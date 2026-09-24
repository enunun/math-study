//! 式で書いた曲面を，三角形の網にして，視線から見た隠れ方と，輪郭を求める．
//!
//! 投影は平行なので，視線は画面の1点に写る．点が網に隠れているかは，その点の画面の位置が，
//! 投影した三角形の中にあり，三角形の奥行きが，点よりカメラに近いかで決まる．
//! 輪郭は，頂点の法線と視線の内積が0になる所で，網の辺の上で補間して結ぶ．

use std::collections::{BTreeMap, BTreeSet};

use crate::refine::{Curve, Node};

/// 空間の点．
pub type Point3 = [f64; 3];

/// 画面の向き．
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    /// 画面の右向き．
    pub right: Point3,
    /// 画面の上向き．
    pub up: Point3,
    /// 点からカメラへの向き．
    pub toward: Point3,
}

/// 隠れ方の判定に使う，投影した三角形．
struct Triangle {
    /// 画面の位置．
    screen: [[f64; 2]; 3],
    /// 各頂点の奥行き(カメラに近いほど大きい)．
    depth: [f64; 3],
    min: [f64; 2],
    max: [f64; 2],
    /// 重心座標の計算に使う，分母．
    denominator: f64,
}

/// 投影した三角形を，画面の一様な格子の升目に振り分けた表．点の問い合わせで，その点の升目にある
/// 三角形だけを調べ，全三角形を調べずに済ませる．
struct ScreenGrid {
    origin: [f64; 2],
    cell: [f64; 2],
    counts: [usize; 2],
    /// 升目ごとの，三角形の番号．
    buckets: Vec<Vec<usize>>,
}

/// 値が入る升目の番号．範囲の外は，端の升目にする．
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn cell_of(value: f64, origin: f64, size: f64, count: usize) -> usize {
    let last = count.saturating_sub(1);
    let index = ((value - origin) / size).floor();
    if index.is_nan() || index <= 0.0 {
        0
    } else if index >= f64::from(u32::try_from(last).unwrap_or(u32::MAX)) {
        last
    } else {
        // 0以上last以下の整数である．
        index as usize
    }
}

impl ScreenGrid {
    /// 升目の数の上限(1辺)．
    const MAX_CELLS: usize = 256;

    fn new(triangles: &[Triangle]) -> Self {
        let (mut low, mut high) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for triangle in triangles {
            low = [low[0].min(triangle.min[0]), low[1].min(triangle.min[1])];
            high = [high[0].max(triangle.max[0]), high[1].max(triangle.max[1])];
        }
        // 三角形の数の平方根を，1辺の升目の数にする．
        let side = (1..=Self::MAX_CELLS)
            .find(|n| n.saturating_mul(*n) >= triangles.len())
            .unwrap_or(Self::MAX_CELLS);
        let counts = [side; 2];
        let cell_width = |width: f64| {
            if width.is_finite() && width > 0.0 {
                width / f64::from(u32::try_from(side).unwrap_or(1))
            } else {
                1.0
            }
        };
        let cell = [cell_width(high[0] - low[0]), cell_width(high[1] - low[1])];
        let origin = if low.iter().all(|c| c.is_finite()) {
            low
        } else {
            [0.0; 2]
        };
        let mut buckets = vec![Vec::new(); side.saturating_mul(side)];
        for (index, triangle) in triangles.iter().enumerate() {
            let locate = |at: [f64; 2]| {
                [
                    cell_of(at[0], origin[0], cell[0], side),
                    cell_of(at[1], origin[1], cell[1], side),
                ]
            };
            let first = locate(triangle.min);
            let last = locate(triangle.max);
            for column in first[0]..=last[0] {
                for row in first[1]..=last[1] {
                    if let Some(bucket) =
                        buckets.get_mut(row.saturating_mul(side).saturating_add(column))
                    {
                        bucket.push(index);
                    }
                }
            }
        }
        Self {
            origin,
            cell,
            counts,
            buckets,
        }
    }

    /// 画面の点の升目にある，三角形の番号．
    fn candidates(&self, at: [f64; 2]) -> &[usize] {
        let column = cell_of(at[0], self.origin[0], self.cell[0], self.counts[0]);
        let row = cell_of(at[1], self.origin[1], self.cell[1], self.counts[1]);
        self.buckets
            .get(row.saturating_mul(self.counts[0]).saturating_add(column))
            .map_or(&[], Vec::as_slice)
    }
}

impl Triangle {
    /// 画面の点が三角形の中にあり，三角形が，その点(の奥行き`depth`)よりカメラに近いか．
    fn covers(&self, at: [f64; 2], depth: f64) -> bool {
        if at[0] < self.min[0] || at[0] > self.max[0] || at[1] < self.min[1] || at[1] > self.max[1]
        {
            return false;
        }
        let [a, b, c] = self.screen;
        let first =
            ((b[1] - c[1]) * (at[0] - c[0]) + (c[0] - b[0]) * (at[1] - c[1])) / self.denominator;
        let second =
            ((c[1] - a[1]) * (at[0] - c[0]) + (a[0] - c[0]) * (at[1] - c[1])) / self.denominator;
        let weights = [first, second, 1.0 - first - second];
        let inside = weights.iter().all(|w| *w >= -1e-9);
        let surface_depth: f64 = weights.iter().zip(self.depth).map(|(w, d)| w * d).sum();
        inside && surface_depth > depth + 1e-12
    }
}

