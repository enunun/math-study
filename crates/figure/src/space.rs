//! 空間の図を，描画の中間表現にする．
//!
//! 平行投影で，画面の右向きを`(-sin a, cos a, 0)`，上向きを`(-sin e cos a, -sin e sin a, cos e)`，
//! カメラへの向きを`(cos e cos a, cos e sin a, sin e)`とする(`a`は方位角，`e`は仰角)．
//! 球は，不透明な殻として，点を隠す．点から，カメラへの視線が，球に当たれば，その点は隠れている．
//! 線は，まず点で刻み，隠れ方が変わる区間を二分法で詰めて，隠れた部分と見える部分に分ける．
//! 刻みの間に，隠れ方が2回変わる細かい隠れは，見つけられない．

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::f64::consts::TAU;

use crate::bezier::{bezier_curve_point, bezier_point};
use crate::compile::{
    Compiled, CurvePlot, CutPlot, GridPlot, LabelPlot, LinkPlot, Plot, PointPlot, SurfacePlot,
    TangentPlanePlot,
};
use crate::derivative::central_difference_point;
use crate::figure::{Bounds, DotItem, Figure, Item, LabelItem, Path, Stroke};
use crate::render::{
    AXIS_WIDTH, CURVE_WIDTH, DOT_RADIUS, GRID_WIDTH, MARGIN, arrow_head, multiples, stroke_of,
    with_variable,
};
use crate::sample::{sample, sample_with_parameters};
use crate::scene::{
    Anchor, Arrow, Axis, Complex, Cut, Direction, Hidden, Intersection, Label, Line, Object, Point,
    Scene, SpaceView, Sphere, Style, Surface, TangentPlane,
};
use crate::spline::catmull_rom_point;
use crate::surface::{Frame, Mesh, Rim};
use crate::transform::Transform;

/// 空間の点．
type Point3 = [f64; 3];

/// 軸を刻む数．隠れ方が変わる区間を見つけるために使う．
const AXIS_STEPS: u16 = 64;
/// 隠れ方が変わる区間を詰める，二分法の回数．
const BISECTIONS: u32 = 60;
/// 球の面の上の点は，計算の誤差で，面の内側にも外側にもなる．隠す球の半径を，この割合だけ小さく見て，
/// 面の上の点を，外側にある点として扱う．
const SURFACE_MARGIN: f64 = 1e-12;
/// 軸の名前の向きを決める，8方向の境目の角度(度)．
const SECTOR: f64 = 22.5;
/// 球のワイヤーフレームの，経線の本数．
const SPHERE_MERIDIANS: usize = 6;
/// 球のワイヤーフレームの，緯線の本数．両極は含めない．
const SPHERE_PARALLELS: usize = 3;

fn lerp(low: f64, high: f64, t: f64) -> f64 {
    low + (high - low) * t
}

/// 2点`from`，`to`を結ぶ線分の，割合`t`の点．
fn lerp3(from: Point3, to: Point3, t: f64) -> Point3 {
    let [x0, y0, z0] = from;
    let [x1, y1, z1] = to;
    [lerp(x0, x1, t), lerp(y0, y1, t), lerp(z0, z1, t)]
}

/// `0`から`count + 1`等分した，内側の`count`個の位置(両端は含めない)．
fn interior_fractions(count: usize) -> impl Iterator<Item = f64> {
    let divisions = count.saturating_add(1);
    (1..=count).map(move |k| {
        f64::from(u32::try_from(k).unwrap_or(0)) / f64::from(u32::try_from(divisions).unwrap_or(1))
    })
}

/// 画面への投影．
struct Camera {
    right: Point3,
    up: Point3,
    toward: Point3,
    /// 1単位の実寸(cm)．
    unit: f64,
}

impl Camera {
    fn frame(&self) -> Frame {
        Frame {
            right: self.right,
            up: self.up,
            toward: self.toward,
        }
    }

    fn new(view: &SpaceView) -> Self {
        let (a, e) = (view.azimuth.to_radians(), view.elevation.to_radians());
        Self {
            right: [-a.sin(), a.cos(), 0.0],
            up: [-e.sin() * a.cos(), -e.sin() * a.sin(), e.cos()],
            toward: [e.cos() * a.cos(), e.cos() * a.sin(), e.sin()],
            unit: view.unit.to_cm(),
        }
    }

    /// 空間の点の，画面の位置(cm)．
    fn project(&self, point: Point3) -> [f64; 2] {
        [
            dot(point, self.right) * self.unit,
            dot(point, self.up) * self.unit,
        ]
    }
}

fn dot(a: Point3, b: Point3) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn minus(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Point3, s: f64) -> Point3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// 長さ1に直した向き．長さが0か有限でなければ，`None`．
fn normalized3(v: Point3) -> Option<Point3> {
    let length = dot(v, v).sqrt();
    (length.is_finite() && length > 0.0).then(|| scale(v, 1.0 / length))
}

/// 空間の図で，点を隠す球．
struct Ball {
    center: Point3,
    radius: f64,
}

impl Ball {
    /// 点から，カメラへの視線が，球に当たるか．球の内側の点も，隠れている．
    fn hides(&self, point: Point3, toward: Point3) -> bool {
        let offset = minus(point, self.center);
        let along = dot(offset, toward);
        let radius = self.radius * (1.0 - SURFACE_MARGIN);
        let discriminant = along * along - (dot(offset, offset) - radius * radius);
        discriminant > 0.0 && discriminant.sqrt() - along > 0.0
    }
}

/// 2つの変数から，曲面の点を返す，式の関数．
type SurfaceMap<'a> = Box<dyn Fn(f64, f64) -> Option<Point3> + 'a>;

