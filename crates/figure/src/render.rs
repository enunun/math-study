//! シーンを，描画の中間表現にする．

use std::collections::HashMap;

use crate::arrow::Stealth;
use crate::bezier::bezier_curve_point;
use crate::clip::{clip_polygon, clip_polyline};
use crate::compile::{
    Compiled, CurvePlot, FractalPlot, GraphPlot, GridPlot, LabelPlot, LinkPlot, Plot, PointPlot,
    PolygonPlot, TangentLinePlot, TaylorPlot, TickPlot, compile,
};
use crate::derivative::central_difference_point;
use crate::error::Error;
use crate::figure::{ArrowHead, Bounds, DotItem, Figure, FillItem, Item, LabelItem, Path, Stroke};
use crate::image::expand_images;
use crate::region::region_items;
use crate::sample::sample;
use crate::scene::{
    Anchor, Arrow, Axis, CM_PER_PT, Curve, Direction, Fractal, Graph, Label, Line, Object,
    PlaneView, Point, Polygon, Scene, Style, TangentLine, Taylor, View,
};
use crate::space::render_space;
use crate::spline::catmull_rom_point;
use crate::transform::Transform;

/// 軸の線幅(pt)．`TikZ`の`semithick`である．
pub const AXIS_WIDTH: f64 = 0.6;
/// 曲線の線幅(pt)．`TikZ`の`thick`である．
pub const CURVE_WIDTH: f64 = 0.8;
/// 格子の線幅(pt)．目盛と軸より細い．
pub const GRID_WIDTH: f64 = 0.3;
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
    let scene = expand_images(scene)?;
    let compiled = compile(&scene)?;
    match &scene.view {
        View::Plane(view) => render_plane(&scene, view, &compiled),
        View::Space(view) => Ok(render_space(&scene, view, &compiled)),
    }
}

fn render_plane(scene: &Scene, view: &PlaneView, compiled: &Compiled) -> Result<Figure, Error> {
    let scale = Scale {
        x: view.unit.x.to_cm(),
        y: view.unit.y.to_cm(),
    };
    let context = PlaneContext {
        view,
        scale,
        // グラフと曲線は，見える範囲で切り取る．
        window: [
            scale.point(view.x[0], view.y[0]),
            scale.point(view.x[1], view.y[1]),
        ],
        // 領域が挟むグラフと，接線が接する対象は，`id`から引く．
        graphs: graphs_of(scene, compiled),
        curves: plane_curves_of(scene, compiled),
        compiled,
    };
    let mut items = Vec::new();
    for ((object, plot), transform) in scene
        .objects
        .iter()
        .zip(&compiled.plots)
        .zip(&compiled.transforms)
    {
        items.extend(object_items(object, plot, transform, &context)?);
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

/// 平面の図の，オブジェクトによらない描き方．
struct PlaneContext<'a> {
    view: &'a PlaneView,
    scale: Scale,
    /// 見える範囲(cm)．
    window: [[f64; 2]; 2],
    /// グラフの`id`から，グラフ．
    graphs: HashMap<&'a str, &'a GraphPlot>,
    /// グラフと曲線の`id`から，媒介変数の関数として見たもの．
    curves: HashMap<&'a str, PlaneCurve<'a>>,
    compiled: &'a Compiled,
}

/// オブジェクト1つの，描く要素．
fn object_items(
    object: &Object,
    plot: &Plot,
    transform: &Transform,
    context: &PlaneContext,
) -> Result<Vec<Item>, Error> {
    let PlaneContext {
        view,
        scale,
        window,
        compiled,
        ..
    } = *context;
    Ok(match (object, plot) {
        (Object::Region(region), Plot::Region(placed)) => region_items(
            region,
            placed,
            &context.graphs,
            &compiled.parameters,
            [scale.x, scale.y],
            window,
        )?,
        (Object::Axis(axis), _) => {
            let ticks = match plot {
                Plot::Axis(axis_plot) => axis_plot.ticks.as_slice(),
                _ => &[],
            };
            axis_items(axis, ticks, view, scale)
        }
        (Object::Label(label), Plot::Label(placed)) => label_item(label, placed, scale)
            .map(Item::Label)
            .into_iter()
            .collect(),
        (Object::Graph(Graph { id, style, .. }) | Object::Curve(Curve { id, style, .. }), _) => {
            context
                .curves
                .get(id.as_str())
                .map(|curve| plot_items(curve, *style, compiled, scale, window))
                .unwrap_or_default()
        }
        (Object::TangentLine(tangent), Plot::TangentLine(placed)) => {
            tangent_line_items(tangent, placed, &context.curves, compiled, scale, window)
        }
        (Object::Grid(grid), Plot::Grid(grid_plot)) => {
            grid_items(&grid.style, grid_plot, transform, scale, window)
        }
        (Object::Point(point), Plot::Point(placed)) => point_items(point, placed, scale),
        (Object::Vector(vector), Plot::Link(link)) => {
            link_item(&vector.style, vector.arrow, link, scale)
                .into_iter()
                .collect()
        }
        (Object::Segment(segment), Plot::Link(link)) => {
            link_item(&segment.style, Arrow::None, link, scale)
                .into_iter()
                .collect()
        }
        (Object::Fractal(fractal), Plot::Fractal(placed)) => {
            fractal_items(fractal, placed, scale, window)
        }
        (Object::Polygon(polygon), Plot::Polygon(placed)) => {
            polygon_items(polygon, placed, transform, scale, window)
        }
        (Object::Taylor(taylor), Plot::Taylor(placed)) => {
            taylor_items(taylor, placed, scale, window)
        }
        _ => Vec::new(),
    })
}

/// グラフを，`id`から引けるようにする．領域が挟むグラフを探すために使う．
fn graphs_of<'a>(scene: &'a Scene, compiled: &'a Compiled) -> HashMap<&'a str, &'a GraphPlot> {
    scene
        .objects
        .iter()
        .zip(&compiled.plots)
        .filter_map(|(object, plot)| match (object, plot) {
            (Object::Graph(graph), Plot::Graph(placed)) => Some((graph.id.as_str(), placed)),
            _ => None,
        })
        .collect()
}

