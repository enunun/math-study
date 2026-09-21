//! シーンの式を読み，定義域を評価する．描画は，ここで作る`Compiled`から式を評価する．

use std::collections::HashMap;

use crate::error::{Error, ErrorKind};
use crate::expr::{Expr, is_reserved_name};
use crate::scene::{
    Anchor, Axis, Bound, Curve, Direction, Graph, Grid, Label, Object, Point, Region, Scene, View,
};
use crate::validate::curve_expressions;

/// 式を読んだ後の，グラフ．
pub struct GraphPlot {
    /// 変数と媒介変数を入れて評価する式．名前の順は，変数，媒介変数である．
    pub expr: Expr,
    /// 評価した定義域．
    pub domain: [f64; 2],
}

/// 式を読んだ後の，媒介変数表示の曲線．
pub struct CurvePlot {
    /// 各座標の式(平面では2個，空間では3個)．名前の順は，変数，媒介変数である．
    pub exprs: Vec<Expr>,
    /// 評価した媒介変数の範囲．
    pub domain: [f64; 2],
}

/// 式を読んだ後の，軸の目盛．
pub struct TickPlot {
    /// 評価した位置．
    pub at: f64,
    /// 名前の式．
    pub label: Option<String>,
    /// 指定された名前の向き．
    pub anchor: Option<Anchor>,
}

/// 式を読んだ後の，軸．
pub struct AxisPlot {
    /// 目盛．書かれた順に並ぶ．
    pub ticks: Vec<TickPlot>,
}

/// 式を読んだ後の，格子．
pub struct GridPlot {
    /// 評価したx方向の刻み．
    pub x_step: Option<f64>,
    /// 評価したy方向の刻み．
    pub y_step: Option<f64>,
}

/// 格子の，方向ごとの線の数の上限．
const MAX_GRID_LINES: f64 = 200.0;

/// 式を読んだ後の，点．
pub struct PointPlot {
    /// 評価した座標(数学の座標)．
    pub at: [f64; 2],
}

/// 式を読んだ後の，ラベルの位置．
pub struct LabelPlot {
    /// 評価した座標(数学の座標)．平面の図では2個，空間の図では3個である．
    pub at: Vec<f64>,
}

/// 式を読んだ後の，ベクトルか線分．両端の点の座標を持つ．
pub struct LinkPlot {
    /// 始点(数学の座標)．
    pub from: [f64; 2],
    /// 終点(数学の座標)．
    pub to: [f64; 2],
}

/// 式を読んだ後の，領域．
pub struct RegionPlot {
    /// 評価した，xの範囲．
    pub domain: [f64; 2],
}

/// 式を読んだ後の，描く対象．
pub enum Plot {
    /// 領域．
    Region(RegionPlot),
    /// 点．
    Point(PointPlot),
    /// ラベル．
    Label(LabelPlot),
    /// ベクトルか線分．
    Link(LinkPlot),
    /// 格子．
    Grid(GridPlot),
    /// 座標軸．
    Axis(AxisPlot),
    /// 関数のグラフ．
    Graph(GraphPlot),
    /// 曲線．
    Curve(CurvePlot),
    /// 式のないオブジェクト．
    None,
}

/// 式を読んだ後のシーン．
pub struct Compiled {
    /// 式の中の名前の値．媒介変数(シーンの中の順)のあとに，点の座標(`x`，`y`の順で，点の順)が続く．
    /// あるオブジェクトの式が使える名前は，この並びの先頭の部分だけである．
    pub parameters: Vec<f64>,
    /// オブジェクトごとの，描く対象．`scene.objects`と同じ順に並ぶ．
    pub plots: Vec<Plot>,
}

