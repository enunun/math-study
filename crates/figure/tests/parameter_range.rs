//! 媒介変数の範囲(`range`)を確かめる．範囲は，編集画面のスライダーが動かせる値の幅で，描画には使わない．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use figure::scene::Object;
use figure::{Error, ErrorKind, parse_scene, render};

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

#[test]
fn 媒介変数は_範囲を持てる() {
    let json = scene(r#"{ "id": "t", "type": "parameter", "value": 0.5, "range": [-1, 1] }"#);
    let parsed = parse_scene(&json).expect("読める");
    let Object::Parameter(parameter) = &parsed.objects[0] else {
        panic!("媒介変数である");
    };
    assert_eq!(parameter.range, Some([-1.0, 1.0]));
    let written = serde_json::to_string(&parsed).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), parsed);
    assert!(render(&parsed).is_ok());
}

#[test]
fn 範囲のない媒介変数は_範囲を書き出さない() {
    let parsed = parse_scene(&scene(
        r#"{ "id": "t", "type": "parameter", "value": 0.5 }"#,
    ))
    .expect("読める");
    let written = serde_json::to_string(&parsed).expect("書き出せる");
    assert!(!written.contains("range"), "{written}");
}

#[test]
fn 範囲は_小さい方から書く() {
    let error = error_of(r#"{ "id": "t", "type": "parameter", "value": 0, "range": [1, -1] }"#);
    assert!(
        matches!(error.kind, ErrorKind::InvalidRange("range")),
        "{error}"
    );
    assert_eq!(error.object.as_deref(), Some("t"));
}

#[test]
fn 値は_範囲の中にある() {
    for value in ["-1.5", "2"] {
        let error = error_of(&format!(
            r#"{{ "id": "t", "type": "parameter", "value": {value}, "range": [-1, 1] }}"#
        ));
        assert!(error.to_string().contains("範囲"), "{value}: {error}");
        assert_eq!(error.object.as_deref(), Some("t"));
    }
    // 端は範囲に含む．
    for value in ["-1", "1"] {
        let json = scene(&format!(
            r#"{{ "id": "t", "type": "parameter", "value": {value}, "range": [-1, 1] }}"#
        ));
        assert!(parse_scene(&json).is_ok(), "{value}");
    }
}

#[test]
fn 範囲のある媒介変数は_刻みを持てる() {
    let json =
        scene(r#"{ "id": "h", "type": "parameter", "value": 1, "range": [-3, 3], "step": 1 }"#);
    let parsed = parse_scene(&json).expect("読める");
    let Object::Parameter(parameter) = &parsed.objects[0] else {
        panic!("媒介変数である");
    };
    assert_eq!(parameter.step, Some(1.0));
    let written = serde_json::to_string(&parsed).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), parsed);
    let without = parse_scene(&scene(
        r#"{ "id": "t", "type": "parameter", "value": 0.5, "range": [0, 1] }"#,
    ))
    .expect("読める");
    let written = serde_json::to_string(&without).expect("書き出せる");
    assert!(!written.contains("step"), "{written}");
}

#[test]
fn 刻みは_正の数で_範囲と一緒に書く() {
    for step in ["0", "-1"] {
        let error = error_of(&format!(
            r#"{{ "id": "h", "type": "parameter", "value": 0, "range": [-3, 3], "step": {step} }}"#
        ));
        assert!(error.to_string().contains("step"), "{error}");
        assert_eq!(error.object.as_deref(), Some("h"));
    }
    let error = error_of(r#"{ "id": "h", "type": "parameter", "value": 0, "step": 1 }"#);
    assert!(error.to_string().contains("range"), "{error}");
}
