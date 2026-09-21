//! 空間の図を，描画の中間表現にする．
//!
//! 平行投影で，画面の右向きを`(-sin a, cos a, 0)`，上向きを`(-sin e cos a, -sin e sin a, cos e)`，
//! カメラへの向きを`(cos e cos a, cos e sin a, sin e)`とする(`a`は方位角，`e`は仰角)．
//! 球は，不透明な殻として，点を隠す．点から，カメラへの視線が，球に当たれば，その点は隠れている．
//! 線は，まず点で刻み，隠れ方が変わる区間を二分法で詰めて，隠れた部分と見える部分に分ける．
//! 刻みの間に，隠れ方が2回変わる細かい隠れは，見つけられない．

use std::f64::consts::TAU;

use crate::compile::{Compiled, CurvePlot, LabelPlot, Plot};
use crate::figure::{Bounds, Figure, Item, LabelItem, Path, Stroke};
use crate::render::{AXIS_WIDTH, CURVE_WIDTH, MARGIN, arrow_head, stroke_of, with_variable};
use crate::sample::{sample, sample_with_parameters};
use crate::scene::{
    Anchor, Axis, Direction, Hidden, Label, Line, Object, Scene, SpaceView, Sphere, Style,
};

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

/// 画面への投影．
struct Camera {
    right: Point3,
    up: Point3,
    toward: Point3,
    /// 1単位の実寸(cm)．
    unit: f64,
}

impl Camera {
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

/// 図の全体．投影と，点を隠す球．
struct Space {
    camera: Camera,
    balls: Vec<Ball>,
}

impl Space {
    fn hidden(&self, point: Point3) -> bool {
        self.balls
            .iter()
            .any(|ball| ball.hides(point, self.camera.toward))
    }
}

/// 空間の図を，描画の中間表現にする．
pub fn render_space(scene: &Scene, view: &SpaceView, compiled: &Compiled) -> Figure {
    let space = Space {
        camera: Camera::new(view),
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
    };
    let mut items = Vec::new();
    for (object, plot) in scene.objects.iter().zip(&compiled.plots) {
        match (object, plot) {
            (Object::Axis(axis), _) => items.extend(axis_items(axis, &space)),
            (Object::Label(label), Plot::Label(placed)) => {
                items.extend(label_item(label, placed, &space.camera).map(Item::Label));
            }
            (Object::Sphere(sphere), _) => items.push(outline(sphere, &space.camera)),
            (Object::Curve(curve), Plot::Curve(plot)) => {
                items.extend(curve_items(curve.style, plot, compiled, &space));
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
    let pieces = split_by_visibility(&steps, &at, space);
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

/// 曲線の線と，隠れた部分の線．
fn curve_items(style: Style, plot: &CurvePlot, compiled: &Compiled, space: &Space) -> Vec<Item> {
    let stroke = stroke_of(&style, Line::Solid, CURVE_WIDTH);
    let at = |t: f64| -> Option<Point3> {
        let values = with_variable(t, &compiled.parameters);
        let [x, y, z] = plot.exprs.as_slice() else {
            return None;
        };
        let point = [x.eval(&values), y.eval(&values), z.eval(&values)];
        point.iter().all(|c| c.is_finite()).then_some(point)
    };
    let [start, end] = plot.domain;
    let lines = sample_with_parameters(
        |t| at(t).map(|point| space.camera.project(point)),
        start,
        end,
    );
    let mut items = Vec::new();
    for line in lines {
        let steps: Vec<f64> = line.iter().map(|(t, _)| *t).collect();
        for piece in split_by_visibility(&steps, &|t| at(t).unwrap_or([f64::NAN; 3]), space) {
            let Some(kind) = piece_line(piece.hidden, stroke.line, style.hidden) else {
                continue;
            };
            items.push(Item::Path(Path {
                points: piece
                    .points
                    .iter()
                    .map(|point| space.camera.project(*point))
                    .collect(),
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
fn split_by_visibility(steps: &[f64], at: &dyn Fn(f64) -> Point3, space: &Space) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut previous: Option<(f64, bool)> = None;
    for &t in steps {
        let point = at(t);
        let hidden = space.hidden(point);
        match (previous, pieces.last_mut()) {
            (Some((before, was_hidden)), Some(current)) if was_hidden != hidden => {
                let switch = find_switch(before, t, was_hidden, at, space);
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
    space: &Space,
) -> Point3 {
    let (mut near, mut far) = (from, to);
    for _ in 0..BISECTIONS {
        let middle = f64::midpoint(near, far);
        if space.hidden(at(middle)) == hidden_at_from {
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
