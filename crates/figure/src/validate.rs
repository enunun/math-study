//! 読み込んだシーンの検査．型では表せない条件を確かめる．

use std::collections::HashSet;

use crate::compile::compile;
use crate::error::{Error, ErrorKind};
use crate::image::{expand_images, transform_of};
use crate::scene::{
    Axis, Bound, CM_PER_PT, Complex, Curve, Cut, Direction, Fill, Fractal, FunctionDef, Graph,
    Grid, Intersection, Label, MAX_TRANSFORM_STEPS, MAX_WIDTH_PT, Map, Object, Point, Polygon,
    Position, Region, Scene, SpaceView, Sphere, Style, Surface, TangentPlane, Taylor,
    TransformStep, View,
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
        if let Object::Image(image) = object {
            check_transform_steps(&image.transform, true)
                .map_err(|kind| Error::in_object(id, kind))?;
        }
    }
    // 像は，元のオブジェクトの複製に置き換えてから，ほかのオブジェクトと同じに確かめる．
    let scene = expand_images(scene)?;
    let ids_of = |wanted: fn(&Object) -> bool| -> HashSet<&str> {
        scene
            .objects
            .iter()
            .filter(|object| wanted(object))
            .map(Object::id)
            .collect()
    };
    let points = ids_of(|object| matches!(object, Object::Point(_)));
    let graphs = ids_of(|object| matches!(object, Object::Graph(_)));
    let transformed_graphs =
        ids_of(|object| matches!(object, Object::Graph(graph) if !graph.transform.is_empty()));
    let curves = ids_of(|object| matches!(object, Object::Curve(_)));
    let surfaces = ids_of(|object| matches!(object, Object::Surface(_)));
    for object in &scene.objects {
        let id = object.id();
        let fail = |kind| Error::in_object(id, kind);
        validate_object(object, &scene.view).map_err(fail)?;
        check_object_transform(object).map_err(fail)?;
        check_endpoints(object, &points).map_err(fail)?;
        check_graphs(object, &graphs, &transformed_graphs).map_err(fail)?;
        check_surface(object, &surfaces).map_err(fail)?;
        check_tangent_target(object, &graphs, &curves).map_err(fail)?;
    }
    // 式の構文，名前，定義域は，型では確かめられないので，式を読んで確かめる．
    compile(&scene).map(drop)
}

/// 変換(`transform`)の手順の数と，写像を使えるかを確かめる．線分・ベクトル・複体・正多面体は，
/// 直線や平らな面が曲がらないように，アフィン変換(写像以外)だけを使える．
fn check_object_transform(object: &Object) -> Result<(), ErrorKind> {
    let Some(steps) = transform_of(object) else {
        return Ok(());
    };
    let allows_map = !matches!(
        object,
        Object::Segment(_) | Object::Vector(_) | Object::Complex(_) | Object::Polyhedron(_)
    );
    check_transform_steps(steps, allows_map)
}

