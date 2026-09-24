//! 領域を，斜線の線にする．領域は，2つのグラフ(かグラフとx軸)を，定義域の両端で結んだ多角形である．
//! 変換があれば，多角形の縁に施す．写像では，まっすぐな縁(両端の縦の辺とx軸)も曲がるので，標本化する．

use std::collections::HashMap;

use crate::clip::{clip_polygon, clip_polyline};
use crate::compile::{GraphPlot, RegionPlot};
use crate::error::{Error, ErrorKind};
use crate::figure::{FillItem, Item, Path};
use crate::hatch::hatch_lines;
use crate::render::{stroke_of, with_variable};
use crate::sample::{Point, sample};
use crate::scene::{Line, Region};
use crate::transform::Transform;

/// 斜線の線幅(pt)．`TikZ`の`thin`である．
const HATCH_WIDTH: f64 = 0.4;

/// 領域を埋める斜線を，見える範囲`window`で切り取った，線の並びにする．
///
/// `unit`は，数学の座標から，cmの座標への倍率である．
///
/// # Errors
///
/// 領域の範囲で，グラフが途切れているときは，誤りを返す(多角形が作れない)．
pub fn region_items(
    region: &Region,
    placed: &RegionPlot,
    graphs: &HashMap<&str, &GraphPlot>,
    parameters: &[f64],
    transform: &Transform,
    unit: [f64; 2],
    window: [Point; 2],
) -> Result<Vec<Item>, Error> {
    let [start, end] = placed.domain;
    let fail = |kind| Error::in_object(&region.id, kind);
    let broken = |what: &str| {
        fail(ErrorKind::Invalid(format!(
            "領域の範囲で，{what}が途切れている(値のない点があるか，大きすぎる値がある)．"
        )))
    };
    // 数学の座標の点を，変換してcmの座標にする．
    let placed_at = |[x, y]: Point| -> Option<Point> {
        let [x, y] = transform.apply2([x, y])?;
        Some([x * unit[0], y * unit[1]])
    };
    // 縁の1本を標本化する．途切れれば`None`．
    let traced = |at: &dyn Fn(f64) -> Option<Point>, from: f64, to: f64| -> Option<Vec<Point>> {
        let mut paths = sample(|t| placed_at(at(t)?), from, to);
        match (paths.pop(), paths.is_empty()) {
            (Some(path), true) => Some(path),
            _ => None,
        }
    };
    let height = |name: &str| -> Result<Box<dyn Fn(f64) -> f64 + '_>, Error> {
        let plot = graphs
            .get(name)
            .ok_or_else(|| fail(ErrorKind::UnknownGraph(name.to_owned())))?;
        Ok(Box::new(move |t: f64| {
            plot.expr.eval(&with_variable(t, parameters))
        }))
    };
    let mut names = region.between.iter();
    let Some(first) = names.next() else {
        return Ok(Vec::new());
    };
    let upper = height(first)?;
    // 2つ目のグラフか，x軸．
    let (lower, lower_name): (Box<dyn Fn(f64) -> f64>, String) = match names.next() {
        Some(second) => (height(second)?, format!("グラフ「{second}」")),
        None => (Box::new(|_| 0.0), "x軸".to_owned()),
    };
    let graph_edge = |f: &dyn Fn(f64) -> f64| traced(&|t| Some([t, f(t)]), start, end);
    let mut polygon = graph_edge(&upper).ok_or_else(|| broken(&format!("グラフ「{first}」")))?;
    let mut lower_edge = graph_edge(&lower).ok_or_else(|| broken(&lower_name))?;
    // 1つだけのときのx軸は，アフィン変換ならまっすぐなので，両端だけでよい．
    if region.between.len() < 2
        && let (Some(first), Some(last)) = (lower_edge.first().copied(), lower_edge.last().copied())
        && transform.as_affine().is_some()
    {
        lower_edge = vec![first, last];
    }
    // 両端の縦の辺は，アフィン変換ならまっすぐで，多角形の辺そのものになる．写像なら曲がるので，
    // 標本化して，途中の点を加える．
    let side = |x: f64, from: f64, to: f64| -> Result<Vec<Point>, Error> {
        if transform.as_affine().is_some() {
            return Ok(Vec::new());
        }
        let line = traced(&|s| Some([x, from + (to - from) * s]), 0.0, 1.0)
            .ok_or_else(|| broken("変換した領域の縁"))?;
        Ok(line
            .get(1..line.len().saturating_sub(1))
            .unwrap_or_default()
            .to_vec())
    };
    polygon.extend(side(end, upper(end), lower(end))?);
    // 2つ目のグラフ(かx軸)は，逆向きにたどる．
    polygon.extend(lower_edge.into_iter().rev());
    polygon.extend(side(start, lower(start), upper(start))?);
    let stroke = stroke_of(&region.style, Line::Solid, HATCH_WIDTH);
    let [min, max] = window;
    let mut items = Vec::new();
    // 塗りは，斜線の下に敷く．
    if let Some(fill) = &region.fill {
        let points = clip_polygon(&polygon, min, max);
        if !points.is_empty() {
            items.push(Item::Fill(FillItem {
                points,
                color: fill.color.or(region.style.color),
                opacity: fill.opacity,
            }));
        }
    }
    if region.hatch {
        items.extend(
            hatch_lines(&polygon, region.angle, region.gap.to_cm())
                .iter()
                .flat_map(|line| clip_polyline(line, min, max))
                .map(|points| {
                    Item::Path(Path {
                        points,
                        stroke,
                        arrow: None,
                    })
                }),
        );
    }
    Ok(items)
}
