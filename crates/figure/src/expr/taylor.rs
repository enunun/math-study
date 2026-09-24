//! 式のテイラー係数．式の各節を，打ち切ったべき級数(0次から`order`次までの係数)として計算する．
//!
//! 四則演算は級数の演算，初等関数は係数の漸化式(`exp`なら`e' = a' e`を係数で書いたもの)で求める．
//! 数値微分と違い，次数が高くても誤差が増えない．漸化式のない特殊関数は展開しない．

use super::{BinaryOp, Function, Node};

/// 打ち切ったべき級数．`k`番目が`k`次の係数である．長さは，どれも`order + 1`である．
type Series = Vec<f64>;

/// 指数が整数のとき，掛け算を繰り返して計算する，指数の絶対値の上限．負の数の整数乗も展開できる．
const MAX_INTEGER_POWER: f64 = 64.0;

/// 式`node`の，変数`var`についての，`values`のまわりのテイラー係数．
pub fn coefficients(node: &Node, var: usize, values: &[f64], order: usize) -> Option<Series> {
    let length = order.checked_add(1)?;
    let series = Expander {
        var,
        values,
        length,
    }
    .expand(node)?;
    series.iter().all(|c| c.is_finite()).then_some(series)
}

struct Expander<'a> {
    var: usize,
    values: &'a [f64],
    length: usize,
}

/// 整数を，係数の計算に使う浮動小数点数にする．
fn float(k: usize) -> f64 {
    f64::from(u32::try_from(k).unwrap_or(u32::MAX))
}

/// 級数の`k`次の係数．範囲の外は0である．
fn at(series: &[f64], k: usize) -> f64 {
    series.get(k).copied().unwrap_or(0.0)
}

impl Expander<'_> {
    fn constant(&self, value: f64) -> Series {
        let mut series = vec![0.0; self.length];
        if let Some(first) = series.first_mut() {
            *first = value;
        }
        series
    }

    fn expand(&self, node: &Node) -> Option<Series> {
        match node {
            Node::Number(value) => Some(self.constant(*value)),
            Node::Variable(index) => {
                let value = self.values.get(*index).copied().unwrap_or(f64::NAN);
                let mut series = self.constant(value);
                if *index == self.var
                    && let Some(linear) = series.get_mut(1)
                {
                    *linear = 1.0;
                }
                Some(series)
            }
            Node::Neg(operand) => Some(self.expand(operand)?.iter().map(|c| -c).collect()),
            Node::Binary(op, left, right) => {
                let (left, right) = (self.expand(left)?, self.expand(right)?);
                match op {
                    BinaryOp::Add => Some(zip_with(&left, &right, |a, b| a + b)),
                    BinaryOp::Sub => Some(zip_with(&left, &right, |a, b| a - b)),
                    BinaryOp::Mul => Some(multiply(&left, &right)),
                    BinaryOp::Div => divide(&left, &right),
                    BinaryOp::Pow => self.power(&left, &right),
                }
            }
            Node::Call(function, argument) => self.call(*function, &self.expand(argument)?),
        }
    }

    /// べき乗．指数が定数の整数なら掛け算を繰り返し，そうでなければ`exp(b log a)`にする．
    fn power(&self, base: &[f64], exponent: &[f64]) -> Option<Series> {
        let p = at(exponent, 0);
        let constant_exponent = exponent.iter().skip(1).all(|c| *c == 0.0);
        if constant_exponent && p.fract() == 0.0 && p.abs() <= MAX_INTEGER_POWER {
            let mut result = self.constant(1.0);
            let mut remaining = p.abs();
            while remaining >= 1.0 {
                result = multiply(&result, base);
                remaining -= 1.0;
            }
            return if p < 0.0 {
                divide(&self.constant(1.0), &result)
            } else {
                Some(result)
            };
        }
        Some(exp(&multiply(exponent, &log(base)?)))
    }

    fn call(&self, function: Function, a: &[f64]) -> Option<Series> {
        match function {
            Function::Sin => Some(sin_cos(a, false).0),
            Function::Cos => Some(sin_cos(a, false).1),
            Function::Tan => {
                let (sin, cos) = sin_cos(a, false);
                divide(&sin, &cos)
            }
            Function::Sinh => Some(sin_cos(a, true).0),
            Function::Cosh => Some(sin_cos(a, true).1),
            Function::Tanh => {
                let (sinh, cosh) = sin_cos(a, true);
                divide(&sinh, &cosh)
            }
            Function::Exp => Some(exp(a)),
            Function::Log => log(a),
            Function::Sqrt => sqrt(a),
            Function::Abs => {
                let sign = at(a, 0).signum();
                (at(a, 0) != 0.0).then(|| a.iter().map(|c| c * sign).collect())
            }
            Function::Asin | Function::Acos => {
                // asin(a)' = a' / sqrt(1 - a^2)．acosは，その符号を変えたもの．
                let one_minus_square = zip_with(&self.constant(1.0), &multiply(a, a), |x, y| x - y);
                let derivative = divide(&differentiate(a), &sqrt(&one_minus_square)?)?;
                let (start, sign) = if function == Function::Asin {
                    (at(a, 0).asin(), 1.0)
                } else {
                    (at(a, 0).acos(), -1.0)
                };
                let scaled: Series = derivative.iter().map(|c| c * sign).collect();
                Some(integrate(&scaled, start))
            }
            Function::Atan => {
                // atan(a)' = a' / (1 + a^2)．
                let one_plus_square = zip_with(&self.constant(1.0), &multiply(a, a), |x, y| x + y);
                let derivative = divide(&differentiate(a), &one_plus_square)?;
                Some(integrate(&derivative, at(a, 0).atan()))
            }
            Function::Gamma
            | Function::LnGamma
            | Function::Erf
            | Function::Erfc
            | Function::Dawson
            | Function::LambertW
            | Function::BesselJ0
            | Function::BesselJ1
            | Function::BesselY0
            | Function::BesselY1
            | Function::BesselI0
            | Function::BesselI1
            | Function::BesselK0
            | Function::BesselK1 => None,
        }
    }
}

