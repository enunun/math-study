//! シーンの式を読み，定義域を評価する．描画は，ここで作る`Compiled`から式を評価する．

use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{Error, ErrorKind};
use crate::expr::{Expr, Functions, is_reserved_name};
use crate::fractal;
use crate::image::transform_of;
use crate::scene::{
    Anchor, Axis, Bound, Curve, Cut, Direction, Factor, Fractal, Graph, Grid, Label, Object, Point,
    Polygon, Position, Region, Scene, Surface, TangentLine, TangentPlane, Taylor, TransformStep,
    View,
};
use crate::transform::{Affine, MapFn, Transform, Vector3};
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
    /// 各座標の式(平面では2個，空間では3個)．ベジエ曲線・スプライン曲線では空．
    /// 名前の順は，変数，媒介変数である．
    pub exprs: Vec<Expr>,
    /// 評価した媒介変数の範囲．ベジエ曲線・スプライン曲線では`[0.0, 1.0]`である．
    pub domain: [f64; 2],
    /// ベジエ曲線の制御点の座標(各点，平面では2個，空間では3個)．それ以外の曲線では`None`．
    pub net: Option<Vec<Vec<f64>>>,
    /// スプライン曲線が順に通る点の座標(各点，平面では2個，空間では3個)．それ以外の曲線では`None`．
    pub spline: Option<Vec<Vec<f64>>>,
}

/// 式を読んだ後の，接線．
pub struct TangentLinePlot {
    /// 接する点(`of`がグラフなら変数の値，曲線なら媒介変数の値)．
    pub at: f64,
}

/// 式を読んだ後の，接平面．
pub struct TangentPlanePlot {
    /// 接する点の，曲面の2つの変数の値．
    pub at: [f64; 2],
    /// 接平面の半径(cm)．
    pub size: f64,
}

/// 式を読んだ後の，フラクタル図形．反復関数系(IFS)の展開まで，あらかじめ計算してある．
pub struct FractalPlot {
    /// 展開したあとの，線をなす点の並び(数学の座標)の列．基本図形1つにつき1本の並びである．
    pub paths: Vec<Vec<[f64; 2]>>,
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
    /// 線を引くxの範囲．平面の図で範囲を省けば，見える範囲である．
    pub x_range: [f64; 2],
    /// 線を引くyの範囲．
    pub y_range: [f64; 2],
}

/// 格子の，方向ごとの線の数の上限．
const MAX_GRID_LINES: f64 = 200.0;

/// 式を読んだ後の，点．
pub struct PointPlot {
    /// 評価した座標(数学の座標)．平面の図では2個，空間の図では3個である．
    pub at: Vec<f64>,
}

/// 式を読んだ後の，ラベルの位置．
pub struct LabelPlot {
    /// 評価した座標(数学の座標)．平面の図では2個，空間の図では3個である．
    pub at: Vec<f64>,
}

/// 式を読んだ後の，ベクトルか線分．両端の点の座標を持つ．
pub struct LinkPlot {
    /// 始点(数学の座標)．
    pub from: Vec<f64>,
    /// 終点(数学の座標)．
    pub to: Vec<f64>,
}

/// 式を読んだ後の，切り口の平面．
pub struct CutPlot {
    /// 平面の法線．
    pub normal: [f64; 3],
    /// 平面の定数．
    pub offset: f64,
}

/// 式を読んだ後の，曲面．
pub struct SurfacePlot {
    /// x，y，z座標の式．名前の順は，2つの変数，媒介変数と点の座標である．ベジエ曲面では空である．
    pub exprs: Vec<Expr>,
    /// 評価した，各変数の範囲．ベジエ曲面では，どちらも0から1である．
    pub domain: [[f64; 2]; 2],
    /// 評価した，ベジエ曲面の制御点の網．式で書く曲面では`None`である．
    pub net: Option<Vec<Vec<[f64; 3]>>>,
    /// ワイヤーフレームの断面を引く，uの値とvの値．`wireframe`がなければ，どちらも空である．
    pub wireframe: [Vec<f64>; 2],
}

/// 式を読んだ後の，領域．
pub struct RegionPlot {
    /// 評価した，xの範囲．
    pub domain: [f64; 2],
}

/// 式を読んだ後の，描く対象．
pub enum Plot {
    /// 切り口．
    Cut(CutPlot),
    /// 曲面．
    Surface(SurfacePlot),
    /// 領域．
    Region(RegionPlot),
    /// フラクタル図形．
    Fractal(FractalPlot),
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
    /// 接線．
    TangentLine(TangentLinePlot),
    /// 接平面．
    TangentPlane(TangentPlanePlot),
    /// 多角形．
    Polygon(PolygonPlot),
    /// テイラー展開の多項式．
    Taylor(TaylorPlot),
    /// 式のないオブジェクト．
    None,
}

/// 式を読んだ後の，多角形．
pub struct PolygonPlot {
    /// 頂点(数学の座標)．周に沿って並ぶ．変換(`transform`)は，まだ施していない．
    pub vertices: Vec<[f64; 2]>,
}

/// 式を読んだ後の，テイラー展開の多項式．
pub struct TaylorPlot {
    /// 展開の中心．
    pub at: f64,
    /// 描く範囲．
    pub domain: [f64; 2],
    /// 0次からの係数．
    pub coefficients: Vec<f64>,
}

impl TaylorPlot {
    /// 多項式の値．ホーナー法で計算する．
    #[must_use]
    pub fn value(&self, x: f64) -> f64 {
        let h = x - self.at;
        self.coefficients
            .iter()
            .rev()
            .fold(0.0, |sum, coefficient| sum.mul_add(h, *coefficient))
    }
}