/// 平面の図のグラフか曲線を，変換まで含めて，媒介変数の関数として見たもの．グラフ`y = f(x)`は，
/// 曲線`(t, f(t))`として扱う．
struct PlaneCurve<'a> {
    shape: Shape<'a>,
    transform: &'a Transform,
    /// 媒介変数(グラフでは変数)の範囲．
    domain: [f64; 2],
}

enum Shape<'a> {
    Graph(&'a GraphPlot),
    Curve(&'a CurvePlot),
}

impl PlaneCurve<'_> {
    /// 媒介変数`t`の点(数学の座標)．変換を施してある．
    fn at(&self, t: f64, parameters: &[f64]) -> Option<[f64; 2]> {
        let point = match self.shape {
            Shape::Graph(graph) => [t, graph.expr.eval(&with_variable(t, parameters))],
            Shape::Curve(curve) => curve_point(curve, t, parameters)?,
        };
        self.transform.apply2(point)
    }
}

/// グラフと曲線を，`id`から引けるようにする．接線が接する対象を探すためにも使う．
fn plane_curves_of<'a>(
    scene: &'a Scene,
    compiled: &'a Compiled,
) -> HashMap<&'a str, PlaneCurve<'a>> {
    scene
        .objects
        .iter()
        .zip(&compiled.plots)
        .zip(&compiled.transforms)
        .filter_map(|((object, plot), transform)| {
            let (shape, domain) = match plot {
                Plot::Graph(graph) => (Shape::Graph(graph), graph.domain),
                Plot::Curve(curve) => (Shape::Curve(curve), curve.domain),
                _ => return None,
            };
            Some((
                object.id(),
                PlaneCurve {
                    shape,
                    transform,
                    domain,
                },
            ))
        })
        .collect()
}

