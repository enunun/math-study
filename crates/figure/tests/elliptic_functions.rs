//! 楕円積分と楕円関数(Jacobi，Weierstrass)を式の中で使えることを確かめる．
//! 値は，既知の値(完全楕円積分の特殊値)と，関数の間の恒等式と，微分方程式で確かめる．

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use std::f64::consts::{FRAC_PI_2, PI, SQRT_2};

use figure::expr::{Expr, ExprErrorKind};

fn eval(source: &str) -> f64 {
    Expr::compile(source, &[]).expect("式を読める").eval(&[])
}

fn eval_at(source: &str, x: f64) -> f64 {
    Expr::compile(source, &["x"])
        .expect("式を読める")
        .eval(&[x])
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} と {expected} の差が {tolerance} を超える"
    );
}

/// 中心差分による微分．
fn derivative(source: &str, x: f64) -> f64 {
    let h = 1e-5;
    (eval_at(source, x + h) - eval_at(source, x - h)) / (2.0 * h)
}

#[test]
fn 完全楕円積分は既知の値をとる() {
    assert_close(eval("ellipk(0)"), FRAC_PI_2, 1e-15);
    assert_close(eval("ellipe(0)"), FRAC_PI_2, 1e-15);
    assert_close(eval("ellipe(1)"), 1.0, 1e-15);
    assert!(eval("ellipk(1)").is_infinite());
    // K(1/√2) = Γ(1/4)^2 / (4√π)，E(1/√2) = K/2 + π/(4K)(Legendreの関係から)．
    let k = puruspe::gamma(0.25).powi(2) / (4.0 * PI.sqrt());
    assert_close(eval("ellipk(1 / sqrt(2))"), k, 1e-14);
    assert_close(eval("ellipe(1 / sqrt(2))"), k / 2.0 + PI / (4.0 * k), 1e-14);
    // 母数の符号によらない．
    assert_eq!(eval("ellipk(-0.3)"), eval("ellipk(0.3)"));
}

#[test]
fn legendreの関係式が成り立つ() {
    for k in [0.1_f64, 0.5, 0.9, 0.999] {
        let kp = (1.0 - k * k).sqrt();
        let first = eval(&format!("ellipk({k})"));
        let second = eval(&format!("ellipe({k})"));
        let first_complement = eval(&format!("ellipk({kp})"));
        let second_complement = eval(&format!("ellipe({kp})"));
        assert_close(
            second * first_complement + second_complement * first - first * first_complement,
            FRAC_PI_2,
            1e-13,
        );
    }
}

#[test]
fn 不完全楕円積分は振幅がpi_2で完全楕円積分になる() {
    assert_close(eval("ellipf(pi / 2, 0.6)"), eval("ellipk(0.6)"), 1e-14);
    assert_close(eval("ellipeinc(pi / 2, 0.6)"), eval("ellipe(0.6)"), 1e-14);
    assert_eq!(eval("ellipf(0, 0.6)"), 0.0);
    // 母数0では，振幅そのもの．
    assert_close(eval("ellipf(1.2, 0)"), 1.2, 1e-15);
    assert_close(eval("ellipeinc(1.2, 0)"), 1.2, 1e-15);
    // 振幅がpi/2を超えても，Kずつ増える．
    assert_close(
        eval("ellipf(pi + 0.4, 0.6)"),
        2.0 * eval("ellipk(0.6)") + eval("ellipf(0.4, 0.6)"),
        1e-13,
    );
    assert_close(
        eval("ellipeinc(-0.4, 0.6)"),
        -eval("ellipeinc(0.4, 0.6)"),
        1e-15,
    );
    // 被積分関数の微分．
    let k = 0.7_f64;
    for phi in [0.3_f64, 1.0, 2.5] {
        let s = phi.sin();
        let expected = 1.0 / (1.0 - k * k * s * s).sqrt();
        assert_close(derivative("ellipf(x, 0.7)", phi), expected, 1e-8);
        assert_close(derivative("ellipeinc(x, 0.7)", phi), 1.0 / expected, 1e-8);
    }
}

#[test]
fn jacobiの楕円関数は恒等式を満たす() {
    for k in [0.0, 0.3, 0.8, 0.999_999, 1.0, 1.5] {
        for u in [-3.7, -0.2, 0.0, 0.5, 1.3, 10.0, 123.4] {
            let sn = eval(&format!("sn({u}, {k})"));
            let cn = eval(&format!("cn({u}, {k})"));
            let dn = eval(&format!("dn({u}, {k})"));
            assert_close(sn * sn + cn * cn, 1.0, 1e-12);
            assert_close(dn * dn + k * k * sn * sn, 1.0, 1e-12);
        }
    }
}

#[test]
fn jacobiの楕円関数は母数0で三角関数_1で双曲線関数になる() {
    for u in [-2.0_f64, 0.3, 1.7] {
        assert_close(eval(&format!("sn({u}, 0)")), u.sin(), 1e-15);
        assert_close(eval(&format!("cn({u}, 0)")), u.cos(), 1e-15);
        assert_close(eval(&format!("dn({u}, 0)")), 1.0, 1e-15);
        assert_close(eval(&format!("am({u}, 0)")), u, 1e-15);
        assert_close(eval(&format!("sn({u}, 1)")), u.tanh(), 1e-15);
        assert_close(eval(&format!("cn({u}, 1)")), 1.0 / u.cosh(), 1e-15);
        assert_close(eval(&format!("dn({u}, 1)")), 1.0 / u.cosh(), 1e-15);
    }
}

