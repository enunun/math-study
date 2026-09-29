//! 複素数と，複素数の式の評価．複素数の式では，`i`が虚数単位になる(`Expr::compile_complex`)．
//!
//! 初等関数とガンマ関数は，複素数の主値で評価する．Jacobiの楕円関数は，実の母数`k`(`0 <= k <= 1`)で，
//! 引数の実部と虚部の加法定理から求める．Weierstrassの`℘`は，実の不変量で，Jacobiの楕円関数から
//! 求める．実数でしか定めていない特殊関数(誤差関数，Bessel関数など)は，引数の虚部が0でなければ非数を返す．

// 複素数の四則演算(`Add`などの実装)は浮動小数点の演算で，あふれて止まることはない．
#![allow(clippy::arithmetic_side_effects)]

use std::ops::{Add, Div, Mul, Neg, Sub};

use super::elliptic::{Jacobi, WeierstrassForm, jacobi, weierstrass_form};
use super::{BinaryOp, Function, Node};

/// 複素数．
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    /// 実部．
    pub re: f64,
    /// 虚部．
    pub im: f64,
}

impl Complex {
    /// 虚数単位．
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    /// 実部と虚部から作る．
    #[must_use]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// 実数．
    #[must_use]
    pub const fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    /// 絶対値．
    #[must_use]
    pub fn norm(self) -> f64 {
        self.re.hypot(self.im)
    }

    /// 偏角(`-π < θ <= π`)．虚部の`-0`は`0`とみなす(`-1`は，`1`の符号を変えた`-0`の虚部をもつ)．
    #[must_use]
    pub fn arg(self) -> f64 {
        (self.im + 0.0).atan2(self.re)
    }

    /// 共役．
    #[must_use]
    pub const fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    /// 値が有限か．
    #[must_use]
    pub const fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }

    /// 非数．
    const NAN: Self = Self::new(f64::NAN, f64::NAN);

    fn scale(self, factor: f64) -> Self {
        Self::new(self.re * factor, self.im * factor)
    }

    fn exp(self) -> Self {
        let (sin, cos) = self.im.sin_cos();
        let radius = self.re.exp();
        Self::new(radius * cos, radius * sin)
    }

    /// 対数の主値．
    fn ln(self) -> Self {
        Self::new(self.norm().ln(), self.arg())
    }

    /// 平方根の主値．
    fn sqrt(self) -> Self {
        if self.im == 0.0 {
            return if self.re >= 0.0 {
                Self::real(self.re.sqrt())
            } else {
                Self::new(0.0, (-self.re).sqrt())
            };
        }
        let radius = self.norm();
        let re = f64::midpoint(radius, self.re).sqrt();
        let im = f64::midpoint(radius, -self.re).sqrt().copysign(self.im);
        Self::new(re, im)
    }

    fn sin(self) -> Self {
        let (sin, cos) = self.re.sin_cos();
        Self::new(sin * self.im.cosh(), cos * self.im.sinh())
    }

    fn cos(self) -> Self {
        let (sin, cos) = self.re.sin_cos();
        Self::new(cos * self.im.cosh(), -sin * self.im.sinh())
    }

    fn sinh(self) -> Self {
        let (sin, cos) = self.im.sin_cos();
        Self::new(self.re.sinh() * cos, self.re.cosh() * sin)
    }

    fn cosh(self) -> Self {
        let (sin, cos) = self.im.sin_cos();
        Self::new(self.re.cosh() * cos, self.re.sinh() * sin)
    }

    /// べき乗の主値．指数が整数なら，掛け算を繰り返す(負の数の整数乗を正確にするため)．
    fn pow(self, exponent: Self) -> Self {
        if exponent.im == 0.0 && exponent.re.fract() == 0.0 && exponent.re.abs() <= 64.0 {
            let mut result = Self::real(1.0);
            let mut remaining = exponent.re.abs();
            while remaining > 0.0 {
                result = result * self;
                remaining -= 1.0;
            }
            return if exponent.re < 0.0 {
                Self::real(1.0) / result
            } else {
                result
            };
        }
        if self.re == 0.0 && self.im == 0.0 {
            return if exponent.re > 0.0 {
                Self::real(0.0)
            } else {
                Self::NAN
            };
        }
        (exponent * self.ln()).exp()
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.re.mul_add(other.re, -(self.im * other.im)),
            self.re.mul_add(other.im, self.im * other.re),
        )
    }
}

