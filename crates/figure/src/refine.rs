//! 曲線の点を，厳密な式の上へ磨き，曲がりに合わせて点を足す．
//!
//! 三角形の網から求めた曲線(交線や切り口)は，網の辺の上でしか点を取れず，網が曲面から外れる分だけ，
//! 曲面の上からずれる．曲線を，パラメータの組`x`の，残差が0になる点の集まりとして表し，各点を，
//! Newton法で，最も近い解へ動かす(残差の数がパラメータの数より少ないので，最小の動きを選ぶ)．
//! 隣り合う2点のパラメータの中点も，同じ方法で磨き，弦から許容より離れていれば，点として足す．
//! これを繰り返すので，曲がりの強い所だけ，点が密になる．

/// 空間の点．
pub type Point3 = [f64; 3];

/// 磨く反復の回数の上限．
const MAX_STEPS: usize = 12;
/// 中点を足す再帰の深さの上限(1本の弦を，最大で2の累乗に割る)．
const MAX_DEPTH: u32 = 5;
/// 残差が0とみなせる大きさ(座標の大きさに対する割合)．
const RESIDUAL_TOLERANCE: f64 = 1e-10;
/// 磨いた点が，元の点から動いてよい距離(座標の大きさに対する割合)．別の枝へ飛ぶのを防ぐ．
const MAX_MOVE: f64 = 0.125;
/// 弦からの許容の距離(座標の大きさに対する割合)．これより離れた中点を，点として足す．
const CHORD_TOLERANCE: f64 = 1e-3;
/// 数値微分の刻み(パラメータの範囲に対する割合)．
const DERIVATIVE_STEP: f64 = 1e-6;

/// 曲線の上の点と，そのパラメータの組．
#[derive(Debug, Clone)]
pub struct Node {
    /// パラメータの組．
    pub params: Vec<f64>,
    /// 空間の点．
    pub point: Point3,
}

/// 曲線を，パラメータの組の，残差が0になる点の集まりとして表す．
pub struct Curve<'a> {
    /// 残差．すべて0になる所が曲線である．定義域の外や，値のない所では，`None`を返す．
    pub residual: &'a dyn Fn(&[f64]) -> Option<Vec<f64>>,
    /// パラメータの組から，空間の点．
    pub position: &'a dyn Fn(&[f64]) -> Option<Point3>,
    /// 各パラメータの範囲．
    pub bounds: Vec<[f64; 2]>,
    /// 座標の大きさ．許容の距離の基準にする．
    pub scale: f64,
}