/// シーンの式をすべて読み，定義域を評価する．
///
/// # Errors
///
/// 式の誤り，使えない名前，正しくない定義域があると，原因のオブジェクトの`id`つきの誤りを返す．
pub fn compile(scene: &Scene) -> Result<Compiled, Error> {
    let mut names: Vec<String> = Vec::new();
    let mut values = Vec::new();
    for object in &scene.objects {
        if let Object::Parameter(parameter) = object {
            if is_reserved_name(&parameter.id) {
                return Err(Error::in_object(
                    &parameter.id,
                    ErrorKind::ReservedName(parameter.id.clone()),
                ));
            }
            names.push(parameter.id.clone());
            values.push(parameter.value);
        }
    }
    // オブジェクトを順に読む．点の座標は，読んだあとに，あとのオブジェクトが使える名前と値に加わる．
    let mut plots = Vec::with_capacity(scene.objects.len());
    let mut coordinates: HashMap<&str, [f64; 2]> = HashMap::new();
    for object in &scene.objects {
        let visible: Vec<&str> = names.iter().map(String::as_str).collect();
        let plot = compile_object(object, &visible, &values, &scene.view)?;
        if let (Object::Point(point), Plot::Point(placed)) = (object, &plot) {
            for (suffix, value) in [("x", placed.at[0]), ("y", placed.at[1])] {
                let name = format!("{}_{suffix}", point.id);
                if names.contains(&name) {
                    return Err(Error::in_object(&point.id, ErrorKind::NameConflict(name)));
                }
                names.push(name);
                values.push(value);
            }
            coordinates.insert(point.id.as_str(), placed.at);
        }
        plots.push(plot);
    }
    check_region_domains(scene, &plots)?;
    // ベクトルと線分は，点の座標がすべて決まってから，端の座標を引く．
    for (object, plot) in scene.objects.iter().zip(&mut plots) {
        let (from, to) = match object {
            Object::Vector(vector) => (&vector.from, &vector.to),
            Object::Segment(segment) => (&segment.from, &segment.to),
            _ => continue,
        };
        if let (Some(from), Some(to)) =
            (coordinates.get(from.as_str()), coordinates.get(to.as_str()))
        {
            *plot = Plot::Link(LinkPlot {
                from: *from,
                to: *to,
            });
        }
    }
    Ok(Compiled {
        parameters: values,
        plots,
    })
}

fn compile_object(
    object: &Object,
    names: &[&str],
    parameters: &[f64],
    view: &View,
) -> Result<Plot, Error> {
    match object {
        Object::Axis(axis) => compile_axis(axis, names, parameters, view)
            .map(Plot::Axis)
            .map_err(|kind| Error::in_object(&axis.id, kind)),
        Object::Graph(graph) => compile_graph(graph, names, parameters)
            .map(Plot::Graph)
            .map_err(|kind| Error::in_object(&graph.id, kind)),
        Object::Curve(curve) => compile_curve(curve, names, parameters, curve_expressions(view))
            .map(Plot::Curve)
            .map_err(|kind| Error::in_object(&curve.id, kind)),
        Object::Grid(grid) => compile_grid(grid, names, parameters, view)
            .map(Plot::Grid)
            .map_err(|kind| Error::in_object(&grid.id, kind)),
        Object::Label(label) => compile_label(label, names, parameters)
            .map(Plot::Label)
            .map_err(|kind| Error::in_object(&label.id, kind)),
        Object::Point(point) => compile_point(point, names, parameters)
            .map(Plot::Point)
            .map_err(|kind| Error::in_object(&point.id, kind)),
        Object::Region(region) => compile_region(region, names, parameters)
            .map(Plot::Region)
            .map_err(|kind| Error::in_object(&region.id, kind)),
        Object::Parameter(_) | Object::Sphere(_) | Object::Vector(_) | Object::Segment(_) => {
            Ok(Plot::None)
        }
    }
}

