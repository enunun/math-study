//! シーンを，描画の中間表現にする．

use crate::arrow::Stealth;
use crate::compile::{Compiled, Plot, compile};
use crate::error::Error;
use crate::figure::{ArrowHead, Bounds, Figure, Item, LabelItem, Path, Stroke};
use crate::sample::sample;
use crate::scene::{
    Anchor, Arrow, Axis, CM_PER_PT, Direction, Label, Line, Object, PlaneView, Scene, Style, View,
};
use crate::space::render_space;

/// 軸の線幅(pt)．`TikZ`の`semithick`である．
pub const AXIS_WIDTH: f64 = 0.6;
/// 曲線の線幅(pt)．`TikZ`の`thick`である．
pub const CURVE_WIDTH: f64 = 0.8;
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
        View::Plane(view) => Ok(render_plane(scene, view, &compiled)),
        View::Space(view) => Ok(render_space(scene, view, &compiled)),
    }
}

fn render_plane(scene: &Scene, view: &PlaneView, compiled: &Compiled) -> Figure {
    let scale = Scale {
        x: view.unit.x.to_cm(),
        y: view.unit.y.to_cm(),
    };
    let mut items = Vec::new();
    for (object, plot) in scene.objects.iter().zip(&compiled.plots) {
        match object {
            Object::Axis(axis) => items.extend(axis_items(axis, view, scale)),
            Object::Label(label) => items.extend(label_item(label, scale).map(Item::Label)),
            Object::Graph(_) | Object::Curve(_) => {
                items.extend(plot_items(object, plot, compiled, scale));
            }
            Object::Parameter(_) | Object::Sphere(_) => {}
        }
    }
    Figure {
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
    }
}

fn label_item(label: &Label, scale: Scale) -> Option<LabelItem> {
    // 位置の数は，検査で確かめてある．
    let [x, y] = label.at.as_slice() else {
        return None;
    };
    Some(LabelItem {
        at: scale.point(*x, *y),
        anchor: label.anchor,
        tex: label.tex.clone(),
    })
}

/// 軸の線と，先端の矢じり，軸の名前．
fn axis_items(axis: &Axis, view: &PlaneView, scale: Scale) -> Vec<Item> {
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
    let mut items = vec![Item::Path(Path {
        points: vec![start, end],
        stroke: Stroke {
            line: Line::Solid,
            width: AXIS_WIDTH,
        },
        arrow: arrow_head(axis.arrow, end, direction),
    })];
    if let Some(text) = &axis.label {
        items.push(Item::Label(LabelItem {
            at: end,
            anchor,
            tex: format!("${text}$"),
        }));
    }
    items
}

/// 軸の端`end`に，向き`direction`(単位ベクトル)の矢じりを付ける．矢じりなしなら`None`．
pub fn arrow_head(arrow: Arrow, end: [f64; 2], direction: [f64; 2]) -> Option<ArrowHead> {
    match arrow {
        Arrow::Stealth => {
            let stealth = Stealth::new(AXIS_WIDTH);
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

/// グラフや曲線を標本化した，折れ線．線が切れると，折れ線が分かれる．
fn plot_items(object: &Object, plot: &Plot, compiled: &Compiled, scale: Scale) -> Vec<Item> {
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
    paths
        .into_iter()
        .map(|points| Item::Path(curve_path(points, style)))
        .collect()
}

fn curve_path(points: Vec<[f64; 2]>, style: Style) -> Path {
    Path {
        points,
        stroke: Stroke {
            line: style.line,
            width: CURVE_WIDTH,
        },
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