fn check_transform_steps(steps: &[TransformStep], allows_map: bool) -> Result<(), ErrorKind> {
    if steps.len() > MAX_TRANSFORM_STEPS {
        return Err(ErrorKind::Invalid(format!(
            "変換(`transform`)の手順は，{MAX_TRANSFORM_STEPS}個以下にする．"
        )));
    }
    if !allows_map && steps.iter().any(|step| step.map.is_some()) {
        return Err(ErrorKind::Invalid(
            "線分・ベクトル・複体・正多面体の変換には，写像(`map`)は使えない(直線が曲がるため)．\
             写像で写すときは，曲線(`curve`)で書く．"
                .to_owned(),
        ));
    }
    Ok(())
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
        // 接する対象(`of`)の存在は，`check_tangent_target`が確かめる．接する点(`at`)は，
        // 数か式(`Bound`)なので，ここで確かめることはない．
        Object::TangentLine(_) => plane_only("tangent_line", view),
        Object::Sphere(sphere) => space_only("sphere", view).and_then(|()| validate_sphere(sphere)),
        Object::Grid(grid) => validate_grid(grid),
        Object::Point(point) => validate_point(point, view),
        Object::Region(region) => plane_only("region", view).and_then(|()| validate_region(region)),
        Object::Fractal(fractal) => {
            plane_only("fractal", view).and_then(|()| validate_fractal(fractal))
        }
        Object::Surface(surface) => {
            space_only("surface", view).and_then(|()| validate_surface(surface))
        }
        Object::Cut(cut) => space_only("cut", view).and_then(|()| validate_cut(cut)),
        Object::Intersection(found) => {
            space_only("intersection", view).and_then(|()| validate_intersection(found))
        }
        Object::TangentPlane(tangent) => {
            space_only("tangent_plane", view).and_then(|()| validate_tangent_plane(tangent))
        }
        Object::Complex(complex) => {
            space_only("complex", view).and_then(|()| validate_complex(complex))
        }
        Object::Polyhedron(polyhedron) => space_only("polyhedron", view).and_then(|()| {
            if polyhedron.radius.is_finite() && polyhedron.radius > 0.0 {
                Ok(())
            } else {
                Err(ErrorKind::Invalid(
                    "正多面体の半径(`radius`)は，正の有限の数にする．".to_owned(),
                ))
            }
        }),
        Object::Polygon(polygon) => {
            plane_only("polygon", view).and_then(|()| validate_polygon(polygon))
        }
        Object::Function(function) => validate_function(function),
        Object::Map(map) => validate_map(map),
        Object::Taylor(taylor) => plane_only("taylor", view).and_then(|()| validate_taylor(taylor)),
        // 像は，検査の前に，元のオブジェクトの複製に置き換えてある．
        Object::Image(_) | Object::Vector(_) | Object::Segment(_) | Object::Parameter(_) => Ok(()),
    }
}

fn style_of(object: &Object) -> Option<&Style> {
    match object {
        Object::Axis(o) => Some(&o.style),
        Object::Graph(o) => Some(&o.style),
        Object::Curve(o) => Some(&o.style),
        Object::TangentLine(o) => Some(&o.style),
        Object::Sphere(o) => Some(&o.style),
        Object::Grid(o) => Some(&o.style),
        Object::Point(o) => Some(&o.style),
        Object::Vector(o) => Some(&o.style),
        Object::Segment(o) => Some(&o.style),
        Object::Region(o) => Some(&o.style),
        Object::Fractal(o) => Some(&o.style),
        Object::Surface(o) => Some(&o.style),
        Object::Cut(o) => Some(&o.style),
        Object::Intersection(o) => Some(&o.style),
        Object::TangentPlane(o) => Some(&o.style),
        Object::Complex(o) => Some(&o.style),
        Object::Polyhedron(o) => Some(&o.style),
        Object::Polygon(o) => Some(&o.style),
        Object::Image(o) => Some(&o.style),
        Object::Taylor(o) => Some(&o.style),
        Object::Label(_) | Object::Parameter(_) | Object::Function(_) | Object::Map(_) => None,
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
    if let Some(style) = &surface.wireframe {
        check_style(style)?;
    }
    // 式で書いた刻みは，評価してから`compile.rs`で確かめる．
    if surface
        .wireframe_step
        .iter()
        .flatten()
        .any(|step| matches!(step, Bound::Number(value) if !(value.is_finite() && *value > 0.0)))
    {
        return Err(ErrorKind::Invalid(
            "ワイヤーフレームの刻み(`wireframe_step`)は，正の有限の数にする．".to_owned(),
        ));
    }
    match (&surface.control_net, &surface.bezier) {
        (Some(style), Some(_)) => check_style(style),
        (Some(_), None) => Err(ErrorKind::Invalid(
            "制御点の網(`control_net`)は，ベジエ曲面(`bezier`)にだけ使える．".to_owned(),
        )),
        (None, _) => Ok(()),
    }
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
        Object::TangentPlane(tangent) => vec![&tangent.of],
        _ => return Ok(()),
    };
    for name in names {
        if !surfaces.contains(name.as_str()) {
            return Err(ErrorKind::UnknownSurface(name.clone()));
        }
    }
    Ok(())
}