/// 頂点の番号の組．辺の名前に使う．
type Edge = (usize, usize);

/// 輪郭の上の点と，その点での曲面の法線(長さ1)．
pub type Rim = (Point3, Point3);

/// 2つの変数から，曲面の点を返す関数．値がなければ`None`を返す．
pub type SurfaceFn<'a> = &'a dyn Fn(f64, f64) -> Option<Point3>;

/// 値が0になる線の上の点．`rim`は，点と法線で，`params`は，曲面のパラメータ(2つの変数の値)である．
#[derive(Debug, Clone, Copy)]
struct LevelPoint {
    rim: Rim,
    params: [f64; 2],
}

/// 曲面の三角形の網．
pub struct Mesh {
    /// 頂点．値のない点は`None`である．行が第1変数，列が第2変数の方向に並ぶ．
    points: Vec<Option<Point3>>,
    /// 1行の頂点の数．
    columns: usize,
    /// 頂点の番号の三角形．
    indices: Vec<[usize; 3]>,
    /// 隠れ方の判定に使う，投影した三角形．
    triangles: Vec<Triangle>,
    /// 投影した三角形の，画面の升目への振り分け．
    grid: ScreenGrid,
    /// 視線の向きに，点をカメラへずらす長さ．曲面の上の点を，網の外側の点として扱うためである．
    offset: f64,
    /// 座標の大きさ．合っているかを比べる誤差の基準にする．
    scale: f64,
    frame: Frame,
    /// 各変数の範囲．
    domain: [[f64; 2]; 2],
    /// 各変数の方向の分割数．
    divisions: [usize; 2],
}

fn fraction(index: usize, count: usize) -> f64 {
    f64::from(u32::try_from(index).unwrap_or(u32::MAX))
        / f64::from(u32::try_from(count).unwrap_or(1).max(1))
}

fn lerp(low: f64, high: f64, t: f64) -> f64 {
    low + (high - low) * t
}

