//! 軸の目盛(`ticks`)の読み込みと検査を確かめる．目盛は，軸に直角な短い線と，任意の名前である．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::scene::{Bound, Object};
use figure::{Error, ErrorKind, Scene, parse_scene};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-4, 4], "y": [-2, 2], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn axis(extra: &str) -> String {
    format!(r#"{{ "id": "x_axis", "type": "axis", "direction": "x" {extra} }}"#)
}

fn error_of(json: &str) -> Error {
    parse_scene(json).expect_err("誤りになる")
}

#[test]
fn 目盛は_数か式の位置と_任意の名前を持つ() {
    let scene = parse_scene(&plane_scene(&axis(
        r#", "ticks": [ { "at": 1 }, { "at": "pi/2", "label": "\\frac{\\pi}{2}" } ]"#,
    )))
    .expect("読める");
    let Object::Axis(axis) = &scene.objects[0] else {
        panic!("軸である");
    };
    assert_eq!(axis.ticks.len(), 2);
    assert_eq!(axis.ticks[0].at, Bound::Number(1.0));
    assert_eq!(axis.ticks[0].label, None);
    assert_eq!(axis.ticks[1].at, Bound::Expression("pi/2".to_owned()));
    assert_eq!(axis.ticks[1].label.as_deref(), Some(r"\frac{\pi}{2}"));
}

#[test]
fn 目盛のない軸は_書き出しに目盛の項目を出さない() {
    let scene = parse_scene(&plane_scene(&axis(""))).expect("読める");
    let written = serde_json::to_string(&scene).expect("書き出せる");
    assert!(!written.contains("ticks"), "{written}");
}

#[test]
fn 目盛のある軸は_書き出して読み直すと同じになる() {
    let scene = parse_scene(&plane_scene(&format!(
        r#"{{ "id": "a", "type": "parameter", "value": 1 }}, {}"#,
        axis(r#", "ticks": [ { "at": -1, "label": "-1" }, { "at": "2*a" } ]"#)
    )))
    .expect("読める");
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    let again: Scene = parse_scene(&written).expect("読み直せる");
    assert_eq!(scene, again);
}

#[test]
fn 目盛の未知の項目は誤りになり_軸のidを返す() {
    let error = error_of(&plane_scene(&axis(
        r#", "ticks": [ { "at": 1, "colour": "red" } ]"#,
    )));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_eq!(error.object.as_deref(), Some("x_axis"));
    assert!(error.to_string().contains("colour"), "{error}");
}

#[test]
fn 目盛には_位置が必要である() {
    let error = error_of(&plane_scene(&axis(r#", "ticks": [ { "label": "1" } ]"#)));
    assert!(error.to_string().contains("at"), "{error}");
}

#[test]
fn 目盛の名前は_空にできない() {
    let error = error_of(&plane_scene(&axis(
        r#", "ticks": [ { "at": 1, "label": " " } ]"#,
    )));
    assert_eq!(error.kind, ErrorKind::EmptyText("label"));
    assert_eq!(error.object.as_deref(), Some("x_axis"));
}

#[test]
fn 位置の式の誤りは_目盛の番号と式の中の位置を示す() {
    let error = error_of(&plane_scene(&axis(
        r#", "ticks": [ { "at": 1 }, { "at": "pi/" } ]"#,
    )));
    let ErrorKind::Expression { field, index, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!((*field, *index), ("ticks", 1));
    assert_eq!(error.object.as_deref(), Some("x_axis"));
}

#[test]
fn 位置の式は_媒介変数と定数を使え_変数は使えない() {
    let ok = parse_scene(&plane_scene(&format!(
        r#"{{ "id": "a", "type": "parameter", "value": 1.5 }}, {}"#,
        axis(r#", "ticks": [ { "at": "a*pi/2" } ]"#)
    )));
    assert!(ok.is_ok(), "{ok:?}");
    let error = error_of(&plane_scene(&axis(r#", "ticks": [ { "at": "x" } ]"#)));
    assert!(
        matches!(error.kind, ErrorKind::Expression { .. }),
        "{error}"
    );
}

#[test]
fn 軸の範囲の外の目盛は_誤りになる() {
    // 範囲を省いた軸は，見える範囲(x: -4から4)である．
    let error = error_of(&plane_scene(&axis(r#", "ticks": [ { "at": 5 } ]"#)));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert_eq!(error.object.as_deref(), Some("x_axis"));
    assert!(error.to_string().contains("範囲"), "{error}");
    let ranged = error_of(&plane_scene(&axis(
        r#", "range": [0, 2], "ticks": [ { "at": 3 } ]"#,
    )));
    assert!(matches!(ranged.kind, ErrorKind::Invalid(_)), "{ranged}");
    // 範囲の端は，範囲の中である．
    assert!(
        parse_scene(&plane_scene(&axis(
            r#", "ticks": [ { "at": 4 }, { "at": -4 } ]"#
        )))
        .is_ok()
    );
}

#[test]
fn 空間の図の軸には_目盛をまだ付けられない() {
    let error = error_of(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "azimuth": 0, "elevation": 0, "unit": "1cm" },
             "objects": [ { "id": "x_axis", "type": "axis", "direction": "x", "range": [-1, 1],
                            "ticks": [ { "at": 0.5 } ] } ] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert_eq!(error.object.as_deref(), Some("x_axis"));
    assert!(error.to_string().contains("空間"), "{error}");
}
