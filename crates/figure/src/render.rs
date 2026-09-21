//! シーンを，描画の中間表現にする．

use std::collections::HashMap;

use crate::arrow::Stealth;
use crate::clip::clip_polyline;
use crate::compile::{
    Compiled, GraphPlot, GridPlot, LabelPlot, LinkPlot, Plot, PointPlot, TickPlot, compile,
};
use crate::error::Error;
use crate::figure::{ArrowHead, Bounds, DotItem, Figure, Item, LabelItem, Path, Stroke};
use crate::region::region_items;
use crate::sample::sample;
use crate::scene::{
    Anchor, Arrow, Axis, CM_PER_PT, Direction, Label, Line, Object, PlaneView, Point, Scene, Style,
    View,
};
use crate::space::render_space;

/// 軸の線幅(pt)．`TikZ`の`semithick`である．
pub const AXIS_WIDTH: f64 = 0.6;
/// 曲線の線幅(pt)．`TikZ`の`thick`である．
pub const CURVE_WIDTH: f64 = 0.8;
/// 格子の線幅(pt)．目盛と軸より細い．
const GRID_WIDTH: f64 = 0.3;
/// 刻みの倍数の位置を，範囲の端に含めるための，割った値の許容．
const GRID_EPSILON: f64 = 1e-9;
/// 目盛の線の，軸から片側への長さ(pt)．
const TICK_HALF_LENGTH: f64 = 3.0;
/// 見える範囲の外側に足す余白(cm)．軸の名前が，範囲の端の外に出る分である．
pub const MARGIN: f64 = 0.6;

/// 数学の座標から，cmの座標への倍率．
#[derive(Clone, Copy)]
struct Scale {
    x: f64,
    y: f64,
}

impl Scale {
    fn point(self, x: f64, y: f64) -> [f64; 2] {
        [x * self.x, y * self.y]
    }
}

/// シーンを，描画の中間表現にする．
///
/// # Errors
///
/// 式の誤りがあると，誤りを返す．
pub fn render(scene: &Scene) -> Result<Figure, Error> {
    let compiled = compile(scene)?;
    match &scene.view {
        View::Plane(view) => render_plane(scene, view, &compiled),
        View::Space(view) => Ok(render_space(scene, view, &compiled)),
    }
}

fn render_plane(scene: &Scene, view: &PlaneView, compiled: &Compiled) -> Result<Figure, Error> {
    let scale = Scale {
        x: view.unit.x.to_cm(),
        y: view.unit.y.to_cm(),
    };
    // グラフと曲線は，見える範囲で切り取る．
    let window = [
        scale.point(view.x[0], view.y[0]),
        scale.point(view.x[1], view.y[1]),
    ];
    // 領域が挟むグラフは，`id`から引く．
    let graphs: HashMap<&str, &GraphPlot> = scene
        .objects
        .iter()
        .zip(&compiled.plots)
        .filter_map(|(object, plot)| match (object, plot) {
            (Object::Graph(graph), Plot::Graph(placed)) => Some((graph.id.as_str(), placed)),
            _ => None,
        })
        .collect();
    let mut items = Vec::new();
    for (object, plot) in scene.objects.iter().zip(&compiled.plots) {
        match object {
            Object::Region(region) => {
                if let Plot::Region(placed) = plot {
                    let unit = [scale.x, scale.y];
                    items.extend(region_items(
                        region,
                        placed,
                        &graphs,
                        &compiled.parameters,
                        unit,
                        window,
                    )?);
                }
            }
            Object::Axis(axis) => {
                let ticks = match plot {
                    Plot::Axis(axis_plot) => axis_plot.ticks.as_slice(),
                    _ => &[],
                };
                items.extend(axis_items(axis, ticks, view, scale));
            }
            Object::Label(label) => {
                if let Plot::Label(placed) = plot {
                    items.extend(label_item(label, placed, scale).map(Item::Label));
                }
            }
            Object::Graph(_) | Object::Curve(_) => {
                items.extend(plot_items(object, plot, compiled, scale, window));
            }
            Object::Grid(grid) => {
                if let Plot::Grid(grid_plot) = plot {
                    items.extend(grid_items(&grid.style, grid_plot, view, scale));
                }
            }
            Object::Point(point) => {
                if let Plot::Point(placed) = plot {
                    items.extend(point_items(point, placed, scale));
                }
            }
            Object::Vector(vector) => {
                if let Plot::Link(link) = plot {
                    items.extend(link_item(&vector.style, vector.arrow, link, scale));
                }
            }
            Object::Segment(segment) => {
                if let Plot::Link(link) = plot {
                    items.extend(link_item(&segment.style, Arrow::None, link, scale));
                }
            }
            Object::Parameter(_)
            | Object::Sphere(_)
            | Object::Surface(_)
            | Object::Cut(_)
            | Object::Intersection(_) => {}
        }
    }
    Ok(Figure {
        description: scene.description.clone(),
        bounds: Bounds {
            min: {
                let [x, y] = scale.point(view.x[0], view.y[0]);
                [x - MARGIN, y - MARGIN]
            },
            max: {
                let [x, y] = scale.point(view.x[1], view.y[1]);
                [x + MARGIN, y + MARGIN]
            },
        },
        items,
    })
}