fn zip(a: Point3, b: Point3, f: impl Fn(f64, f64) -> f64) -> Point3 {
    [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2])]
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn minus(a: Point3, b: Point3) -> Point3 {
    zip(a, b, |x, y| x - y)
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn distance(a: Point3, b: Point3) -> f64 {
    let d = minus(a, b);
    dot(d, d).sqrt()
}

/// 行と列の番号から，頂点の番号を作る．
fn vertex(row: usize, column: usize, columns: usize) -> usize {
    row.saturating_mul(columns).saturating_add(column)
}

impl Mesh {
    /// 曲面を，`mesh`(各変数の方向の分割数)の網にする．`point_at`は，2つの変数から点を返し，
    /// 値がなければ`None`を返す．
    pub fn build(
        point_at: &dyn Fn(f64, f64) -> Option<Point3>,
        domain: [[f64; 2]; 2],
        mesh: [usize; 2],
        frame: Frame,
    ) -> Self {
        let [rows, columns] = mesh;
        let width = columns.saturating_add(1);
        let [[u0, u1], [v0, v1]] = domain;
        let mut points = Vec::new();
        for i in 0..=rows {
            for j in 0..=columns {
                points.push(point_at(
                    lerp(u0, u1, fraction(i, rows)),
                    lerp(v0, v1, fraction(j, columns)),
                ));
            }
        }
        let mut indices = Vec::new();
        for i in 0..rows {
            for j in 0..columns {
                let a = vertex(i, j, width);
                let b = vertex(i.saturating_add(1), j, width);
                let c = vertex(i.saturating_add(1), j.saturating_add(1), width);
                let d = vertex(i, j.saturating_add(1), width);
                indices.push([a, b, c]);
                indices.push([a, c, d]);
            }
        }
        let scale = points
            .iter()
            .flatten()
            .flat_map(|p| p.iter().map(|c| c.abs()))
            .fold(1.0_f64, f64::max);
        // 網が曲面から外れる大きさを，各升目の中心で見積もる．
        let mut sag = 0.0_f64;
        for i in 0..rows {
            for j in 0..columns {
                let (i1, j1) = (i.saturating_add(1), j.saturating_add(1));
                let corners = [
                    vertex(i, j, width),
                    vertex(i1, j, width),
                    vertex(i1, j1, width),
                    vertex(i, j1, width),
                ]
                .map(|k| points.get(k).copied().flatten());
                let [Some(a), Some(b), Some(c), Some(d)] = corners else {
                    continue;
                };
                let middle = point_at(
                    lerp(u0, u1, f64::midpoint(fraction(i, rows), fraction(i1, rows))),
                    lerp(
                        v0,
                        v1,
                        f64::midpoint(fraction(j, columns), fraction(j1, columns)),
                    ),
                );
                if let Some(middle) = middle {
                    let sum = zip(zip(a, b, |x, y| x + y), zip(c, d, |x, y| x + y), |x, y| {
                        x + y
                    });
                    sag = sag.max(distance(middle, sum.map(|x| x / 4.0)));
                }
            }
        }
        let mut result = Self {
            points,
            columns: width,
            indices,
            triangles: Vec::new(),
            grid: ScreenGrid::new(&[]),
            offset: 2.0 * sag + 1e-9 * scale,
            scale,
            frame,
            domain,
            divisions: mesh,
        };
        result.triangles = result.project_triangles();
        result.grid = ScreenGrid::new(&result.triangles);
        result
    }

    /// 三角形を直接指定して，網を作る．複体(平らな面の集まり)のように，パラメータの升目からではなく，
    /// 頂点と三角形(頂点の番号)が決まっているときに使う．面は平らなので，曲率のずれ(`sag`)はなく，
    /// `offset`(隠れ判定でカメラ側へずらす量)は，丸め誤差だけを吸収するごく小さい値でよい．
    /// この網は，`hides`(ほかのオブジェクトを隠すか)にだけ使い，`silhouette`や`boundary`は使わない
    /// (曲面のパラメータの升目を前提にしており，直接指定した三角形には合わない)．
    pub fn from_triangles(points: Vec<Point3>, indices: Vec<[usize; 3]>, frame: Frame) -> Self {
        let scale = points
            .iter()
            .flat_map(|p| p.iter().map(|c| c.abs()))
            .fold(1.0_f64, f64::max);
        let mut result = Self {
            points: points.into_iter().map(Some).collect(),
            columns: 0,
            indices,
            triangles: Vec::new(),
            grid: ScreenGrid::new(&[]),
            offset: 1e-9 * scale,
            scale,
            frame,
            domain: [[0.0, 0.0], [0.0, 0.0]],
            divisions: [0, 0],
        };
        result.triangles = result.project_triangles();
        result.grid = ScreenGrid::new(&result.triangles);
        result
    }

    /// 頂点の，曲面のパラメータ(2つの変数の値)．
    fn vertex_params(&self, vertex: usize) -> [f64; 2] {
        let row = vertex.checked_div(self.columns).unwrap_or(0);
        let column = vertex.checked_rem(self.columns).unwrap_or(0);
        let [[u0, u1], [v0, v1]] = self.domain;
        [
            lerp(u0, u1, fraction(row, self.divisions[0])),
            lerp(v0, v1, fraction(column, self.divisions[1])),
        ]
    }

    fn point(&self, index: usize) -> Option<Point3> {
        self.points.get(index).copied().flatten()
    }

    fn project_triangles(&self) -> Vec<Triangle> {
        let project = |p: Point3| [dot(p, self.frame.right), dot(p, self.frame.up)];
        self.indices
            .iter()
            .filter_map(|corners| {
                let space_corners = corners.map(|k| self.point(k));
                let [Some(first), Some(second), Some(third)] = space_corners else {
                    return None;
                };
                let screen = [project(first), project(second), project(third)];
                let [top, left, right] = screen;
                let denominator = (left[1] - right[1]) * (top[0] - right[0])
                    + (right[0] - left[0]) * (top[1] - right[1]);
                (denominator.abs() > 1e-14).then(|| Triangle {
                    screen,
                    depth: [first, second, third].map(|point| dot(point, self.frame.toward)),
                    min: [
                        top[0].min(left[0]).min(right[0]),
                        top[1].min(left[1]).min(right[1]),
                    ],
                    max: [
                        top[0].max(left[0]).max(right[0]),
                        top[1].max(left[1]).max(right[1]),
                    ],
                    denominator,
                })
            })
            .collect()
    }

    /// 点が，この網に隠れているか．点から，カメラへ向かう視線が，網の三角形に当たれば，隠れている．
    /// 曲面の上の点も，網の外側にあるものとして判定できるよう，カメラの側へわずかにずらして調べる．
    /// 画面の升目にある三角形だけを調べる．
    #[must_use]
    pub fn hides(&self, point: Point3) -> bool {
        let (at, depth) = self.query(point);
        self.grid
            .candidates(at)
            .iter()
            .filter_map(|index| self.triangles.get(*index))
            .any(|triangle| triangle.covers(at, depth))
    }

    /// `hides`と同じ答えを，全三角形を調べて求める．`hides`の照合に使う．
    #[must_use]
    pub fn hides_exhaustive(&self, point: Point3) -> bool {
        let (at, depth) = self.query(point);
        self.triangles
            .iter()
            .any(|triangle| triangle.covers(at, depth))
    }

    /// 点を，カメラの側へずらし，画面の位置と，奥行きにする．
    fn query(&self, point: Point3) -> ([f64; 2], f64) {
        let shifted = zip(point, self.frame.toward, |p, t| p + self.offset * t);
        (
            [dot(shifted, self.frame.right), dot(shifted, self.frame.up)],
            dot(shifted, self.frame.toward),
        )
    }

    /// 頂点の法線(長さ1)．位置が同じ頂点(継ぎ目や極)は，法線を合わせて，同じ値にする．
    fn vertex_normals(&self) -> Vec<Point3> {
        let mut normals = vec![[0.0_f64; 3]; self.points.len()];
        for corners in &self.indices {
            let [a, b, c] = corners.map(|k| self.point(k));
            let (Some(a), Some(b), Some(c)) = (a, b, c) else {
                continue;
            };
            let normal = cross(minus(b, a), minus(c, a));
            for k in corners {
                if let Some(sum) = normals.get_mut(*k) {
                    *sum = zip(*sum, normal, |x, y| x + y);
                }
            }
        }
        // 位置が同じ頂点の法線を，足し合わせる．
        let quantum = 1e-9 * self.scale;
        let key = |p: Point3| p.map(|c| ((c / quantum).round() + 0.0).to_bits());
        let mut merged: BTreeMap<[u64; 3], Point3> = BTreeMap::new();
        for (index, normal) in normals.iter().enumerate() {
            if let Some(p) = self.point(index) {
                let sum = merged.entry(key(p)).or_insert([0.0; 3]);
                *sum = zip(*sum, *normal, |x, y| x + y);
            }
        }
        (0..self.points.len())
            .map(|index| {
                let Some(p) = self.point(index) else {
                    return [0.0; 3];
                };
                let normal = merged.get(&key(p)).copied().unwrap_or([0.0; 3]);
                let length = dot(normal, normal).sqrt();
                if length > 1e-12 {
                    normal.map(|x| x / length)
                } else {
                    [0.0; 3]
                }
            })
            .collect()
    }

    /// 輪郭の上の点が，この網に隠れているか．輪郭は，曲面の縁の見える所なので，そのままでは，
    /// 曲面自身の三角形に，ぎりぎりで当たる．曲面の法線の両側へずらした点が，どちらも隠れていれば，
    /// 隠れているとする(片側が空いていれば，輪郭は見えている)．
    #[must_use]
    pub fn rim_hidden(&self, (point, normal): Rim) -> bool {
        [1.0, -1.0]
            .into_iter()
            .all(|side| self.hides(zip(point, normal, |p, n| p + side * self.offset * n)))
    }

    /// 輪郭の線．視線が曲面に接する所を，網の辺の上で補間して結んだ，空間の折れ線である．
    /// 閉じた輪郭は，始めと終わりが同じ点になる．
    #[must_use]
    pub fn silhouette(&self) -> Vec<Vec<Rim>> {
        let normals = self.vertex_normals();
        let facing: Vec<f64> = normals
            .iter()
            .map(|normal| dot(*normal, self.frame.toward))
            .collect();
        let (chains, points) = self.level_chains(&facing, &normals);
        let lines = chains
            .iter()
            .map(|ids| {
                ids.iter()
                    .filter_map(|edge| points.get(edge).map(|found| found.rim))
                    .collect()
            })
            .collect();
        self.tidy(lines)
    }

    /// 平面`normal・p = offset`による，曲面の切り口の線．網の辺の上で，平面の式の値が0になる点を結び，
    /// 厳密な式`surface`の上へ磨き，曲がりに合わせて点を足した，空間の折れ線である．
    #[must_use]
    pub fn cut(&self, normal: Point3, offset: f64, surface: SurfaceFn) -> Vec<Vec<Rim>> {
        let values: Vec<f64> = (0..self.points.len())
            .map(|k| self.point(k).map_or(0.0, |p| dot(normal, p) - offset))
            .collect();
        let (chains, points) = self.level_chains(&values, &self.vertex_normals());
        let residual = |params: &[f64]| -> Option<Vec<f64>> {
            let [u, v] = params else {
                return None;
            };
            surface(*u, *v).map(|p| vec![dot(normal, p) - offset])
        };
        let position = |params: &[f64]| -> Option<Point3> {
            let [u, v] = params else {
                return None;
            };
            surface(*u, *v)
        };
        let curve = Curve {
            residual: &residual,
            position: &position,
            bounds: self.domain.to_vec(),
            scale: self.scale,
        };
        let lines = chains
            .iter()
            .map(|ids| {
                let nodes: Vec<Node> = ids
                    .iter()
                    .filter_map(|edge| points.get(edge))
                    .map(|found| Node {
                        params: found.params.to_vec(),
                        point: found.rim.0,
                    })
                    .collect();
                curve
                    .refine(&nodes)
                    .into_iter()
                    .map(|node| (node.point, [0.0; 3]))
                    .collect()
            })
            .collect();
        self.tidy(lines)
    }

    /// 折れ線の端が重なるものをつなぎ，同じ点の続きを1つにする．
    fn tidy(&self, lines: Vec<Vec<Rim>>) -> Vec<Vec<Rim>> {
        join_chains(lines, 1e-7 * self.scale)
            .into_iter()
            .filter_map(|line| finish_line(line, 1e-7 * self.scale))
            .collect()
    }

    /// 頂点ごとの値が0になる所を，網の辺の上で補間して結んだ線．辺ごとの点と，辺の並びを返す．
    /// 閉じた線は，始めと終わりが同じ辺になる．
    fn level_chains(
        &self,
        values: &[f64],
        normals: &[Point3],
    ) -> (Vec<Vec<Edge>>, BTreeMap<Edge, LevelPoint>) {
        let value = |k: usize| values.get(k).copied().unwrap_or(0.0);
        // 辺の上の，値が0の点と，同じ三角形の中で結ばれる，2つの辺．
        let mut positions: BTreeMap<Edge, LevelPoint> = BTreeMap::new();
        let mut links: BTreeMap<Edge, Vec<Edge>> = BTreeMap::new();
        for corners in &self.indices {
            if corners.iter().any(|k| self.point(*k).is_none()) {
                continue;
            }
            let [v0, v1, v2] = *corners;
            let mut crossing = Vec::new();
            for (from, to) in [(v0, v1), (v1, v2), (v2, v0)] {
                if (value(from) > 0.0) == (value(to) > 0.0) {
                    continue;
                }
                let (low, high) = (from.min(to), from.max(to));
                if let (Some(start), Some(end)) = (self.point(low), self.point(high)) {
                    let ratio = value(low) / (value(low) - value(high));
                    let point = zip(start, end, |from, to| lerp(from, to, ratio));
                    let normal = match (normals.get(low), normals.get(high)) {
                        (Some(low_normal), Some(high_normal)) => {
                            zip(*low_normal, *high_normal, |from, to| lerp(from, to, ratio))
                        }
                        _ => [0.0; 3],
                    };
                    let (low_params, high_params) =
                        (self.vertex_params(low), self.vertex_params(high));
                    positions.insert(
                        (low, high),
                        LevelPoint {
                            rim: (point, normal),
                            params: [
                                lerp(low_params[0], high_params[0], ratio),
                                lerp(low_params[1], high_params[1], ratio),
                            ],
                        },
                    );
                    crossing.push((low, high));
                }
            }
            if let [first, second] = crossing.as_slice() {
                links.entry(*first).or_default().push(*second);
                links.entry(*second).or_default().push(*first);
            }
        }
        (chain_ids(&links), positions)
    }

    /// 定義域の縁の線．4つの辺のうち，1点に縮んでおらず，継ぎ目でもない辺の，空間の折れ線である．
    /// 向かい合う2つの辺が(向きを逆にしても)点ごとに重なるなら，そこは継ぎ目なので，どちらも描かない．
    #[must_use]
    pub fn boundary(&self) -> Vec<Vec<Point3>> {
        let rows = self.points.len().checked_div(self.columns).unwrap_or(0);
        let last_row = rows.saturating_sub(1);
        let last_column = self.columns.saturating_sub(1);
        let edges: [Vec<usize>; 4] = [
            (0..self.columns).collect(),
            (0..self.columns)
                .map(|j| vertex(last_row, j, self.columns))
                .collect(),
            (0..rows).map(|i| vertex(i, 0, self.columns)).collect(),
            (0..rows)
                .map(|i| vertex(i, last_column, self.columns))
                .collect(),
        ];
        let seams = [
            self.is_seam(&edges[0], &edges[1]),
            self.is_seam(&edges[2], &edges[3]),
        ];
        edges
            .iter()
            .enumerate()
            .filter(|(at, _)| !seams.get(at / 2).copied().unwrap_or(false))
            .filter_map(|(_, edge)| {
                let line: Option<Vec<Point3>> = edge.iter().map(|k| self.point(*k)).collect();
                let line = line?;
                let length: f64 = line
                    .windows(2)
                    .map(|pair| match pair {
                        [a, b] => distance(*a, *b),
                        _ => 0.0,
                    })
                    .sum();
                (length > 1e-9 * self.scale).then_some(line)
            })
            .collect()
    }

    /// 向かい合う2つの辺が，同じ向きか逆の向きで，点ごとに重なるか(継ぎ目か)．
    fn is_seam(&self, first: &[usize], second: &[usize]) -> bool {
        let tolerance = 1e-7 * self.scale;
        let close = |a: &usize, b: &usize| match (self.point(*a), self.point(*b)) {
            (Some(p), Some(q)) => distance(p, q) <= tolerance,
            _ => false,
        };
        let same = first.iter().zip(second).all(|(a, b)| close(a, b));
        let reversed = first
            .iter()
            .zip(second.iter().rev())
            .all(|(a, b)| close(a, b));
        !first.is_empty() && (same || reversed)
    }
}

/// 名前でつながった点を，名前の並びにたどる．端(つながりが1つの点)から始め，残りは輪である．
/// 輪は，始めの名前に戻って，閉じる．
fn chain_ids<K: Ord + Copy>(links: &BTreeMap<K, Vec<K>>) -> Vec<Vec<K>> {
    let mut visited: BTreeSet<K> = BTreeSet::new();
    let mut chains = Vec::new();
    let ends = links
        .iter()
        .filter(|(_, next)| next.len() == 1)
        .map(|(key, _)| *key);
    for start in ends.chain(links.keys().copied()) {
        if visited.contains(&start) {
            continue;
        }
        let mut line = Vec::new();
        let mut current = Some(start);
        let mut last = start;
        while let Some(key) = current {
            last = key;
            visited.insert(key);
            line.push(key);
            current = links
                .get(&key)
                .into_iter()
                .flatten()
                .copied()
                .find(|candidate| !visited.contains(candidate));
        }
        if line.len() >= 3
            && last != start
            && links.get(&last).is_some_and(|next| next.contains(&start))
        {
            line.push(start);
        }
        chains.push(line);
    }
    chains
}

/// 端が(誤差の中で)重なる折れ線を，つなぐ．曲面の継ぎ目で切れた輪郭が，1本になる．
fn join_chains(mut lines: Vec<Vec<Rim>>, tolerance: f64) -> Vec<Vec<Rim>> {
    let near = |p: Option<&Rim>, q: Option<&Rim>| matches!((p, q), (Some(p), Some(q)) if distance(p.0, q.0) < tolerance);
    loop {
        let mut joined = None;
        'search: for a in 0..lines.len() {
            for b in a.saturating_add(1)..lines.len() {
                let (Some(x), Some(y)) = (lines.get(a), lines.get(b)) else {
                    continue;
                };
                let combined: Option<Vec<Rim>> = if near(x.last(), y.first()) {
                    Some(x.iter().chain(y.iter().skip(1)).copied().collect())
                } else if near(y.last(), x.first()) {
                    Some(y.iter().chain(x.iter().skip(1)).copied().collect())
                } else if near(x.last(), y.last()) {
                    Some(x.iter().chain(y.iter().rev().skip(1)).copied().collect())
                } else if near(x.first(), y.first()) {
                    Some(x.iter().rev().chain(y.iter().skip(1)).copied().collect())
                } else {
                    None
                };
                if let Some(line) = combined {
                    joined = Some((a, b, line));
                    break 'search;
                }
            }
        }
        let Some((a, b, line)) = joined else {
            return lines;
        };
        lines.remove(b);
        lines.remove(a);
        lines.push(line);
    }
}

