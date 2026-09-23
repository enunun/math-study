//! 特殊関数(ガンマ関数，誤差関数，ベッセル関数など)を式の中で使えることを確かめる．
//! 数値そのものの正しさは`puruspe`が検証済みなので，ここでは式のエンジンが正しい関数へ
//! つないでいることだけを確かめる．

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use figure::expr::Expr;

fn eval(source: &str) -> f64 {
    Expr::compile(source, &[]).expect("式を読める").eval(&[])
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-12
}

/// `puruspe`の近似には，桁落ちで`1e-12`よりゆるい誤差になるものがある．
fn close_loosely(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-9
}

#[test]
fn ガンマ関数は階乗を一般化する() {
    assert!(close(eval("gamma(5)"), 24.0));
    assert!(close(eval("gamma(1)"), 1.0));
    assert!(close(eval("gamma(0.5)"), std::f64::consts::PI.sqrt()));
    assert_eq!(eval("gamma(5)"), puruspe::gamma(5.0));
}

#[test]
fn 対数ガンマ関数はガンマ関数の自然対数である() {
    assert!(close_loosely(eval("loggamma(1)"), 0.0));
    assert!(close_loosely(eval("loggamma(5)"), 24.0_f64.ln()));
    assert_eq!(eval("lgamma(5)"), eval("loggamma(5)"));
    assert_eq!(eval("loggamma(30)"), puruspe::ln_gamma(30.0));
}

#[test]
fn 誤差関数は原点で0_無限遠で1に近づく() {
    assert_eq!(eval("erf(0)"), 0.0);
    assert!(close(eval("erf(1)") + eval("erfc(1)"), 1.0));
    assert_eq!(eval("erf(1)"), puruspe::erf(1.0));
    assert_eq!(eval("erfc(1)"), puruspe::erfc(1.0));
}

#[test]
fn ドーソン関数は原点で0である() {
    assert_eq!(eval("dawson(0)"), 0.0);
    assert_eq!(eval("dawson(1)"), puruspe::dawson(1.0));
}

#[test]
fn ランベルトのw関数は_xe_xの逆関数である() {
    assert!(close(eval("lambertw(0)"), 0.0));
    assert!(close(eval("lambertw(e)"), 1.0));
    assert_eq!(eval("lambertw(2)"), puruspe::lambert_w0(2.0));
}

#[test]
fn ベッセル関数は次数0と1を使える() {
    assert!(close_loosely(eval("besselj0(0)"), 1.0));
    assert_eq!(eval("besselj1(0)"), 0.0);
    assert!(close_loosely(eval("besseli0(0)"), 1.0));
    assert_eq!(eval("besseli1(0)"), 0.0);
    assert_eq!(eval("besselj0(1)"), puruspe::Jn(0, 1.0));
    assert_eq!(eval("besselj1(1)"), puruspe::Jn(1, 1.0));
    assert_eq!(eval("bessely0(1)"), puruspe::Yn(0, 1.0));
    assert_eq!(eval("bessely1(1)"), puruspe::Yn(1, 1.0));
    assert_eq!(eval("besseli0(1)"), puruspe::In(0, 1.0));
    assert_eq!(eval("besseli1(1)"), puruspe::In(1, 1.0));
    assert_eq!(eval("besselk0(1)"), puruspe::Kn(0, 1.0));
    assert_eq!(eval("besselk1(1)"), puruspe::Kn(1, 1.0));
}
