//! シーンの式を読み，定義域を評価する．描画は，ここで作る`Compiled`から式を評価する．

use crate::error::{Error, ErrorKind};
use crate::expr::{Expr, is_reserved_name};
use crate::scene::{Axis, Bound, Curve, Direction, Graph, Object, Scene, View};
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
}

/// 式を読んだ後の，軸．
pub struct AxisPlot {
    /// 目盛．書かれた順に並ぶ．
    pub ticks: Vec<TickPlot>,
}

/// 式を読んだ後の，描く対象．
pub enum Plot {
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
    /// 媒介変数の値．シーンの中の媒介変数の順に並ぶ．
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
    let mut names = Vec::new();
    let mut parameters = Vec::new();
    for object in &scene.objects {
        if let Object::Parameter(parameter) = object {
            if is_reserved_name(&parameter.id) {
                return Err(Error::in_object(
                    &parameter.id,
                    ErrorKind::ReservedName(parameter.id.clone()),
                ));
            }
            names.push(parameter.id.as_str());
            parameters.push(parameter.value);
        }
    }
    let plots = scene
        .objects
        .iter()
        .map(|object| compile_object(object, &names, &parameters, &scene.view))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Compiled { parameters, plots })
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
        Object::Label(_) | Object::Parameter(_) | Object::Sphere(_) => Ok(Plot::None),
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