/// 同じ点が続く所を1つにし，始めと終わりが(誤差の中で)重なる線は，ぴったり閉じる．
/// 点が2つに満たない線は，捨てる．
fn finish_line(line: Vec<Rim>, tolerance: f64) -> Option<Vec<Rim>> {
    let mut points: Vec<Rim> = Vec::with_capacity(line.len());
    for point in line {
        if points
            .last()
            .is_none_or(|last| distance(last.0, point.0) > 1e-12)
        {
            points.push(point);
        }
    }
    if points.len() >= 3
        && let (Some(first), Some(last)) = (points.first().copied(), points.last_mut())
        && distance(first.0, last.0) < tolerance
    {
        *last = first;
    }
    (points.len() >= 2).then_some(points)
}

// ---- 2つの網の交線 ----

/// 空間の三角形．
type Solid = [Point3; 3];

/// 三角形を，空間の一様な格子の升目に振り分けた表．
struct SpaceGrid {
    origin: Point3,
    cell: f64,
    side: usize,
    buckets: Vec<Vec<usize>>,
}

impl SpaceGrid {
    /// 1辺の升目の数の上限．
    const MAX_CELLS: usize = 64;

    fn new(triangles: &[Solid]) -> Self {
        let mut low = [f64::INFINITY; 3];
        let mut high = [f64::NEG_INFINITY; 3];
        for corner in triangles.iter().flatten() {
            low = zip(low, *corner, f64::min);
            high = zip(high, *corner, f64::max);
        }
        // 三角形の数の立方根を，1辺の升目の数にする．
        let side = (1..=Self::MAX_CELLS)
            .find(|n| n.saturating_mul(*n).saturating_mul(*n) >= triangles.len())
            .unwrap_or(Self::MAX_CELLS);
        let extent = high
            .iter()
            .zip(&low)
            .map(|(top, bottom)| top - bottom)
            .fold(0.0_f64, f64::max);
        let cell = if extent.is_finite() && extent > 0.0 {
            extent / f64::from(u32::try_from(side).unwrap_or(1))
        } else {
            1.0
        };
        let origin = if low.iter().all(|c| c.is_finite()) {
            low
        } else {
            [0.0; 3]
        };
        let mut buckets = vec![Vec::new(); side.saturating_pow(3)];
        for (index, triangle) in triangles.iter().enumerate() {
            let (first, last) = Self::cells_of(origin, cell, side, triangle);
            for x in first[0]..=last[0] {
                for y in first[1]..=last[1] {
                    for z in first[2]..=last[2] {
                        if let Some(bucket) = buckets.get_mut(Self::slot(side, [x, y, z])) {
                            bucket.push(index);
                        }
                    }
                }
            }
        }
        Self {
            origin,
            cell,
            side,
            buckets,
        }
    }