/// 式を読んだ後のシーン．
pub struct Compiled {
    /// 式の中の名前の値．媒介変数(シーンの中の順)のあとに，点の座標(`x`，`y`の順で，点の順)が続く．
    /// あるオブジェクトの式が使える名前は，この並びの先頭の部分だけである．
    pub parameters: Vec<f64>,
    /// オブジェクトごとの，描く対象．`scene.objects`と同じ順に並ぶ．
    pub plots: Vec<Plot>,
    /// オブジェクトごとの変換(`transform`)．`scene.objects`と同じ順に並ぶ．変換を持たないオブジェクトは，
    /// 何もしない変換である．点・線分・ベクトル・フラクタルの変換は，ここで施してあり，描画では使わない．
    pub transforms: Vec<Transform>,
}

/// 式を読むときに使える名前と，その値と，利用者が定義した関数．
#[derive(Clone, Copy)]
struct Env<'a> {
    /// 名前．媒介変数と，先に置いた点の座標である．
    names: &'a [&'a str],
    /// 名前の値．`names`より長くてもよい(先頭の部分だけを使う)．
    parameters: &'a [f64],
    /// 利用者が定義した関数．
    functions: &'a Functions,
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
    let parameter_names: Vec<&str> = names.iter().map(String::as_str).collect();
    let functions = compile_functions(scene, &parameter_names)?;
    let parameter_env = Env {
        names: &parameter_names,
        parameters: &values,
        functions: &functions,
    };
    let dimension = curve_expressions(&scene.view);
    let maps = compile_maps(scene, dimension, &parameter_env)?;
    // オブジェクトを順に読む．点の座標は，読んだあとに，あとのオブジェクトが使える名前と値に加わる．
    let mut plots = Vec::with_capacity(scene.objects.len());
    let mut transforms = Vec::with_capacity(scene.objects.len());
    let mut coordinates: HashMap<&str, Vec<f64>> = HashMap::new();
    let parameter_count = names.len();
    // 点の式に使える，先に置いた点(idと座標)．
    let mut placed_points: Vec<(&str, Vec<f64>)> = Vec::new();
    for object in &scene.objects {
        let visible: Vec<&str> = names.iter().map(String::as_str).collect();
        let env = Env {
            names: &visible,
            parameters: &values,
            functions: &functions,
        };
        let scope = VectorScope {
            parameter_names: visible.get(..parameter_count).unwrap_or_default(),
            parameter_values: values.get(..parameter_count).unwrap_or_default(),
            points: &placed_points,
            dimension,
            functions: &functions,
        };
        let transform = compile_transform(
            transform_of(object).unwrap_or_default(),
            dimension,
            &env,
            &maps,
        )
        .map_err(|kind| Error::in_object(object.id(), kind))?;
        let plot = compile_object(object, &env, &scene.view, &scope, &transform)?;
        if let (Object::Point(point), Plot::Point(placed)) = (object, &plot) {
            if is_reserved_name(&point.id) {
                return Err(Error::in_object(
                    &point.id,
                    ErrorKind::ReservedName(point.id.clone()),
                ));
            }
            placed_points.push((point.id.as_str(), placed.at.clone()));
            for (suffix, value) in ["x", "y", "z"].into_iter().zip(&placed.at) {
                let name = format!("{}_{suffix}", point.id);
                if names.contains(&name) {
                    return Err(Error::in_object(&point.id, ErrorKind::NameConflict(name)));
                }
                names.push(name);
                values.push(*value);
            }
            coordinates.insert(point.id.as_str(), placed.at.clone());
        }
        plots.push(plot);
        transforms.push(transform);
    }
    check_region_domains(scene, &plots)?;
    // ベクトルと線分は，点の座標がすべて決まってから，端の座標を引いて，変換を施す．
    for ((object, plot), transform) in scene.objects.iter().zip(&mut plots).zip(&transforms) {
        let (from, to) = match object {
            Object::Vector(vector) => (&vector.from, &vector.to),
            Object::Segment(segment) => (&segment.from, &segment.to),
            _ => continue,
        };
        let moved = |id: &str| coordinates.get(id).and_then(|at| transform.apply(at));
        if let (Some(from), Some(to)) = (moved(from), moved(to)) {
            *plot = Plot::Link(LinkPlot { from, to });
        }
    }
    expand_taylor_polynomials(scene, &mut plots, &values)?;
    Ok(Compiled {
        parameters: values,
        plots,
        transforms,
    })
}

/// 関数(`function`)を，置いた順に定義する．本体は，媒介変数と，先に定義した関数を使える．
fn compile_functions(scene: &Scene, parameter_names: &[&str]) -> Result<Functions, Error> {
    let mut functions = Functions::default();
    for object in &scene.objects {
        let Object::Function(function) = object else {
            continue;
        };
        let fail = |kind| Error::in_object(&function.id, kind);
        if is_reserved_name(&function.id) {
            return Err(fail(ErrorKind::ReservedName(function.id.clone())));
        }
        if parameter_names.contains(&function.id.as_str()) {
            return Err(fail(ErrorKind::NameConflict(function.id.clone())));
        }
        let mut vars: Vec<&str> = Vec::with_capacity(function.vars.len());
        for var in &function.vars {
            if is_reserved_name(var) {
                return Err(fail(ErrorKind::ReservedName(var.clone())));
            }
            if parameter_names.contains(&var.as_str()) || vars.contains(&var.as_str()) {
                return Err(fail(ErrorKind::NameConflict(var.clone())));
            }
            vars.push(var);
        }
        functions
            .define(&function.id, &vars, &function.expr, parameter_names)
            .map_err(|error| {
                fail(ErrorKind::Expression {
                    field: "expr",
                    index: 0,
                    error,
                })
            })?;
    }
    Ok(functions)
}