fn norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn distance(first: Point3, second: Point3) -> f64 {
    let (dx, dy, dz) = (
        first[0] - second[0],
        first[1] - second[1],
        first[2] - second[2],
    );
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// 点から，線分までの距離．
fn distance_to_segment(point: Point3, start: Point3, end: Point3) -> f64 {
    let direction = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let offset = [
        point[0] - start[0],
        point[1] - start[1],
        point[2] - start[2],
    ];
    let length_squared: f64 = direction.iter().map(|c| c * c).sum();
    let along = if length_squared > 0.0 {
        (direction
            .iter()
            .zip(offset)
            .map(|(d, o)| d * o)
            .sum::<f64>()
            / length_squared)
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    let nearest = [
        start[0] + direction[0] * along,
        start[1] + direction[1] * along,
        start[2] + direction[2] * along,
    ];
    distance(point, nearest)
}

/// 連立方程式`matrix * y = rhs`を，ガウスの消去法で解く．特異なら`None`を返す．
fn solve(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let size = rhs.len();
    for column in 0..size {
        // 列の中で，絶対値が最大の行を，軸にする．
        let pivot = (column..size).max_by(|first, second| {
            let magnitude = |row: &usize| {
                matrix
                    .get(*row)
                    .and_then(|line| line.get(column))
                    .map_or(0.0, |value| value.abs())
            };
            magnitude(first).total_cmp(&magnitude(second))
        })?;
        matrix.swap(column, pivot);
        rhs.swap(column, pivot);
        let pivot_value = *matrix.get(column)?.get(column)?;
        if pivot_value.abs() < f64::MIN_POSITIVE {
            return None;
        }
        for row in column.saturating_add(1)..size {
            let factor = matrix.get(row)?.get(column)? / pivot_value;
            let upper: Vec<f64> = matrix.get(column)?.clone();
            for (cell, above) in matrix.get_mut(row)?.iter_mut().zip(upper) {
                *cell -= factor * above;
            }
            let above_rhs = *rhs.get(column)?;
            *rhs.get_mut(row)? -= factor * above_rhs;
        }
    }
    let mut solution = vec![0.0; size];
    for row in (0..size).rev() {
        let tail: f64 = (row.saturating_add(1)..size)
            .map(|column| {
                matrix
                    .get(row)
                    .and_then(|line| line.get(column))
                    .copied()
                    .unwrap_or(0.0)
                    * solution.get(column).copied().unwrap_or(0.0)
            })
            .sum();
        let diagonal = *matrix.get(row)?.get(row)?;
        *solution.get_mut(row)? = (rhs.get(row)? - tail) / diagonal;
    }
    Some(solution)
}

impl Curve<'_> {
    /// 範囲の中に収めたパラメータの組．
    fn clamped(&self, params: &[f64]) -> Vec<f64> {
        params
            .iter()
            .zip(&self.bounds)
            .map(|(value, [low, high])| value.clamp(*low, *high))
            .collect()
    }

    /// 残差の，パラメータごとの偏微分(中心差分)．パラメータごとに，残差の数の長さの列を返す．
    fn columns(&self, params: &[f64]) -> Option<Vec<Vec<f64>>> {
        let mut columns = Vec::with_capacity(params.len());
        for (index, (value, [low, high])) in params.iter().zip(&self.bounds).enumerate() {
            let step = ((high - low) * DERIVATIVE_STEP).max(1e-12);
            let shifted = |amount: f64| -> Vec<f64> {
                let mut moved = params.to_vec();
                if let Some(slot) = moved.get_mut(index) {
                    *slot = (value + amount).clamp(*low, *high);
                }
                moved
            };
            let (forward, backward) = (shifted(step), shifted(-step));
            let width = forward.get(index)? - backward.get(index)?;
            if width <= 0.0 {
                columns.push(vec![0.0; (self.residual)(params)?.len()]);
                continue;
            }
            let (after, before) = ((self.residual)(&forward)?, (self.residual)(&backward)?);
            columns.push(
                after
                    .iter()
                    .zip(&before)
                    .map(|(high_value, low_value)| (high_value - low_value) / width)
                    .collect(),
            );
        }
        Some(columns)
    }

    /// 1回の反復．残差を0にする，最小の動きを求める．
    fn step(&self, params: &[f64], residual: &[f64]) -> Option<Vec<f64>> {
        let columns = self.columns(params)?;
        let size = residual.len();
        let mut gram = vec![vec![0.0; size]; size];
        for column in &columns {
            for (row, row_value) in column.iter().enumerate() {
                for (col, col_value) in column.iter().enumerate() {
                    *gram.get_mut(row)?.get_mut(col)? += row_value * col_value;
                }
            }
        }
        // 接する所で，行列が特異に近くなるので，対角に小さな値を足す．
        let largest = (0..size)
            .filter_map(|index| gram.get(index)?.get(index).copied())
            .fold(0.0_f64, f64::max);
        for index in 0..size {
            *gram.get_mut(index)?.get_mut(index)? += 1e-12 * largest + f64::MIN_POSITIVE;
        }
        let combination = solve(gram, residual.to_vec())?;
        Some(
            columns
                .iter()
                .map(|column| {
                    -column
                        .iter()
                        .zip(&combination)
                        .map(|(c, y)| c * y)
                        .sum::<f64>()
                })
                .collect(),
        )
    }

    /// パラメータの組を，最も近い解へ動かす．収束しなければ，`None`を返す．
    fn solve_from(&self, start: &[f64]) -> Option<Vec<f64>> {
        let tolerance = RESIDUAL_TOLERANCE * self.scale;
        let mut params = self.clamped(start);
        for _ in 0..MAX_STEPS {
            let residual = (self.residual)(&params)?;
            if norm(&residual) <= tolerance {
                return Some(params);
            }
            let change = self.step(&params, &residual)?;
            let next: Vec<f64> = self.clamped(
                &params
                    .iter()
                    .zip(&change)
                    .map(|(value, delta)| value + delta)
                    .collect::<Vec<_>>(),
            );
            if norm(&change) < 1e-15 {
                break;
            }
            params = next;
        }
        let residual = (self.residual)(&params)?;
        (norm(&residual) <= tolerance * 10.0).then_some(params)
    }

    /// 点を，曲線の上へ磨く．収束しないか，遠くへ動くなら，元の点のままにする．
    #[must_use]
    pub fn polish(&self, node: &Node) -> Node {
        let polished = self
            .solve_from(&node.params)
            .and_then(|params| (self.position)(&params).map(|point| (params, point)))
            .filter(|(_, point)| distance(*point, node.point) <= MAX_MOVE * self.scale);
        match polished {
            Some((params, point)) => Node { params, point },
            None => node.clone(),
        }
    }

    /// 隣り合う2点の間を，曲がりに合わせて，点を足して細かくする．足した点は，順に`out`へ入れる．
    fn refine_between(&self, first: &Node, second: &Node, depth: u32, out: &mut Vec<Node>) {
        let chord = distance(first.point, second.point);
        if depth == 0 || chord < 4.0 * CHORD_TOLERANCE * self.scale {
            return;
        }
        let middle: Vec<f64> = first
            .params
            .iter()
            .zip(&second.params)
            .map(|(a, b)| f64::midpoint(*a, *b))
            .collect();
        let Some(params) = self.solve_from(&middle) else {
            return;
        };
        let Some(point) = (self.position)(&params) else {
            return;
        };
        // 弦の中点から，弦の長さの半分より遠い点は，別の枝へ飛んだ点なので，使わない．
        let chord_middle = [
            f64::midpoint(first.point[0], second.point[0]),
            f64::midpoint(first.point[1], second.point[1]),
            f64::midpoint(first.point[2], second.point[2]),
        ];
        if distance(point, chord_middle) > chord / 2.0 {
            return;
        }
        if distance_to_segment(point, first.point, second.point) <= CHORD_TOLERANCE * self.scale {
            return;
        }
        let inserted = Node { params, point };
        let next = depth.saturating_sub(1);
        self.refine_between(first, &inserted, next, out);
        out.push(inserted.clone());
        self.refine_between(&inserted, second, next, out);
    }

    /// 弦の許容の中で，直線とみなせる点を間引く(Douglas-Peuckerの方法)．両端は残す．
    fn simplify(&self, nodes: Vec<Node>) -> Vec<Node> {
        let tolerance = CHORD_TOLERANCE * self.scale * 0.5;
        let mut keep = vec![false; nodes.len()];
        let mut stack = vec![(0_usize, nodes.len().saturating_sub(1))];
        if let Some(first) = keep.first_mut() {
            *first = true;
        }
        if let Some(last) = keep.last_mut() {
            *last = true;
        }
        while let Some((start, end)) = stack.pop() {
            let (Some(from), Some(to)) = (nodes.get(start), nodes.get(end)) else {
                continue;
            };
            // 始めと終わりの間で，弦から最も遠い点．
            let farthest = (start.saturating_add(1)..end)
                .filter_map(|index| {
                    let node = nodes.get(index)?;
                    Some((index, distance_to_segment(node.point, from.point, to.point)))
                })
                .max_by(|first, second| first.1.total_cmp(&second.1));
            if let Some((index, gap)) = farthest
                && gap > tolerance
            {
                if let Some(flag) = keep.get_mut(index) {
                    *flag = true;
                }
                stack.push((start, index));
                stack.push((index, end));
            }
        }
        nodes
            .into_iter()
            .zip(keep)
            .filter_map(|(node, kept)| kept.then_some(node))
            .collect()
    }

    /// 点の列を，磨き，直線とみなせる点を間引き，曲がりに合わせて点を足す．点の順は，保たれる．
    #[must_use]
    pub fn refine(&self, nodes: &[Node]) -> Vec<Node> {
        let polished = self.simplify(nodes.iter().map(|node| self.polish(node)).collect());
        let mut result = Vec::with_capacity(polished.len());
        for (index, node) in polished.iter().enumerate() {
            result.push(node.clone());
            if let Some(next) = polished.get(index.saturating_add(1)) {
                self.refine_between(node, next, MAX_DEPTH, &mut result);
            }
        }
        result
    }
}
