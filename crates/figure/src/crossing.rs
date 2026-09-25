//! 空間の図で，ほかの線の奥を通る線を，交わる所で少し切る(`style.crossing_gap`)．
//!
//! 線が交わっているのではなく，前後にすれ違っていることを，奥の線の切れ目で示す．結び目の図と同じ描き方である．
//! 画面に投影した線分どうしの交点で，2本の線の奥行きを比べ，奥の線を，交点を中心に，線に沿って
//! `crossing_gap`の長さだけ描かない．手前の線は，どの線でもよい(ほかのオブジェクトの線でも，同じ線の
//! 別の所でもよい)．奥行きの差が`DEPTH_TOLERANCE`以下なら，空間で交わっているとみなして，切らない．
//! 線分は，画面を格子に区切った箱に分けておき，同じ箱に入る線分どうしだけを比べる．

use crate::figure::{Item, Path};

/// 空間で交わっているとみなす，奥行きの差(cm)．曲線を折れ線で近似した誤差を吸収する．
const DEPTH_TOLERANCE: f64 = 0.02;
/// 画面を区切る格子の，一辺の箱の数の上限．
const MAX_CELLS: usize = 256;

/// 画面の点(cm)．
type Point = [f64; 2];

/// 奥行きつきの折れ線．
pub struct Traced {
    /// 画面の折れ線．
    pub path: Path,
    /// 各点の奥行き(cm)．`path.points`と同じ順に並び，大きいほどカメラに近い．
    pub depths: Vec<f64>,
    /// ほかの線の奥を通る所で切る長さ(cm)．なければ切らない．
    pub gap: Option<f64>,
}

/// 空間の図の，描く要素．線は，切れ目を入れるまで，奥行きを持っておく．
pub enum Drawn {
    /// 奥行きつきの折れ線．
    Line(Traced),
    /// 線でない要素(ラベル，点の印)．
    Other(Item),
}

/// 画面の線分．どの線の，何番目の線分か，と，両端の奥行き．
struct Segment {
    line: usize,
    index: usize,
    from: Point,
    to: Point,
    depths: [f64; 2],
}

