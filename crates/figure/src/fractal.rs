//! フラクタル図形．反復関数系(IFS)：基本図形を，アフィン変換の集まりで再帰的に写す．

/// 平面のアフィン変換．`x' = a*x + b*y + e`，`y' = c*x + d*y + f`．
#[derive(Clone, Copy)]
pub struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Affine {
    /// 恒等変換(何もしない変換)．
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    #[must_use]
    pub fn translate(x: f64, y: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: x,
            f: y,
        }
    }

    #[must_use]
    pub fn rotate(degrees: f64) -> Self {
        let radians = degrees.to_radians();
        let (sin, cos) = (radians.sin(), radians.cos());
        Self {
            a: cos,
            b: -sin,
            c: sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    #[must_use]
    pub fn scale(x: f64, y: f64) -> Self {
        Self {
            a: x,
            b: 0.0,
            c: 0.0,
            d: y,
            e: 0.0,
            f: 0.0,
        }
    }

    #[must_use]
    pub fn shear(x: f64, y: f64) -> Self {
        Self {
            a: 1.0,
            b: x,
            c: y,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// `self`のあとに`next`を施した，合成の変換．
    #[must_use]
    pub fn then(&self, next: &Self) -> Self {
        Self {
            a: next.a * self.a + next.b * self.c,
            b: next.a * self.b + next.b * self.d,
            c: next.c * self.a + next.d * self.c,
            d: next.c * self.b + next.d * self.d,
            e: next.a * self.e + next.b * self.f + next.e,
            f: next.c * self.e + next.d * self.f + next.f,
        }
    }

    #[must_use]
    pub fn apply(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        [
            self.a * x + self.b * y + self.e,
            self.c * x + self.d * y + self.f,
        ]
    }
}

/// 手順の並びを，1つのアフィン変換に合成する．並びの順に施す(最初の手順が，点に最初にかかる)．
#[must_use]
pub fn compose(steps: &[Affine]) -> Affine {
    steps
        .iter()
        .fold(Affine::IDENTITY, |combined, step| combined.then(step))
}

/// 基本図形(`base`)を，`transforms`で`depth`回，再帰的に写す．深さ0は`base`そのもの，深さ`n`は，
/// 深さ`n - 1`の図形全体に，それぞれの変換を施したものをすべて集めたものである．
#[must_use]
pub fn instances(base: &[[f64; 2]], transforms: &[Affine], depth: u32) -> Vec<Vec<[f64; 2]>> {
    if depth == 0 {
        return vec![base.to_vec()];
    }
    let previous = instances(base, transforms, depth.saturating_sub(1));
    let mut result = Vec::with_capacity(previous.len().saturating_mul(transforms.len()));
    for transform in transforms {
        for path in &previous {
            result.push(path.iter().map(|point| transform.apply(*point)).collect());
        }
    }
    result
}
