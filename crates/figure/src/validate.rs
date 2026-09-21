//! 読み込んだシーンの検査．型では表せない条件を確かめる．

use std::collections::HashSet;

use crate::compile::compile;
use crate::error::{Error, ErrorKind};
use crate::scene::{
    Axis, Bound, CM_PER_PT, Curve, Cut, Direction, Graph, Grid, Intersection, Label, MAX_WIDTH_PT,
    Object, Point, Position, Region, Scene, SpaceView, Sphere, Style, Surface, View,
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
    let points: HashSet<&str> = scene
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Point(point) => Some(point.id.as_str()),
            _ => None,
        })
        .collect();
    let graphs: HashSet<&str> = scene
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Graph(graph) => Some(graph.id.as_str()),
            _ => None,
        })
        .collect();
    let surfaces: HashSet<&str> = scene
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Surface(surface) => Some(surface.id.as_str()),
            _ => None,
        })
        .collect();
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
        check_endpoints(object, &points).map_err(|kind| Error::in_object(id, kind))?;
        check_graphs(object, &graphs).map_err(|kind| Error::in_object(id, kind))?;
        check_surface(object, &surfaces).map_err(|kind| Error::in_object(id, kind))?;
    }
    // 式の構文，名前，定義域は，型では確かめられないので，式を読んで確かめる．
    compile(scene).map(drop)
}

fn validate_object(object: &Object, view: &View) -> Result<(), ErrorKind> {
    if let Some(style) = style_of(object) {
        check_style(style)?;
    }
    match object {
        Object::Axis(axis) => validate_axis(axis, view),
        Object::Label(label) => validate_label(label, view),
        Object::Graph(graph) => plane_only("graph", view).and_then(|()| validate_graph(graph)),
        Object::Curve(curve) => validate_curve(curve, view),
        Object::Sphere(sphere) => space_only("sphere", view).and_then(|()| validate_sphere(sphere)),
        Object::Grid(grid) => plane_only("grid", view).and_then(|()| validate_grid(grid)),
        Object::Point(point) => validate_point(point, view),
        Object::Region(region) => plane_only("region", view).and_then(|()| validate_region(region)),
        Object::Surface(surface) => {
            space_only("surface", view).and_then(|()| validate_surface(surface))
        }
        Object::Cut(cut) => space_only("cut", view).and_then(|()| validate_cut(cut)),
        Object::Intersection(found) => {
            space_only("intersection", view).and_then(|()| validate_intersection(found))
        }
        Object::Vector(_) | Object::Segment(_) | Object::Parameter(_) => Ok(()),
    }
}

fn style_of(object: &Object) -> Option<&Style> {
    match object {
        Object::Axis(o) => Some(&o.style),
        Object::Graph(o) => Some(&o.style),
        Object::Curve(o) => Some(&o.style),
        Object::Sphere(o) => Some(&o.style),
        Object::Grid(o) => Some(&o.style),
        Object::Point(o) => Some(&o.style),
        Object::Vector(o) => Some(&o.style),
        Object::Segment(o) => Some(&o.style),
        Object::Region(o) => Some(&o.style),
        Object::Surface(o) => Some(&o.style),
        Object::Cut(o) => Some(&o.style),
        Object::Intersection(o) => Some(&o.style),
        Object::Label(_) | Object::Parameter(_) => None,
    }
}

/// 線の太さは，0より大きく，上限以下である(0以下の長さは，読み込みで断られる)．
fn check_style(style: &Style) -> Result<(), ErrorKind> {
    match style.width {
        Some(width) if width.to_cm() / CM_PER_PT > MAX_WIDTH_PT + 1e-9 => Err(ErrorKind::Invalid(
            format!("`width`は，0より大きく，{MAX_WIDTH_PT}pt以下にする．"),
        )),
        _ => Ok(()),
    }
}

fn validate_point(point: &Point, view: &View) -> Result<(), ErrorKind> {
    let (kind, expected) = match view {
        View::Plane(_) => ("平面", 2),
        View::Space(_) => ("空間", 3),
    };
    if let Position::Coordinates(list) = &point.at
        && list.len() != expected
    {
        return Err(ErrorKind::Invalid(format!(
            "点の座標(`at`)は，{kind}の図では{expected}個の数で書く．"
        )));
    }
    match &point.label {
        Some(label) => non_empty("label", label),
        None => Ok(()),
    }
}

/// ベクトルと線分の端は，`point`オブジェクトの`id`でなければならない．
fn check_endpoints(object: &Object, points: &HashSet<&str>) -> Result<(), ErrorKind> {
    let (from, to) = match object {
        Object::Vector(vector) => (&vector.from, &vector.to),
        Object::Segment(segment) => (&segment.from, &segment.to),
        _ => return Ok(()),
    };
    for name in [from, to] {
        if !points.contains(name.as_str()) {
            return Err(ErrorKind::UnknownPoint(name.clone()));
        }
    }
    Ok(())
}

fn validate_surface(surface: &Surface) -> Result<(), ErrorKind> {
    match &surface.bezier {
        Some(net) => validate_bezier(surface, net)?,
        None => validate_formula(surface)?,
    }
    if surface
        .mesh
        .iter()
        .any(|count| !(Surface::MIN_MESH..=Surface::MAX_MESH).contains(count))
    {
        return Err(ErrorKind::Invalid(format!(
            "網の細かさ(`mesh`)は，各方向とも，{}以上{}以下にする．",
            Surface::MIN_MESH,
            Surface::MAX_MESH
        )));
    }
    Ok(())
}