/// 図の全体．投影と，点を隠す球と曲面．
struct Space<'a> {
    camera: Camera,
    balls: Vec<Ball>,
    /// 曲面の網．シーンの中の曲面の順に並ぶ．
    meshes: Vec<Mesh>,
    /// 複体の網．シーンの中の複体の順に並ぶ．面を三角形分割したもので，`hides`にだけ使う
    /// (複体自身の稜が隠れるかは，稜に隣接する面の向きで決めるので，ここは使わない)．
    complex_meshes: Vec<Mesh>,
    /// 曲面の式の関数．`meshes`と同じ順に並び，交線と切り口を，厳密な式の上へ磨くために使う．
    surfaces: Vec<SurfaceMap<'a>>,
    /// 曲面の`id`から，`meshes`と`surfaces`の中の番号．
    mesh_of: HashMap<String, usize>,
    /// 曲面の`id`から，2つの変数それぞれの範囲．接平面の偏微分の刻みを決めるために使う．
    surface_domains: HashMap<String, [[f64; 2]; 2]>,
}

impl Space<'_> {
    /// 曲面自身の輪郭の点が隠れているか．自身の網には，輪郭用の判定を使い，ほかの網と球には，そのまま使う．
    fn rim_hidden(&self, rim: Rim, own: &Mesh) -> bool {
        self.balls
            .iter()
            .any(|ball| ball.hides(rim.0, self.camera.toward))
            || self.meshes.iter().any(|mesh| {
                if std::ptr::eq(mesh, own) {
                    mesh.rim_hidden(rim)
                } else {
                    mesh.hides(rim.0)
                }
            })
    }

    fn hidden(&self, point: Point3) -> bool {
        self.balls
            .iter()
            .any(|ball| ball.hides(point, self.camera.toward))
            || self.meshes.iter().any(|mesh| mesh.hides(point))
            || self.complex_meshes.iter().any(|mesh| mesh.hides(point))
    }

    /// 点が，自分(`own`)以外のオブジェクト(球，曲面，ほかの複体)に隠れているか．複体自身の稜の
    /// 隠れ方は，稜に隣接する面の向きで別に決めるので，自分の網はここでは調べない．
    fn hidden_by_others(&self, point: Point3, own: &Mesh) -> bool {
        self.balls
            .iter()
            .any(|ball| ball.hides(point, self.camera.toward))
            || self.meshes.iter().any(|mesh| mesh.hides(point))
            || self
                .complex_meshes
                .iter()
                .any(|mesh| !std::ptr::eq(mesh, own) && mesh.hides(point))
    }
}

/// 曲面の式の関数．2つの変数から，点を返す．値が有限でなければ，`None`を返す．ベジエ曲面では，制御点の網から求める．
/// 曲面の変換(`transform`)は，ここで施すので，切り口・交線・接平面も，変換した曲面の上に求まる．
fn surface_map<'a>(
    plot: &'a SurfacePlot,
    compiled: &'a Compiled,
    transform: &'a Transform,
) -> SurfaceMap<'a> {
    if let Some(net) = &plot.net {
        return Box::new(move |u: f64, v: f64| transform.apply3(bezier_point(net, u, v)?));
    }
    Box::new(move |u: f64, v: f64| {
        let mut values = vec![u, v];
        values.extend_from_slice(&compiled.parameters);
        let [x, y, z] = plot.exprs.as_slice() else {
            return None;
        };
        let point = [x.eval(&values), y.eval(&values), z.eval(&values)];
        if point.iter().all(|c| c.is_finite()) {
            transform.apply3(point)
        } else {
            None
        }
    })
}

/// 面(頂点の番号の周)を，最初の頂点を要にした扇形に，三角形分割する．
fn fan_triangles(face: &[usize]) -> impl Iterator<Item = [usize; 3]> + '_ {
    let first = face.first().copied().unwrap_or(0);
    let seconds = face.iter().copied().skip(1);
    let thirds = face.iter().copied().skip(2);
    seconds.zip(thirds).map(move |(b, c)| [first, b, c])
}

/// 複体として描くオブジェクト(複体と正多面体)の，変換(アフィン変換に限る)を施した複体．ほかのオブジェクト
/// なら`None`．対称移動のように向きが裏返る変換では，面の頂点の並びを逆にして，面の向きを外向きに保つ．
fn as_complex<'a>(object: &'a Object, transform: &Transform) -> Option<Cow<'a, Complex>> {
    let complex = match object {
        Object::Complex(complex) => Cow::Borrowed(complex),
        Object::Polyhedron(polyhedron) => Cow::Owned(polyhedron.to_complex()),
        _ => return None,
    };
    if transform.is_identity() {
        return Some(complex);
    }
    let affine = transform.as_affine()?;
    let mut moved = complex.into_owned();
    for vertex in &mut moved.vertices {
        *vertex = affine.apply3(*vertex);
    }
    if affine.determinant() < 0.0 {
        for face in &mut moved.faces {
            face.reverse();
        }
    }
    Some(Cow::Owned(moved))
}

/// 複体の面を三角形分割して，ほかのオブジェクトを隠すための網にする．
fn complex_mesh(complex: &Complex, frame: Frame) -> Mesh {
    let indices = complex
        .faces
        .iter()
        .flat_map(|face| fan_triangles(face))
        .collect();
    Mesh::from_triangles(complex.vertices.clone(), indices, frame)
}

