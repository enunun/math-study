//! シーンの検査を確かめる．誤りは，原因のオブジェクトの`id`を持つ．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use figure::{Error, ErrorKind, parse_scene};

/// 指定したオブジェクトを持つシーンを読んで，誤りを返す．
fn error_of_objects(objects: &str) -> Error {
    parse_scene(&scene(objects, "\"試験の図\"", "[-1, 1]")).expect_err("誤りになる")
}

fn scene(objects: &str, description: &str, x_range: &str) -> String {
    format!(
        r#"{{
            "version": "0.1.0",
            "description": {description},
            "view": {{ "x": {x_range}, "y": [-1, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
            "objects": [{objects}]
        }}"#
    )
}

fn assert_in_object(error: &Error, id: &str) {
    assert_eq!(error.object.as_deref(), Some(id), "{error}");
}

#[test]
fn 未知の項目は誤りになる() {
    let error = parse_scene(
        r#"{ "version": "0.1.0", "description": "a", "foo": 1,
             "view": { "x": [0, 1], "y": [0, 1], "unit": { "x": "1cm", "y": "1cm" } },
             "objects": [] }"#,
    )
    .expect_err("誤りになる");
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert!(error.to_string().contains("foo"), "{error}");
}

#[test]
fn オブジェクトの未知の項目は誤りになり_idを返す() {
    let error = error_of_objects(
        r#"{ "id": "x_axis", "type": "axis", "direction": "x", "colour": "red" }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_in_object(&error, "x_axis");
    assert!(error.to_string().contains("colour"), "{error}");
}

#[test]
fn 未知の種類は誤りになり_idを返す() {
    let error = error_of_objects(r#"{ "id": "h", "type": "hexagon" }"#);
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_in_object(&error, "h");
    assert!(error.to_string().contains("hexagon"), "{error}");
}

#[test]
fn 欠けた項目は誤りになり_idを返す() {
    let error = error_of_objects(r#"{ "id": "f", "type": "graph", "var": "x", "domain": [0, 1] }"#);
    assert_in_object(&error, "f");
    assert!(error.to_string().contains("expr"), "{error}");
}

#[test]
fn 型の違う項目は誤りになり_idを返す() {
    let error = error_of_objects(r#"{ "id": "a", "type": "label", "at": "origin", "tex": "O" }"#);
    assert_in_object(&error, "a");
}

#[test]
fn idがないオブジェクトは誤りになる() {
    let error = error_of_objects(r#"{ "type": "parameter", "value": 1 }"#);
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_eq!(error.object, None);
    assert!(error.to_string().contains("id"), "{error}");
}

#[test]
fn 識別子でないidは誤りになる() {
    for id in ["x-axis", "", "1a", "日本語", "a b", "_a"] {
        let error = error_of_objects(&format!(
            r#"{{ "id": "{id}", "type": "parameter", "value": 1 }}"#
        ));
        assert_eq!(error.kind, ErrorKind::InvalidId(id.to_owned()), "{id}");
        assert_in_object(&error, id);
    }
}

#[test]
fn 重なったidは誤りになる() {
    let error = error_of_objects(
        r#"{ "id": "a", "type": "parameter", "value": 1 },
           { "id": "a", "type": "parameter", "value": 2 }"#,
    );
    assert_eq!(error.kind, ErrorKind::DuplicateId("a".to_owned()));
    assert_in_object(&error, "a");
}

#[test]
fn 変数の名前は識別子にする() {
    for name in ["", "2x", "x y", "x-1"] {
        let error = error_of_objects(&format!(
            r#"{{ "id": "f", "type": "graph", "var": "{name}", "expr": "1", "domain": [0, 1] }}"#
        ));
        assert_eq!(
            error.kind,
            ErrorKind::InvalidVariable(name.to_owned()),
            "{name}"
        );
        assert_in_object(&error, "f");
    }
}

#[test]
fn 図の説明が空だと誤りになる() {
    for description in ["\"\"", "\"  \""] {
        let error = parse_scene(&scene("", description, "[-1, 1]")).expect_err("誤りになる");
        assert_eq!(error.kind, ErrorKind::EmptyText("description"));
        assert_eq!(error.object, None);
    }
}

#[test]
fn 見える範囲は下端が上端より小さい有限の数にする() {
    for range in ["[1, 1]", "[2, 1]"] {
        let error = parse_scene(&scene("", "\"a\"", range)).expect_err("誤りになる");
        assert_eq!(error.kind, ErrorKind::InvalidRange("view.x"), "{range}");
    }
}

#[test]
fn 軸の範囲は下端が上端より小さくする() {
    let error = error_of_objects(
        r#"{ "id": "x_axis", "type": "axis", "direction": "x", "range": [5, 0] }"#,
    );
    assert_eq!(error.kind, ErrorKind::InvalidRange("range"));
    assert_in_object(&error, "x_axis");
}

#[test]
fn 数の定義域は下端が上端より小さくする() {
    let error = error_of_objects(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "x", "domain": [1, 0] }"#,
    );
    assert_eq!(error.kind, ErrorKind::InvalidRange("domain"));
    assert_in_object(&error, "f");
}

#[test]
fn 曲線の式は2個にする() {
    for (exprs, found) in [(r#"["t"]"#, 1), (r#"["t", "t", "t"]"#, 3), ("[]", 0)] {
        let error = error_of_objects(&format!(
            r#"{{ "id": "c", "type": "curve", "var": "t", "expr": {exprs}, "domain": [0, 1] }}"#
        ));
        assert_eq!(
            error.kind,
            ErrorKind::ExpressionCount { expected: 2, found }
        );
        assert_in_object(&error, "c");
    }
}

#[test]
fn 式とラベルの文字列が空だと誤りになる() {
    let error = error_of_objects(r#"{ "id": "a", "type": "label", "at": [0, 0], "tex": " " }"#);
    assert_eq!(error.kind, ErrorKind::EmptyText("tex"));
    assert_in_object(&error, "a");

    let error = error_of_objects(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "", "domain": [0, 1] }"#,
    );
    assert_eq!(error.kind, ErrorKind::EmptyText("expr"));
    assert_in_object(&error, "f");
}

#[test]
fn 読めない長さは誤りになる() {
    for length in ["1parsec", "-1cm", "0cm", "cm", "1", "abccm"] {
        let error = parse_scene(&format!(
            r#"{{ "version": "0.1.0", "description": "a",
                 "view": {{ "x": [0, 1], "y": [0, 1], "unit": {{ "x": "{length}", "y": "1cm" }} }},
                 "objects": [] }}"#
        ))
        .expect_err("誤りになる");
        assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{length}");
        assert!(error.to_string().contains("長さ"), "{length}: {error}");
    }
}

#[test]
fn 誤りの説明にオブジェクトのidが出る() {
    let error = error_of_objects(r#"{ "id": "x_axis", "type": "axis", "direction": "w" }"#);
    assert!(error.to_string().contains("x_axis"), "{error}");
}
