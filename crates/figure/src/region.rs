//! 領域を，斜線の線にする．領域は，2つのグラフ(かグラフとx軸)を，定義域の両端で結んだ多角形である．

use std::collections::HashMap;

use crate::clip::clip_polyline;
use crate::compile::{GraphPlot, RegionPlot};
use crate::error::{Error, ErrorKind};
use crate::figure::{Item, Path};
use crate::hatch::hatch_lines;
use crate::render::{stroke_of, with_variable};
use crate::sample::{Point, sample};
use crate::scene::{Line, Region};

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
    unit: [f64; 2],
    window: [Point; 2],
) -> Result<Vec<Item>, Error> {
    let [start, end] = placed.domain;
    let fail = |kind| Error::in_object(&region.id, kind);
    let boundary = |name: &str| -> Result<Vec<Point>, Error> {
        let plot = graphs
            .get(name)
            .ok_or_else(|| fail(ErrorKind::UnknownGraph(name.to_owned())))?;
        let mut paths = sample(
            |t| {
                let y = plot.expr.eval(&with_variable(t, parameters));
                Some([t * unit[0], y * unit[1]])
            },
            start,
            end,
        );
        match (paths.pop(), paths.is_empty()) {
            (Some(path), true) => Ok(path),
            _ => Err(fail(ErrorKind::Invalid(format!(
                "領域の範囲で，グラフ「{name}」が途切れている(値のない点があるか，大きすぎる値がある)．"
            )))),
        }
    };
    let mut polygon = Vec::new();
    let mut names = region.between.iter();
    if let Some(first) = names.next() {
        polygon.extend(boundary(first)?);
    }
    match names.next() {
        // 2つ目のグラフは，逆向きにたどる．
        Some(second) => polygon.extend(boundary(second)?.into_iter().rev()),
        // 1つだけなら，x軸に沿って，終わりから始まりへ戻る．
        None => polygon.extend([[end * unit[0], 0.0], [start * unit[0], 0.0]]),
    }
    let stroke = stroke_of(&region.style, Line::Solid, HATCH_WIDTH);
    let [min, max] = window;
    Ok(hatch_lines(&polygon, region.angle, region.gap.to_cm())
        .iter()
        .flat_map(|line| clip_polyline(line, min, max))
        .map(|points| {
            Item::Path(Path {
                points,
                stroke,
                arrow: None,
            })
        })
        .collect())
}