/// 図全体(投影と，点を隠す球と曲面)を組み立てる．
fn build_space<'a>(scene: &'a Scene, view: &SpaceView, compiled: &'a Compiled) -> Space<'a> {
    let camera = Camera::new(view);
    let placed_surfaces: Vec<(&Surface, &SurfacePlot, &Transform)> = scene
        .objects
        .iter()
        .zip(&compiled.plots)
        .zip(&compiled.transforms)
        .filter_map(|((object, plot), transform)| match (object, plot) {
            (Object::Surface(surface), Plot::Surface(placed)) => Some((surface, placed, transform)),
            _ => None,
        })
        .collect();
    let surfaces: Vec<SurfaceMap> = placed_surfaces
        .iter()
        .map(|(_, placed, transform)| surface_map(placed, compiled, transform))
        .collect();
    let meshes = placed_surfaces
        .iter()
        .zip(&surfaces)
        .map(|((surface, placed, _), map)| {
            Mesh::build(map.as_ref(), placed.domain, surface.mesh, camera.frame())
        })
        .collect();
    let mesh_of: HashMap<String, usize> = scene
        .objects
        .iter()
        .filter_map(|object| match object {
            Object::Surface(surface) => Some(surface.id.clone()),
            _ => None,
        })
        .enumerate()
        .map(|(index, id)| (id, index))
        .collect();
    let surface_domains: HashMap<String, [[f64; 2]; 2]> = placed_surfaces
        .iter()
        .map(|(surface, placed, _)| (surface.id.clone(), placed.domain))
        .collect();
    let complex_meshes: Vec<Mesh> = scene
        .objects
        .iter()
        .zip(&compiled.transforms)
        .filter_map(|(object, transform)| {
            as_complex(object, transform).map(|complex| complex_mesh(&complex, camera.frame()))
        })
        .collect();
    Space {
        camera,
        meshes,
        complex_meshes,
        surfaces,
        mesh_of,
        surface_domains,
        balls: scene
            .objects
            .iter()
            .filter_map(|object| match object {
                Object::Sphere(sphere) => Some(Ball {
                    center: sphere.center,
                    radius: sphere.radius,
                }),
                _ => None,
            })
            .collect(),
    }
}

/// 空間の図を，描画の中間表現にする．
pub fn render_space(scene: &Scene, view: &SpaceView, compiled: &Compiled) -> Figure {
    let space = build_space(scene, view, compiled);
    let mut items = Vec::new();
    let mut meshes = space.meshes.iter();
    let mut maps = space.surfaces.iter();
    let mut complex_meshes = space.complex_meshes.iter();
    for ((object, plot), transform) in scene
        .objects
        .iter()
        .zip(&compiled.plots)
        .zip(&compiled.transforms)
    {
        if let Some(complex) = as_complex(object, transform) {
            if let Some(mesh) = complex_meshes.next() {
                items.extend(complex_items(&complex, mesh, &space));
            }
            continue;
        }
        match (object, plot) {
            (Object::Surface(surface), Plot::Surface(placed)) => {
                if let (Some(mesh), Some(map)) = (meshes.next(), maps.next()) {
                    items.extend(surface_items(surface, mesh, &space));
                    items.extend(surface_wireframe_items(surface, placed, map, &space));
                    items.extend(control_net_items(surface, placed, transform, &space));
                }
            }
            (Object::Point(point), Plot::Point(placed)) => {
                items.extend(point_items(point, placed, &space));
            }
            (Object::Vector(vector), Plot::Link(link)) => {
                items.extend(link_items(&vector.style, vector.arrow, link, &space));
            }
            (Object::Segment(segment), Plot::Link(link)) => {
                items.extend(link_items(&segment.style, Arrow::None, link, &space));
            }
            (Object::Intersection(found), _) => {
                items.extend(intersection_items(found, &space));
            }
            (Object::Cut(cut), Plot::Cut(placed)) => {
                items.extend(cut_items(cut, placed, &space));
            }
            (Object::TangentPlane(tangent), Plot::TangentPlane(placed)) => {
                items.extend(tangent_plane_items(tangent, placed, &space));
            }
            (Object::Axis(axis), _) => items.extend(axis_items(axis, &space)),
            (Object::Label(label), Plot::Label(placed)) => {
                items.extend(label_item(label, placed, &space.camera).map(Item::Label));
            }
            (Object::Sphere(sphere), _) => {
                items.push(outline(sphere, &space.camera));
                items.extend(sphere_wireframe_items(sphere, &space));
            }
            (Object::Curve(curve), Plot::Curve(plot)) => {
                items.extend(curve_items(curve.style, plot, compiled, transform, &space));
            }
            (Object::Grid(grid), Plot::Grid(placed)) => {
                items.extend(grid_items(&grid.style, placed, transform, &space));
            }
            _ => {}
        }
    }
    Figure {
        description: scene.description.clone(),
        bounds: bounds_of(&items),
        items,
    }
}

/// 空間の位置に置くラベル．ラベルは，球に隠れない．
fn label_item(label: &Label, placed: &LabelPlot, camera: &Camera) -> Option<LabelItem> {
    // 位置の数は，検査で確かめてある．
    let [x, y, z] = placed.at.as_slice() else {
        return None;
    };
    Some(LabelItem {
        at: camera.project([*x, *y, *z]),
        anchor: label.anchor,
        tex: label.tex.clone(),
    })
}

/// 隠れた部分の線の種類．描かないときは`None`．
const fn hidden_line(hidden: Hidden) -> Option<Line> {
    match hidden {
        Hidden::Dotted => Some(Line::Dotted),
        Hidden::Dashed => Some(Line::Dashed),
        Hidden::None => None,
    }
}

/// 球の輪郭．中心の投影を中心とする，半径の円である．
fn outline(sphere: &Sphere, camera: &Camera) -> Item {
    let center = camera.project(sphere.center);
    let radius = sphere.radius * camera.unit;
    let mut paths = sample(
        |t| Some([center[0] + radius * t.cos(), center[1] + radius * t.sin()]),
        0.0,
        TAU,
    );
    let mut points = paths.pop().unwrap_or_default();
    // 一周して戻る点は，計算の誤差で，始めの点とずれる．閉じるために，重ねる．
    if let Some(first) = points.first().copied()
        && let Some(last) = points.last_mut()
    {
        *last = first;
    }
    Item::Path(Path {
        points,
        stroke: stroke_of(&sphere.style, Line::Solid, CURVE_WIDTH),
        arrow: None,
    })
}

