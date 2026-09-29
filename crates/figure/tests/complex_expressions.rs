//! 複素数の式を確かめる．複素数の式(`Expr::compile_complex`)では，`i`が虚数単位になり，初等関数，ガンマ関数，
//! Jacobiの楕円関数，Weierstrassの楕円関数を複素数で評価する．実数でしか定めていない特殊関数は，
//! 引数が実数でなければ非数になる．

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use std::f64::consts::{FRAC_PI_2, PI};

use figure::expr::{Complex, Expr, ExprErrorKind, Functions};

fn eval(source: &str) -> Complex {
    Expr::compile_complex(source, &[], &Functions::default())
        .expect("式を読める")
        .eval_complex(&[])
}

fn eval_at(source: &str, z: Complex) -> Complex {
    Expr::compile_complex(source, &["z"], &Functions::default())
        .expect("式を読める")
        .eval_complex(&[z])
}

fn c(re: f64, im: f64) -> Complex {
    Complex::new(re, im)
}

fn assert_close(actual: Complex, expected: Complex, tolerance: f64) {
    let error = (actual.re - expected.re).hypot(actual.im - expected.im);
    assert!(
        error <= tolerance,
        "{actual:?} と {expected:?} の差 {error} が {tolerance} を超える"
    );
}

#[test]
fn 虚数単位の2乗は負の1である() {
    assert_close(eval("i * i"), c(-1.0, 0.0), 0.0);
    assert_close(eval("i^2"), c(-1.0, 0.0), 1e-15);
    assert_close(eval("(1 + i)^2"), c(0.0, 2.0), 1e-15);
    assert_close(eval("1 / i"), c(0.0, -1.0), 0.0);
    assert_close(eval("(3 + 4*i) / (1 - 2*i)"), c(-1.0, 2.0), 1e-15);
}

#[test]
fn eulerの公式が成り立つ() {
    assert_close(eval("exp(i * pi)"), c(-1.0, 0.0), 1e-15);
    assert_close(eval("exp(i * pi / 2)"), c(0.0, 1.0), 1e-15);
    assert_close(eval("cos(1 + 2*i)^2 + sin(1 + 2*i)^2"), c(1.0, 0.0), 1e-13);
    assert_close(eval("sin(i)"), c(0.0, 1.0_f64.sinh()), 1e-15);
    assert_close(eval("cosh(i * pi)"), c(-1.0, 0.0), 1e-15);
}

#[test]
fn 対数と平方根は主値である() {
    assert_close(eval("log(-1)"), c(0.0, PI), 1e-15);
    assert_close(eval("sqrt(-4)"), c(0.0, 2.0), 1e-15);
    assert_close(eval("sqrt(i)^2"), c(0.0, 1.0), 1e-15);
    assert_close(eval("exp(log(2 - 3*i))"), c(2.0, -3.0), 1e-14);
    assert_close(eval("(-8)^(1/3)"), c(1.0, 3.0_f64.sqrt()), 1e-14);
    assert_close(eval("0^2"), c(0.0, 0.0), 0.0);
}

#[test]
fn 逆三角関数は元の関数の逆である() {
    for (re, im) in [(0.3, 0.4), (-1.2, 0.5), (2.0, -1.0)] {
        let z = c(re, im);
        assert_close(eval_at("sin(asin(z))", z), z, 1e-13);
        assert_close(eval_at("cos(acos(z))", z), z, 1e-13);
        assert_close(eval_at("tan(atan(z))", z), z, 1e-13);
    }
}

#[test]
fn 実部_虚部_共役_偏角_絶対値() {
    assert_close(eval("re(3 + 4*i)"), c(3.0, 0.0), 0.0);
    assert_close(eval("im(3 + 4*i)"), c(4.0, 0.0), 0.0);
    assert_close(eval("conj(3 + 4*i)"), c(3.0, -4.0), 0.0);
    assert_close(eval("abs(3 + 4*i)"), c(5.0, 0.0), 1e-15);
    assert_close(eval("arg(i)"), c(FRAC_PI_2, 0.0), 1e-15);
    assert_close(eval("arg(-1)"), c(PI, 0.0), 1e-15);
}

#[test]
fn ガンマ関数は複素数でも使える() {
    assert_close(eval("gamma(0.5)"), c(PI.sqrt(), 0.0), 1e-14);
    assert_close(eval("gamma(5)"), c(24.0, 0.0), 1e-12);
    assert_close(eval("gamma(-0.5)"), c(-2.0 * PI.sqrt(), 0.0), 1e-13);
    // Γ(1 + i)．
    assert_close(
        eval("gamma(1 + i)"),
        c(0.498_015_668_118_356, -0.154_949_828_301_811),
        1e-14,
    );
    // Γ(z + 1) = z Γ(z)．
    let z = c(-2.3, 1.7);
    let ratio = eval_at("gamma(z + 1) / (z * gamma(z))", z);
    assert_close(ratio, c(1.0, 0.0), 1e-12);
    // 鏡映公式Γ(z)Γ(1 - z) = π / sin(πz)．
    let reflected = eval_at("gamma(z) * gamma(1 - z) * sin(pi * z) / pi", z);
    assert_close(reflected, c(1.0, 0.0), 1e-12);
}

