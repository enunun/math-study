//! 楕円積分と楕円関数．母数は`k`(母数の2乗`m = k^2`ではない)で渡す．
//!
//! 完全楕円積分は算術幾何平均(AGM)，不完全楕円積分はCarlsonの対称形`R_F`，`R_D`，Jacobiの楕円関数は
//! AGMの下降Landen変換で求める．Weierstrassの`℘`は，`4t^3 - g2 t - g3`の実根からJacobiの楕円関数に
//! 帰着させる．どれも倍精度の丸め誤差の程度まで収束させる．

use std::f64::consts::FRAC_PI_2;

/// 反復の上限．AGMは2次収束なので，倍精度ではどれも10回以内に収まる．
const MAX_ITERATIONS: usize = 40;

/// 収束の判定に使う相対誤差．
const TOLERANCE: f64 = 1e-16;

/// 算術幾何平均の列．`(a_n, b_n, c_n)`を，`c_n`が十分小さくなるまで並べる．
fn agm_sequence(a: f64, b: f64, c: f64) -> Vec<(f64, f64, f64)> {
    let mut steps = vec![(a, b, c)];
    let (mut a, mut b, mut c) = (a, b, c);
    for _ in 0..MAX_ITERATIONS {
        if c.abs() <= TOLERANCE * a.abs() {
            break;
        }
        (a, b, c) = (f64::midpoint(a, b), (a * b).sqrt(), (a - b) / 2.0);
        steps.push((a, b, c));
    }
    steps
}

/// 補母数`k' = sqrt(1 - k^2)`．
fn complementary(k: f64) -> f64 {
    (1.0 - k * k).max(0.0).sqrt()
}

/// 第1種完全楕円積分`K(k)`．`|k| = 1`で無限大，`|k| > 1`で非数である．
#[must_use]
pub fn ellipk(k: f64) -> f64 {
    let k = k.abs();
    if k > 1.0 || k.is_nan() {
        return f64::NAN;
    }
    if k >= 1.0 {
        return f64::INFINITY;
    }
    let steps = agm_sequence(1.0, complementary(k), k);
    let a = steps.last().map_or(f64::NAN, |step| step.0);
    FRAC_PI_2 / a
}

/// 第2種完全楕円積分`E(k)`．`E = K (1 - Σ 2^(n-1) c_n^2)`で求める．
#[must_use]
pub fn ellipe(k: f64) -> f64 {
    let k = k.abs();
    if k > 1.0 || k.is_nan() {
        return f64::NAN;
    }
    if k >= 1.0 {
        return 1.0;
    }
    let steps = agm_sequence(1.0, complementary(k), k);
    let mut sum = 0.0;
    let mut weight = 0.5;
    for &(_, _, c) in &steps {
        sum += weight * c * c;
        weight *= 2.0;
    }
    ellipk(k) * (1.0 - sum)
}

/// Carlsonの対称形の第1種楕円積分`R_F(x, y, z)`．
fn carlson_rf(x: f64, y: f64, z: f64) -> f64 {
    let (mut x, mut y, mut z) = (x, y, z);
    for _ in 0..MAX_ITERATIONS {
        let mean = (x + y + z) / 3.0;
        let spread = (mean - x).abs().max((mean - y).abs()).max((mean - z).abs());
        if spread <= 1e-4 * mean.abs() {
            break;
        }
        let lambda = (x * y).sqrt() + (y * z).sqrt() + (z * x).sqrt();
        (x, y, z) = ((x + lambda) / 4.0, (y + lambda) / 4.0, (z + lambda) / 4.0);
    }
    // 平均のまわりの展開(Carlson 1995，5次まで)．
    let mean = (x + y + z) / 3.0;
    let (dx, dy) = (1.0 - x / mean, 1.0 - y / mean);
    let dz = -dx - dy;
    let e2 = dx * dy - dz * dz;
    let e3 = dx * dy * dz;
    (1.0 - e2 / 10.0 + e3 / 14.0 + e2 * e2 / 24.0 - 3.0 * e2 * e3 / 44.0) / mean.sqrt()
}