/// 軸の線，先端の矢じり，軸の名前．
fn axis_items(axis: &Axis, space: &Space) -> Vec<Item> {
    let Some([low, high]) = axis.range else {
        return Vec::new();
    };
    let unit_vector = unit_vector(axis.direction);
    let at = |t: f64| unit_vector.map(|c| c * t);
    let steps: Vec<f64> = (0..=AXIS_STEPS)
        .map(|step| {
            if step == AXIS_STEPS {
                high
            } else {
                low + (high - low) * f64::from(step) / f64::from(AXIS_STEPS)
            }
        })
        .collect();
    let pieces = split_by_visibility(&steps, &at, &|t| space.hidden(at(t)));
    let direction = normalized(space.camera.project(unit_vector));
    let end = space.camera.project(at(high));
    let last = pieces.len().saturating_sub(1);
    let stroke = stroke_of(&axis.style, Line::Solid, AXIS_WIDTH);
    let mut items = Vec::new();
    for (index, piece) in pieces.iter().enumerate() {
        let Some(line) = piece_line(piece.hidden, stroke.line, axis.style.hidden) else {
            continue;
        };
        // 軸はまっすぐなので，各部分は，両端だけで描く．
        let (Some(first), Some(final_point)) = (piece.points.first(), piece.points.last()) else {
            continue;
        };
        let arrow = match direction {
            Some(direction) if index == last && !piece.hidden => {
                arrow_head(axis.arrow, end, direction, stroke.width)
            }
            _ => None,
        };
        items.push(Item::Path(Path {
            points: vec![
                space.camera.project(*first),
                space.camera.project(*final_point),
            ],
            stroke: Stroke { line, ..stroke },
            arrow,
        }));
    }
    if let Some(text) = &axis.label {
        items.push(Item::Label(LabelItem {
            at: end,
            anchor: direction.map_or(Anchor::South, anchor_beyond),
            tex: format!("${text}$"),
        }));
    }
    items
}

const fn unit_vector(direction: Direction) -> Point3 {
    match direction {
        Direction::X => [1.0, 0.0, 0.0],
        Direction::Y => [0.0, 1.0, 0.0],
        Direction::Z => [0.0, 0.0, 1.0],
    }
}

/// 長さ1に直した向き．長さが0なら`None`．軸がカメラの方を向いていると，画面では点になる．
fn normalized([x, y]: [f64; 2]) -> Option<[f64; 2]> {
    let length = x.hypot(y);
    (length > f64::EPSILON).then(|| [x / length, y / length])
}

/// 軸の先に置く名前の`anchor`．名前の箱は，軸の向きの先に延びるので，箱の，向きと反対の側を，軸の端に合わせる．
fn anchor_beyond([x, y]: [f64; 2]) -> Anchor {
    let degrees = y.atan2(x).to_degrees();
    let upper = degrees >= 0.0;
    match degrees.abs() {
        d if d <= SECTOR => Anchor::West,
        d if d <= 3.0 * SECTOR => {
            if upper {
                Anchor::SouthWest
            } else {
                Anchor::NorthWest
            }
        }
        d if d <= 5.0 * SECTOR => {
            if upper {
                Anchor::South
            } else {
                Anchor::North
            }
        }
        d if d <= 7.0 * SECTOR => {
            if upper {
                Anchor::SouthEast
            } else {
                Anchor::NorthEast
            }
        }
        _ => Anchor::East,
    }
}

/// 曲線の線と，隠れた部分の線．変換があれば，曲線の点に施す．
fn curve_items(
    style: Style,
    plot: &CurvePlot,
    compiled: &Compiled,
    transform: &Transform,
    space: &Space,
) -> Vec<Item> {
    let stroke = stroke_of(&style, Line::Solid, CURVE_WIDTH);
    let at = |t: f64| -> Option<Point3> {
        let point = if let Some(net) = &plot.net {
            bezier_curve_point(net, t)?
        } else if let Some(points) = &plot.spline {
            catmull_rom_point(points, t)?
        } else {
            let values = with_variable(t, &compiled.parameters);
            plot.exprs.iter().map(|expr| expr.eval(&values)).collect()
        };
        let point = point3(&point)?;
        if point.iter().all(|c| c.is_finite()) {
            transform.apply3(point)
        } else {
            None
        }
    };
    let [start, end] = plot.domain;
    traced_items(stroke, style.hidden, &at, start, end, space)
}

/// パラメータ`t`が`start`から`end`まで動く線を，隠れ方に分けて描く．画面での滑らかさに合わせて刻み，
/// 隠れ方が変わる点を二分法で詰める．値のない所で，線は切れる．
fn traced_items(
    stroke: Stroke,
    hidden: Hidden,
    at: &dyn Fn(f64) -> Option<Point3>,
    start: f64,
    end: f64,
    space: &Space,
) -> Vec<Item> {
    let point = |t: f64| at(t).unwrap_or([f64::NAN; 3]);
    let lines = sample_with_parameters(|t| at(t).map(|p| space.camera.project(p)), start, end);
    let mut items = Vec::new();
    for line in lines {
        let steps: Vec<f64> = line.iter().map(|(t, _)| *t).collect();
        let pieces = split_by_visibility(&steps, &point, &|t| space.hidden(point(t)));
        items.extend(piece_items(&pieces, stroke, hidden, &space.camera));
    }
    items
}

/// 両端`from`，`to`の線分を，隠れ方に分けて描く．まっすぐなので，各部分は両端だけで描く．
fn straight_items(
    from: Point3,
    to: Point3,
    stroke: Stroke,
    hidden: Hidden,
    space: &Space,
) -> Vec<Item> {
    let at = |t: f64| lerp3(from, to, t);
    let steps: Vec<f64> = (0..=AXIS_STEPS)
        .map(|step| f64::from(step) / f64::from(AXIS_STEPS))
        .collect();
    let pieces = split_by_visibility(&steps, &at, &|t| space.hidden(at(t)));
    let ends: Vec<Piece> = pieces
        .into_iter()
        .filter_map(|piece| {
            let (first, last) = (*piece.points.first()?, *piece.points.last()?);
            Some(Piece {
                hidden: piece.hidden,
                points: vec![first, last],
            })
        })
        .collect();
    piece_items(&ends, stroke, hidden, &space.camera)
}