/// 線に切れ目を入れて，描く要素にする．切れ目を指定した線がなければ，そのまま並べる．
pub fn break_crossings(drawn: Vec<Drawn>) -> Vec<Item> {
    let lines: Vec<&Traced> = drawn
        .iter()
        .filter_map(|element| match element {
            Drawn::Line(traced) => Some(traced),
            Drawn::Other(_) => None,
        })
        .collect();
    let gaps: Vec<Vec<f64>> = if lines.iter().any(|traced| traced.gap.is_some()) {
        let segments = segments_of(&lines);
        let grid = Grid::new(&segments);
        lines
            .iter()
            .enumerate()
            .map(|(line, traced)| {
                if traced.gap.is_some() {
                    crossings(line, &segments, &grid)
                } else {
                    Vec::new()
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    let mut items = Vec::new();
    let mut line = 0;
    for element in drawn {
        match element {
            Drawn::Other(item) => items.push(item),
            Drawn::Line(traced) => {
                match (traced.gap, gaps.get(line)) {
                    (Some(gap), Some(cuts)) if !cuts.is_empty() => {
                        items.extend(
                            cut_path(&traced.path, cuts, gap)
                                .into_iter()
                                .map(Item::Path),
                        );
                    }
                    _ => items.push(Item::Path(traced.path)),
                }
                line = line.saturating_add(1);
            }
        }
    }
    items
}

fn segments_of(lines: &[&Traced]) -> Vec<Segment> {
    let mut segments = Vec::new();
    for (line, traced) in lines.iter().enumerate() {
        let pairs = traced.path.points.windows(2).zip(traced.depths.windows(2));
        for (index, (points, depths)) in pairs.enumerate() {
            if let ([from, to], [near, far]) = (points, depths) {
                segments.push(Segment {
                    line,
                    index,
                    from: *from,
                    to: *to,
                    depths: [*near, *far],
                });
            }
        }
    }
    segments
}

/// 線`line`が，ほかの線の奥を通る交点の，線の始まりから測った長さ(cm)．
fn crossings(line: usize, segments: &[Segment], grid: &Grid) -> Vec<f64> {
    let mut found = Vec::new();
    let mut start = 0.0;
    // 線分は，線の順に並んでいる．
    let first = segments.partition_point(|segment| segment.line < line);
    let own_segments = segments
        .iter()
        .skip(first)
        .take_while(|segment| segment.line == line);
    for own in own_segments {
        let length = distance(own.from, own.to);
        for other in grid.near(own).into_iter().filter_map(|k| segments.get(k)) {
            // 隣り合う線分は，端を共有するだけである．
            if other.line == line && other.index.abs_diff(own.index) <= 1 {
                continue;
            }
            if let Some([s, t]) = intersect(own, other)
                && lerp(other.depths, t) - lerp(own.depths, s) > DEPTH_TOLERANCE
            {
                found.push(start + s * length);
            }
        }
        start += length;
    }
    found
}

/// 2つの線分の交点の，それぞれの線分の上の割合．交わらないか，平行なら`None`．
fn intersect(own: &Segment, other: &Segment) -> Option<[f64; 2]> {
    let own_span = minus(own.to, own.from);
    let other_span = minus(other.to, other.from);
    let denominator = cross(own_span, other_span);
    let lengths = own_span[0].hypot(own_span[1]) * other_span[0].hypot(other_span[1]);
    if denominator.abs() <= f64::EPSILON * lengths {
        return None;
    }
    let offset = minus(other.from, own.from);
    let on_own = cross(offset, other_span) / denominator;
    let on_other = cross(offset, own_span) / denominator;
    let within = |ratio: f64| (0.0..=1.0).contains(&ratio);
    (within(on_own) && within(on_other)).then_some([on_own, on_other])
}

/// 折れ線から，各交点を中心に，線に沿って長さ`gap`の部分を除いた，残りの折れ線．矢じりは，
/// 終わりの端が残ったときだけ，最後の部分に付ける．
fn cut_path(path: &Path, cuts: &[f64], gap: f64) -> Vec<Path> {
    let lengths = cumulative_lengths(&path.points);
    let total = lengths.last().copied().unwrap_or(0.0);
    let mut removed: Vec<[f64; 2]> = cuts
        .iter()
        .map(|cut| [cut - gap / 2.0, cut + gap / 2.0])
        .collect();
    removed.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut kept = Vec::new();
    let mut current = 0.0;
    for [low, high] in removed {
        if low > current {
            kept.push([current, low.min(total)]);
        }
        current = f64::max(current, high);
    }
    if current < total {
        kept.push([current, total]);
    }
    let reaches_end = kept.last().is_some_and(|range| range[1] >= total);
    let last = kept.len().saturating_sub(1);
    kept.iter()
        .enumerate()
        .filter(|(_, [low, high])| high > low)
        .map(|(index, [low, high])| Path {
            points: sub_polyline(&path.points, &lengths, *low, *high),
            stroke: path.stroke,
            arrow: if index == last && reaches_end {
                path.arrow.clone()
            } else {
                None
            },
        })
        .collect()
}

/// 各点までの，始まりから測った長さ．
fn cumulative_lengths(points: &[Point]) -> Vec<f64> {
    let mut total = 0.0;
    let mut lengths = vec![0.0];
    for pair in points.windows(2) {
        if let [from, to] = pair {
            total += distance(*from, *to);
            lengths.push(total);
        }
    }
    lengths
}

/// 折れ線の，長さ`low`から`high`までの部分．両端は，線分の上に補う．
fn sub_polyline(points: &[Point], lengths: &[f64], low: f64, high: f64) -> Vec<Point> {
    let mut part = vec![point_at(points, lengths, low)];
    part.extend(
        points
            .iter()
            .zip(lengths)
            .filter(|(_, length)| **length > low && **length < high)
            .map(|(point, _)| *point),
    );
    part.push(point_at(points, lengths, high));
    part
}

/// 折れ線の，始まりから長さ`at`の点．
fn point_at(points: &[Point], lengths: &[f64], at: f64) -> Point {
    let pairs = points.windows(2).zip(lengths.windows(2));
    for (pair, span) in pairs {
        if let ([from, to], [start, end]) = (pair, span)
            && at <= *end
        {
            let width = end - start;
            let ratio = if width > 0.0 {
                (at - start) / width
            } else {
                0.0
            };
            return [
                from[0] + (to[0] - from[0]) * ratio,
                from[1] + (to[1] - from[1]) * ratio,
            ];
        }
    }
    points.last().copied().unwrap_or([0.0; 2])
}

/// 画面を区切る格子．各箱に，その箱にかかる線分の番号を持つ．
struct Grid {
    min: Point,
    cell: f64,
    columns: usize,
    rows: usize,
    cells: Vec<Vec<usize>>,
}

impl Grid {
    fn new(segments: &[Segment]) -> Self {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        for segment in segments {
            for point in [segment.from, segment.to] {
                min = [min[0].min(point[0]), min[1].min(point[1])];
                max = [max[0].max(point[0]), max[1].max(point[1])];
            }
        }
        let extent = f64::max(max[0] - min[0], max[1] - min[1]);
        let side = count_to_f64(segments.len())
            .sqrt()
            .ceil()
            .clamp(1.0, count_to_f64(MAX_CELLS));
        let cell = if extent.is_finite() && extent > 0.0 {
            extent / side
        } else {
            1.0
        };
        let count = |span: f64| index_of(span / cell).saturating_add(1).min(MAX_CELLS);
        let (columns, rows) = (count(max[0] - min[0]), count(max[1] - min[1]));
        let mut grid = Self {
            min: if min[0].is_finite() { min } else { [0.0; 2] },
            cell,
            columns,
            rows,
            cells: vec![Vec::new(); columns.saturating_mul(rows)],
        };
        for (number, segment) in segments.iter().enumerate() {
            for cell in grid.cells_of(segment) {
                if let Some(entries) = grid.cells.get_mut(cell) {
                    entries.push(number);
                }
            }
        }
        grid
    }

    /// 線分の外枠にかかる，箱の番号．
    fn cells_of(&self, segment: &Segment) -> Vec<usize> {
        let column =
            |x: f64| index_of((x - self.min[0]) / self.cell).min(self.columns.saturating_sub(1));
        let row = |y: f64| index_of((y - self.min[1]) / self.cell).min(self.rows.saturating_sub(1));
        let (x0, x1) = (column(segment.from[0]), column(segment.to[0]));
        let (y0, y1) = (row(segment.from[1]), row(segment.to[1]));
        let mut cells = Vec::new();
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                cells.push(y.saturating_mul(self.columns).saturating_add(x));
            }
        }
        cells
    }

    /// 線分と同じ箱に入る線分の番号．重なりは除く．
    fn near(&self, segment: &Segment) -> Vec<usize> {
        let mut found: Vec<usize> = self
            .cells_of(segment)
            .into_iter()
            .filter_map(|cell| self.cells.get(cell))
            .flatten()
            .copied()
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    }
}

/// 0以上の数を，切り捨てて番号にする．負の数や有限でない数は0にする．
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn index_of(value: f64) -> usize {
    if value.is_finite() && value > 0.0 {
        // 0以上MAX_CELLS以下の数である．
        value.min(count_to_f64(MAX_CELLS)) as usize
    } else {
        0
    }
}

fn count_to_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

fn lerp([from, to]: [f64; 2], t: f64) -> f64 {
    from + (to - from) * t
}

fn minus(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}

fn cross(a: Point, b: Point) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

fn distance(a: Point, b: Point) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