impl Div for Complex {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        // Smithの方法．分母の大きい方の成分で割り，あふれを防ぐ．
        if other.im == 0.0 {
            return Self::new(self.re / other.re, self.im / other.re);
        }
        if other.re.abs() >= other.im.abs() {
            let ratio = other.im / other.re;
            let denominator = other.im.mul_add(ratio, other.re);
            Self::new(
                self.im.mul_add(ratio, self.re) / denominator,
                (-self.re).mul_add(ratio, self.im) / denominator,
            )
        } else {
            let ratio = other.re / other.im;
            let denominator = other.re.mul_add(ratio, other.im);
            Self::new(
                self.re.mul_add(ratio, self.im) / denominator,
                self.im.mul_add(ratio, -self.re) / denominator,
            )
        }
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.re, -self.im)
    }
}

/// Lanczos近似の係数(g = 7，9項)．
const LANCZOS: [f64; 9] = [
    0.999_999_999_999_809_9,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_1,
    -176.615_029_162_140_6,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    9.984_369_578_019_572e-6,
    1.505_632_735_149_311_6e-7,
];

/// ガンマ関数．実部が1/2より小さいときは，鏡映公式で折り返す．
fn gamma(z: Complex) -> Complex {
    if z.im == 0.0 {
        return Complex::real(puruspe::gamma(z.re));
    }
    if z.re < 0.5 {
        let pi = Complex::real(std::f64::consts::PI);
        return pi / ((pi * z).sin() * gamma(Complex::real(1.0) - z));
    }
    let z = z - Complex::real(1.0);
    let mut sum = Complex::real(LANCZOS[0]);
    let mut shift = 0.0;
    for coefficient in LANCZOS.iter().skip(1) {
        shift += 1.0;
        sum = sum + Complex::real(*coefficient) / (z + Complex::real(shift));
    }
    let t = z + Complex::real(7.5);
    let half = z + Complex::real(0.5);
    (Complex::real(std::f64::consts::TAU.sqrt()) * t.pow(half)) * (-t).exp() * sum
}

/// 実数でしか定めていない関数を，虚部が0の引数でだけ評価する．
fn real_only(function: Function, arguments: &[Complex]) -> Complex {
    if arguments.iter().all(|argument| argument.im == 0.0) {
        let values: Vec<f64> = arguments.iter().map(|argument| argument.re).collect();
        Complex::real(function.apply(&values))
    } else {
        Complex::NAN
    }
}

/// 複素数の引数と実の母数の，Jacobiの楕円関数`(sn, cn, dn)`．加法定理で，実部と虚部に分けて求める．
fn jacobi_complex(u: Complex, k: f64) -> (Complex, Complex, Complex) {
    let k = k.abs();
    if u.im == 0.0 {
        let Jacobi { sn, cn, dn, .. } = jacobi(u.re, k);
        return (Complex::real(sn), Complex::real(cn), Complex::real(dn));
    }
    if k > 1.0 {
        return (Complex::NAN, Complex::NAN, Complex::NAN);
    }
    let complement = (1.0 - k * k).max(0.0).sqrt();
    // A&S 16.21．実部の値と，補母数での虚部の値から組み立てる．
    let along = jacobi(u.re, k);
    let across = jacobi(u.im, complement);
    let cross = k * along.sn * across.sn;
    let scale = 1.0 / cross.mul_add(cross, across.cn * across.cn);
    let sn = Complex::new(
        along.sn * across.dn,
        along.cn * along.dn * across.sn * across.cn,
    );
    let cn = Complex::new(
        along.cn * across.cn,
        -along.sn * along.dn * across.sn * across.dn,
    );
    let dn = Complex::new(
        along.dn * across.cn * across.dn,
        -k * k * along.sn * along.cn * across.sn,
    );
    (sn.scale(scale), cn.scale(scale), dn.scale(scale))
}

/// 複素数の引数と実の不変量の，Weierstrassの`℘`と`℘'`．
fn weierstrass_complex(z: Complex, g2: f64, g3: f64) -> (Complex, Complex) {
    if z.im == 0.0 {
        let (value, slope) = super::elliptic::weierstrass(z.re, g2, g3);
        return (Complex::real(value), Complex::real(slope));
    }
    if g2 == 0.0 && g3 == 0.0 {
        let square = z * z;
        return (
            Complex::real(1.0) / square,
            Complex::real(-2.0) / (square * z),
        );
    }
    let one = Complex::real(1.0);
    match weierstrass_form(g2, g3) {
        WeierstrassForm::ThreeRoots {
            e3,
            spread,
            scale,
            k,
        } => {
            let (sn, cn, dn) = jacobi_complex(z.scale(scale), k);
            let square = sn * sn;
            let value = Complex::real(e3) + Complex::real(spread) / square;
            let slope = Complex::real(-2.0 * spread * scale) * cn * dn / (square * sn);
            (value, slope)
        }
        WeierstrassForm::OneRoot { e2, h, root, k } => {
            let (sn, cn, dn) = jacobi_complex(z.scale(2.0 * root), k);
            let gap = one - cn;
            let value = Complex::real(e2) + Complex::real(h) * (one + cn) / gap;
            let slope = Complex::real(-4.0 * h * root) * sn * dn / (gap * gap);
            (value, slope)
        }
    }
}