    fn slot(side: usize, [x, y, z]: [usize; 3]) -> usize {
        x.saturating_add(side.saturating_mul(y.saturating_add(side.saturating_mul(z))))
    }

    /// 三角形の外枠が重なる，升目の範囲(両端を含む)．
    fn cells_of(
        origin: Point3,
        cell: f64,
        side: usize,
        triangle: &Solid,
    ) -> ([usize; 3], [usize; 3]) {
        let mut low = [f64::INFINITY; 3];
        let mut high = [f64::NEG_INFINITY; 3];
        for corner in triangle {
            low = zip(low, *corner, f64::min);
            high = zip(high, *corner, f64::max);
        }
        let locate = |at: Point3| {
            [
                cell_of(at[0], origin[0], cell, side),
                cell_of(at[1], origin[1], cell, side),
                cell_of(at[2], origin[2], cell, side),
            ]
        };
        (locate(low), locate(high))
    }

    /// 三角形の外枠と重なる升目にある，三角形の番号(重複を含む)．
    fn candidates(&self, triangle: &Solid) -> impl Iterator<Item = usize> + '_ {
        let (first, last) = Self::cells_of(self.origin, self.cell, self.side, triangle);
        let side = self.side;
        (first[0]..=last[0]).flat_map(move |x| {
            (first[1]..=last[1]).flat_map(move |y| {
                (first[2]..=last[2]).flat_map(move |z| {
                    self.buckets
                        .get(Self::slot(side, [x, y, z]))
                        .into_iter()
                        .flatten()
                        .copied()
                })
            })
        })
    }
}