/// Carlsonの対称形の第2種楕円積分`R_D(x, y, z)`．
fn carlson_rd(x: f64, y: f64, z: f64) -> f64 {
    let (mut x, mut y, mut z) = (x, y, z);
    let mut sum = 0.0;
    let mut factor = 1.0;
    for _ in 0..MAX_ITERATIONS {
        let mean = (x + y + 3.0 * z) / 5.0;
        let spread = (mean - x).abs().max((mean - y).abs()).max((mean - z).abs());
        if spread <= 1e-4 * mean.abs() {
            break;
        }
        let lambda = (x * y).sqrt() + (y * z).sqrt() + (z * x).sqrt();
        sum += factor / (z.sqrt() * (z + lambda));
        factor /= 4.0;
        (x, y, z) = ((x + lambda) / 4.0, (y + lambda) / 4.0, (z + lambda) / 4.0);
    }
    let mean = (x + y + 3.0 * z) / 5.0;
    let (dx, dy) = (1.0 - x / mean, 1.0 - y / mean);
    let dz = -(dx + dy) / 3.0;
    let ea = dx * dy;
    let eb = dz * dz;
    let ec = ea - eb;
    let ed = ea - 6.0 * eb;
    let ef = ed + ec + ec;
    let series = 1.0
        + ed * (-3.0 / 14.0 + 9.0 / 88.0 * ed - 9.0 / 52.0 * dz * ef)
        + dz * (ef / 6.0 + dz * (-9.0 / 22.0 * ec + dz * 3.0 / 26.0 * ea));
    3.0 * sum + factor * series / (mean * mean.sqrt())
}

/// 振幅を，`-π/2`から`π/2`の間の値と，`π`の何倍ずれているかに分ける．
fn reduce_amplitude(phi: f64) -> (f64, f64) {
    let turns = (phi / std::f64::consts::PI).round();
    (phi - turns * std::f64::consts::PI, turns)
}

/// 第1種不完全楕円積分`F(φ, k)`．`|k| > 1`では，被積分関数が実数になる振幅でだけ値をもつ．
#[must_use]
pub fn ellipf(phi: f64, k: f64) -> f64 {
    let (rest, turns) = reduce_amplitude(phi);
    let (s, c) = rest.sin_cos();
    let m = k * k;
    let partial = s * carlson_rf(c * c, 1.0 - m * s * s, 1.0);
    if turns == 0.0 {
        partial
    } else {
        2.0 * turns * ellipk(k) + partial
    }
}

/// 第2種不完全楕円積分`E(φ, k)`．
#[must_use]
pub fn ellipeinc(phi: f64, k: f64) -> f64 {
    let (rest, turns) = reduce_amplitude(phi);
    let (s, c) = rest.sin_cos();
    let m = k * k;
    let delta = 1.0 - m * s * s;
    let partial =
        s * carlson_rf(c * c, delta, 1.0) - m * s * s * s * carlson_rd(c * c, delta, 1.0) / 3.0;
    if turns == 0.0 {
        partial
    } else {
        2.0 * turns * ellipe(k) + partial
    }
}

/// Jacobiの楕円関数の値の組．
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jacobi {
    /// 振幅`am(u, k)`．
    pub am: f64,
    /// `sn(u, k)`．
    pub sn: f64,
    /// `cn(u, k)`．
    pub cn: f64,
    /// `dn(u, k)`．
    pub dn: f64,
}

/// Jacobiの楕円関数`am`，`sn`，`cn`，`dn`をまとめて求める．`k`の符号によらない．
/// `|k| > 1`は，逆数の母数への変換(`sn(u, k) = sn(k u, 1/k) / k`など)で求める．
#[must_use]
pub fn jacobi(u: f64, k: f64) -> Jacobi {
    let k = k.abs();
    if k.is_nan() || !u.is_finite() {
        return Jacobi {
            am: f64::NAN,
            sn: f64::NAN,
            cn: f64::NAN,
            dn: f64::NAN,
        };
    }
    if k > 1.0 {
        let inner = jacobi(k * u, 1.0 / k);
        let sn = inner.sn / k;
        return Jacobi {
            am: sn.asin(),
            sn,
            cn: inner.dn,
            dn: inner.cn,
        };
    }
    if k >= 1.0 {
        let sech = 1.0 / u.cosh();
        return Jacobi {
            am: u.sinh().atan(),
            sn: u.tanh(),
            cn: sech,
            dn: sech,
        };
    }
    if k == 0.0 {
        let (sn, cn) = u.sin_cos();
        return Jacobi {
            am: u,
            sn,
            cn,
            dn: 1.0,
        };
    }
    // 周期4Kで，u を[-2K, 2K]に寄せる．振幅は，寄せた分だけ2πずつずらす．
    let quarter = ellipk(k);
    let periods = (u / (4.0 * quarter)).round();
    let reduced = u - periods * 4.0 * quarter;
    let steps = agm_sequence(1.0, complementary(k), k);
    let Some(&(a_last, _, _)) = steps.last() else {
        return Jacobi {
            am: f64::NAN,
            sn: f64::NAN,
            cn: f64::NAN,
            dn: f64::NAN,
        };
    };
    let count = steps.len().saturating_sub(1);
    let mut phi = f64::from(u32::try_from(count).unwrap_or(0)).exp2() * a_last * reduced;
    for &(a, _, c) in steps.iter().skip(1).rev() {
        phi = f64::midpoint(phi, (c / a * phi.sin()).clamp(-1.0, 1.0).asin());
    }
    let (sn, cn) = phi.sin_cos();
    // k < 1 では dn > 0 である．cos φ0 / cos(φ1 - φ0) は cn = 0 の近くで 0/0 になるので，使わない．
    let dn = (1.0 - k * k * sn * sn).sqrt();
    Jacobi {
        am: phi + periods * std::f64::consts::TAU,
        sn,
        cn,
        dn,
    }
}

