//! 利用者が定義した関数(`Functions`)を確かめる．呼び出しは，読むときに本体へ展開するので，
//! 合成(`f(g(x))`)も，媒介変数を使う本体も，ふつうの式と同じに評価できる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::expr::{Expr, ExprErrorKind, Functions};

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-12
}

#[test]
fn 定義した関数を呼べる() {
    let mut functions = Functions::default();
    functions
        .define("f", &["x"], "x^2 + 1", &[])
        .expect("定義できる");
    let expr = Expr::compile_with("f(3)", &[], &functions).expect("読める");
    assert_eq!(expr.eval(&[]), 10.0);
}

#[test]
fn 関数を合成できる() {
    let mut functions = Functions::default();
    functions
        .define("f", &["x"], "sin(x)", &[])
        .expect("定義できる");
    functions
        .define("g", &["x"], "x^2", &[])
        .expect("定義できる");
    // 先に定義した関数は，あとの関数の本体でも使える．
    functions
        .define("h", &["x"], "f(g(x))", &[])
        .expect("定義できる");
    let direct = Expr::compile_with("f(g(t))", &["t"], &functions).expect("読める");
    let composed = Expr::compile_with("h(t)", &["t"], &functions).expect("読める");
    for t in [-1.5, 0.0, 0.7, 2.0] {
        assert!(close(direct.eval(&[t]), (t * t).sin()));
        assert!(close(composed.eval(&[t]), (t * t).sin()));
    }
}

#[test]
fn 引数を複数とれる() {
    let mut functions = Functions::default();
    functions
        .define("r", &["x", "y"], "sqrt(x^2 + y^2)", &[])
        .expect("定義できる");
    let expr = Expr::compile_with("r(3, 4)", &[], &functions).expect("読める");
    assert_eq!(expr.eval(&[]), 5.0);
}

#[test]
fn 本体は定義のときの媒介変数を使え_呼ぶ側の名前の並びに合わせる() {
    let mut functions = Functions::default();
    functions
        .define("f", &["x"], "a * x", &["a"])
        .expect("定義できる");
    // 呼ぶ側では，媒介変数aは2番目にある．
    let expr = Expr::compile_with("f(t)", &["t", "a"], &functions).expect("読める");
    assert_eq!(expr.eval(&[3.0, 2.0]), 6.0);
}

#[test]
fn 引数の数が違えば誤りになる() {
    let mut functions = Functions::default();
    functions.define("f", &["x"], "x", &[]).expect("定義できる");
    let error = Expr::compile_with("f(1, 2)", &[], &functions).expect_err("誤りになる");
    assert_eq!(
        error.kind,
        ExprErrorKind::ArgumentCount {
            name: "f".to_owned(),
            expected: 1,
            found: 2
        }
    );
}

#[test]
fn 組み込みの関数は引数を1つだけとる() {
    let error = Expr::compile("sin(1, 2)", &[]).expect_err("誤りになる");
    assert_eq!(error.kind, ExprErrorKind::UnexpectedToken(",".to_owned()));
}

#[test]
fn 定義していない関数は呼べない() {
    let functions = Functions::default();
    let error = Expr::compile_with("f(1)", &[], &functions).expect_err("誤りになる");
    assert_eq!(error.kind, ExprErrorKind::UnknownFunction("f".to_owned()));
}

#[test]
fn 展開して長くなりすぎる式は誤りになる() {
    let mut functions = Functions::default();
    functions
        .define("f0", &["x"], "x + x", &[])
        .expect("定義できる");
    // 1段ごとに，引数が2回ずつ現れるので，展開した木は段ごとに倍になる．
    for level in 1..30 {
        let body = format!("f{}(x) + f{}(x)", level - 1, level - 1);
        if functions
            .define(&format!("f{level}"), &["x"], &body, &[])
            .is_err()
        {
            return;
        }
    }
    panic!("展開した式の大きさに，上限がない");
}