/// 空間の図の格子．xy平面(z = 0)の上に，範囲を刻みの倍数の位置で区切る線を引き，変換で動かす．
/// 線は，曲面や球や複体に隠れる．写像で写すと線が曲がるので，そのときは標本化する．
fn grid_items(style: &Style, grid: &GridPlot, transform: &Transform, space: &Space) -> Vec<Item> {
    let stroke = stroke_of(style, Line::Dotted, GRID_WIDTH);
    let [x_low, x_high] = grid.x_range;
    let [y_low, y_high] = grid.y_range;
    let vertical = grid.x_step.into_iter().flat_map(|step| {
        multiples(step, grid.x_range).map(move |x| ([x, y_low, 0.0], [x, y_high, 0.0]))
    });
    let horizontal = grid.y_step.into_iter().flat_map(|step| {
        multiples(step, grid.y_range).map(move |y| ([x_low, y, 0.0], [x_high, y, 0.0]))
    });
    let affine = transform.as_affine();
    vertical
        .chain(horizontal)
        .flat_map(|(from, to)| match affine {
            Some(affine) => straight_items(
                affine.apply3(from),
                affine.apply3(to),
                stroke,
                style.hidden,
                space,
            ),
            None => traced_items(
                stroke,
                style.hidden,
                &|t| transform.apply3(lerp3(from, to, t)),
                0.0,
                1.0,
                space,
            ),
        })
        .collect()
}

/// 曲面の輪郭と，縁(`boundary`)の線．隠れた部分は，隠れた部分の線の種類で描く．
fn surface_items(surface: &Surface, mesh: &Mesh, space: &Space) -> Vec<Item> {
    let stroke = stroke_of(&surface.style, Line::Solid, CURVE_WIDTH);
    let indices = |count: usize| -> Vec<f64> {
        (0..count)
            .map(|k| f64::from(u32::try_from(k).unwrap_or(u32::MAX)))
            .collect()
    };
    let mut items = Vec::new();
    for line in &mesh.silhouette() {
        let at = |t: f64| polyline_at(line, t);
        let pieces = split_by_visibility(&indices(line.len()), &|t| at(t).0, &|t| {
            space.rim_hidden(at(t), mesh)
        });
        items.extend(piece_items(
            &pieces,
            stroke,
            surface.style.hidden,
            &space.camera,
        ));
    }
    if surface.boundary {
        for line in &mesh.boundary() {
            let rims: Vec<Rim> = line.iter().map(|p| (*p, [0.0; 3])).collect();
            let at = |t: f64| polyline_at(&rims, t).0;
            let pieces = split_by_visibility(&indices(line.len()), &at, &|t| space.hidden(at(t)));
            items.extend(piece_items(
                &pieces,
                stroke,
                surface.style.hidden,
                &space.camera,
            ));
        }
    }
    items
}

/// パラメータ`t`が`start`から`end`まで動く曲線を，隠れ方に分けて描く．軸や曲線と同じ手順で，
/// 画面での滑らかさに合わせて刻み，隠れ方が変わる点を二分法で詰める．`default_line`は，`style`が
/// 線の種類を指定しないときに使う(ワイヤーフレームと制御点の網は点線，接平面は実線)．
fn wireframe_line(
    style: &Style,
    default_line: Line,
    at: &dyn Fn(f64) -> Option<Point3>,
    start: f64,
    end: f64,
    space: &Space,
) -> Vec<Item> {
    let stroke = stroke_of(style, default_line, CURVE_WIDTH);
    traced_items(stroke, style.hidden, at, start, end, space)
}

/// 曲面のワイヤーフレーム．`u`一定・`v`一定の断面を，`plot.wireframe`(刻みから`compile.rs`が求めた値)
/// の所に引く．式で書いた曲面でもベジエ曲面でも，`map`(曲面の式の関数)が同じ形なので，同じように描ける．
fn surface_wireframe_items(
    surface: &Surface,
    plot: &SurfacePlot,
    map: &SurfaceMap,
    space: &Space,
) -> Vec<Item> {
    let Some(style) = &surface.wireframe else {
        return Vec::new();
    };
    let [[u0, u1], [v0, v1]] = plot.domain;
    let [us, vs] = &plot.wireframe;
    let mut items = Vec::new();
    for &u in us {
        items.extend(wireframe_line(
            style,
            Line::Dotted,
            &|v| map(u, v),
            v0,
            v1,
            space,
        ));
    }
    for &v in vs {
        items.extend(wireframe_line(
            style,
            Line::Dotted,
            &|u| map(u, v),
            u0,
            u1,
            space,
        ));
    }
    items
}

/// ベジエ曲面の制御点の網(行と列を結ぶ折れ線)．曲面自身と同じく，ほかの曲面や球に隠れる．
fn control_net_items(
    surface: &Surface,
    plot: &SurfacePlot,
    transform: &Transform,
    space: &Space,
) -> Vec<Item> {
    let (Some(style), Some(net)) = (&surface.control_net, &plot.net) else {
        return Vec::new();
    };
    let link = |a: Point3, b: Point3| -> Vec<Item> {
        let mix = |t: f64| {
            [
                lerp(a[0], b[0], t),
                lerp(a[1], b[1], t),
                lerp(a[2], b[2], t),
            ]
        };
        wireframe_line(
            style,
            Line::Dotted,
            &|t| transform.apply3(mix(t)),
            0.0,
            1.0,
            space,
        )
    };
    let mut items = Vec::new();
    for row in net {
        for pair in row.windows(2) {
            if let [a, b] = *pair {
                items.extend(link(a, b));
            }
        }
    }
    let columns = net.first().map_or(0, Vec::len);
    for column in 0..columns {
        for pair in net.windows(2) {
            if let [row_a, row_b] = pair
                && let (Some(a), Some(b)) = (row_a.get(column), row_b.get(column))
            {
                items.extend(link(*a, *b));
            }
        }
    }
    items
}

