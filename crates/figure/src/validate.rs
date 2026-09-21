//! 読み込んだシーンの検査．型では表せない条件を確かめる．

use std::collections::HashSet;

use crate::compile::compile;
use crate::error::{Error, ErrorKind};
use crate::scene::{
    Axis, Bound, Curve, Direction, Graph, Grid, Label, Object, Scene, SpaceView, Sphere, View,
};

/// 仰角の絶対値の上限(度)．
const MAX_ELEVATION: f64 = 90.0;

/// シーンを検査する．
///
/// # Errors
///
/// 図の説明が空，範囲が正しくない，`id`が識別子でないか重なっている，
/// 式や文字列が空，曲線の式の数が合わない，のいずれかで誤りを返す．
pub fn validate(scene: &Scene) -> Result<(), Error> {
    if is_blank(&scene.description) {
        return Err(Error::new(ErrorKind::EmptyText("description")));
    }
    check_view(&scene.view)?;
    let mut seen = HashSet::new();
    for object in &scene.objects {
        let id = object.id();
        if !is_identifier(id) {
            return Err(Error::in_object(id, ErrorKind::InvalidId(id.to_owned())));
        }
        if !seen.insert(id) {
            return Err(Error::in_object(id, ErrorKind::DuplicateId(id.to_owned())));
        }
        validate_object(object, &scene.view).map_err(|kind| Error::in_object(id, kind))?;
    }
    // 式の構文，名前，定義域は，型では確かめられないので，式を読んで確かめる．
    compile(scene).map(drop)
}

fn validate_object(object: &Object, view: &View) -> Result<(), ErrorKind> {
    match object {
        Object::Axis(axis) => validate_axis(axis, view),
        Object::Label(label) => validate_label(label, view),
        Object::Graph(graph) => plane_only("graph", view).and_then(|()| validate_graph(graph)),
        Object::Curve(curve) => validate_curve(curve, view),
        Object::Sphere(sphere) => space_only("sphere", view).and_then(|()| validate_sphere(sphere)),
        Object::Grid(grid) => plane_only("grid", view).and_then(|()| validate_grid(grid)),
        Object::Parameter(_) => Ok(()),
    }
}

fn validate_grid(grid: &Grid) -> Result<(), ErrorKind> {
    if grid.x_step.is_none() && grid.y_step.is_none() {
        return Err(ErrorKind::Invalid(
            "格子には，`x_step`か`y_step`の，少なくとも一方が必要である．".to_owned(),
        ));
    }
    for (field, step) in [("x_step", &grid.x_step), ("y_step", &grid.y_step)] {
        if let Some(Bound::Number(value)) = step
            && !(value.is_finite() && *value > 0.0)
        {
            return Err(ErrorKind::Invalid(format!(
                "`{field}`は，正の有限の数にする．"
            )));
        }
    }
    Ok(())
}

/// 平面の図でだけ使えるオブジェクトを，空間の図に置いていないか．
fn plane_only(type_name: &str, view: &View) -> Result<(), ErrorKind> {
    match view {
        View::Plane(_) => Ok(()),
        View::Space(_) => Err(ErrorKind::Invalid(format!(
            "「{type_name}」は，空間の図では使えない．平面の図(`view`に`x`，`y`)で使う．"
        ))),
    }
}

/// 空間の図でだけ使えるオブジェクトを，平面の図に置いていないか．
fn space_only(type_name: &str, view: &View) -> Result<(), ErrorKind> {
    match view {
        View::Space(_) => Ok(()),
        View::Plane(_) => Err(ErrorKind::Invalid(format!(
            "「{type_name}」は，平面の図では使えない．空間の図(`view`に`azimuth`，`elevation`)で使う．"
        ))),
    }
}

