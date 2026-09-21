//! 式の構文，優先順位，関数，誤りの位置，上限を確かめる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use std::f64::consts::{E, FRAC_PI_2, FRAC_PI_4, PI};

use figure::expr::{Expr, ExprError, ExprErrorKind};

fn eval(source: &str) -> f64 {
    Expr::compile(source, &[]).expect("式を読める").eval(&[])
}

fn error(source: &str) -> ExprError {
    Expr::compile(source, &["x"]).expect_err("誤りになる")
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-12
}

#[test]
fn 四則演算は掛け算と割り算を先に計算する() {
    assert_eq!(eval("1 + 2 * 3"), 7.0);
    assert_eq!(eval("(1 + 2) * 3"), 9.0);
    assert_eq!(eval("10 / 4"), 2.5);
    assert_eq!(eval("2 + 6 / 3"), 4.0);
}

#[test]
fn 同じ優先順位は左から計算する() {
    assert_eq!(eval("1 - 2 - 3"), -4.0);
    assert_eq!(eval("8 / 2 / 2"), 2.0);
}

#[test]
fn 累乗は右から結び_単項のマイナスより先に計算する() {
    assert_eq!(eval("2 ^ 3 ^ 2"), 512.0);
    assert_eq!(eval("-2 ^ 2"), -4.0);
    assert_eq!(eval("2 ^ -1"), 0.5);
    assert_eq!(eval("-(-3)"), 3.0);
    assert_eq!(eval("+3"), 3.0);
}

#[test]
fn 小数を読める() {
    assert_eq!(eval("2.5 * 2"), 5.0);
    assert_eq!(eval("0.125"), 0.125);
}

#[test]
fn 関数を評価する() {
    assert_eq!(eval("sin(0)"), 0.0);
    assert_eq!(eval("cos(0)"), 1.0);
    assert_eq!(eval("tan(0)"), 0.0);
    assert_eq!(eval("sqrt(4)"), 2.0);
    assert_eq!(eval("abs(-3)"), 3.0);
    assert_eq!(eval("exp(0)"), 1.0);
    assert!(close(eval("atan(1)"), FRAC_PI_4));
    assert!(close(eval("asin(1)"), FRAC_PI_2));
    assert!(close(eval("acos(1)"), 0.0));
    assert!(close(eval("sinh(0)"), 0.0));
    assert!(close(eval("cosh(0)"), 1.0));
    assert!(close(eval("tanh(0)"), 0.0));
}

#[test]
fn logとlnは自然対数である() {
    assert!(close(eval("log(e)"), 1.0));
    assert!(close(eval("ln(e ^ 2)"), 2.0));
}

#[test]
fn 定数を使える() {
    assert!(close(eval("pi"), PI));
    assert!(close(eval("e"), E));
    assert!(close(eval("sin(pi / 2)"), 1.0));
}

#[test]
fn 変数に値を入れて評価する() {
    let expr = Expr::compile("a * x + 1", &["x", "a"]).expect("式を読める");
    assert_eq!(expr.eval(&[3.0, 2.0]), 7.0);
    assert_eq!(expr.eval(&[0.0, 5.0]), 1.0);
}

#[test]
fn 関数の引数に式を書ける() {
    let expr = Expr::compile("sin(x - shift)", &["x", "shift"]).expect("式を読める");
    assert!(close(expr.eval(&[1.0, 1.0]), 0.0));
}

#[test]
fn 数学の関数の定義域の外では有限でない値になる() {
    assert!(eval("1 / 0").is_infinite());
    assert!(eval("sqrt(-1)").is_nan());
    assert!(eval("ln(0)").is_infinite());
}

#[test]
fn 値が足りないときは非数になり_停止しない() {
    let expr = Expr::compile("x + 1", &["x"]).expect("式を読める");
    assert!(expr.eval(&[]).is_nan());
}

#[test]
fn 知らない名前は名前の位置を示す() {
    let error = error("x + y");
    assert_eq!(error.kind, ExprErrorKind::UnknownName("y".to_owned()));
    assert_eq!(error.span, 4..5);
}

#[test]
fn 知らない関数は関数の名前の位置を示す() {
    let error = error("1 + foo(x)");
    assert_eq!(error.kind, ExprErrorKind::UnknownFunction("foo".to_owned()));
    assert_eq!(error.span, 4..7);
}

#[test]
fn 引数のない関数の名前は誤りになる() {
    let error = error("sin + 1");
    assert_eq!(
        error.kind,
        ExprErrorKind::FunctionNeedsArgument("sin".to_owned())
    );
    assert_eq!(error.span, 0..3);
}

#[test]
fn 置けない位置の記号は位置を示す() {
    let error = error("1 + * 2");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedToken("*".to_owned()));
    assert_eq!(error.span, 4..5);
}

#[test]
fn 数の次の数は誤りになる() {
    let error = error("1 2");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedToken("2".to_owned()));
    assert_eq!(error.span, 2..3);
}

#[test]
fn 掛け算の記号の省略は誤りになる() {
    let error = error("2x");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedToken("x".to_owned()));
    assert_eq!(error.span, 1..2);
}

#[test]
fn 途中で終わった式は終わりの位置を示す() {
    let error = error("1 +");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedEnd);
    assert_eq!(error.span, 3..3);
    assert_eq!(self::error("").kind, ExprErrorKind::UnexpectedEnd);
}

#[test]
fn 閉じ括弧がないと誤りになる() {
    let error = error("(1 + 2");
    assert_eq!(error.kind, ExprErrorKind::MissingClosingParenthesis);
}

#[test]
fn 余分な閉じ括弧は置けない記号として示す() {
    let error = error("1 + 2)");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedToken(")".to_owned()));
    assert_eq!(error.span, 5..6);
}

#[test]
fn 使えない文字は位置を示す() {
    let error = error("1 $ 2");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedCharacter('$'));
    assert_eq!(error.span, 2..3);
}

#[test]
fn 位置はバイトではなく文字の番号で数える() {
    let error = error("x + 𠮷");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedCharacter('𠮷'));
    assert_eq!(error.span, 4..5);
}

#[test]
fn 数の小数点の前後には数字が要る() {
    assert!(Expr::compile(".5", &[]).is_err());
    assert!(Expr::compile("1.", &[]).is_err());
}

#[test]
fn 長すぎる式は誤りになる() {
    let source = "1+".repeat(1500) + "1";
    let error = Expr::compile(&source, &[]).expect_err("誤りになる");
    assert_eq!(error.kind, ExprErrorKind::TooLong);
}

#[test]
fn 深すぎる入れ子は誤りになり_停止しない() {
    let source = format!("{}1{}", "(".repeat(500), ")".repeat(500));
    let error = Expr::compile(&source, &[]).expect_err("誤りになる");
    assert_eq!(error.kind, ExprErrorKind::TooDeep);
    let unary = format!("{}1", "-".repeat(500));
    assert_eq!(
        Expr::compile(&unary, &[]).expect_err("誤りになる").kind,
        ExprErrorKind::TooDeep
    );
}

#[test]
fn 深さの上限の内側の入れ子は読める() {
    let source = format!("{}1{}", "(".repeat(100), ")".repeat(100));
    assert_eq!(eval(&source), 1.0);
}