/// 球のワイヤーフレーム(経線と緯線)．経線は，方位角`theta`を一定にして，仰角`phi`を動かす．
/// 緯線は，`phi`を一定にして，`theta`を動かす．両極を通る経線どうしが重なることは気にしない．
fn sphere_wireframe_items(sphere: &Sphere, space: &Space) -> Vec<Item> {
    let Some(style) = &sphere.wireframe else {
        return Vec::new();
    };
    let at = |theta: f64, phi: f64| -> Point3 {
        [
            sphere.center[0] + sphere.radius * phi.cos() * theta.cos(),
            sphere.center[1] + sphere.radius * phi.cos() * theta.sin(),
            sphere.center[2] + sphere.radius * phi.sin(),
        ]
    };
    let quarter_turn = TAU / 4.0;
    let mut items = Vec::new();
    for k in 0..SPHERE_MERIDIANS {
        let theta = TAU * f64::from(u32::try_from(k).unwrap_or(0))
            / f64::from(u32::try_from(SPHERE_MERIDIANS).unwrap_or(1));
        items.extend(wireframe_line(
            style,
            Line::Dotted,
            &|phi| Some(at(theta, phi)),
            -quarter_turn,
            quarter_turn,
            space,
        ));
    }
    for t in interior_fractions(SPHERE_PARALLELS) {
        let phi = lerp(-quarter_turn, quarter_turn, t);
        items.extend(wireframe_line(
            style,
            Line::Dotted,
            &|theta| Some(at(theta, phi)),
            0.0,
            TAU,
            space,
        ));
    }
    items
}

/// 空間の座標を，3個の数から作る．
fn point3(values: &[f64]) -> Option<Point3> {
    let [x, y, z] = values else {
        return None;
    };
    Some([*x, *y, *z])
}

/// 点の印と，点の名前．曲面や球に隠れた点の印は，描かない(塗った丸は，点線にできない)．
/// 名前は，隠れていても，描く．
fn point_items(point: &Point, placed: &PointPlot, space: &Space) -> Vec<Item> {
    let Some(position) = point3(&placed.at) else {
        return Vec::new();
    };
    let at = space.camera.project(position);
    let mut items = Vec::new();
    if point.dot && !space.hidden(position) {
        items.push(Item::Dot(DotItem {
            at,
            radius: DOT_RADIUS,
            color: point.style.color,
        }));
    }
    if let Some(text) = &point.label {
        items.push(Item::Label(LabelItem {
            at,
            anchor: point.anchor.unwrap_or(Anchor::SouthWest),
            tex: format!("${text}$"),
        }));
    }
    items
}