fn validate_axis(axis: &Axis, view: &View) -> Result<(), ErrorKind> {
    if matches!(view, View::Space(_)) && !axis.ticks.is_empty() {
        return Err(ErrorKind::Invalid(
            "空間の図の軸には，目盛(`ticks`)をまだ付けられない．".to_owned(),
        ));
    }
    for tick in &axis.ticks {
        if let Some(label) = &tick.label {
            non_empty("label", label)?;
        }
    }
    match (view, axis.direction, axis.range) {
        (View::Plane(_), Direction::Z, _) => Err(ErrorKind::Invalid(
            "z軸は，空間の図でだけ使える．平面の図(`view`に`x`，`y`)では，x軸とy軸を使う．"
                .to_owned(),
        )),
        (View::Space(_), _, None) => Err(ErrorKind::Invalid(
            "空間の図の軸には，`range`が必要である．".to_owned(),
        )),
        (_, _, Some(range)) if !is_increasing(range) => Err(ErrorKind::InvalidRange("range")),
        _ => Ok(()),
    }
}

fn validate_label(label: &Label, view: &View) -> Result<(), ErrorKind> {
    let (kind, expected) = match view {
        View::Plane(_) => ("平面", 2),
        View::Space(_) => ("空間", 3),
    };
    if label.at.len() != expected {
        return Err(ErrorKind::Invalid(format!(
            "ラベルの位置`at`は，{kind}の図では{expected}個の数で書く．"
        )));
    }
    non_empty("tex", &label.tex)
}

fn validate_graph(graph: &Graph) -> Result<(), ErrorKind> {
    check_variable(&graph.var)?;
    non_empty("expr", &graph.expr)?;
    check_domain(&graph.domain)
}

fn validate_sphere(sphere: &Sphere) -> Result<(), ErrorKind> {
    if sphere.radius.is_finite() && sphere.radius > 0.0 {
        Ok(())
    } else {
        Err(ErrorKind::Invalid(
            "`radius`は，正の有限の数にする．".to_owned(),
        ))
    }
}

fn validate_curve(curve: &Curve, view: &View) -> Result<(), ErrorKind> {
    check_variable(&curve.var)?;
    let expected = curve_expressions(view);
    if curve.expr.len() != expected {
        return Err(ErrorKind::ExpressionCount {
            expected,
            found: curve.expr.len(),
        });
    }
    for expr in &curve.expr {
        non_empty("expr", expr)?;
    }
    check_domain(&curve.domain)
}

/// 曲線の式の数．平面の曲線は2個，空間の曲線は3個である．
pub const fn curve_expressions(view: &View) -> usize {
    match view {
        View::Plane(_) => 2,
        View::Space(_) => 3,
    }
}

fn check_view(view: &View) -> Result<(), Error> {
    match view {
        View::Plane(plane) => {
            check_view_range("view.x", plane.x)?;
            check_view_range("view.y", plane.y)
        }
        View::Space(space) => check_space_view(space),
    }
}

fn check_space_view(view: &SpaceView) -> Result<(), Error> {
    let elevation_ok = view.elevation.is_finite() && view.elevation.abs() <= MAX_ELEVATION;
    if !elevation_ok {
        return Err(Error::new(ErrorKind::Invalid(
            "`elevation`は，-90以上90以下の度数で書く．".to_owned(),
        )));
    }
    if !view.azimuth.is_finite() {
        return Err(Error::new(ErrorKind::Invalid(
            "`azimuth`は，有限の度数で書く．".to_owned(),
        )));
    }
    Ok(())
}

fn check_view_range(field: &'static str, range: [f64; 2]) -> Result<(), Error> {
    if is_increasing(range) {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidRange(field)))
    }
}

/// 数で書かれた定義域だけを確かめる．式で書かれた端は，式を評価する段階で確かめる．
fn check_domain(domain: &[Bound; 2]) -> Result<(), ErrorKind> {
    match domain {
        [Bound::Number(low), Bound::Number(high)] if !is_increasing([*low, *high]) => {
            Err(ErrorKind::InvalidRange("domain"))
        }
        _ => Ok(()),
    }
}

fn check_variable(name: &str) -> Result<(), ErrorKind> {
    if is_identifier(name) {
        Ok(())
    } else {
        Err(ErrorKind::InvalidVariable(name.to_owned()))
    }
}

fn non_empty(field: &'static str, text: &str) -> Result<(), ErrorKind> {
    if is_blank(text) {
        Err(ErrorKind::EmptyText(field))
    } else {
        Ok(())
    }
}

fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn is_increasing([low, high]: [f64; 2]) -> bool {
    low.is_finite() && high.is_finite() && low < high
}

/// 英字で始まり，英数字と`_`だけでできた名前．式の中で，そのまま使える．
fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