fn label_item(label: &Label, placed: &LabelPlot, scale: Scale) -> Option<LabelItem> {
    // 位置の数は，検査で確かめてある．
    let [x, y] = placed.at.as_slice() else {
        return None;
    };
    Some(LabelItem {
        at: scale.point(*x, *y),
        anchor: label.anchor,
        tex: label.tex.clone(),
    })
}

/// 点の印(塗った丸)の半径(pt)．
pub const DOT_RADIUS: f64 = 2.0;

/// 点の印と，点の名前．名前の箱は，既定では，点の右上に置く．
fn point_items(point: &Point, placed: &PointPlot, scale: Scale) -> Vec<Item> {
    // 座標の数は，検査で確かめてある．
    let [x, y] = placed.at.as_slice() else {
        return Vec::new();
    };
    let at = scale.point(*x, *y);
    let mut items = Vec::new();
    if point.dot {
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

/// ベクトルか線分の線．終点に矢じりを付けられる．長さがなければ，何も描かない．
fn link_item(style: &Style, arrow: Arrow, link: &LinkPlot, scale: Scale) -> Option<Item> {
    let ([from_x, from_y], [to_x, to_y]) = (link.from.as_slice(), link.to.as_slice()) else {
        return None;
    };
    let start = scale.point(*from_x, *from_y);
    let end = scale.point(*to_x, *to_y);
    let length = (end[0] - start[0]).hypot(end[1] - start[1]);
    if length <= f64::EPSILON {
        return None;
    }
    let stroke = stroke_of(style, Line::Solid, CURVE_WIDTH);
    let direction = [(end[0] - start[0]) / length, (end[1] - start[1]) / length];
    Some(Item::Path(Path {
        points: vec![start, end],
        stroke,
        arrow: arrow_head(arrow, end, direction, stroke.width),
    }))
}

/// 軸の線と，先端の矢じり，軸の名前．
fn axis_items(axis: &Axis, ticks: &[TickPlot], view: &PlaneView, scale: Scale) -> Vec<Item> {
    let (start, end, direction, anchor) = match axis.direction {
        Direction::X => {
            let [low, high] = axis.range.unwrap_or(view.x);
            (
                scale.point(low, 0.0),
                scale.point(high, 0.0),
                [1.0, 0.0],
                Anchor::West,
            )
        }
        Direction::Y => {
            let [low, high] = axis.range.unwrap_or(view.y);
            (
                scale.point(0.0, low),
                scale.point(0.0, high),
                [0.0, 1.0],
                Anchor::South,
            )
        }
        // z軸は，空間の図でだけ使える．検査で，平面の図から除かれている．
        Direction::Z => return Vec::new(),
    };
    let stroke = stroke_of(&axis.style, Line::Solid, AXIS_WIDTH);
    let mut items = vec![Item::Path(Path {
        points: vec![start, end],
        stroke,
        arrow: arrow_head(axis.arrow, end, direction, stroke.width),
    })];
    // 目盛は，軸の色と太さで，実線に引く．
    let tick_stroke = Stroke {
        line: Line::Solid,
        ..stroke
    };
    for tick in ticks {
        items.extend(tick_items(tick, axis.direction, tick_stroke, scale));
    }
    if let Some(text) = &axis.label {
        items.push(Item::Label(LabelItem {
            at: end,
            anchor,
            tex: format!("${text}$"),
        }));
    }
    items
}

/// 格子の線．見える範囲を，原点から数えた刻みの倍数の位置で区切る．縦の線，横の線の順に並ぶ．
fn grid_items(style: &Style, grid: &GridPlot, view: &PlaneView, scale: Scale) -> Vec<Item> {
    let stroke = stroke_of(style, Line::Dotted, GRID_WIDTH);
    let make = |points: [[f64; 2]; 2]| {
        Item::Path(Path {
            points: points.to_vec(),
            stroke,
            arrow: None,
        })
    };
    let vertical = grid.x_step.into_iter().flat_map(|step| {
        multiples(step, view.x)
            .map(|x| make([scale.point(x, view.y[0]), scale.point(x, view.y[1])]))
    });
    let horizontal = grid.y_step.into_iter().flat_map(|step| {
        multiples(step, view.y)
            .map(|y| make([scale.point(view.x[0], y), scale.point(view.x[1], y)]))
    });
    vertical.chain(horizontal).collect()
}

/// 範囲の中にある，`step`の整数倍．範囲の端も含む．
fn multiples(step: f64, [low, high]: [f64; 2]) -> impl Iterator<Item = f64> {
    let first = (low / step - GRID_EPSILON).ceil();
    let last = (high / step + GRID_EPSILON).floor();
    std::iter::successors(Some(first), move |k| (k + 1.0 <= last).then_some(k + 1.0))
        .take_while(move |k| *k <= last)
        .map(move |k| (k * step).clamp(low, high))
}

/// 目盛の線と，名前．線は，軸に直角で，軸をまたぐ．名前は，x軸では線の下，y軸では線の左に置く．
fn tick_items(tick: &TickPlot, direction: Direction, stroke: Stroke, scale: Scale) -> Vec<Item> {
    let half = TICK_HALF_LENGTH * CM_PER_PT;
    let ([start, end], name_at, default_anchor) = match direction {
        Direction::X => {
            let [x, y] = scale.point(tick.at, 0.0);
            ([[x, y - half], [x, y + half]], [x, y - half], Anchor::North)
        }
        Direction::Y => {
            let [x, y] = scale.point(0.0, tick.at);
            ([[x - half, y], [x + half, y]], [x - half, y], Anchor::East)
        }
        // z軸は，空間の図でだけ使える．検査で，平面の図から除かれている．
        Direction::Z => return Vec::new(),
    };
    let mut items = vec![Item::Path(Path {
        points: vec![start, end],
        stroke,
        arrow: None,
    })];
    if let Some(text) = &tick.label {
        items.push(Item::Label(LabelItem {
            at: name_at,
            anchor: tick.anchor.unwrap_or(default_anchor),
            tex: format!("${text}$"),
        }));
    }
    items
}

/// スタイルから，線の種類，太さ(pt)，色を決める．省いた項目は，既定である．
pub fn stroke_of(style: &Style, default_line: Line, default_width: f64) -> Stroke {
    Stroke {
        line: style.line.unwrap_or(default_line),
        width: style.width_pt().unwrap_or(default_width),
        color: style.color,
    }
}

/// 軸の端`end`に，向き`direction`(単位ベクトル)の矢じりを付ける．矢じりなしなら`None`．
/// 矢じりの大きさは，線幅`width`(pt)で決まる．
pub fn arrow_head(
    arrow: Arrow,
    end: [f64; 2],
    direction: [f64; 2],
    width: f64,
) -> Option<ArrowHead> {
    match arrow {
        Arrow::Stealth => {
            let stealth = Stealth::new(width);
            let placed = stealth.place(end, direction, CM_PER_PT);
            Some(ArrowHead {
                kind: Arrow::Stealth,
                polygon: placed.polygon,
                line_width: stealth.line_width,
                line_end: placed.line_end,
            })
        }
        Arrow::None => None,
    }
}

/// グラフや曲線を標本化して，見える範囲で切り取った，折れ線．線が切れるか，範囲を出ると，折れ線が分かれる．
fn plot_items(
    object: &Object,
    plot: &Plot,
    compiled: &Compiled,
    scale: Scale,
    window: [[f64; 2]; 2],
) -> Vec<Item> {
    let (paths, style) = match (object, plot) {
        (Object::Graph(graph), Plot::Graph(plot)) => {
            let [start, end] = plot.domain;
            let paths = sample(
                |t| {
                    let y = plot.expr.eval(&with_variable(t, &compiled.parameters));
                    Some(scale.point(t, y))
                },
                start,
                end,
            );
            (paths, graph.style)
        }
        (Object::Curve(curve), Plot::Curve(plot)) => {
            let [start, end] = plot.domain;
            let paths = sample(
                |t| {
                    let values = with_variable(t, &compiled.parameters);
                    let [x_expr, y_expr] = plot.exprs.as_slice() else {
                        return None;
                    };
                    Some(scale.point(x_expr.eval(&values), y_expr.eval(&values)))
                },
                start,
                end,
            );
            (paths, curve.style)
        }
        _ => return Vec::new(),
    };
    let [min, max] = window;
    paths
        .iter()
        .flat_map(|points| clip_polyline(points, min, max))
        .map(|points| Item::Path(curve_path(points, style)))
        .collect()
}

fn curve_path(points: Vec<[f64; 2]>, style: Style) -> Path {
    Path {
        points,
        stroke: stroke_of(&style, Line::Solid, CURVE_WIDTH),
        arrow: None,
    }
}

/// 変数の値を先頭に，媒介変数の値を続けた，式に渡す値の並び．
pub fn with_variable(value: f64, parameters: &[f64]) -> Vec<f64> {
    let mut values = Vec::with_capacity(parameters.len().saturating_add(1));
    values.push(value);
    values.extend_from_slice(parameters);
    values
}