/// 変数の名前を検査し，変数を先頭にした，式の名前の並びを作る．
fn expression_names<'a>(var: &'a str, names: &[&'a str]) -> Result<Vec<&'a str>, ErrorKind> {
    if is_reserved_name(var) {
        return Err(ErrorKind::ReservedName(var.to_owned()));
    }
    if names.contains(&var) {
        return Err(ErrorKind::NameConflict(var.to_owned()));
    }
    Ok(std::iter::once(var).chain(names.iter().copied()).collect())
}

fn compile_expr(
    field: &'static str,
    index: usize,
    source: &str,
    names: &[&str],
) -> Result<Expr, ErrorKind> {
    Expr::compile(source, names).map_err(|error| ErrorKind::Expression {
        field,
        index,
        error,
    })
}

fn compile_graph(
    graph: &Graph,
    names: &[&str],
    parameters: &[f64],
) -> Result<GraphPlot, ErrorKind> {
    let with_var = expression_names(&graph.var, names)?;
    let expr = compile_expr("expr", 0, &graph.expr, &with_var)?;
    let domain = evaluate_domain(&graph.domain, names, parameters)?;
    Ok(GraphPlot { expr, domain })
}

fn compile_curve(
    curve: &Curve,
    names: &[&str],
    parameters: &[f64],
    size: usize,
) -> Result<CurvePlot, ErrorKind> {
    let with_var = expression_names(&curve.var, names)?;
    let exprs = curve
        .expr
        .iter()
        .enumerate()
        .map(|(index, source)| compile_expr("expr", index, source, &with_var))
        .collect::<Result<Vec<_>, _>>()?;
    if exprs.len() != size {
        return Err(ErrorKind::ExpressionCount {
            expected: size,
            found: exprs.len(),
        });
    }
    let domain = evaluate_domain(&curve.domain, names, parameters)?;
    Ok(CurvePlot { exprs, domain })
}

fn compile_region(
    region: &Region,
    names: &[&str],
    parameters: &[f64],
) -> Result<RegionPlot, ErrorKind> {
    Ok(RegionPlot {
        domain: evaluate_domain(&region.domain, names, parameters)?,
    })
}

/// 領域の範囲は，挟むグラフの定義域の中でなければならない．グラフは，領域よりあとに置いてもよい．
fn check_region_domains(scene: &Scene, plots: &[Plot]) -> Result<(), Error> {
    let graph_domains: HashMap<&str, [f64; 2]> = scene
        .objects
        .iter()
        .zip(plots)
        .filter_map(|(object, plot)| match (object, plot) {
            (Object::Graph(graph), Plot::Graph(placed)) => Some((graph.id.as_str(), placed.domain)),
            _ => None,
        })
        .collect();
    for (object, plot) in scene.objects.iter().zip(plots) {
        let (Object::Region(region), Plot::Region(placed)) = (object, plot) else {
            continue;
        };
        for name in &region.between {
            let Some([low, high]) = graph_domains.get(name.as_str()) else {
                continue;
            };
            let [from, to] = placed.domain;
            if from < *low - DOMAIN_TOLERANCE || to > *high + DOMAIN_TOLERANCE {
                return Err(Error::in_object(
                    &region.id,
                    ErrorKind::Invalid(format!(
                        "領域の範囲[{from}, {to}]が，グラフ「{name}」の定義域[{low}, {high}]の外にある．"
                    )),
                ));
            }
        }
    }
    Ok(())
}

/// 領域の範囲とグラフの定義域を比べるときの，誤差の許容．
const DOMAIN_TOLERANCE: f64 = 1e-9;

/// 座標の式を評価し，有限の数にする．
fn evaluate_coordinates(
    bounds: &[Bound],
    names: &[&str],
    parameters: &[f64],
) -> Result<Vec<f64>, ErrorKind> {
    bounds
        .iter()
        .enumerate()
        .map(|(index, bound)| {
            let value = evaluate_bound("at", bound, index, names, parameters)?;
            if value.is_finite() {
                Ok(value)
            } else {
                Err(ErrorKind::Invalid(
                    "座標(`at`)は，有限の数にする．".to_owned(),
                ))
            }
        })
        .collect()
}

fn compile_label(
    label: &Label,
    names: &[&str],
    parameters: &[f64],
) -> Result<LabelPlot, ErrorKind> {
    Ok(LabelPlot {
        at: evaluate_coordinates(&label.at, names, parameters)?,
    })
}

fn compile_point(
    point: &Point,
    names: &[&str],
    parameters: &[f64],
) -> Result<PointPlot, ErrorKind> {
    let at = evaluate_coordinates(&point.at, names, parameters)?;
    match at.as_slice() {
        [x, y] => Ok(PointPlot { at: [*x, *y] }),
        _ => Err(ErrorKind::Invalid(
            "点の座標(`at`)は，2個の数で書く．".to_owned(),
        )),
    }
}

/// 格子の刻みを評価し，正の有限の数で，線が多すぎないことを確かめる．
fn compile_grid(
    grid: &Grid,
    names: &[&str],
    parameters: &[f64],
    view: &View,
) -> Result<GridPlot, ErrorKind> {
    let (x_range, y_range) = match view {
        View::Plane(plane) => (plane.x, plane.y),
        View::Space(_) => ([0.0, 0.0], [0.0, 0.0]),
    };
    let step = |field: &'static str, bound: &Option<Bound>, range: [f64; 2]| {
        let Some(bound) = bound else {
            return Ok(None);
        };
        let value = evaluate_bound(field, bound, 0, names, parameters)?;
        if !(value.is_finite() && value > 0.0) {
            return Err(ErrorKind::Invalid(format!(
                "`{field}`は，正の有限の数にする．"
            )));
        }
        let lines = ((range[1] - range[0]) / value).floor() + 1.0;
        if lines > MAX_GRID_LINES {
            return Err(ErrorKind::Invalid(format!(
                "刻み`{field}`が細かすぎる．線が{lines}本になる(上限は{MAX_GRID_LINES}本)．"
            )));
        }
        Ok(Some(value))
    };
    Ok(GridPlot {
        x_step: step("x_step", &grid.x_step, x_range)?,
        y_step: step("y_step", &grid.y_step, y_range)?,
    })
}