#[test]
fn 実数でしか定めていない関数は_実数でない引数で非数になる() {
    assert!(eval("erf(1 + i)").re.is_nan());
    assert_close(eval("erf(1 + 0*i)"), c(puruspe::erf(1.0), 0.0), 0.0);
    assert!(eval("besselj0(i)").re.is_nan());
    assert!(eval("ellipk(i)").re.is_nan());
}

#[test]
fn jacobiの楕円関数は複素数の引数でも恒等式を満たす() {
    let k = 0.6_f64;
    for (re, im) in [(0.3, 0.4), (1.7, -0.8), (-2.2, 1.9)] {
        let z = c(re, im);
        let sn = eval_at("sn(z, 0.6)", z);
        let cn = eval_at("cn(z, 0.6)", z);
        let dn = eval_at("dn(z, 0.6)", z);
        assert_close(sn * sn + cn * cn, c(1.0, 0.0), 1e-12);
        assert_close(dn * dn + sn * sn * c(k * k, 0.0), c(1.0, 0.0), 1e-12);
    }
    // 実数の引数では，実数の値と一致する．
    assert_close(
        eval("sn(0.7, 0.6)"),
        c(Expr::compile("sn(0.7, 0.6)", &[]).unwrap().eval(&[]), 0.0),
        1e-15,
    );
}

#[test]
fn jacobiの虚数変換() {
    // sn(iy, k) = i sn(y, k') / cn(y, k')．k = 0.6なら，k' = 0.8．
    let y = 0.9;
    let expected = eval(&format!("i * sn({y}, 0.8) / cn({y}, 0.8)"));
    assert_close(eval(&format!("sn({y} * i, 0.6)")), expected, 1e-13);
}

#[test]
fn jacobiの楕円関数は虚の周期をもつ() {
    // sn(z + 2iK', k) = sn(z, k)．K' = K(k')．
    let z = c(0.4, 0.3);
    let shifted = eval_at("sn(z + 2 * i * ellipk(0.8), 0.6)", z);
    assert_close(shifted, eval_at("sn(z, 0.6)", z), 1e-11);
}

#[test]
fn weierstrassのペー関数は複素数で微分方程式と二重周期をもつ() {
    for (g2, g3) in [(4.0, 1.0), (1.0, 2.0)] {
        for (re, im) in [(0.3, 0.4), (0.7, -0.5)] {
            let z = c(re, im);
            let p = eval_at(&format!("wp(z, {g2}, {g3})"), z);
            let dp = eval_at(&format!("wpd(z, {g2}, {g3})"), z);
            let rhs = p * p * p * c(4.0, 0.0) - p * c(g2, 0.0) - c(g3, 0.0);
            assert_close(dp * dp, rhs, 1e-8 * (1.0 + rhs.norm()));
        }
    }
    // g2 = 4，g3 = 0：e1 = 1，e3 = -1，k = 1/√2．周期は2ω1 = 2K/√2と2ω3 = 2iK'/√2．
    let z = c(0.3, 0.2);
    let base = eval_at("wp(z, 4, 0)", z);
    let real_shift = eval_at("wp(z + 2 * ellipk(1/sqrt(2)) / sqrt(2), 4, 0)", z);
    let imaginary_shift = eval_at("wp(z + 2 * i * ellipk(1/sqrt(2)) / sqrt(2), 4, 0)", z);
    assert_close(real_shift, base, 1e-10);
    assert_close(imaginary_shift, base, 1e-10);
}

#[test]
fn 実数の式ではiは使えない_複素数の式ではiはいつも虚数単位である() {
    let error = Expr::compile("i", &[]).expect_err("実数の式ではiを使えない");
    assert_eq!(error.kind, ExprErrorKind::UnknownName("i".to_owned()));
    // 名前の並びに`i`があっても，虚数単位を表す．
    let expr = Expr::compile_complex("i", &["i"], &Functions::default()).unwrap();
    assert_close(expr.eval_complex(&[c(5.0, 0.0)]), c(0.0, 1.0), 0.0);
}

#[test]
fn 実数の式でも実部などの関数を使える() {
    let value = |source: &str| Expr::compile(source, &[]).unwrap().eval(&[]);
    assert_eq!(value("re(-2)"), -2.0);
    assert_eq!(value("im(-2)"), 0.0);
    assert_eq!(value("conj(-2)"), -2.0);
    assert_eq!(value("arg(-2)"), PI);
    assert_eq!(value("arg(2)"), 0.0);
}

#[test]
fn 利用者の関数は複素数の式でも使える() {
    let mut functions = Functions::default();
    functions.define("f", &["w"], "w^2 + 1", &[]).unwrap();
    let expr = Expr::compile_complex("f(z)", &["z"], &functions).unwrap();
    assert_close(expr.eval_complex(&[c(0.0, 1.0)]), c(0.0, 0.0), 1e-15);
}
