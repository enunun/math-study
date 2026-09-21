//! 折れ線を，長方形で切り取る．外に出た部分は捨て，境目には，長方形の辺の上の点を置く．
//!
//! 辺の上の点は，中にあるとして残す．線は，長方形を出るたびに分かれる．

use crate::sample::Point;

/// 折れ線を，`min`と`max`を対角とする長方形で切り取る．外に出て戻る線は，2本以上に分かれる．
/// 点が2つに満たない線と，長さのない線(辺や角に触れるだけの線)は捨てる．
#[must_use]
pub fn clip_polyline(points: &[Point], min: Point, max: Point) -> Vec<Vec<Point>> {
    let mut finished = Vec::new();
    let mut current: Vec<Point> = Vec::new();
    for pair in points.windows(2) {
        let [from, to] = pair else {
            continue;
        };
        let Some((enter, leave)) = clip_segment(*from, *to, min, max) else {
            flush(&mut current, &mut finished);
            continue;
        };
        let start = if enter <= 0.0 {
            *from
        } else {
            on_segment(*from, *to, enter, min, max)
        };
        let end = if leave >= 1.0 {
            *to
        } else {
            on_segment(*from, *to, leave, min, max)
        };
        // 入り直す線は，前の線とつながらない．
        if enter > 0.0 {
            flush(&mut current, &mut finished);
        }
        if current.is_empty() {
            current.push(start);
        }
        current.push(end);
        if leave < 1.0 {
            flush(&mut current, &mut finished);
        }
    }
    flush(&mut current, &mut finished);
    finished
}

/// 長さのない線を除いて，線を確定する．
fn flush(current: &mut Vec<Point>, finished: &mut Vec<Vec<Point>>) {
    let line = std::mem::take(current);
    let moves = line.windows(2).any(|pair| pair.first() != pair.get(1));
    if line.len() >= 2 && moves {
        finished.push(line);
    }
}

/// 線分`from`から`to`の，`t`の所の点．計算の誤差で長方形の外に出ないよう，長方形の中に収める．
fn on_segment(from: Point, to: Point, t: f64, min: Point, max: Point) -> Point {
    [
        (from[0] + (to[0] - from[0]) * t).clamp(min[0], max[0]),
        (from[1] + (to[1] - from[1]) * t).clamp(min[1], max[1]),
    ]
}

/// 線分が長方形の中にある，パラメータの範囲(Liang-Barskyの方法)．線分が長方形と交わらないか，
/// 1点でだけ交わるときは`None`を返す．
fn clip_segment(from: Point, to: Point, min: Point, max: Point) -> Option<(f64, f64)> {
    let delta = [to[0] - from[0], to[1] - from[1]];
    let mut range = (0.0_f64, 1.0_f64);
    // 各辺について，(進む向きの成分の符号を反転した値，辺までの距離)．
    let edges = [
        (-delta[0], from[0] - min[0]),
        (delta[0], max[0] - from[0]),
        (-delta[1], from[1] - min[1]),
        (delta[1], max[1] - from[1]),
    ];
    for (p, q) in edges {
        if p.abs() < f64::MIN_POSITIVE {
            // 辺に平行な線分は，辺の外側にあれば，全体が外にある．
            if q < 0.0 {
                return None;
            }
            continue;
        }
        let ratio = q / p;
        if p < 0.0 {
            range.0 = range.0.max(ratio);
        } else {
            range.1 = range.1.min(ratio);
        }
    }
    (range.0 < range.1).then_some(range)
}

/// 多角形を，`min`と`max`を対角とする長方形で切り取る(Sutherland-Hodgmanの方法)．
/// 長方形の外は捨て，境目には，長方形の辺の上の点を置く．共通部分がなければ，空である．
/// 凹んだ多角形で，共通部分が離れた2つになるときは，辺の上の線でつながった，1つの多角形になる．
#[must_use]
pub fn clip_polygon(points: &[Point], min: Point, max: Point) -> Vec<Point> {
    if points.len() < 3 {
        return Vec::new();
    }
    // (横の座標を見るか，長方形の内側は，境界より大きい側か，境界の値)．
    let edges = [
        (true, true, min[0]),
        (true, false, max[0]),
        (false, true, min[1]),
        (false, false, max[1]),
    ];
    let mut current = points.to_vec();
    for (horizontal, keep_greater, bound) in edges {
        current = clip_polygon_at(&current, horizontal, keep_greater, bound);
        if current.is_empty() {
            return current;
        }
    }
    if current.len() < 3 {
        Vec::new()
    } else {
        current
    }
}

/// 多角形を，1本の辺で切る．
fn clip_polygon_at(
    points: &[Point],
    horizontal: bool,
    keep_greater: bool,
    bound: f64,
) -> Vec<Point> {
    let coordinate = |point: Point| if horizontal { point[0] } else { point[1] };
    let inside = |point: Point| {
        if keep_greater {
            coordinate(point) >= bound
        } else {
            coordinate(point) <= bound
        }
    };
    let crossing = |from: Point, to: Point| -> Point {
        let ratio = (bound - coordinate(from)) / (coordinate(to) - coordinate(from));
        let x = from[0] + (to[0] - from[0]) * ratio;
        let y = from[1] + (to[1] - from[1]) * ratio;
        // 辺の上の点は，辺の値にそろえる．
        if horizontal { [bound, y] } else { [x, bound] }
    };
    let mut result = Vec::new();
    for (index, current) in points.iter().enumerate() {
        let previous = points
            .get(
                index
                    .checked_sub(1)
                    .unwrap_or(points.len().saturating_sub(1)),
            )
            .copied()
            .unwrap_or(*current);
        match (inside(previous), inside(*current)) {
            (true, true) => result.push(*current),
            (false, true) => {
                result.push(crossing(previous, *current));
                result.push(*current);
            }
            (true, false) => result.push(crossing(previous, *current)),
            (false, false) => {}
        }
    }
    result
}