/// 接線が接する対象は，`graph`か`curve`オブジェクトの`id`でなければならない．
fn check_tangent_target(
    object: &Object,
    graphs: &HashSet<&str>,
    curves: &HashSet<&str>,
) -> Result<(), ErrorKind> {
    let Object::TangentLine(tangent) = object else {
        return Ok(());
    };
    if graphs.contains(tangent.of.as_str()) || curves.contains(tangent.of.as_str()) {
        Ok(())
    } else {
        Err(ErrorKind::UnknownTangentTarget(tangent.of.clone()))
    }
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
    check_fill(region.fill.as_ref())?;
    check_domain(&region.domain)
}

/// 各変換の中身(数や式)は，`compile.rs`が確かめる．
fn validate_fractal(fractal: &Fractal) -> Result<(), ErrorKind> {
    if fractal.base.len() < Fractal::MIN_BASE_POINTS {
        return Err(ErrorKind::Invalid(format!(
            "基本図形の点(`base`)は，{}点以上並べる．",
            Fractal::MIN_BASE_POINTS
        )));
    }
    if fractal.base.len() > Fractal::MAX_BASE_POINTS {
        return Err(ErrorKind::Invalid(format!(
            "基本図形の点(`base`)は，{}点以下にする．",
            Fractal::MAX_BASE_POINTS
        )));
    }
    if fractal.transforms.is_empty() {
        return Err(ErrorKind::Invalid(
            "`transforms`には，変換を1つ以上書く．".to_owned(),
        ));
    }
    if fractal.transforms.len() > Fractal::MAX_TRANSFORMS {
        return Err(ErrorKind::Invalid(format!(
            "`transforms`の変換は，{}個以下にする．",
            Fractal::MAX_TRANSFORMS
        )));
    }
    if fractal.depth > Fractal::MAX_DEPTH {
        return Err(ErrorKind::Invalid(format!(
            "`depth`(再帰の深さ)は，{}以下にする．",
            Fractal::MAX_DEPTH
        )));
    }
    // 描く図形の数は，深さ`depth`だけなら変換の数の`depth`乗，`all_depths`なら，0乗からの和である．
    let first_depth = if fractal.all_depths { 0 } else { fractal.depth };
    let instances = (first_depth..=fractal.depth).try_fold(0_usize, |sum, depth| {
        sum.checked_add(fractal.transforms.len().checked_pow(depth)?)
    });
    if instances.is_none_or(|count| count > Fractal::MAX_INSTANCES) {
        return Err(ErrorKind::Invalid(format!(
            "フラクタルの図形の数(変換の数の{}乗)が多すぎる．`depth`を減らすか，\
             `transforms`を見直す．",
            fractal.depth
        )));
    }
    Ok(())
}

