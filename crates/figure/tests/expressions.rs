//! シーンの中の式の検査を確かめる．式の構文，使える名前，定義域の評価．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use figure::expr::ExprErrorKind;
use figure::{Error, ErrorKind, parse_scene};

fn scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "a",
             "view": {{ "x": [-1, 1], "y": [-1, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn error_of(objects: &str) -> Error {
    parse_scene(&scene(objects)).expect_err("誤りになる")
}

fn graph(expr: &str, domain: &str) -> String {
    format!(r#"{{ "id": "f", "type": "graph", "var": "x", "expr": "{expr}", "domain": {domain} }}"#)
}

const SHIFT: &str = r#"{ "id": "shift", "type": "parameter", "value": 1 },"#;

#[test]
fn 媒介変数を使った式を読める() {
    let json = scene(&format!("{SHIFT}{}", graph("sin(x - shift)", "[0, 1]")));
    assert!(parse_scene(&json).is_ok());
}

#[test]
fn 式の構文の誤りは項目と位置を示す() {
    let error = error_of(&graph("1 + * x", "[0, 1]"));
    let ErrorKind::Expression {
        field,
        index,
        error: expr,
    } = &error.kind
    else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!((*field, *index), ("expr", 0));
    assert_eq!(expr.kind, ExprErrorKind::UnexpectedToken("*".to_owned()));
    assert_eq!(expr.span, 4..5);
    assert_eq!(error.object.as_deref(), Some("f"));
    assert_eq!(error.kind.code(), "expression");
}

#[test]
fn 定義されていない名前は誤りになる() {
    let error = error_of(&graph("x + a", "[0, 1]"));
    let ErrorKind::Expression { error: expr, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!(expr.kind, ExprErrorKind::UnknownName("a".to_owned()));
}

#[test]
fn 曲線の式の誤りは何番目の式かを示す() {
    let error = error_of(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["cos(t)", "sin(t"], "domain": [0, 1] }"#,
    );
    let ErrorKind::Expression { field, index, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!((*field, *index), ("expr", 1));
}

#[test]
fn 定義域の端の式は媒介変数と定数を使える() {
    let json = scene(
        r#"{ "id": "a", "type": "parameter", "value": 3 },
           { "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": [0, "2*pi + a"] }"#,
    );
    assert!(parse_scene(&json).is_ok());
}

#[test]
fn 定義域の端の式は変数を使えない() {
    let error = error_of(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": [0, "t"] }"#,
    );
    let ErrorKind::Expression {
        field,
        index,
        error: expr,
    } = &error.kind
    else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!((*field, *index), ("domain", 1));
    assert_eq!(expr.kind, ExprErrorKind::UnknownName("t".to_owned()));
}

#[test]
fn 式で書いた定義域は評価して下端が上端より小さいか確かめる() {
    let error = error_of(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": ["2*pi", 0] }"#,
    );
    assert_eq!(error.kind, ErrorKind::InvalidRange("domain"));
    assert_eq!(error.object.as_deref(), Some("c"));
}

#[test]
fn 媒介変数の値で決まる定義域も確かめる() {
    let objects = |a: f64| {
        format!(
            r#"{{ "id": "a", "type": "parameter", "value": {a} }},
               {{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": ["a", 0] }}"#
        )
    };
    assert_eq!(
        error_of(&objects(3.0)).kind,
        ErrorKind::InvalidRange("domain")
    );
    assert!(parse_scene(&scene(&objects(-3.0))).is_ok());
}

#[test]
fn 有限でない定義域の端は誤りになる() {
    let error = error_of(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": [0, "1/0"] }"#,
    );
    assert_eq!(error.kind, ErrorKind::InvalidRange("domain"));
}

#[test]
fn 関数や定数の名前の媒介変数は使えない() {
    for name in ["sin", "pi", "e", "sqrt"] {
        let error = error_of(&format!(
            r#"{{ "id": "{name}", "type": "parameter", "value": 1 }}"#
        ));
        assert_eq!(error.kind, ErrorKind::ReservedName(name.to_owned()));
        assert_eq!(error.object.as_deref(), Some(name));
    }
}

#[test]
fn 関数や定数の名前の変数は使えない() {
    let error =
        error_of(r#"{ "id": "f", "type": "graph", "var": "pi", "expr": "1", "domain": [0, 1] }"#);
    assert_eq!(error.kind, ErrorKind::ReservedName("pi".to_owned()));
}

#[test]
fn 変数の名前は媒介変数のidと重ねられない() {
    let error = error_of(&format!(
        "{SHIFT}{}",
        r#"{ "id": "f", "type": "graph", "var": "shift", "expr": "1", "domain": [0, 1] }"#
    ));
    assert_eq!(error.kind, ErrorKind::NameConflict("shift".to_owned()));
    assert_eq!(error.object.as_deref(), Some("f"));
}

#[test]
fn 誤りの説明に位置と説明が出る() {
    let error = error_of(&graph("1 + * x", "[0, 1]"));
    let message = error.to_string();
    assert!(message.contains("5文字目"), "{message}");
    assert!(message.contains('*'), "{message}");
    assert!(message.contains('f'), "{message}");
}