/// `4t^3 - g2 t - g3 = 0`の根．判別式が0以上なら，実の3根を大きい順に，負なら，実の1根だけを返す．
fn weierstrass_roots(g2: f64, g3: f64) -> Result<[f64; 3], f64> {
    let discriminant = g2 * g2 * g2 - 27.0 * g3 * g3;
    // t^3 + p t + q = 0 に直す．
    let p = -g2 / 4.0;
    let q = -g3 / 4.0;
    if discriminant >= 0.0 {
        if p == 0.0 {
            return Ok([0.0; 3]);
        }
        let radius = 2.0 * (-p / 3.0).sqrt();
        let angle = ((3.0 * q / (2.0 * p)) * (-3.0 / p).sqrt())
            .clamp(-1.0, 1.0)
            .acos()
            / 3.0;
        let third = std::f64::consts::TAU / 3.0;
        let mut roots = [
            radius * angle.cos(),
            radius * (angle - third).cos(),
            radius * (angle - 2.0 * third).cos(),
        ];
        roots.sort_by(|a, b| b.total_cmp(a));
        Ok(roots)
    } else {
        let root = (q * q / 4.0 + p * p * p / 27.0).sqrt();
        Err((-q / 2.0 + root).cbrt() + (-q / 2.0 - root).cbrt())
    }
}

/// Weierstrassの`℘`を，Jacobiの楕円関数で書くための定数．
pub enum WeierstrassForm {
    /// 判別式が0以上(実根が3つ)．`℘ = e3 + spread / sn^2(scale z, k)`．
    ThreeRoots {
        /// いちばん小さい根`e3`．
        e3: f64,
        /// `e1 - e3`．
        spread: f64,
        /// `sqrt(e1 - e3)`．
        scale: f64,
        /// 母数`k = sqrt((e2 - e3) / (e1 - e3))`．
        k: f64,
    },
    /// 判別式が負(実根が1つ)．`℘ = e2 + h (1 + cn(2 root z, k)) / (1 - cn(2 root z, k))`．
    OneRoot {
        /// 実根`e2`．
        e2: f64,
        /// `H = sqrt(3 e2^2 - g2/4)`．
        h: f64,
        /// `sqrt(H)`．
        root: f64,
        /// 母数`k = sqrt(1/2 - 3 e2 / (4H))`．
        k: f64,
    },
}

/// 不変量から，`℘`をJacobiの楕円関数で書くための定数を求める．
#[must_use]
pub fn weierstrass_form(g2: f64, g3: f64) -> WeierstrassForm {
    match weierstrass_roots(g2, g3) {
        Ok([e1, e2, e3]) => {
            let spread = e1 - e3;
            WeierstrassForm::ThreeRoots {
                e3,
                spread,
                scale: spread.sqrt(),
                k: ((e2 - e3) / spread).clamp(0.0, 1.0).sqrt(),
            }
        }
        Err(e2) => {
            let h = (3.0 * e2).mul_add(e2, -g2 / 4.0).sqrt();
            WeierstrassForm::OneRoot {
                e2,
                h,
                root: h.sqrt(),
                k: (0.5 - 3.0 * e2 / (4.0 * h)).clamp(0.0, 1.0).sqrt(),
            }
        }
    }
}

/// Weierstrassの`℘(z; g2, g3)`と，その導関数`℘'(z)`．不変量は実数に限る．
#[must_use]
pub fn weierstrass(z: f64, g2: f64, g3: f64) -> (f64, f64) {
    if g2 == 0.0 && g3 == 0.0 {
        return (1.0 / (z * z), -2.0 / (z * z * z));
    }
    match weierstrass_form(g2, g3) {
        WeierstrassForm::ThreeRoots {
            e3,
            spread,
            scale,
            k,
        } => {
            let Jacobi { sn, cn, dn, .. } = jacobi(scale * z, k);
            let value = e3 + spread / (sn * sn);
            let slope = -2.0 * spread * scale * cn * dn / (sn * sn * sn);
            (value, slope)
        }
        WeierstrassForm::OneRoot { e2, h, root, k } => {
            let Jacobi { sn, cn, dn, .. } = jacobi(2.0 * root * z, k);
            let value = e2 + h * (1.0 + cn) / (1.0 - cn);
            let slope = -4.0 * h * root * sn * dn / ((1.0 - cn) * (1.0 - cn));
            (value, slope)
        }
    }
}