/// 写像(`map`)の式を読む．式は，座標の変数と，媒介変数と，関数を使える．
fn compile_maps<'a>(
    scene: &'a Scene,
    dimension: usize,
    env: &Env,
) -> Result<HashMap<&'a str, Rc<MapFn>>, Error> {
    let mut maps = HashMap::new();
    for object in &scene.objects {
        let Object::Map(map) = object else {
            continue;
        };
        let fail = |kind| Error::in_object(&map.id, kind);
        if map.vars.len() != dimension || map.expr.len() != dimension {
            return Err(fail(ErrorKind::Invalid(format!(
                "写像の変数(`vars`)と式(`expr`)は，どちらも{dimension}個(図の次元の数)書く．"
            ))));
        }
        let mut scope: Vec<&str> = Vec::with_capacity(dimension.saturating_add(env.names.len()));
        for var in &map.vars {
            if is_reserved_name(var) {
                return Err(fail(ErrorKind::ReservedName(var.clone())));
            }
            if env.names.contains(&var.as_str()) || scope.contains(&var.as_str()) {
                return Err(fail(ErrorKind::NameConflict(var.clone())));
            }
            scope.push(var);
        }
        scope.extend(env.names);
        let exprs = map
            .expr
            .iter()
            .enumerate()
            .map(|(index, source)| compile_expr("expr", index, source, &scope, env.functions))
            .collect::<Result<Vec<_>, _>>()
            .map_err(fail)?;
        maps.insert(
            map.id.as_str(),
            Rc::new(MapFn::new(exprs, env.parameters.to_vec())),
        );
    }
    Ok(maps)
}

/// 変換の手順を評価し，1つの変換にする．
fn compile_transform(
    steps: &[TransformStep],
    dimension: usize,
    env: &Env,
    maps: &HashMap<&str, Rc<MapFn>>,
) -> Result<Transform, ErrorKind> {
    let mut transform = Transform::default();
    for step in steps {
        if let Some(affine) = step_affine(step, dimension, env)? {
            transform.push_affine(affine);
        } else {
            let id = step.map.as_deref().unwrap_or_default();
            let map = maps.get(id).ok_or_else(|| {
                ErrorKind::Invalid(format!(
                    "写像「{id}」がない．`map`には，`map`オブジェクトの`id`を書く．"
                ))
            })?;
            transform.push_map(Rc::clone(map));
        }
    }
    Ok(transform)
}

/// 変換の手順1つの，操作の名前．手順には，操作をちょうど1つ書く．
fn step_operation(step: &TransformStep) -> Result<&'static str, ErrorKind> {
    let present: Vec<&'static str> = [
        ("translate", step.translate.is_some()),
        ("rotate", step.rotate.is_some()),
        ("scale", step.scale.is_some()),
        ("reflect", step.reflect.is_some()),
        ("shear", step.shear.is_some()),
        ("map", step.map.is_some()),
    ]
    .into_iter()
    .filter_map(|(name, is_present)| is_present.then_some(name))
    .collect();
    match present.as_slice() {
        [operation] => Ok(operation),
        _ => Err(ErrorKind::Invalid(
            "変換の手順には，`translate`，`rotate`，`scale`，`reflect`，`shear`，`map`のどれか1つだけを書く．"
                .to_owned(),
        )),
    }
}