/// 三角形が平面`normal・(p - origin) = 0`と交わる線分の端(2点)．平面の上の辺は，その辺である．
fn plane_segment(triangle: &Solid, distances: [f64; 3]) -> Option<[Point3; 2]> {
    let mut points: Vec<Point3> = Vec::new();
    for (corner, distance_to_plane) in triangle.iter().zip(distances) {
        if distance_to_plane == 0.0 {
            points.push(*corner);
        }
    }
    for (from, to) in [(0, 1), (1, 2), (2, 0)] {
        let (Some(start), Some(end)) = (triangle.get(from), triangle.get(to)) else {
            continue;
        };
        let (Some(before), Some(after)) = (distances.get(from), distances.get(to)) else {
            continue;
        };
        if before * after < 0.0 {
            let ratio = before / (before - after);
            points.push(zip(*start, *end, |s, e| lerp(s, e, ratio)));
        }
    }
    match points.as_slice() {
        [first, .., last] => Some([*first, *last]),
        _ => None,
    }
}

/// 2つの三角形が交わる線分．平面が平行か，同じ平面の上にあるか，交わらなければ`None`である．
fn triangle_intersection(first: &Solid, second: &Solid, scale: f64) -> Option<[Point3; 2]> {
    let normal = |t: &Solid| cross(minus(t[1], t[0]), minus(t[2], t[0]));
    let (first_normal, second_normal) = (normal(first), normal(second));
    let lengths = [
        dot(first_normal, first_normal).sqrt(),
        dot(second_normal, second_normal).sqrt(),
    ];
    if lengths.iter().any(|length| *length < 1e-14 * scale * scale) {
        return None;
    }
    // 頂点から，もう一方の平面までの符号つきの距離(の定数倍)．平面の上の頂点は，0にそろえる．
    let signed = |triangle: &Solid, origin: Point3, plane_normal: Point3, length: f64| {
        triangle.map(|corner| {
            let value = dot(plane_normal, minus(corner, origin));
            if value.abs() < 1e-12 * length * scale {
                0.0
            } else {
                value
            }
        })
    };
    let to_second = signed(first, second[0], second_normal, lengths[1]);
    let to_first = signed(second, first[0], first_normal, lengths[0]);
    let same_side = |d: &[f64; 3]| d.iter().all(|v| *v > 0.0) || d.iter().all(|v| *v < 0.0);
    let coplanar = |d: &[f64; 3]| d.iter().all(|v| *v == 0.0);
    if same_side(&to_second) || same_side(&to_first) || coplanar(&to_second) || coplanar(&to_first)
    {
        return None;
    }
    let along = cross(first_normal, second_normal);
    let along_length = dot(along, along).sqrt();
    if along_length < 1e-14 * lengths[0] * lengths[1] {
        return None;
    }
    let direction = along.map(|c| c / along_length);
    let mine = plane_segment(first, to_second)?;
    let theirs = plane_segment(second, to_first)?;
    let range = |segment: &[Point3; 2]| {
        let (a, b) = (dot(direction, segment[0]), dot(direction, segment[1]));
        (a.min(b), a.max(b))
    };
    let ((mine_low, mine_high), (theirs_low, theirs_high)) = (range(&mine), range(&theirs));
    let (low, high) = (mine_low.max(theirs_low), mine_high.min(theirs_high));
    if high - low <= 1e-12 * scale {
        return None;
    }
    // 交わりの線の上の点を，線分の1つの端から，向きに沿って求める．
    let base = mine[0];
    let base_position = dot(direction, base);
    let at = |position: f64| zip(base, direction, |b, d| b + d * (position - base_position));
    Some([at(low), at(high)])
}