#[test]
fn jacobiの楕円関数は周期4kをもち_kで特別な値をとる() {
    let big_k = eval("ellipk(0.6)");
    assert_close(eval("sn(ellipk(0.6), 0.6)"), 1.0, 1e-14);
    assert_close(eval("cn(ellipk(0.6), 0.6)"), 0.0, 1e-14);
    assert_close(eval("dn(ellipk(0.6), 0.6)"), 0.8, 1e-14);
    assert_close(eval("am(ellipk(0.6), 0.6)"), FRAC_PI_2, 1e-14);
    for u in [0.1, 0.9, 2.2] {
        let shifted = u + 4.0 * big_k;
        assert_close(
            eval_at("sn(x, 0.6)", shifted),
            eval_at("sn(x, 0.6)", u),
            1e-13,
        );
        assert_close(
            eval_at("cn(x, 0.6)", shifted),
            eval_at("cn(x, 0.6)", u),
            1e-13,
        );
        assert_close(
            eval_at("dn(x, 0.6)", u + 2.0 * big_k),
            eval_at("dn(x, 0.6)", u),
            1e-13,
        );
    }
}

#[test]
fn jacobiの楕円関数の導関数() {
    let k = 0.75;
    for u in [0.2, 1.1, 2.9, -1.4] {
        let sn = eval_at("sn(x, 0.75)", u);
        let cn = eval_at("cn(x, 0.75)", u);
        let dn = eval_at("dn(x, 0.75)", u);
        assert_close(derivative("sn(x, 0.75)", u), cn * dn, 1e-8);
        assert_close(derivative("cn(x, 0.75)", u), -sn * dn, 1e-8);
        assert_close(derivative("dn(x, 0.75)", u), -k * k * sn * cn, 1e-8);
        assert_close(derivative("am(x, 0.75)", u), dn, 1e-8);
    }
}

#[test]
fn 振幅関数は第1種不完全楕円積分の逆関数である() {
    for phi in [0.2, 1.0, 1.5, 3.0, -2.0] {
        assert_close(eval(&format!("am(ellipf({phi}, 0.9), 0.9)")), phi, 1e-12);
    }
}

#[test]
fn weierstrassのペー関数は微分方程式を満たす() {
    // 判別式が正(実の3根)，負(実の1根)，レムニスケートの場合．
    for (g2, g3) in [
        (4.0, 1.0),
        (1.0, 2.0),
        (1.0, 0.0),
        (0.0, 1.0),
        (-3.0, 0.5),
        (3.0, -1.0),
    ] {
        for z in [0.3, 0.7, 1.1] {
            let p = eval(&format!("wp({z}, {g2}, {g3})"));
            let dp = eval(&format!("wpd({z}, {g2}, {g3})"));
            let rhs = 4.0 * p * p * p - g2 * p - g3;
            assert_close(dp * dp, rhs, 1e-9 * rhs.abs().max(1.0));
            let numeric = derivative(&format!("wp(x, {g2}, {g3})"), z);
            assert_close(dp, numeric, 1e-6 * dp.abs().max(1.0));
        }
    }
}

#[test]
fn weierstrassのペー関数は原点の近くで1_z2に近い() {
    assert_close(eval("wp(0.001, 2, 1)") * 1e-6, 1.0, 1e-6);
    assert_close(eval("wp(0.5, 0, 0)"), 4.0, 1e-15);
    assert_close(eval("wpd(0.5, 0, 0)"), -16.0, 1e-13);
    // 偶関数．
    assert_close(eval("wp(-0.4, 2, 1)"), eval("wp(0.4, 2, 1)"), 1e-13);
    assert_close(eval("wpd(-0.4, 2, 1)"), -eval("wpd(0.4, 2, 1)"), 1e-12);
}

#[test]
fn weierstrassのペー関数は実の周期をもつ() {
    // g2 = 4，g3 = 0 は，e1 = 1，e2 = 0，e3 = -1．周期2ω = 2K(1/√2)/√2．
    let omega = eval("ellipk(1 / sqrt(2))") / SQRT_2;
    assert_close(eval_at("wp(x, 4, 0)", omega), 1.0, 1e-12);
    for z in [0.3, 0.9] {
        assert_close(
            eval_at("wp(x, 4, 0)", z + 2.0 * omega),
            eval_at("wp(x, 4, 0)", z),
            1e-10,
        );
    }
}

#[test]
fn 引数の数が違えば誤りになる() {
    let error = Expr::compile("sn(1)", &[]).expect_err("引数が足りない");
    assert_eq!(
        error.kind,
        ExprErrorKind::ArgumentCount {
            name: "sn".to_owned(),
            expected: 2,
            found: 1
        }
    );
    let error = Expr::compile("sin(1, 2)", &[]).expect_err("引数が多すぎる");
    assert_eq!(
        error.kind,
        ExprErrorKind::ArgumentCount {
            name: "sin".to_owned(),
            expected: 1,
            found: 2
        }
    );
    let error = Expr::compile("wp(1, 2)", &[]).expect_err("引数が足りない");
    assert_eq!(
        error.kind,
        ExprErrorKind::ArgumentCount {
            name: "wp".to_owned(),
            expected: 3,
            found: 2
        }
    );
}

#[test]
fn 楕円関数はtaylor展開しない() {
    let expr = Expr::compile("sn(x, 0.5)", &["x"]).expect("式を読める");
    assert_eq!(expr.taylor(0, &[0.3], 3), None);
}