/// 式で書いた曲面．変数，式，定義域が要る．
fn validate_formula(surface: &Surface) -> Result<(), ErrorKind> {
    let Some(domain) = &surface.domain else {
        return Err(ErrorKind::Invalid(
            "曲面は，式(`vars`，`expr`，`domain`)か，ベジエ曲面の制御点の網(`bezier`)で書く．"
                .to_owned(),
        ));
    };
    let [first, second] = surface.vars.as_slice() else {
        return Err(ErrorKind::Invalid(
            "曲面の変数(`vars`)は，2つの名前で書く．".to_owned(),
        ));
    };
    check_variable(first)?;
    check_variable(second)?;
    if first == second {
        return Err(ErrorKind::NameConflict(first.clone()));
    }
    if surface.expr.len() != 3 {
        return Err(ErrorKind::ExpressionCount {
            expected: 3,
            found: surface.expr.len(),
        });
    }
    for expr in &surface.expr {
        non_empty("expr", expr)?;
    }
    for domain in domain {
        check_domain(domain)?;
    }
    Ok(())
}

/// ベジエ曲面．制御点の網は，各方向に2点以上を，行の長さを揃えて並べ，各点は3つの座標を持つ．
fn validate_bezier(surface: &Surface, net: &[Vec<Vec<Bound>>]) -> Result<(), ErrorKind> {
    if !surface.vars.is_empty() || !surface.expr.is_empty() || surface.domain.is_some() {
        return Err(ErrorKind::Invalid(
            "ベジエ曲面(`bezier`)は，式(`vars`，`expr`，`domain`)と同時に書けない．".to_owned(),
        ));
    }
    let width = net.first().map_or(0, Vec::len);
    if net.len() < Surface::MIN_CONTROL_POINTS || width < Surface::MIN_CONTROL_POINTS {
        return Err(ErrorKind::Invalid(format!(
            "ベジエ曲面の制御点は，各方向に{}点以上を並べる．",
            Surface::MIN_CONTROL_POINTS
        )));
    }
    if net.len() > Surface::MAX_CONTROL_POINTS || width > Surface::MAX_CONTROL_POINTS {
        return Err(ErrorKind::Invalid(format!(
            "ベジエ曲面の制御点は，各方向に{}点以下にする．",
            Surface::MAX_CONTROL_POINTS
        )));
    }
    for row in net {
        if row.len() != width {
            return Err(ErrorKind::Invalid(
                "ベジエ曲面の制御点の行は，長さを揃える．".to_owned(),
            ));
        }
        for point in row {
            if point.len() != 3 {
                return Err(ErrorKind::Invalid(
                    "ベジエ曲面の制御点は，x，y，zの3つの座標で書く．".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_cut(cut: &Cut) -> Result<(), ErrorKind> {
    if cut.normal.len() == 3 {
        Ok(())
    } else {
        Err(ErrorKind::Invalid(
            "平面の法線(`normal`)は，3個の数か式で書く．".to_owned(),
        ))
    }
}

/// 切り口が切る曲面は，`surface`オブジェクトの`id`でなければならない．
fn check_surface(object: &Object, surfaces: &HashSet<&str>) -> Result<(), ErrorKind> {
    let names: Vec<&String> = match object {
        Object::Cut(cut) => vec![&cut.surface],
        Object::Intersection(found) => found.surfaces.iter().collect(),
        _ => return Ok(()),
    };
    for name in names {
        if !surfaces.contains(name.as_str()) {
            return Err(ErrorKind::UnknownSurface(name.clone()));
        }
    }
    Ok(())
}

fn validate_intersection(found: &Intersection) -> Result<(), ErrorKind> {
    match found.surfaces.as_slice() {
        [first, second] if first != second => Ok(()),
        _ => Err(ErrorKind::Invalid(
            "交線の曲面(`surfaces`)は，違う2つの曲面の`id`で書く．".to_owned(),
        )),
    }
}

fn validate_region(region: &Region) -> Result<(), ErrorKind> {
    if !(1..=2).contains(&region.between.len()) {
        return Err(ErrorKind::Invalid(
            "`between`には，グラフの`id`を1つか2つ書く．".to_owned(),
        ));
    }
    if let Some(fill) = &region.fill
        && !(fill.opacity > 0.0 && fill.opacity <= 1.0)
    {
        return Err(ErrorKind::Invalid(
            "塗りの不透明度(`opacity`)は，0より大きく1以下にする．".to_owned(),
        ));
    }
    check_domain(&region.domain)
}

/// 領域が挟むグラフは，`graph`オブジェクトの`id`でなければならない．
fn check_graphs(object: &Object, graphs: &HashSet<&str>) -> Result<(), ErrorKind> {
    let Object::Region(region) = object else {
        return Ok(());
    };
    for name in &region.between {
        if !graphs.contains(name.as_str()) {
            return Err(ErrorKind::UnknownGraph(name.clone()));
        }
    }
    Ok(())
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
    match &label.at {
        Position::Coordinates(list) if list.len() == expected => {}
        Position::Coordinates(_) => {
            return Err(ErrorKind::Invalid(format!(
                "ラベルの位置`at`は，{kind}の図では{expected}個の数で書く．"
            )));
        }
        Position::Vector(_) => {}
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
