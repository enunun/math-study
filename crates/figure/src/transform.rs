//! 変換(`transform`)．平行移動・回転・拡大縮小・対称移動・せん断(アフィン変換)と，写像(`map`)を，
//! 書いた順に施す．
//!
//! アフィン変換は，平面の図でも空間の図と同じ3次元の行列で持つ(平面の点は，z = 0の点として写す)．
//! 続けて書いたアフィン変換は，1つに合成しておく．写像は，式で書かれた一般の変換である．

use std::rc::Rc;

use crate::expr::Expr;

/// 空間の点か，向き．
pub type Vector3 = [f64; 3];

const IDENTITY_MATRIX: [Vector3; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// アフィン変換．`p' = linear p + offset`．
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    /// 線形の部分．`linear[i]`が，行列の`i`行目である．
    linear: [Vector3; 3],
    /// 平行移動の部分．
    offset: Vector3,
}

fn dot(a: Vector3, b: Vector3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Affine {
    /// 恒等変換(何もしない変換)．
    pub const IDENTITY: Self = Self {
        linear: IDENTITY_MATRIX,
        offset: [0.0; 3],
    };

    /// 平行移動．
    #[must_use]
    pub const fn translate(offset: Vector3) -> Self {
        Self {
            linear: IDENTITY_MATRIX,
            offset,
        }
    }

    /// 線形変換(原点を動かさない変換)．
    #[must_use]
    pub const fn linear(linear: [Vector3; 3]) -> Self {
        Self {
            linear,
            offset: [0.0; 3],
        }
    }

    /// 平面の回転．z軸のまわりに，反時計回りに`degrees`度回す．
    #[must_use]
    pub fn rotate_plane(degrees: f64) -> Self {
        Self::rotate_about_axis(degrees, [0.0, 0.0, 1.0])
    }

    /// 空間の回転．単位ベクトル`axis`のまわりに，右ねじの向きに`degrees`度回す(ロドリゲスの公式)．
    #[must_use]
    pub fn rotate_about_axis(degrees: f64, axis: Vector3) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let [x, y, z] = axis;
        let t = 1.0 - cos;
        Self::linear([
            [t * x * x + cos, t * x * y - sin * z, t * x * z + sin * y],
            [t * x * y + sin * z, t * y * y + cos, t * y * z - sin * x],
            [t * x * z - sin * y, t * y * z + sin * x, t * z * z + cos],
        ])
    }

    /// 方向ごとの拡大縮小．
    #[must_use]
    pub const fn scale([x, y, z]: Vector3) -> Self {
        Self::linear([[x, 0.0, 0.0], [0.0, y, 0.0], [0.0, 0.0, z]])
    }

    /// 原点を通り，単位ベクトル`direction`の向きの直線に関する，平面の対称移動．zは変えない．
    #[must_use]
    pub fn reflect_line([x, y]: [f64; 2]) -> Self {
        Self::linear([
            [2.0 * x * x - 1.0, 2.0 * x * y, 0.0],
            [2.0 * x * y, 2.0 * y * y - 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// 原点を通り，単位ベクトル`normal`を法線とする平面に関する，空間の対称移動．
    #[must_use]
    pub fn reflect_plane(normal: Vector3) -> Self {
        let [x, y, z] = normal;
        Self::linear([
            [1.0 - 2.0 * x * x, -2.0 * x * y, -2.0 * x * z],
            [-2.0 * x * y, 1.0 - 2.0 * y * y, -2.0 * y * z],
            [-2.0 * x * z, -2.0 * y * z, 1.0 - 2.0 * z * z],
        ])
    }

    /// 平面のせん断．`x' = x + a y`，`y' = y + b x`．
    #[must_use]
    pub const fn shear(a: f64, b: f64) -> Self {
        Self::linear([[1.0, a, 0.0], [b, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    /// 同じ変換を，原点の代わりに`center`を動かさない点にして施すもの．
    #[must_use]
    pub fn about(&self, center: Vector3) -> Self {
        Self::translate([-center[0], -center[1], -center[2]])
            .then(self)
            .then(&Self::translate(center))
    }

    /// `self`のあとに`next`を施した，合成の変換．
    #[must_use]
    pub fn then(&self, next: &Self) -> Self {
        let [a, b, c] = self.linear;
        let columns = [[a[0], b[0], c[0]], [a[1], b[1], c[1]], [a[2], b[2], c[2]]];
        Self {
            linear: next
                .linear
                .map(|row| columns.map(|column| dot(row, column))),
            offset: next.apply3(self.offset),
        }
    }

    /// 空間の点を写す．
    #[must_use]
    pub fn apply3(&self, point: Vector3) -> Vector3 {
        let [first, second, third] = self.linear;
        let [dx, dy, dz] = self.offset;
        [
            dot(first, point) + dx,
            dot(second, point) + dy,
            dot(third, point) + dz,
        ]
    }

    /// 平面の点を写す(z = 0の点として写し，zを捨てる)．
    #[must_use]
    pub fn apply2(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        let [x, y, _] = self.apply3([x, y, 0.0]);
        [x, y]
    }

    /// 線形の部分の行列式．負なら，向きが裏返る(対称移動など)．
    #[must_use]
    pub fn determinant(&self) -> f64 {
        let [a, b, c] = self.linear;
        a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0])
    }
}

/// 式で書いた写像．座標の変数に値を入れ，写した先の座標を返す．
#[derive(Debug)]
pub struct MapFn {
    /// 写した先の各座標の式．名前の順は，座標の変数，媒介変数である．
    exprs: Vec<Expr>,
    /// 媒介変数の値．
    parameters: Vec<f64>,
}

impl MapFn {
    /// 写像を作る．`exprs`は，座標の変数と媒介変数(`parameters`の順)を使う式である．
    #[must_use]
    pub const fn new(exprs: Vec<Expr>, parameters: Vec<f64>) -> Self {
        Self { exprs, parameters }
    }

    /// 点を写す．値が有限でなければ，`None`を返す．
    fn apply(&self, point: &[f64]) -> Option<Vec<f64>> {
        let values: Vec<f64> = point.iter().chain(&self.parameters).copied().collect();
        let image: Vec<f64> = self.exprs.iter().map(|expr| expr.eval(&values)).collect();
        image.iter().all(|c| c.is_finite()).then_some(image)
    }
}

/// 変換の手順の1つ．
#[derive(Debug, Clone)]
enum Step {
    Affine(Affine),
    Map(Rc<MapFn>),
}

/// 変換．手順を，書いた順に施す．手順がなければ，何もしない．
#[derive(Debug, Clone, Default)]
pub struct Transform {
    steps: Vec<Step>,
}

impl Transform {
    /// アフィン変換を，最後の手順として加える．直前もアフィン変換なら，1つに合成する．
    pub fn push_affine(&mut self, affine: Affine) {
        if let Some(Step::Affine(last)) = self.steps.last_mut() {
            *last = last.then(&affine);
        } else {
            self.steps.push(Step::Affine(affine));
        }
    }

    /// 写像を，最後の手順として加える．
    pub fn push_map(&mut self, map: Rc<MapFn>) {
        self.steps.push(Step::Map(map));
    }

    /// 何もしない変換か．
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.steps.is_empty()
    }

    /// 写像を含まなければ，全体を1つにしたアフィン変換．含めば`None`．
    #[must_use]
    pub fn as_affine(&self) -> Option<Affine> {
        self.steps
            .iter()
            .try_fold(Affine::IDENTITY, |combined, step| match step {
                Step::Affine(affine) => Some(combined.then(affine)),
                Step::Map(_) => None,
            })
    }

    /// 点(平面の図では2個，空間の図では3個の座標)を写す．写像の値が有限でなければ，`None`を返す．
    #[must_use]
    pub fn apply(&self, point: &[f64]) -> Option<Vec<f64>> {
        let mut current = point.to_vec();
        for step in &self.steps {
            current = match step {
                Step::Affine(affine) => match current.as_slice() {
                    [x, y] => affine.apply2([*x, *y]).to_vec(),
                    [x, y, z] => affine.apply3([*x, *y, *z]).to_vec(),
                    _ => return None,
                },
                Step::Map(map) => map.apply(&current)?,
            };
        }
        Some(current)
    }

    /// 平面の点を写す．
    #[must_use]
    pub fn apply2(&self, [x, y]: [f64; 2]) -> Option<[f64; 2]> {
        match self.apply(&[x, y])?.as_slice() {
            [x, y] => Some([*x, *y]),
            _ => None,
        }
    }

    /// 空間の点を写す．
    #[must_use]
    pub fn apply3(&self, [x, y, z]: Vector3) -> Option<Vector3> {
        match self.apply(&[x, y, z])?.as_slice() {
            [x, y, z] => Some([*x, *y, *z]),
            _ => None,
        }
    }
}