/// 端が(誤差の中で)同じ点を，1つの点にまとめる表．
struct NodeIndex {
    cell: f64,
    tolerance: f64,
    nodes: Vec<Point3>,
    /// 各節の，2つの曲面のパラメータ(4つの値)．はじめに見つかった値である．
    params: Vec<[f64; 4]>,
    lookup: std::collections::HashMap<[i64; 3], Vec<usize>>,
}

/// 値が入る升目の番号．
#[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
fn quantize(value: f64, cell: f64) -> i64 {
    (value / cell).floor().clamp(-1e15, 1e15) as i64
}

impl NodeIndex {
    fn new(tolerance: f64) -> Self {
        Self {
            cell: tolerance * 4.0,
            tolerance,
            nodes: Vec::new(),
            params: Vec::new(),
            lookup: std::collections::HashMap::new(),
        }
    }

    fn key(&self, point: Point3) -> [i64; 3] {
        point.map(|c| quantize(c, self.cell))
    }

    /// 点の節の番号．近くに節がなければ，新しく作る．
    fn node(&mut self, point: Point3, params: [f64; 4]) -> usize {
        let [cx, cy, cz] = self.key(point);
        for dx in -1..=1_i64 {
            for dy in -1..=1_i64 {
                for dz in -1..=1_i64 {
                    let neighbor = [
                        cx.saturating_add(dx),
                        cy.saturating_add(dy),
                        cz.saturating_add(dz),
                    ];
                    let found = self.lookup.get(&neighbor).and_then(|indices| {
                        indices.iter().copied().find(|index| {
                            self.nodes
                                .get(*index)
                                .is_some_and(|node| distance(*node, point) < self.tolerance)
                        })
                    });
                    if let Some(index) = found {
                        return index;
                    }
                }
            }
        }
        let index = self.nodes.len();
        self.nodes.push(point);
        self.params.push(params);
        self.lookup.entry([cx, cy, cz]).or_default().push(index);
        index
    }
}