/// ベクトルか線分の線．隠れた部分は，隠れた部分の線で描く．矢じりは，終点が見えるときだけ付く．
fn link_items(style: &Style, arrow: Arrow, link: &LinkPlot, space: &Space) -> Vec<Item> {
    let (Some(from), Some(to)) = (point3(&link.from), point3(&link.to)) else {
        return Vec::new();
    };
    let (start, end) = (space.camera.project(from), space.camera.project(to));
    let Some(direction) = normalized([end[0] - start[0], end[1] - start[1]]) else {
        return Vec::new();
    };
    let stroke = stroke_of(style, Line::Solid, CURVE_WIDTH);
    let at = |t: f64| {
        [
            from[0] + (to[0] - from[0]) * t,
            from[1] + (to[1] - from[1]) * t,
            from[2] + (to[2] - from[2]) * t,
        ]
    };
    let steps: Vec<f64> = (0..=AXIS_STEPS)
        .map(|step| {
            if step == AXIS_STEPS {
                1.0
            } else {
                f64::from(step) / f64::from(AXIS_STEPS)
            }
        })
        .collect();
    let pieces = split_by_visibility(&steps, &at, &|t| space.hidden(at(t)));
    let last = pieces.len().saturating_sub(1);
    let mut items = Vec::new();
    for (index, piece) in pieces.iter().enumerate() {
        let Some(kind) = piece_line(piece.hidden, stroke.line, style.hidden) else {
            continue;
        };
        // 直線なので，各部分は，両端だけで描く．
        let (Some(first), Some(final_point)) = (piece.points.first(), piece.points.last()) else {
            continue;
        };
        let head = if index == last && !piece.hidden {
            arrow_head(arrow, end, direction, stroke.width)
        } else {
            None
        };
        items.push(Item::Path(Path {
            points: vec![
                space.camera.project(*first),
                space.camera.project(*final_point),
            ],
            stroke: Stroke {
                line: kind,
                ..stroke
            },
            arrow: head,
        }));
    }
    items
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// 面(頂点の番号の周)の，外向きの法線(長さ1)．最初の3頂点から求める．3頂点が一直線に並ぶ
/// (退化した)面では，`None`を返す．
fn face_normal(vertices: &[Point3], face: &[usize]) -> Option<Point3> {
    let point = |index: usize| vertices.get(index).copied();
    let (a, b, c) = (
        point(*face.first()?)?,
        point(*face.get(1)?)?,
        point(*face.get(2)?)?,
    );
    normalized3(cross(minus(b, a), minus(c, a)))
}

/// 複体の稜(頂点の番号の組，小さい方が先)と，それに隣接する面の法線の並び．稜は，面の周にある
/// 隣り合う頂点の組として決まるので，手で指定しない．同じ稜が2つの面にまたがれば，法線は2個になる．
fn complex_edges(complex: &Complex) -> BTreeMap<(usize, usize), Vec<Point3>> {
    let mut edges: BTreeMap<(usize, usize), Vec<Point3>> = BTreeMap::new();
    for face in &complex.faces {
        let Some(normal) = face_normal(&complex.vertices, face) else {
            continue;
        };
        let next = face.iter().copied().cycle().skip(1).take(face.len());
        for (a, b) in face.iter().copied().zip(next) {
            let key = if a < b { (a, b) } else { (b, a) };
            edges.entry(key).or_default().push(normal);
        }
    }
    edges
}

/// 複体の稜．稜の両側の面が2つともカメラを向いていなければ，複体自身に隠れているとする(凸体なら
/// 正確に決まる)．そうでなければ，ほかのオブジェクト(球，曲面，ほかの複体)に隠れているかを調べる．
fn complex_items(complex: &Complex, mesh: &Mesh, space: &Space) -> Vec<Item> {
    let stroke = stroke_of(&complex.style, Line::Solid, CURVE_WIDTH);
    let mut items = Vec::new();
    for ((from_index, to_index), normals) in complex_edges(complex) {
        let (Some(from), Some(to)) = (
            complex.vertices.get(from_index).copied(),
            complex.vertices.get(to_index).copied(),
        ) else {
            continue;
        };
        let hidden_by_own_faces = normals.len() == 2
            && normals
                .iter()
                .all(|normal| dot(*normal, space.camera.toward) <= 0.0);
        let at = |t: f64| add(from, scale(minus(to, from), t));
        let steps: Vec<f64> = (0..=AXIS_STEPS)
            .map(|step| {
                if step == AXIS_STEPS {
                    1.0
                } else {
                    f64::from(step) / f64::from(AXIS_STEPS)
                }
            })
            .collect();
        let pieces = split_by_visibility(&steps, &at, &|t| {
            hidden_by_own_faces || space.hidden_by_others(at(t), mesh)
        });
        for piece in &pieces {
            let Some(kind) = piece_line(piece.hidden, stroke.line, complex.style.hidden) else {
                continue;
            };
            let (Some(first), Some(final_point)) = (piece.points.first(), piece.points.last())
            else {
                continue;
            };
            items.push(Item::Path(Path {
                points: vec![
                    space.camera.project(*first),
                    space.camera.project(*final_point),
                ],
                stroke: Stroke {
                    line: kind,
                    ..stroke
                },
                arrow: None,
            }));
        }
    }
    items
}

/// 2つの曲面の交線．曲面の上にあるので，曲面に隠れる部分は，隠れた部分の線で描く．
fn intersection_items(found: &Intersection, space: &Space) -> Vec<Item> {
    // 曲面の名前から，網と，式の関数．
    let surface_for = |name: &String| {
        let index = *space.mesh_of.get(name)?;
        Some((space.meshes.get(index)?, space.surfaces.get(index)?))
    };
    let [first, second] = found.surfaces.as_slice() else {
        return Vec::new();
    };
    let (Some((first, first_map)), Some((second, second_map))) =
        (surface_for(first), surface_for(second))
    else {
        return Vec::new();
    };
    let stroke = stroke_of(&found.style, Line::Solid, CURVE_WIDTH);
    let mut items = Vec::new();
    for line in &first.intersection(second, first_map.as_ref(), second_map.as_ref()) {
        let steps: Vec<f64> = (0..line.len())
            .map(|k| f64::from(u32::try_from(k).unwrap_or(u32::MAX)))
            .collect();
        let at = |t: f64| polyline_at(line, t).0;
        let pieces = split_by_visibility(&steps, &at, &|t| space.hidden(at(t)));
        items.extend(piece_items(
            &pieces,
            stroke,
            found.style.hidden,
            &space.camera,
        ));
    }
    items
}

/// 曲面の切り口の線．曲面の上にあるので，曲面に隠れる部分は，隠れた部分の線で描く．
fn cut_items(cut: &Cut, placed: &CutPlot, space: &Space) -> Vec<Item> {
    let Some((mesh, map)) = space
        .mesh_of
        .get(&cut.surface)
        .and_then(|index| Some((space.meshes.get(*index)?, space.surfaces.get(*index)?)))
    else {
        return Vec::new();
    };
    let stroke = stroke_of(&cut.style, Line::Solid, CURVE_WIDTH);
    let mut items = Vec::new();
    for line in &mesh.cut(placed.normal, placed.offset, map.as_ref()) {
        let steps: Vec<f64> = (0..line.len())
            .map(|k| f64::from(u32::try_from(k).unwrap_or(u32::MAX)))
            .collect();
        let at = |t: f64| polyline_at(line, t).0;
        let pieces = split_by_visibility(&steps, &at, &|t| space.hidden(at(t)));
        items.extend(piece_items(
            &pieces,
            stroke,
            cut.style.hidden,
            &space.camera,
        ));
    }
    items
}

/// 接平面．接する曲面の2つの偏微分(中心差分)の向きに，半径`size`だけ広げた平行四辺形として描く．
/// 曲面が見つからないか，偏微分が求められないか，どちらかの向きが0になれば(特異点など)，何も描かない．
fn tangent_plane_items(
    tangent: &TangentPlane,
    placed: &TangentPlanePlot,
    space: &Space,
) -> Vec<Item> {
    let Some((map, domain)) = space.mesh_of.get(&tangent.of).and_then(|index| {
        Some((
            space.surfaces.get(*index)?,
            space.surface_domains.get(&tangent.of)?,
        ))
    }) else {
        return Vec::new();
    };
    let [u0, v0] = placed.at;
    let Some(point) = map(u0, v0) else {
        return Vec::new();
    };
    let [[u_low, u_high], [v_low, v_high]] = *domain;
    let along_u =
        central_difference_point(|u| map(u, v0), u0, u_high - u_low).and_then(normalized3);
    let along_v =
        central_difference_point(|v| map(u0, v), v0, v_high - v_low).and_then(normalized3);
    let (Some(along_u), Some(along_v)) = (along_u, along_v) else {
        return Vec::new();
    };
    let size = placed.size;
    let corner = |u_sign: f64, v_sign: f64| {
        add(
            point,
            add(scale(along_u, u_sign * size), scale(along_v, v_sign * size)),
        )
    };
    let corners = [
        corner(1.0, 1.0),
        corner(1.0, -1.0),
        corner(-1.0, -1.0),
        corner(-1.0, 1.0),
    ];
    let mut items = Vec::new();
    for pair in [
        [corners[0], corners[1]],
        [corners[1], corners[2]],
        [corners[2], corners[3]],
        [corners[3], corners[0]],
    ] {
        let [a, b] = pair;
        let at = |t: f64| {
            Some([
                lerp(a[0], b[0], t),
                lerp(a[1], b[1], t),
                lerp(a[2], b[2], t),
            ])
        };
        items.extend(wireframe_line(
            &tangent.style,
            Line::Solid,
            &at,
            0.0,
            1.0,
            space,
        ));
    }
    items
}

/// 折れ線(点と法線の組)の，番号`t`の位置．頂点の番号の間は，直線で補う．
fn polyline_at(points: &[Rim], t: f64) -> Rim {
    let mut found = points.first().copied().unwrap_or(([f64::NAN; 3], [0.0; 3]));
    let mix = |a: Point3, b: Point3, ratio: f64| {
        [
            a[0] + (b[0] - a[0]) * ratio,
            a[1] + (b[1] - a[1]) * ratio,
            a[2] + (b[2] - a[2]) * ratio,
        ]
    };
    for (k, pair) in points.windows(2).enumerate() {
        let start = f64::from(u32::try_from(k).unwrap_or(u32::MAX));
        if let [from, to] = pair
            && t >= start
        {
            let ratio = (t - start).min(1.0);
            found = (mix(from.0, to.0, ratio), mix(from.1, to.1, ratio));
        }
    }
    found
}

/// 隠れ方の同じ部分を，線にする．隠れた部分は，`hidden`の種類で描き，`none`なら描かない．
fn piece_items(pieces: &[Piece], stroke: Stroke, hidden: Hidden, camera: &Camera) -> Vec<Item> {
    pieces
        .iter()
        .filter_map(|piece| {
            let line = piece_line(piece.hidden, stroke.line, hidden)?;
            Some(Item::Path(Path {
                points: piece
                    .points
                    .iter()
                    .map(|point| camera.project(*point))
                    .collect(),
                stroke: Stroke { line, ..stroke },
                arrow: None,
            }))
        })
        .collect()
}

/// 見える部分と隠れた部分の線の種類．描かないときは`None`．
const fn piece_line(hidden: bool, visible: Line, when_hidden: Hidden) -> Option<Line> {
    if hidden {
        hidden_line(when_hidden)
    } else {
        Some(visible)
    }
}

/// 隠れ方の同じ，続く点．
struct Piece {
    hidden: bool,
    points: Vec<Point3>,
}

/// パラメータの列に沿った曲線を，隠れ方が同じ部分に分ける．
///
/// 隣り合う刻みで隠れ方が変わるところは，二分法で切り替わりの点を詰め，前後の部分が共有する．
fn split_by_visibility(
    steps: &[f64],
    at: &dyn Fn(f64) -> Point3,
    hidden_at: &dyn Fn(f64) -> bool,
) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut previous: Option<(f64, bool)> = None;
    for &t in steps {
        let point = at(t);
        let hidden = hidden_at(t);
        match (previous, pieces.last_mut()) {
            (Some((before, was_hidden)), Some(current)) if was_hidden != hidden => {
                let switch = find_switch(before, t, was_hidden, at, hidden_at);
                current.points.push(switch);
                pieces.push(Piece {
                    hidden,
                    points: vec![switch, point],
                });
            }
            (_, Some(current)) => current.points.push(point),
            (_, None) => pieces.push(Piece {
                hidden,
                points: vec![point],
            }),
        }
        previous = Some((t, hidden));
    }
    pieces
}