/// 目盛の位置を評価し，軸の範囲の中にあることを確かめる．範囲を省いた平面の軸は，見える範囲である．
fn compile_axis(
    axis: &Axis,
    names: &[&str],
    parameters: &[f64],
    view: &View,
) -> Result<AxisPlot, ErrorKind> {
    let range = axis.range.or(match (view, axis.direction) {
        (View::Plane(plane), Direction::X) => Some(plane.x),
        (View::Plane(plane), Direction::Y) => Some(plane.y),
        _ => None,
    });
    let ticks = axis
        .ticks
        .iter()
        .enumerate()
        .map(|(index, tick)| {
            let at = evaluate_bound("ticks", &tick.at, index, names, parameters)?;
            match range {
                Some([low, high]) if !(low <= at && at <= high) => Err(ErrorKind::Invalid(
                    format!("目盛の位置{at}は，軸の範囲[{low}, {high}]の外にある．"),
                )),
                _ => Ok(TickPlot {
                    at,
                    label: tick.label.clone(),
                    anchor: tick.anchor,
                }),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(AxisPlot { ticks })
}

/// 定義域の端を評価し，有限の数で，下端が上端より小さいことを確かめる．端の式は，変数を使えない．
fn evaluate_domain(
    domain: &[Bound; 2],
    names: &[&str],
    parameters: &[f64],
) -> Result<[f64; 2], ErrorKind> {
    let [low, high] = domain;
    let low = evaluate_bound("domain", low, 0, names, parameters)?;
    let high = evaluate_bound("domain", high, 1, names, parameters)?;
    if low.is_finite() && high.is_finite() && low < high {
        Ok([low, high])
    } else {
        Err(ErrorKind::InvalidRange("domain"))
    }
}

fn evaluate_bound(
    field: &'static str,
    bound: &Bound,
    index: usize,
    names: &[&str],
    parameters: &[f64],
) -> Result<f64, ErrorKind> {
    match bound {
        Bound::Number(value) => Ok(*value),
        Bound::Expression(source) => {
            Ok(compile_expr(field, index, source, names)?.eval(parameters))
        }
    }
}