/// 変換の手順1つを評価する．アフィン変換ならそれを，写像なら`None`を返す．
fn step_affine(
    step: &TransformStep,
    dimension: usize,
    env: &Env,
) -> Result<Option<Affine>, ErrorKind> {
    let operation = step_operation(step)?;
    if step.axis.is_some() && !(operation == "rotate" && dimension == SPACE_DIMENSION) {
        return Err(ErrorKind::Invalid(
            "回転の軸(`axis`)は，空間の図の回転(`rotate`)にだけ書く．".to_owned(),
        ));
    }
    if step.center.is_some() && matches!(operation, "translate" | "map") {
        return Err(ErrorKind::Invalid(format!(
            "`center`は，`{operation}`には書けない(回転・拡大縮小・対称移動・せん断にだけ書く)．"
        )));
    }
    let vector = |field: &'static str, bounds: &[Bound]| -> Result<Vector3, ErrorKind> {
        let values = evaluate_numbers(field, bounds, env)?;
        if values.len() != dimension {
            return Err(ErrorKind::Invalid(format!(
                "`{field}`は，{dimension}個の数で書く．"
            )));
        }
        Ok(pad(&values, 0.0))
    };
    let unit = |field: &'static str, bounds: &[Bound]| -> Result<Vector3, ErrorKind> {
        let v = vector(field, bounds)?;
        let length = v.iter().map(|c| c * c).sum::<f64>().sqrt();
        if length > 0.0 {
            Ok(v.map(|c| c / length))
        } else {
            Err(ErrorKind::Invalid(format!(
                "`{field}`は，0でないベクトルにする．"
            )))
        }
    };
    let center = step
        .center
        .as_deref()
        .map(|bounds| vector("center", bounds))
        .transpose()?
        .unwrap_or([0.0; 3]);
    let affine = if let Some(offset) = &step.translate {
        Affine::translate(vector("translate", offset)?)
    } else if let Some(angle) = &step.rotate {
        let degrees = evaluate_numbers("rotate", std::slice::from_ref(angle), env)?
            .first()
            .copied()
            .unwrap_or_default();
        let rotation = if dimension == SPACE_DIMENSION {
            let axis = step.axis.as_deref().ok_or_else(|| {
                ErrorKind::Invalid(
                    "空間の図の回転(`rotate`)には，軸の向き(`axis`)を書く．".to_owned(),
                )
            })?;
            Affine::rotate_about_axis(degrees, unit("axis", axis)?)
        } else {
            Affine::rotate_plane(degrees)
        };
        rotation.about(center)
    } else if let Some(factor) = &step.scale {
        let factors = match factor {
            Factor::Uniform(bound) => {
                let k = evaluate_numbers("scale", std::slice::from_ref(bound), env)?
                    .first()
                    .copied()
                    .unwrap_or(1.0);
                [k; 3]
            }
            Factor::PerAxis(bounds) => {
                let values = evaluate_numbers("scale", bounds, env)?;
                if values.len() != dimension {
                    return Err(ErrorKind::Invalid(format!(
                        "`scale`は，1つの数か，{dimension}個の数で書く．"
                    )));
                }
                pad(&values, 1.0)
            }
        };
        Affine::scale(factors).about(center)
    } else if let Some(mirror) = &step.reflect {
        let [x, y, z] = unit("reflect", mirror)?;
        let reflection = if dimension == SPACE_DIMENSION {
            Affine::reflect_plane([x, y, z])
        } else {
            Affine::reflect_line([x, y])
        };
        reflection.about(center)
    } else if let Some(amounts) = &step.shear {
        if dimension == SPACE_DIMENSION {
            return Err(ErrorKind::Invalid(
                "せん断(`shear`)は，平面の図でだけ使える．".to_owned(),
            ));
        }
        let [a, b, _] = vector("shear", amounts)?;
        Affine::shear(a, b).about(center)
    } else {
        return Ok(None);
    };
    Ok(Some(affine))
}

/// 空間の図の座標の数．
const SPACE_DIMENSION: usize = 3;

/// 2個か3個の数を，3個にする．足りない所は`fill`で埋める．
fn pad(values: &[f64], fill: f64) -> Vector3 {
    [0, 1, 2].map(|index| values.get(index).copied().unwrap_or(fill))
}

/// 数か式の並びを評価し，どれも有限の数であることを確かめる．
fn evaluate_numbers(
    field: &'static str,
    bounds: &[Bound],
    env: &Env,
) -> Result<Vec<f64>, ErrorKind> {
    bounds
        .iter()
        .enumerate()
        .map(|(index, bound)| {
            let value = evaluate_bound(field, bound, index, env)?;
            if value.is_finite() {
                Ok(value)
            } else {
                Err(ErrorKind::Invalid(format!("`{field}`は，有限の数にする．")))
            }
        })
        .collect()
}

/// 点の式が使える名前と値．媒介変数と，先に置いた点である．
struct VectorScope<'a> {
    parameter_names: &'a [&'a str],
    parameter_values: &'a [f64],
    points: &'a [(&'a str, Vec<f64>)],
    /// 座標の数．平面の図では2，空間の図では3である．
    dimension: usize,
    /// 利用者が定義した関数．
    functions: &'a Functions,
}