/// 曲線の，媒介変数`t`の点(数学の座標)．変換は施さない．
fn curve_point(plot: &CurvePlot, t: f64, parameters: &[f64]) -> Option<[f64; 2]> {
    let point = if let Some(net) = &plot.net {
        bezier_curve_point(net, t)?
    } else if let Some(points) = &plot.spline {
        catmull_rom_point(points, t)?
    } else {
        let values = with_variable(t, parameters);
        plot.exprs.iter().map(|expr| expr.eval(&values)).collect()
    };
    match point.as_slice() {
        [x, y] => Some([*x, *y]),
        _ => None,
    }
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

/// 格子の線．範囲を，原点から数えた刻みの倍数の位置で区切る．縦の線，横の線の順に並ぶ．
/// 変換があれば，各線を変換し，見える範囲で切り取る．
fn grid_items(
    style: &Style,
    grid: &GridPlot,
    transform: &Transform,
    scale: Scale,
    window: [[f64; 2]; 2],
) -> Vec<Item> {
    let stroke = stroke_of(style, Line::Dotted, GRID_WIDTH);
    let [x_low, x_high] = grid.x_range;
    let [y_low, y_high] = grid.y_range;
    let vertical = grid
        .x_step
        .into_iter()
        .flat_map(|step| multiples(step, grid.x_range).map(move |x| ([x, y_low], [x, y_high])));
    let horizontal = grid
        .y_step
        .into_iter()
        .flat_map(|step| multiples(step, grid.y_range).map(move |y| ([x_low, y], [x_high, y])));
    let [min, max] = window;
    vertical
        .chain(horizontal)
        .flat_map(|(from, to)| transformed_segment(from, to, transform, scale))
        .flat_map(|points| clip_polyline(&points, min, max))
        .map(|points| {
            Item::Path(Path {
                points,
                stroke,
                arrow: None,
            })
        })
        .collect()
}

/// 線分を変換した線(cm)．アフィン変換なら両端を写すだけで，写像なら曲がるので，標本化する．
fn transformed_segment(
    from: [f64; 2],
    to: [f64; 2],
    transform: &Transform,
    scale: Scale,
) -> Vec<Vec<[f64; 2]>> {
    if let Some(affine) = transform.as_affine() {
        let ([x0, y0], [x1, y1]) = (affine.apply2(from), affine.apply2(to));
        return vec![vec![scale.point(x0, y0), scale.point(x1, y1)]];
    }
    sample(
        |t| {
            let point = [
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t,
            ];
            let [x, y] = transform.apply2(point)?;
            Some(scale.point(x, y))
        },
        0.0,
        1.0,
    )
}

/// 多角形の面(塗り)と辺．辺は，変換した各辺をつないだ，閉じた折れ線である．面は，辺の下に敷く．
/// 写像で辺が途切れたときは，面を塗らない．
fn polygon_items(
    polygon: &Polygon,
    placed: &PolygonPlot,
    transform: &Transform,
    scale: Scale,
    window: [[f64; 2]; 2],
) -> Vec<Item> {
    let count = placed.vertices.len();
    let mut pieces: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut broken = false;
    for (index, from) in placed.vertices.iter().enumerate() {
        let next = index
            .saturating_add(1)
            .checked_rem(count)
            .unwrap_or_default();
        let Some(to) = placed.vertices.get(next) else {
            continue;
        };
        let edge = transformed_segment(*from, *to, transform, scale);
        broken |= edge.len() != 1;
        pieces.extend(edge);
    }
    let [min, max] = window;
    let mut items = Vec::new();
    if !broken {
        let mut boundary: Vec<[f64; 2]> = Vec::new();
        for piece in &pieces {
            // 前の辺の終わりと，次の辺の始まりは，同じ頂点である．
            let skip = usize::from(!boundary.is_empty());
            boundary.extend(piece.iter().skip(skip));
        }
        if let Some(fill) = &polygon.fill {
            // 塗りの多角形は閉じているとみなすので，最初の頂点に戻る点は除く．
            let open = boundary.split_last().map_or(&[][..], |(_, rest)| rest);
            let points = clip_polygon(open, min, max);
            if !points.is_empty() {
                items.push(Item::Fill(FillItem {
                    points,
                    color: fill.color.or(polygon.style.color),
                    opacity: fill.opacity,
                }));
            }
        }
        pieces = vec![boundary];
    }
    items.extend(
        pieces
            .iter()
            .flat_map(|points| clip_polyline(points, min, max))
            .map(|points| Item::Path(curve_path(points, polygon.style))),
    );
    items
}

/// テイラー展開の多項式のグラフ．グラフと同じに標本化し，見える範囲で切り取る．
fn taylor_items(
    taylor: &Taylor,
    placed: &TaylorPlot,
    scale: Scale,
    window: [[f64; 2]; 2],
) -> Vec<Item> {
    let [start, end] = placed.domain;
    let [min, max] = window;
    sample(|x| Some(scale.point(x, placed.value(x))), start, end)
        .iter()
        .flat_map(|points| clip_polyline(points, min, max))
        .map(|points| Item::Path(curve_path(points, taylor.style)))
        .collect()
}

/// 範囲の中にある，`step`の整数倍．範囲の端も含む．
pub fn multiples(step: f64, [low, high]: [f64; 2]) -> impl Iterator<Item = f64> {
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
    curve: &PlaneCurve,
    style: Style,
    compiled: &Compiled,
    scale: Scale,
    window: [[f64; 2]; 2],
) -> Vec<Item> {
    let [start, end] = curve.domain;
    let paths = sample(
        |t| {
            let [x, y] = curve.at(t, &compiled.parameters)?;
            Some(scale.point(x, y))
        },
        start,
        end,
    );
    let [min, max] = window;
    paths
        .iter()
        .flat_map(|points| clip_polyline(points, min, max))
        .map(|points| Item::Path(curve_path(points, style)))
        .collect()
}

/// 接線の，接する点と向き(数学の座標)．曲線(グラフは`(t, f(t))`)を，変換まで含めて微分する．
/// 対象が見つからないか，微分が求められなければ，`None`を返す．
fn tangent_of(
    tangent: &TangentLine,
    placed: &TangentLinePlot,
    curves: &HashMap<&str, PlaneCurve>,
    compiled: &Compiled,
) -> Option<([f64; 2], [f64; 2])> {
    let curve = curves.get(tangent.of.as_str())?;
    let at = |t: f64| curve.at(t, &compiled.parameters);
    let point = at(placed.at)?;
    let width = curve.domain[1] - curve.domain[0];
    let direction = central_difference_point(at, placed.at, width)?;
    (point.iter().chain(&direction).all(|c| c.is_finite())).then_some((point, direction))
}

/// 接線を，接する点を通り，見える範囲いっぱいに引いた線分として描く．向きが求められないか，
/// 長さが0になれば(垂直接線を除く定義域の端など)，何も描かない．
fn tangent_line_items(
    tangent: &TangentLine,
    placed: &TangentLinePlot,
    curves: &HashMap<&str, PlaneCurve>,
    compiled: &Compiled,
    scale: Scale,
    window: [[f64; 2]; 2],
) -> Vec<Item> {
    let Some((point, direction)) = tangent_of(tangent, placed, curves, compiled) else {
        return Vec::new();
    };
    let point = scale.point(point[0], point[1]);
    let direction = [direction[0] * scale.x, direction[1] * scale.y];
    let length = direction[0].hypot(direction[1]);
    if !(length.is_finite() && length > 0.0) {
        return Vec::new();
    }
    let unit = [direction[0] / length, direction[1] / length];
    let [min, max] = window;
    // 見える範囲の対角線より確実に長く延ばしてから，範囲で切り取る．
    let reach = (max[0] - min[0]).hypot(max[1] - min[1]).mul_add(2.0, 1.0);
    let segment = [
        [point[0] - unit[0] * reach, point[1] - unit[1] * reach],
        [point[0] + unit[0] * reach, point[1] + unit[1] * reach],
    ];
    clip_polyline(&segment, min, max)
        .into_iter()
        .map(|points| Item::Path(curve_path(points, tangent.style)))
        .collect()
}

/// フラクタル図形の線．タートルが歩いた点の並びを，数学の座標からcmに直し，見える範囲で切り取る．
fn fractal_items(
    fractal: &Fractal,
    plot: &FractalPlot,
    scale: Scale,
    window: [[f64; 2]; 2],
) -> Vec<Item> {
    let [min, max] = window;
    plot.paths
        .iter()
        .map(|path| {
            path.iter()
                .map(|&[x, y]| scale.point(x, y))
                .collect::<Vec<_>>()
        })
        .flat_map(|points| clip_polyline(&points, min, max))
        .map(|points| Item::Path(curve_path(points, fractal.style)))
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