/// 区間`[from, to]`で，隠れ方が`from`の側から変わる点を，二分法で詰める．
fn find_switch(
    from: f64,
    to: f64,
    hidden_at_from: bool,
    at: &dyn Fn(f64) -> Point3,
    hidden_at: &dyn Fn(f64) -> bool,
) -> Point3 {
    let (mut near, mut far) = (from, to);
    for _ in 0..BISECTIONS {
        let middle = f64::midpoint(near, far);
        if hidden_at(middle) == hidden_at_from {
            near = middle;
        } else {
            far = middle;
        }
    }
    at(f64::midpoint(near, far))
}

/// 描いた要素の外枠に余白を足した，描く範囲．要素がなければ，原点の周りの余白だけである．
fn bounds_of(items: &[Item]) -> Bounds {
    let points = items.iter().flat_map(|item| match item {
        Item::Path(path) => path
            .points
            .iter()
            .copied()
            .chain(path.arrow.iter().flat_map(|arrow| arrow.polygon))
            .collect::<Vec<_>>(),
        Item::Label(label) => vec![label.at],
        Item::Dot(dot) => vec![dot.at],
        Item::Fill(fill) => fill.points.clone(),
    });
    let (min, max) = points.fold(
        ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]),
        |(min, max), point| {
            (
                [min[0].min(point[0]), min[1].min(point[1])],
                [max[0].max(point[0]), max[1].max(point[1])],
            )
        },
    );
    let (min, max) = if min[0] <= max[0] {
        (min, max)
    } else {
        ([0.0; 2], [0.0; 2])
    };
    Bounds {
        min: [min[0] - MARGIN, min[1] - MARGIN],
        max: [max[0] + MARGIN, max[1] + MARGIN],
    }
}