fn compile_object(
    object: &Object,
    env: &Env,
    view: &View,
    scope: &VectorScope,
    transform: &Transform,
) -> Result<Plot, Error> {
    match object {
        Object::Axis(axis) => compile_axis(axis, env, view)
            .map(Plot::Axis)
            .map_err(|kind| Error::in_object(&axis.id, kind)),
        Object::Graph(graph) => compile_graph(graph, env)
            .map(Plot::Graph)
            .map_err(|kind| Error::in_object(&graph.id, kind)),
        Object::Curve(curve) => compile_curve(curve, env, curve_expressions(view))
            .map(Plot::Curve)
            .map_err(|kind| Error::in_object(&curve.id, kind)),
        Object::TangentLine(tangent) => compile_tangent_line(tangent, env)
            .map(Plot::TangentLine)
            .map_err(|kind| Error::in_object(&tangent.id, kind)),
        Object::Grid(grid) => compile_grid(grid, env, view)
            .map(Plot::Grid)
            .map_err(|kind| Error::in_object(&grid.id, kind)),
        Object::Label(label) => compile_label(label, env, scope)
            .map(Plot::Label)
            .map_err(|kind| Error::in_object(&label.id, kind)),
        Object::Point(point) => compile_point(point, env, scope, transform)
            .map(Plot::Point)
            .map_err(|kind| Error::in_object(&point.id, kind)),
        Object::Cut(cut) => compile_cut(cut, env)
            .map(Plot::Cut)
            .map_err(|kind| Error::in_object(&cut.id, kind)),
        Object::Surface(surface) => compile_surface(surface, env)
            .map(Plot::Surface)
            .map_err(|kind| Error::in_object(&surface.id, kind)),
        Object::Region(region) => compile_region(region, env)
            .map(Plot::Region)
            .map_err(|kind| Error::in_object(&region.id, kind)),
        Object::Fractal(fractal) => compile_fractal(fractal, env, transform)
            .map(Plot::Fractal)
            .map_err(|kind| Error::in_object(&fractal.id, kind)),
        Object::TangentPlane(tangent) => compile_tangent_plane(tangent, env)
            .map(Plot::TangentPlane)
            .map_err(|kind| Error::in_object(&tangent.id, kind)),
        Object::Polygon(polygon) => compile_polygon(polygon, env, scope)
            .map(Plot::Polygon)
            .map_err(|kind| Error::in_object(&polygon.id, kind)),
        Object::Taylor(taylor) => compile_taylor(taylor, env)
            .map(Plot::Taylor)
            .map_err(|kind| Error::in_object(&taylor.id, kind)),
        Object::Parameter(_)
        | Object::Sphere(_)
        | Object::Vector(_)
        | Object::Segment(_)
        | Object::Intersection(_)
        | Object::Complex(_)
        | Object::Polyhedron(_)
        | Object::Function(_)
        | Object::Map(_)
        | Object::Image(_) => Ok(Plot::None),
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
    functions: &Functions,
) -> Result<Expr, ErrorKind> {
    Expr::compile_with(source, names, functions).map_err(|error| ErrorKind::Expression {
        field,
        index,
        error,
    })
}

fn compile_graph(graph: &Graph, env: &Env) -> Result<GraphPlot, ErrorKind> {
    let with_var = expression_names(&graph.var, env.names)?;
    let expr = compile_expr("expr", 0, &graph.expr, &with_var, env.functions)?;
    let domain = evaluate_domain(&graph.domain, env)?;
    Ok(GraphPlot { expr, domain })
}

/// 曲線は，式(`var`，`expr`，`domain`)か，ベジエ曲線の制御点(`bezier`)か，
/// スプライン曲線が通る点(`spline`)の，どれか1つで書く．
const CURVE_FORM_HINT: &str = "曲線は，式(`var`，`expr`，`domain`)か，ベジエ曲線の制御点(`bezier`)か，\
     スプライン曲線の点(`spline`)で書く．";

fn compile_curve(curve: &Curve, env: &Env, size: usize) -> Result<CurvePlot, ErrorKind> {
    if let Some(net) = &curve.bezier {
        let evaluated = compile_curve_points(net, env, size, "bezier")?;
        return Ok(CurvePlot {
            exprs: Vec::new(),
            domain: [0.0, 1.0],
            net: Some(evaluated),
            spline: None,
        });
    }
    if let Some(points) = &curve.spline {
        let evaluated = compile_curve_points(points, env, size, "spline")?;
        return Ok(CurvePlot {
            exprs: Vec::new(),
            domain: [0.0, 1.0],
            net: None,
            spline: Some(evaluated),
        });
    }
    let Some(var) = &curve.var else {
        return Err(ErrorKind::Invalid(CURVE_FORM_HINT.to_owned()));
    };
    let with_var = expression_names(var, env.names)?;
    let exprs = curve
        .expr
        .iter()
        .enumerate()
        .map(|(index, source)| compile_expr("expr", index, source, &with_var, env.functions))
        .collect::<Result<Vec<_>, _>>()?;
    if exprs.len() != size {
        return Err(ErrorKind::ExpressionCount {
            expected: size,
            found: exprs.len(),
        });
    }
    let Some(domain) = &curve.domain else {
        return Err(ErrorKind::Invalid(CURVE_FORM_HINT.to_owned()));
    };
    let domain = evaluate_domain(domain, env)?;
    Ok(CurvePlot {
        exprs,
        domain,
        net: None,
        spline: None,
    })
}

/// ベジエ曲線・スプライン曲線に共通の，点の座標を評価する処理．座標は，媒介変数と定数を使え，
/// 有限の数でなければならない．
fn compile_curve_points(
    points: &[Vec<Bound>],
    env: &Env,
    size: usize,
    field: &'static str,
) -> Result<Vec<Vec<f64>>, ErrorKind> {
    points
        .iter()
        .map(|point| {
            let coordinates = point
                .iter()
                .enumerate()
                .map(|(index, bound)| evaluate_bound(field, bound, index, env))
                .collect::<Result<Vec<_>, _>>()?;
            if coordinates.len() == size && coordinates.iter().all(|c| c.is_finite()) {
                Ok(coordinates)
            } else {
                Err(ErrorKind::Invalid(format!(
                    "`{field}`の点の座標は，有限の数にする．"
                )))
            }
        })
        .collect::<Result<Vec<_>, _>>()
}

/// 接線の接する点を評価する．接する対象(`of`)がグラフか曲線かは，描画のときに`id`で引く．
fn compile_tangent_line(tangent: &TangentLine, env: &Env) -> Result<TangentLinePlot, ErrorKind> {
    let at = evaluate_bound("at", &tangent.at, 0, env)?;
    Ok(TangentLinePlot { at })
}

/// 接平面の接する点と半径を評価する．接する曲面(`of`)は，描画のときに`id`で引く．
fn compile_tangent_plane(tangent: &TangentPlane, env: &Env) -> Result<TangentPlanePlot, ErrorKind> {
    let [u, v] = &tangent.at;
    let at = [
        evaluate_bound("at", u, 0, env)?,
        evaluate_bound("at", v, 1, env)?,
    ];
    let size = evaluate_bound("size", &tangent.size, 0, env)?;
    if size.is_finite() && size > 0.0 {
        Ok(TangentPlanePlot { at, size })
    } else {
        Err(ErrorKind::Invalid(
            "接平面の半径(`size`)は，正の有限の数にする．".to_owned(),
        ))
    }
}

/// 切り口の平面の式を評価し，法線が0でない有限のベクトルであることを確かめる．
fn compile_cut(cut: &Cut, env: &Env) -> Result<CutPlot, ErrorKind> {
    let evaluate =
        |field: &'static str, index: usize, bound: &Bound| evaluate_bound(field, bound, index, env);
    let mut normal = [0.0; 3];
    for (index, (slot, bound)) in normal.iter_mut().zip(&cut.normal).enumerate() {
        *slot = evaluate("normal", index, bound)?;
    }
    let offset = evaluate("offset", 0, &cut.offset)?;
    let length = normal.iter().map(|c| c * c).sum::<f64>().sqrt();
    if normal.iter().all(|c| c.is_finite()) && offset.is_finite() && length > 0.0 {
        Ok(CutPlot { normal, offset })
    } else {
        Err(ErrorKind::Invalid(
            "平面の法線(`normal`)は0でない有限のベクトルに，定数(`offset`)は有限の数にする．"
                .to_owned(),
        ))
    }
}

fn compile_surface(surface: &Surface, env: &Env) -> Result<SurfacePlot, ErrorKind> {
    let mut plot = compile_surface_shape(surface, env)?;
    if surface.wireframe.is_some() {
        plot.wireframe = wireframe_values(surface, plot.domain, env)?;
    }
    Ok(plot)
}

fn compile_surface_shape(surface: &Surface, env: &Env) -> Result<SurfacePlot, ErrorKind> {
    if let Some(net) = &surface.bezier {
        return compile_bezier(net, env);
    }
    let [first, second] = surface.vars.as_slice() else {
        return Err(ErrorKind::Invalid(
            "曲面の変数(`vars`)は，2つの名前で書く．".to_owned(),
        ));
    };
    for var in [first, second] {
        if is_reserved_name(var) {
            return Err(ErrorKind::ReservedName(var.clone()));
        }
        if env.names.contains(&var.as_str()) {
            return Err(ErrorKind::NameConflict(var.clone()));
        }
    }
    let with_vars: Vec<&str> = [first.as_str(), second.as_str()]
        .into_iter()
        .chain(env.names.iter().copied())
        .collect();
    let exprs = surface
        .expr
        .iter()
        .enumerate()
        .map(|(index, source)| compile_expr("expr", index, source, &with_vars, env.functions))
        .collect::<Result<Vec<_>, _>>()?;
    let Some([u_domain, v_domain]) = &surface.domain else {
        return Err(ErrorKind::Invalid(
            "式で書く曲面には，変数の範囲(`domain`)が要る．".to_owned(),
        ));
    };
    Ok(SurfacePlot {
        exprs,
        domain: [
            evaluate_domain(u_domain, env)?,
            evaluate_domain(v_domain, env)?,
        ],
        net: None,
        wireframe: [Vec::new(), Vec::new()],
    })
}

/// ワイヤーフレームの断面を引く，uの値とvの値．各変数の刻みの整数倍のうち，定義域の内側(両端を除く)に
/// あるものである．刻みを書かなければ，定義域の幅の`Surface::DEFAULT_WIREFRAME_DIVISIONS`分の1にする．
fn wireframe_values(
    surface: &Surface,
    domain: [[f64; 2]; 2],
    env: &Env,
) -> Result<[Vec<f64>; 2], ErrorKind> {
    let mut values = [Vec::new(), Vec::new()];
    for (index, ([low, high], slot)) in domain.into_iter().zip(&mut values).enumerate() {
        let step = match surface
            .wireframe_step
            .as_ref()
            .and_then(|steps| steps.get(index))
        {
            Some(bound) => evaluate_bound("wireframe_step", bound, index, env)?,
            None => (high - low) / Surface::DEFAULT_WIREFRAME_DIVISIONS,
        };
        if !(step.is_finite() && step > 0.0) {
            return Err(ErrorKind::Invalid(
                "ワイヤーフレームの刻み(`wireframe_step`)は，正の有限の数にする．".to_owned(),
            ));
        }
        if (high - low) / step > Surface::MAX_WIREFRAME_LINES {
            return Err(ErrorKind::Invalid(format!(
                "ワイヤーフレームの断面が多すぎる．刻み(`wireframe_step`)を大きくして，各方向{}本以下にする．",
                Surface::MAX_WIREFRAME_LINES
            )));
        }
        // 端とほとんど重なる断面は，縁や輪郭と重なるので引かない．
        let margin = (high - low) * WIREFRAME_EDGE_MARGIN;
        let mut multiple = (low / step).floor();
        while multiple * step < high - margin {
            let value = multiple * step;
            if value > low + margin {
                slot.push(value);
            }
            multiple += 1.0;
        }
    }
    Ok(values)
}

/// 定義域の端と重なるとみなす，断面の近さ(定義域の幅に対する割合)．
const WIREFRAME_EDGE_MARGIN: f64 = 1e-9;

/// ベジエ曲面の制御点の座標を評価する．座標は，媒介変数と定数を使え，有限の数でなければならない．
fn compile_bezier(net: &[Vec<Vec<Bound>>], env: &Env) -> Result<SurfacePlot, ErrorKind> {
    let evaluated = net
        .iter()
        .map(|row| {
            row.iter()
                .map(|point| {
                    let coordinates = point
                        .iter()
                        .enumerate()
                        .map(|(index, bound)| evaluate_bound("bezier", bound, index, env))
                        .collect::<Result<Vec<_>, _>>()?;
                    match coordinates.as_slice() {
                        [x, y, z] if coordinates.iter().all(|c| c.is_finite()) => Ok([*x, *y, *z]),
                        _ => Err(ErrorKind::Invalid(
                            "ベジエ曲面の制御点の座標は，有限の数にする．".to_owned(),
                        )),
                    }
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SurfacePlot {
        exprs: Vec::new(),
        domain: [[0.0, 1.0], [0.0, 1.0]],
        net: Some(evaluated),
        wireframe: [Vec::new(), Vec::new()],
    })
}

fn compile_region(region: &Region, env: &Env) -> Result<RegionPlot, ErrorKind> {
    Ok(RegionPlot {
        domain: evaluate_domain(&region.domain, env)?,
    })
}

/// 基本図形の点と，変換の並びを評価し，反復関数系(IFS)として展開する．最後に，オブジェクトの変換
/// (`transform`)を，展開した図形の点に施す．
fn compile_fractal(
    fractal: &Fractal,
    env: &Env,
    transform: &Transform,
) -> Result<FractalPlot, ErrorKind> {
    let mut base = fractal
        .base
        .iter()
        .map(|[x, y]| {
            let point = [
                evaluate_bound("base", x, 0, env)?,
                evaluate_bound("base", y, 1, env)?,
            ];
            if point.iter().all(|c| c.is_finite()) {
                Ok(point)
            } else {
                Err(ErrorKind::Invalid(
                    "基本図形(`base`)の点の座標は，有限の数にする．".to_owned(),
                ))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if fractal.closed
        && let Some(first) = base.first().copied()
    {
        base.push(first);
    }
    let no_maps = HashMap::new();
    let transforms = fractal
        .transforms
        .iter()
        .map(|steps| {
            compile_transform(steps, 2, env, &no_maps)?
                .as_affine()
                .ok_or_else(|| {
                    ErrorKind::Invalid(
                        "フラクタルの変換(`transforms`)には，写像(`map`)は使えない．".to_owned(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let paths = fractal::instances(&base, &transforms, fractal.depth, fractal.all_depths)
        .into_iter()
        .map(|path| {
            path.into_iter()
                .filter_map(|point| transform.apply2(point))
                .collect()
        })
        .collect();
    Ok(FractalPlot { paths })
}

/// 多角形の頂点を評価する．正多角形は，底辺が水平になる向きに，中心から半径の所に頂点を置く．
fn compile_polygon(
    polygon: &Polygon,
    env: &Env,
    scope: &VectorScope,
) -> Result<PolygonPlot, ErrorKind> {
    let vertices = if let Some(sides) = polygon.sides {
        let center = polygon.center.as_deref().unwrap_or_default();
        let [cx, cy] = evaluate_numbers("center", center, env)?[..] else {
            return Err(ErrorKind::Invalid(
                "正多角形の中心(`center`)は，2個の数で書く．".to_owned(),
            ));
        };
        let radius = polygon
            .radius
            .as_ref()
            .map(|bound| evaluate_numbers("radius", std::slice::from_ref(bound), env))
            .transpose()?
            .and_then(|values| values.first().copied())
            .unwrap_or_default();
        if radius <= 0.0 {
            return Err(ErrorKind::Invalid(
                "正多角形の半径(`radius`)は，正の数にする．".to_owned(),
            ));
        }
        let n = f64::from(u32::try_from(sides).unwrap_or(u32::MAX));
        // 最初の頂点を，底辺の右端に置く．
        let start = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI / n;
        (0..sides)
            .map(|k| {
                let angle = start
                    + std::f64::consts::TAU * f64::from(u32::try_from(k).unwrap_or(u32::MAX)) / n;
                [cx + radius * angle.cos(), cy + radius * angle.sin()]
            })
            .collect()
    } else {
        polygon
            .vertices
            .iter()
            .map(
                |vertex| match evaluate_position(vertex, env, scope)?.as_slice() {
                    [x, y] => Ok([*x, *y]),
                    _ => Err(ErrorKind::Invalid(
                        "多角形の頂点(`vertices`)は，2個の座標か，点の式で書く．".to_owned(),
                    )),
                },
            )
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(PolygonPlot { vertices })
}

/// テイラー展開の中心と，描く範囲を評価する．係数は，グラフを読んだあとに求める
/// (`expand_taylor_polynomials`)．範囲を書かなければ，グラフの定義域を使う．
fn compile_taylor(taylor: &Taylor, env: &Env) -> Result<TaylorPlot, ErrorKind> {
    let at = evaluate_numbers("at", std::slice::from_ref(&taylor.at), env)?
        .first()
        .copied()
        .unwrap_or_default();
    let domain = match &taylor.domain {
        Some(domain) => evaluate_domain(domain, env)?,
        None => [f64::NAN; 2],
    };
    Ok(TaylorPlot {
        at,
        domain,
        coefficients: Vec::new(),
    })
}

/// テイラー展開の係数を，展開するグラフの式から求める．グラフは，展開よりあとに置いてもよい．
fn expand_taylor_polynomials(
    scene: &Scene,
    plots: &mut [Plot],
    parameters: &[f64],
) -> Result<(), Error> {
    let graphs: HashMap<&str, (Expr, [f64; 2])> = scene
        .objects
        .iter()
        .zip(plots.iter())
        .filter_map(|(object, plot)| match (object, plot) {
            (Object::Graph(graph), Plot::Graph(placed)) => {
                Some((graph.id.as_str(), (placed.expr.clone(), placed.domain)))
            }
            _ => None,
        })
        .collect();
    for (object, plot) in scene.objects.iter().zip(plots.iter_mut()) {
        let (Object::Taylor(taylor), Plot::Taylor(placed)) = (object, plot) else {
            continue;
        };
        let fail = |kind| Error::in_object(&taylor.id, kind);
        let (expr, domain) = graphs
            .get(taylor.of.as_str())
            .ok_or_else(|| fail(ErrorKind::UnknownGraph(taylor.of.clone())))?;
        if placed.domain.iter().any(|end| end.is_nan()) {
            placed.domain = *domain;
        }
        let values: Vec<f64> = std::iter::once(placed.at)
            .chain(parameters.iter().copied())
            .collect();
        placed.coefficients = expr.taylor(0, &values, taylor.order).ok_or_else(|| {
            fail(ErrorKind::Invalid(format!(
                "グラフ「{}」の式は，{}のまわりでテイラー展開できない(特殊関数を含むか，その点で微分できない)．",
                taylor.of, placed.at
            )))
        })?;
    }
    Ok(())
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
fn evaluate_coordinates(bounds: &[Bound], env: &Env) -> Result<Vec<f64>, ErrorKind> {
    bounds
        .iter()
        .enumerate()
        .map(|(index, bound)| {
            let value = evaluate_bound("at", bound, index, env)?;
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

/// 位置を評価する．座標の並びは，各座標を評価し，点の式は，成分ごとに評価する．
fn evaluate_position(
    position: &Position,
    env: &Env,
    scope: &VectorScope,
) -> Result<Vec<f64>, ErrorKind> {
    match position {
        Position::Coordinates(list) => evaluate_coordinates(list, env),
        Position::Vector(source) => evaluate_vector(source, scope),
    }
}

/// 点の式を評価する．点を，原点からの位置ベクトルとして扱い，成分ごとに評価する．
/// 和と差と数倍しか許さないので，成分ごとに評価した結果は，そのままベクトルの計算になる．
fn evaluate_vector(source: &str, scope: &VectorScope) -> Result<Vec<f64>, ErrorKind> {
    let parameter_count = scope.parameter_names.len();
    let names: Vec<&str> = scope
        .parameter_names
        .iter()
        .copied()
        .chain(scope.points.iter().map(|(id, _)| *id))
        .collect();
    let expr = compile_expr("at", 0, source, &names, scope.functions)?;
    if expr.point_kind(&|index| index >= parameter_count) != Some(true) {
        return Err(ErrorKind::Invalid(
            "位置の式は，点の`id`を，和と差と数の倍でつないだ，点の式で書く(点どうしの積，点への数の足し引き，点を関数やべき乗に入れる式，点を含まない式は書けない)．"
                .to_owned(),
        ));
    }
    let component = |axis: usize| -> f64 {
        let values: Vec<f64> = scope
            .parameter_values
            .iter()
            .copied()
            .chain(
                scope
                    .points
                    .iter()
                    .map(|(_, at)| at.get(axis).copied().unwrap_or(f64::NAN)),
            )
            .collect();
        expr.eval(&values)
    };
    let at: Vec<f64> = (0..scope.dimension).map(component).collect();
    if at.iter().all(|value| value.is_finite()) {
        Ok(at)
    } else {
        Err(ErrorKind::Invalid(
            "座標(`at`)は，有限の数にする．".to_owned(),
        ))
    }
}

fn compile_label(label: &Label, env: &Env, scope: &VectorScope) -> Result<LabelPlot, ErrorKind> {
    Ok(LabelPlot {
        at: evaluate_position(&label.at, env, scope)?,
    })
}

fn compile_point(
    point: &Point,
    env: &Env,
    scope: &VectorScope,
    transform: &Transform,
) -> Result<PointPlot, ErrorKind> {
    let at = evaluate_position(&point.at, env, scope)?;
    if at.len() == scope.dimension {
        let at = transform.apply(&at).ok_or_else(|| {
            ErrorKind::Invalid("変換した点の座標が，有限の数にならない．".to_owned())
        })?;
        Ok(PointPlot { at })
    } else {
        Err(ErrorKind::Invalid(format!(
            "点の座標(`at`)は，{}個の数で書く．",
            scope.dimension
        )))
    }
}

/// 格子の刻みを評価し，正の有限の数で，線が多すぎないことを確かめる．
fn compile_grid(grid: &Grid, env: &Env, view: &View) -> Result<GridPlot, ErrorKind> {
    let (x_view, y_view) = match view {
        View::Plane(plane) => (Some(plane.x), Some(plane.y)),
        View::Space(_) => (None, None),
    };
    let (Some(x_range), Some(y_range)) = (grid.x_range.or(x_view), grid.y_range.or(y_view)) else {
        return Err(ErrorKind::Invalid(
            "空間の図の格子には，線を引く範囲(`x_range`と`y_range`)が要る．".to_owned(),
        ));
    };
    let step = |field: &'static str, bound: &Option<Bound>, range: [f64; 2]| {
        let Some(bound) = bound else {
            return Ok(None);
        };
        let value = evaluate_bound(field, bound, 0, env)?;
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
        x_range,
        y_range,
    })
}

/// 目盛の位置を評価し，軸の範囲の中にあることを確かめる．範囲を省いた平面の軸は，見える範囲である．
fn compile_axis(axis: &Axis, env: &Env, view: &View) -> Result<AxisPlot, ErrorKind> {
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
            let at = evaluate_bound("ticks", &tick.at, index, env)?;
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
fn evaluate_domain(domain: &[Bound; 2], env: &Env) -> Result<[f64; 2], ErrorKind> {
    let [low, high] = domain;
    let low = evaluate_bound("domain", low, 0, env)?;
    let high = evaluate_bound("domain", high, 1, env)?;
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
    env: &Env,
) -> Result<f64, ErrorKind> {
    match bound {
        Bound::Number(value) => Ok(*value),
        Bound::Expression(source) => {
            Ok(compile_expr(field, index, source, env.names, env.functions)?.eval(env.parameters))
        }
    }
}