fn zip_with(a: &[f64], b: &[f64], op: impl Fn(f64, f64) -> f64) -> Series {
    a.iter().zip(b).map(|(x, y)| op(*x, *y)).collect()
}

/// 級数の積(コーシー積)．
fn multiply(a: &[f64], b: &[f64]) -> Series {
    (0..a.len())
        .map(|k| (0..=k).map(|j| at(a, j) * at(b, k.saturating_sub(j))).sum())
        .collect()
}

/// 級数の商．`b`の定数項が0なら，展開できない．
fn divide(a: &[f64], b: &[f64]) -> Option<Series> {
    let b0 = at(b, 0);
    if b0 == 0.0 {
        return None;
    }
    let mut c: Series = Vec::with_capacity(a.len());
    for k in 0..a.len() {
        let known: f64 = (1..=k)
            .map(|j| at(b, j) * at(&c, k.saturating_sub(j)))
            .sum();
        c.push((at(a, k) - known) / b0);
    }
    Some(c)
}

/// `exp(a)`．`e' = a' e`から，`k e_k = Σ j a_j e_{k-j}`．
fn exp(a: &[f64]) -> Series {
    let mut e: Series = vec![at(a, 0).exp()];
    for k in 1..a.len() {
        let sum: f64 = (1..=k)
            .map(|j| float(j) * at(a, j) * at(&e, k.saturating_sub(j)))
            .sum();
        e.push(sum / float(k));
    }
    e
}

/// `log(a)`．`a l' = a'`から．`a`の定数項が正でなければ，展開できない．
fn log(a: &[f64]) -> Option<Series> {
    let a0 = at(a, 0);
    if a0 <= 0.0 {
        return None;
    }
    let mut l: Series = vec![a0.ln()];
    for k in 1..a.len() {
        let sum: f64 = (1..k)
            .map(|j| float(j) * at(&l, j) * at(a, k.saturating_sub(j)))
            .sum();
        l.push((at(a, k) - sum / float(k)) / a0);
    }
    Some(l)
}

/// `sqrt(a)`．`r^2 = a`から．`a`の定数項が正でなければ，展開できない．
fn sqrt(a: &[f64]) -> Option<Series> {
    let a0 = at(a, 0);
    if a0 <= 0.0 {
        return None;
    }
    let r0 = a0.sqrt();
    let mut r: Series = vec![r0];
    for k in 1..a.len() {
        let sum: f64 = (1..k)
            .map(|j| at(&r, j) * at(&r, k.saturating_sub(j)))
            .sum();
        r.push((at(a, k) - sum) / (2.0 * r0));
    }
    Some(r)
}

/// `sin(a)`と`cos(a)`(`hyperbolic`なら`sinh(a)`と`cosh(a)`)．`s' = a' c`，`c' = ∓a' s`から．
fn sin_cos(a: &[f64], hyperbolic: bool) -> (Series, Series) {
    let a0 = at(a, 0);
    let (mut s, mut c) = if hyperbolic {
        (vec![a0.sinh()], vec![a0.cosh()])
    } else {
        (vec![a0.sin()], vec![a0.cos()])
    };
    let sign = if hyperbolic { 1.0 } else { -1.0 };
    for k in 1..a.len() {
        let (mut s_sum, mut c_sum) = (0.0, 0.0);
        for j in 1..=k {
            let weight = float(j) * at(a, j);
            s_sum += weight * at(&c, k.saturating_sub(j));
            c_sum += weight * at(&s, k.saturating_sub(j));
        }
        s.push(s_sum / float(k));
        c.push(sign * c_sum / float(k));
    }
    (s, c)
}

/// 級数の導関数．長さは変えず，最高次の係数は0にする．
fn differentiate(a: &[f64]) -> Series {
    (0..a.len())
        .map(|k| float(k.saturating_add(1)) * at(a, k.saturating_add(1)))
        .collect()
}

/// 導関数`derivative`の級数を積分し，定数項を`start`にする．長さは変えない．
fn integrate(derivative: &[f64], start: f64) -> Series {
    std::iter::once(start)
        .chain((1..derivative.len()).map(|k| at(derivative, k.saturating_sub(1)) / float(k)))
        .collect()
}
