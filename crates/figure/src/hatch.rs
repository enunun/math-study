//! 多角形の内側を，平行な斜線で埋める．内外の判定は，偶奇規則である．
//!
//! 線は，向きの法線方向の座標が，刻みの整数倍になる所に引く．原点から数えるので，多角形の位置や，
//! 頂点の並びの始点や向きに，線の位置が左右されない．

use crate::sample::Point;

/// 多角形を，斜線で埋める．`angle`は，x軸からの角度(度)，`gap`は，隣り合う線の間隔である．
/// 頂点が3つに満たない多角形，正でない間隔，有限でない角度では，何も引かない．
///
/// 多角形の辺が線と交わる数え方は，半開区間である(頂点に触れる線は，どちらか一方の側でだけ数える)．
/// 長さのない線は，引かない．
#[must_use]
pub fn hatch_lines(polygon: &[Point], angle: f64, gap: f64) -> Vec<[Point; 2]> {
    if polygon.len() < 3 || !(gap.is_finite() && gap > 0.0) || !angle.is_finite() {
        return Vec::new();
    }
    let (sin, cos) = angle.to_radians().sin_cos();
    let along = |p: Point| p[0] * cos + p[1] * sin;
    let across = |p: Point| -p[0] * sin + p[1] * cos;
    let (low, high) = polygon
        .iter()
        .map(|p| across(*p))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
            (low.min(value), high.max(value))
        });
    let first = (low / gap).ceil();
    let last = (high / gap).floor();
    std::iter::successors(Some(first), |k| (k + 1.0 <= last).then_some(k + 1.0))
        .take_while(|k| *k <= last)
        .flat_map(|k| {
            let level = k * gap;
            let mut crossings = crossings(polygon, level, &across, &along);
            crossings.sort_by(f64::total_cmp);
            crossings
                .as_chunks::<2>()
                .0
                .iter()
                .filter_map(|[from, to]| {
                    // 線上の点は，(along, across)から，元の座標に戻す．
                    let point = |t: f64| [t * cos - level * sin, t * sin + level * cos];
                    (to - from > f64::EPSILON).then(|| [point(*from), point(*to)])
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// 線(法線方向の座標が`level`)と多角形の辺の交点の，線に沿った座標．
fn crossings(
    polygon: &[Point],
    level: f64,
    across: &impl Fn(Point) -> f64,
    along: &impl Fn(Point) -> f64,
) -> Vec<f64> {
    let next = polygon.iter().cycle().skip(1);
    polygon
        .iter()
        .zip(next)
        .filter_map(|(from, to)| {
            let (a, b) = (across(*from), across(*to));
            let crosses = (a <= level && level < b) || (b <= level && level < a);
            crosses.then(|| {
                let ratio = (level - a) / (b - a);
                let point = [
                    from[0] + (to[0] - from[0]) * ratio,
                    from[1] + (to[1] - from[1]) * ratio,
                ];
                along(point)
            })
        })
        .collect()
}