/// 領域が挟むグラフと，テイラー展開するグラフは，`graph`オブジェクトの`id`でなければならない．
/// 変換したグラフは，関数のグラフではなくなるので，使えない．
fn check_graphs(
    object: &Object,
    graphs: &HashSet<&str>,
    transformed: &HashSet<&str>,
) -> Result<(), ErrorKind> {
    let names: Vec<&String> = match object {
        Object::Region(region) => region.between.iter().collect(),
        Object::Taylor(taylor) => vec![&taylor.of],
        _ => return Ok(()),
    };
    for name in names {
        if !graphs.contains(name.as_str()) {
            return Err(ErrorKind::UnknownGraph(name.clone()));
        }
        if transformed.contains(name.as_str()) {
            return Err(ErrorKind::Invalid(format!(
                "グラフ「{name}」は変換(`transform`)してあるので，領域やテイラー展開には使えない．"
            )));
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
    for (field, range) in [("x_range", grid.x_range), ("y_range", grid.y_range)] {
        if let Some(range) = range
            && !is_increasing(range)
        {
            return Err(ErrorKind::InvalidRange(field));
        }
    }
    Ok(())
}

/// 塗りの不透明度は，0より大きく1以下である．
fn check_fill(fill: Option<&Fill>) -> Result<(), ErrorKind> {
    match fill {
        Some(fill) if !(fill.opacity > 0.0 && fill.opacity <= 1.0) => Err(ErrorKind::Invalid(
            "塗りの不透明度(`opacity`)は，0より大きく1以下にする．".to_owned(),
        )),
        _ => Ok(()),
    }
}

/// 多角形は，正多角形の形(`sides`，`center`，`radius`)か，頂点の並び(`vertices`)の，どちらか一方で書く．
fn validate_polygon(polygon: &Polygon) -> Result<(), ErrorKind> {
    check_fill(polygon.fill.as_ref())?;
    let range = Polygon::MIN_VERTICES..=Polygon::MAX_VERTICES;
    let regular = polygon.sides.is_some() || polygon.center.is_some() || polygon.radius.is_some();
    match (regular, polygon.vertices.is_empty()) {
        (true, true) => {
            let sides = polygon.sides.unwrap_or_default();
            if !range.contains(&sides) {
                return Err(ErrorKind::Invalid(format!(
                    "正多角形の辺の数(`sides`)は，{}以上{}以下にする．",
                    Polygon::MIN_VERTICES,
                    Polygon::MAX_VERTICES
                )));
            }
            if polygon
                .center
                .as_ref()
                .is_none_or(|center| center.len() != 2)
                || polygon.radius.is_none()
            {
                return Err(ErrorKind::Invalid(
                    "正多角形には，中心(`center`，2個の数)と半径(`radius`)を書く．".to_owned(),
                ));
            }
            Ok(())
        }
        (false, false) => {
            if range.contains(&polygon.vertices.len()) {
                Ok(())
            } else {
                Err(ErrorKind::Invalid(format!(
                    "多角形の頂点(`vertices`)は，{}個以上{}個以下にする．",
                    Polygon::MIN_VERTICES,
                    Polygon::MAX_VERTICES
                )))
            }
        }
        _ => Err(ErrorKind::Invalid(
            "多角形は，正多角形(`sides`，`center`，`radius`)か，頂点の並び(`vertices`)の，\
             どちらか一方で書く．"
                .to_owned(),
        )),
    }
}

fn validate_function(function: &FunctionDef) -> Result<(), ErrorKind> {
    if !(1..=FunctionDef::MAX_VARS).contains(&function.vars.len()) {
        return Err(ErrorKind::Invalid(format!(
            "関数の引数(`vars`)は，1個以上{}個以下にする．",
            FunctionDef::MAX_VARS
        )));
    }
    for var in &function.vars {
        check_variable(var)?;
    }
    non_empty("expr", &function.expr)
}

fn validate_map(map: &Map) -> Result<(), ErrorKind> {
    for var in &map.vars {
        check_variable(var)?;
    }
    for expr in &map.expr {
        non_empty("expr", expr)?;
    }
    Ok(())
}

fn validate_taylor(taylor: &Taylor) -> Result<(), ErrorKind> {
    if taylor.order > Taylor::MAX_ORDER {
        return Err(ErrorKind::Invalid(format!(
            "テイラー展開の次数(`order`)は，{}以下にする．",
            Taylor::MAX_ORDER
        )));
    }
    match &taylor.domain {
        Some(domain) => check_domain(domain),
        None => Ok(()),
    }
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
    if !(sphere.radius.is_finite() && sphere.radius > 0.0) {
        return Err(ErrorKind::Invalid(
            "`radius`は，正の有限の数にする．".to_owned(),
        ));
    }
    match &sphere.wireframe {
        Some(style) => check_style(style),
        None => Ok(()),
    }
}

/// 接平面の検査．接する曲面(`of`)の存在は，`check_surface`が確かめる．
fn validate_tangent_plane(tangent: &TangentPlane) -> Result<(), ErrorKind> {
    if let Bound::Number(value) = &tangent.size
        && !(value.is_finite() && *value > 0.0)
    {
        return Err(ErrorKind::Invalid(
            "接平面の半径(`size`)は，正の有限の数にする．".to_owned(),
        ));
    }
    Ok(())
}

fn validate_complex(complex: &Complex) -> Result<(), ErrorKind> {
    let count = complex.vertices.len();
    if !(3..=Complex::MAX_VERTICES).contains(&count) {
        return Err(ErrorKind::Invalid(format!(
            "`vertices`(頂点)は，3個以上{}個以下にする．",
            Complex::MAX_VERTICES
        )));
    }
    if complex.faces.is_empty() || complex.faces.len() > Complex::MAX_FACES {
        return Err(ErrorKind::Invalid(format!(
            "`faces`(面)は，1個以上{}個以下にする．",
            Complex::MAX_FACES
        )));
    }
    for face in &complex.faces {
        if !(3..=Complex::MAX_FACE_VERTICES).contains(&face.len()) {
            return Err(ErrorKind::Invalid(format!(
                "面(`faces`の要素)の頂点は，3個以上{}個以下にする．",
                Complex::MAX_FACE_VERTICES
            )));
        }
        let unique: HashSet<usize> = face.iter().copied().collect();
        if unique.len() != face.len() || face.iter().any(|&index| index >= count) {
            return Err(ErrorKind::Invalid(
                "面(`faces`の要素)は，`vertices`の番号(0始まり)を，重ならないように並べる．"
                    .to_owned(),
            ));
        }
    }
    Ok(())
}

/// 曲線は，式(`var`，`expr`，`domain`)か，ベジエ曲線の制御点(`bezier`)か，
/// スプライン曲線が通る点(`spline`)の，どれか1つで書く．
const CURVE_FORM_HINT: &str = "曲線は，式(`var`，`expr`，`domain`)か，ベジエ曲線の制御点(`bezier`)か，\
     スプライン曲線の点(`spline`)で書く．";

fn validate_curve(curve: &Curve, view: &View) -> Result<(), ErrorKind> {
    match (&curve.bezier, &curve.spline) {
        (Some(net), None) => validate_curve_points(curve, net, view, "ベジエ曲線", "bezier"),
        (None, Some(points)) => {
            validate_curve_points(curve, points, view, "スプライン曲線", "spline")
        }
        (Some(_), Some(_)) => Err(ErrorKind::Invalid(
            "ベジエ曲線(`bezier`)とスプライン曲線(`spline`)は，同時に書けない．".to_owned(),
        )),
        (None, None) => validate_curve_formula(curve, view),
    }
}

/// 曲線を式で書く形の検査．
fn validate_curve_formula(curve: &Curve, view: &View) -> Result<(), ErrorKind> {
    let Some(var) = &curve.var else {
        return Err(ErrorKind::Invalid(CURVE_FORM_HINT.to_owned()));
    };
    check_variable(var)?;
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
    let Some(domain) = &curve.domain else {
        return Err(ErrorKind::Invalid(CURVE_FORM_HINT.to_owned()));
    };
    check_domain(domain)
}

/// ベジエ曲線・スプライン曲線に共通の検査．点(制御点か，通る点)は，2点以上12点以下を，
/// 平面なら2個，空間なら3個の座標で並べる．式とは同時に書けない．
fn validate_curve_points(
    curve: &Curve,
    points: &[Vec<Bound>],
    view: &View,
    name: &str,
    field: &str,
) -> Result<(), ErrorKind> {
    if curve.var.is_some() || !curve.expr.is_empty() || curve.domain.is_some() {
        return Err(ErrorKind::Invalid(format!(
            "{name}(`{field}`)は，式(`var`，`expr`，`domain`)と同時に書けない．"
        )));
    }
    if points.len() < Curve::MIN_CONTROL_POINTS {
        return Err(ErrorKind::Invalid(format!(
            "{name}の点は，{}点以上並べる．",
            Curve::MIN_CONTROL_POINTS
        )));
    }
    if points.len() > Curve::MAX_CONTROL_POINTS {
        return Err(ErrorKind::Invalid(format!(
            "{name}の点は，{}点以下にする．",
            Curve::MAX_CONTROL_POINTS
        )));
    }
    let expected = curve_expressions(view);
    for point in points {
        if point.len() != expected {
            return Err(ErrorKind::Invalid(format!(
                "{name}の点は，座標を{expected}個(平面ならx，y，空間ならx，y，z)で書く．"
            )));
        }
    }
    Ok(())
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