impl Function {
    /// 複素数の引数に施す．
    fn apply_complex(self, arguments: &[Complex]) -> Complex {
        let at = |index: usize| arguments.get(index).copied().unwrap_or(Complex::NAN);
        let z = at(0);
        let one = Complex::real(1.0);
        match self {
            Self::Sin => z.sin(),
            Self::Cos => z.cos(),
            Self::Tan => z.sin() / z.cos(),
            Self::Sinh => z.sinh(),
            Self::Cosh => z.cosh(),
            Self::Tanh => z.sinh() / z.cosh(),
            Self::Exp => z.exp(),
            Self::Log => z.ln(),
            Self::Sqrt => z.sqrt(),
            Self::Abs => Complex::real(z.norm()),
            // asin z = -i log(iz + sqrt(1 - z^2))，acos z = π/2 - asin z，
            // atan z = (i/2)(log(1 - iz) - log(1 + iz))．
            Self::Asin => -Complex::I * (Complex::I * z + (one - z * z).sqrt()).ln(),
            Self::Acos => {
                Complex::real(std::f64::consts::FRAC_PI_2)
                    + Complex::I * (Complex::I * z + (one - z * z).sqrt()).ln()
            }
            Self::Atan => {
                Complex::new(0.0, 0.5) * ((one - Complex::I * z).ln() - (one + Complex::I * z).ln())
            }
            Self::Gamma => gamma(z),
            Self::LnGamma => gamma(z).ln(),
            Self::Re => Complex::real(z.re),
            Self::Im => Complex::real(z.im),
            Self::Conj => z.conj(),
            Self::Arg => Complex::real(z.arg()),
            Self::Sn | Self::Cn | Self::Dn => {
                let k = at(1);
                if k.im != 0.0 {
                    return Complex::NAN;
                }
                let (sn, cn, dn) = jacobi_complex(z, k.re);
                match self {
                    Self::Sn => sn,
                    Self::Cn => cn,
                    _ => dn,
                }
            }
            Self::Wp | Self::Wpd => {
                let (g2, g3) = (at(1), at(2));
                if g2.im != 0.0 || g3.im != 0.0 {
                    return Complex::NAN;
                }
                let (value, slope) = weierstrass_complex(z, g2.re, g3.re);
                if self == Self::Wp { value } else { slope }
            }
            Self::Erf
            | Self::Erfc
            | Self::Dawson
            | Self::LambertW
            | Self::BesselJ0
            | Self::BesselJ1
            | Self::BesselY0
            | Self::BesselY1
            | Self::BesselI0
            | Self::BesselI1
            | Self::BesselK0
            | Self::BesselK1
            | Self::EllipK
            | Self::EllipE
            | Self::EllipF
            | Self::EllipEInc
            | Self::Am => real_only(self, arguments),
        }
    }
}

impl Node {
    /// 複素数で評価する．
    pub(super) fn eval_complex(&self, values: &[Complex]) -> Complex {
        match self {
            Self::Number(value) => Complex::real(*value),
            Self::Imaginary => Complex::I,
            Self::Variable(index) => values.get(*index).copied().unwrap_or(Complex::NAN),
            Self::Neg(operand) => -operand.eval_complex(values),
            Self::Binary(op, left, right) => {
                let (left, right) = (left.eval_complex(values), right.eval_complex(values));
                match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    BinaryOp::Div => left / right,
                    BinaryOp::Pow => left.pow(right),
                }
            }
            Self::Call(function, arguments) => {
                let values: Vec<Complex> = arguments
                    .iter()
                    .map(|argument| argument.eval_complex(values))
                    .collect();
                function.apply_complex(&values)
            }
            // ベクトルの関数は，点の式でだけ使える．
            Self::Vector(..) => Complex::NAN,
        }
    }
}