/// 三角形の中の点の，曲面のパラメータ．頂点のパラメータを，重心座標で混ぜる．
fn triangle_params(solid: &Solid, vertex_params: [[f64; 2]; 3], point: Point3) -> [f64; 2] {
    let (first_edge, second_edge) = (minus(solid[1], solid[0]), minus(solid[2], solid[0]));
    let offset = minus(point, solid[0]);
    let (d00, d01, d11) = (
        dot(first_edge, first_edge),
        dot(first_edge, second_edge),
        dot(second_edge, second_edge),
    );
    let (d20, d21) = (dot(offset, first_edge), dot(offset, second_edge));
    let denominator = d00 * d11 - d01 * d01;
    let (weight_one, weight_two) = if denominator.abs() > f64::MIN_POSITIVE {
        (
            (d11 * d20 - d01 * d21) / denominator,
            (d00 * d21 - d01 * d20) / denominator,
        )
    } else {
        (0.0, 0.0)
    };
    let weight_zero = 1.0 - weight_one - weight_two;
    let mix = |axis: usize| {
        let [zero, one, two] = vertex_params.map(|params| params.get(axis).copied().unwrap_or(0.0));
        weight_zero * zero + weight_one * one + weight_two * two
    };
    [mix(0), mix(1)]
}

/// 2つの網の三角形が交わる線分と，端の，2つの曲面のパラメータ．
struct Meeting {
    ends: [Point3; 2],
    params: [[f64; 4]; 2],
}

impl Mesh {
    /// 網の三角形と，頂点の番号(順に，`indices`のうち，すべての頂点に値がある三角形)．
    fn solids(&self) -> Vec<(Solid, [usize; 3])> {
        self.indices
            .iter()
            .filter_map(|corners| {
                let [a, b, c] = corners.map(|k| self.point(k));
                Some(([a?, b?, c?], *corners))
            })
            .collect()
    }

    fn triangle_params(&self, solid: &Solid, corners: [usize; 3], point: Point3) -> [f64; 2] {
        triangle_params(solid, corners.map(|k| self.vertex_params(k)), point)
    }

    /// 別の網との交線．2つの網の三角形が交わる線分を求め，端が重なるものをつなぎ，2つの厳密な式
    /// (`own`は，この網の曲面，`theirs`は，相手の曲面)の上へ磨き，曲がりに合わせて点を足した，
    /// 空間の折れ線である．
    #[must_use]
    pub fn intersection(&self, other: &Mesh, own: SurfaceFn, theirs: SurfaceFn) -> Vec<Vec<Rim>> {
        let (mine, others) = (self.solids(), other.solids());
        let scale = self.scale.max(other.scale);
        let solid_list: Vec<Solid> = others.iter().map(|(solid, _)| *solid).collect();
        let grid = SpaceGrid::new(&solid_list);
        let mut seen = vec![usize::MAX; others.len()];
        let mut meetings: Vec<Meeting> = Vec::new();
        for (index, (triangle, corners)) in mine.iter().enumerate() {
            for candidate in grid.candidates(triangle) {
                if seen.get(candidate) == Some(&index) {
                    continue;
                }
                if let Some(slot) = seen.get_mut(candidate) {
                    *slot = index;
                }
                let Some((other_triangle, other_corners)) = others.get(candidate) else {
                    continue;
                };
                if let Some(ends) = triangle_intersection(triangle, other_triangle, scale) {
                    let params = ends.map(|end| {
                        let [u, v] = self.triangle_params(triangle, *corners, end);
                        let [s, t] = other.triangle_params(other_triangle, *other_corners, end);
                        [u, v, s, t]
                    });
                    meetings.push(Meeting { ends, params });
                }
            }
        }
        let tolerance = 1e-7 * scale;
        let mut nodes = NodeIndex::new(tolerance);
        let mut links: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for meeting in meetings {
            let [start, end] = meeting.ends;
            let [start_params, end_params] = meeting.params;
            let (from, to) = (nodes.node(start, start_params), nodes.node(end, end_params));
            if from != to {
                links.entry(from).or_default().push(to);
                links.entry(to).or_default().push(from);
            }
        }
        let residual = |params: &[f64]| -> Option<Vec<f64>> {
            let [u, v, s, t] = params else {
                return None;
            };
            let (first, second) = (own(*u, *v)?, theirs(*s, *t)?);
            Some(minus(first, second).to_vec())
        };
        let position = |params: &[f64]| -> Option<Point3> {
            let [u, v, _, _] = params else {
                return None;
            };
            own(*u, *v)
        };
        let curve = Curve {
            residual: &residual,
            position: &position,
            bounds: self
                .domain
                .iter()
                .chain(other.domain.iter())
                .copied()
                .collect(),
            scale,
        };
        let joined = chain_ids(&links)
            .iter()
            .map(|ids| {
                let chain: Vec<Node> = ids
                    .iter()
                    .filter_map(|id| Some((nodes.nodes.get(*id)?, nodes.params.get(*id)?)))
                    .map(|(point, params)| Node {
                        params: params.to_vec(),
                        point: *point,
                    })
                    .collect();
                curve
                    .refine(&chain)
                    .into_iter()
                    .map(|node| (node.point, [0.0; 3]))
                    .collect()
            })
            .collect();
        self.tidy(joined)
    }
}
