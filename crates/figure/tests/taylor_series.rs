//! 式のテイラー係数(`Expr::taylor`)を確かめる．数値微分ではなく，べき級数の演算で求めるので，
//! 高い次数でも正確である．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::cast_precision_loss,
    clippy::as_conversions
)]

use figure::expr::Expr;

fn coefficients(source: &str, at: f64, order: usize) -> Vec<f64> {
    Expr::compile(source, &["x"])
        .expect("読める")
        .taylor(0, &[at], order)
        .expect("展開できる")
}

fn factorial(n: usize) -> f64 {
    (1..=n).map(|k| k as f64).product()
}

fn assert_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a - e).abs() < 1e-10 * e.abs().max(1.0),
            "{index}次の係数：{a}と{e}"
        );
    }
}

#[test]
fn 指数関数の係数は階乗の逆数である() {
    let expected: Vec<f64> = (0..=12).map(|k| 1.0 / factorial(k)).collect();
    assert_close(&coefficients("exp(x)", 0.0, 12), &expected);
}

#[test]
fn 正弦の係数は奇数次だけで符号が交互に変わる() {
    let expected: Vec<f64> = (0..=11)
        .map(|k| {
            if k % 2 == 0 {
                0.0
            } else {
                let sign = if (k / 2) % 2 == 0 { 1.0 } else { -1.0 };
                sign / factorial(k)
            }
        })
        .collect();
    assert_close(&coefficients("sin(x)", 0.0, 11), &expected);
}

#[test]
fn 多項式はそのまま係数になる() {
    // (x - 1)^3を，x = 1のまわりで展開する．
    assert_close(
        &coefficients("x^3 - 3*x^2 + 3*x - 1", 1.0, 5),
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    );
}

#[test]
fn 対数と割り算と平方根() {
    // log(1 + x) = x - x^2/2 + x^3/3 - ...
    assert_close(
        &coefficients("log(1 + x)", 0.0, 5),
        &[0.0, 1.0, -0.5, 1.0 / 3.0, -0.25, 0.2],
    );
    // 1/(1 - x) = 1 + x + x^2 + ...
    assert_close(&coefficients("1/(1 - x)", 0.0, 6), &[1.0; 7]);
    // sqrt(1 + x) = 1 + x/2 - x^2/8 + x^3/16 - ...
    assert_close(
        &coefficients("sqrt(1 + x)", 0.0, 3),
        &[1.0, 0.5, -0.125, 0.0625],
    );
}

#[test]
fn 逆三角関数() {
    // atan(x) = x - x^3/3 + x^5/5
    assert_close(
        &coefficients("atan(x)", 0.0, 5),
        &[0.0, 1.0, 0.0, -1.0 / 3.0, 0.0, 0.2],
    );
    // asin(x) = x + x^3/6 + 3x^5/40
    assert_close(
        &coefficients("asin(x)", 0.0, 5),
        &[0.0, 1.0, 0.0, 1.0 / 6.0, 0.0, 3.0 / 40.0],
    );
}

#[test]
fn 展開の中心が0でなくても導関数の値と合う() {
    // tan(x)のx = 0.5での係数：tan，sec^2，sec^2 tan，…
    let at: f64 = 0.5;
    let c = coefficients("tan(x)", at, 2);
    let sec2 = 1.0 / at.cos().powi(2);
    assert_close(&c, &[at.tan(), sec2, sec2 * at.tan()]);
}

#[test]
fn 負の数の整数乗も展開できる() {
    // x^2をx = -2のまわりで：4 - 4(x + 2) + (x + 2)^2
    assert_close(&coefficients("x^2", -2.0, 3), &[4.0, -4.0, 1.0, 0.0]);
}

#[test]
fn 展開できない関数は展開しない() {
    let expr = Expr::compile("gamma(x)", &["x"]).expect("読める");
    assert!(expr.taylor(0, &[1.0], 3).is_none());
    // 定義域の外(log(0))も展開しない．
    let expr = Expr::compile("log(x)", &["x"]).expect("読める");
    assert!(expr.taylor(0, &[0.0], 3).is_none());
}
