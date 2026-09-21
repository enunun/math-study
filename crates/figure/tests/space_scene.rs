//! 空間の図のシーンを，読み込みと検査で確かめる．平面と空間で，使えるオブジェクトが違う．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::scene::{Hidden, Object, View};
use figure::{Error, ErrorKind, Scene, parse_scene};

const SPHERE_WITH_AXES: &str = include_str!("../../../site/src/figures/sphere-with-axes.json");

fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-1, 1], "y": [-1, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn error_of(json: &str) -> Error {
    parse_scene(json).expect_err("誤りになる")
}

fn assert_in_object(error: &Error, id: &str) {
    assert_eq!(error.object.as_deref(), Some(id), "{error}");
}

#[test]
fn 空間の図を読める() {
    let scene = parse_scene(SPHERE_WITH_AXES).expect("読める");
    let View::Space(view) = &scene.view else {
        panic!("空間の図ではない: {:?}", scene.view);
    };
    assert_eq!(view.azimuth, 60.0);
    assert_eq!(view.elevation, 20.0);
    assert_eq!(view.unit.to_cm(), 0.8);
    let kinds: Vec<&str> = scene.objects.iter().map(Object::type_name).collect();
    assert_eq!(kinds, ["axis", "axis", "axis", "sphere"]);
}

#[test]
fn 書き出して読み直すと_同じ空間のシーンになる() {
    let scene = parse_scene(SPHERE_WITH_AXES).expect("読める");
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    let again: Scene = parse_scene(&written).expect("読み直せる");
    assert_eq!(scene, again);
}

#[test]
fn 平面の図は_平面の見え方として読む() {
    let scene = parse_scene(&plane_scene("")).expect("読める");
    assert!(matches!(scene.view, View::Plane(_)));
}

#[test]
fn 平面でも空間でもない見え方は誤りになる() {
    let error = error_of(
        r#"{ "version": "0.1.0", "description": "a", "view": { "unit": "1cm" }, "objects": [] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert!(error.to_string().contains("view"), "{error}");
}

#[test]
fn 空間の見え方の未知の項目は誤りになる() {
    let error = error_of(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "azimuth": 1, "elevation": 1, "unit": "1cm", "zoom": 2 }, "objects": [] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert!(error.to_string().contains("zoom"), "{error}");
}

#[test]
fn 平面と空間の項目が混ざった見え方は誤りになる() {
    let error = error_of(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "x": [0, 1], "azimuth": 1, "elevation": 1, "unit": "1cm" }, "objects": [] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
}

#[test]
fn 仰角が範囲の外なら誤りになる() {
    for elevation in ["91", "-91"] {
        let error = error_of(&format!(
            r#"{{ "version": "0.1.0", "description": "a",
                 "view": {{ "azimuth": 0, "elevation": {elevation}, "unit": "1cm" }}, "objects": [] }}"#
        ));
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{elevation}: {error}"
        );
        assert!(error.to_string().contains("elevation"), "{error}");
    }
}

#[test]
fn 空間の見え方の単位は_長さの文字列である() {
    let error = error_of(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "azimuth": 0, "elevation": 0, "unit": "abc" }, "objects": [] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
}

#[test]
fn z軸は_空間の図で使える() {
    let scene = parse_scene(&space_scene(
        r#"{ "id": "z_axis", "type": "axis", "direction": "z", "range": [-1, 1] }"#,
    ))
    .expect("読める");
    assert_eq!(scene.objects.len(), 1);
}

#[test]
fn z軸は_平面の図では誤りになる() {
    let error = error_of(&plane_scene(
        r#"{ "id": "z_axis", "type": "axis", "direction": "z", "range": [-1, 1] }"#,
    ));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_in_object(&error, "z_axis");
}

#[test]
fn 空間の軸には_範囲が必要である() {
    let error = error_of(&space_scene(
        r#"{ "id": "x_axis", "type": "axis", "direction": "x" }"#,
    ));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_in_object(&error, "x_axis");
    assert!(error.to_string().contains("range"), "{error}");
}

#[test]
fn 空間では_グラフとラベルは使えない() {
    for object in [
        r#"{ "id": "g", "type": "graph", "var": "x", "expr": "x", "domain": [0, 1] }"#,
        r#"{ "id": "g", "type": "label", "at": [0, 0], "tex": "O" }"#,
    ] {
        let error = error_of(&space_scene(object));
        assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
        assert_in_object(&error, "g");
        assert!(error.to_string().contains("空間"), "{error}");
    }
}

#[test]
fn 平面では_球は使えない() {
    let error = error_of(&plane_scene(
        r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 1 }"#,
    ));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_in_object(&error, "ball");
    assert!(error.to_string().contains("平面"), "{error}");
}

#[test]
fn 球の半径は_正の有限の数でなければならない() {
    for radius in ["0", "-1"] {
        let error = error_of(&space_scene(&format!(
            r#"{{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": {radius} }}"#
        )));
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{radius}: {error}"
        );
        assert_in_object(&error, "ball");
        assert!(error.to_string().contains("radius"), "{error}");
    }
}

#[test]
fn 曲線の式の数は_平面で2個_空間で3個である() {
    let space = error_of(&space_scene(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": [0, 1] }"#,
    ));
    assert_eq!(
        space.kind,
        ErrorKind::ExpressionCount {
            expected: 3,
            found: 2
        }
    );
    assert_in_object(&space, "c");
    let plane = error_of(&plane_scene(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t", "t"], "domain": [0, 1] }"#,
    ));
    assert_eq!(
        plane.kind,
        ErrorKind::ExpressionCount {
            expected: 2,
            found: 3
        }
    );
}

#[test]
fn 空間の曲線を読める() {
    let scene = parse_scene(&space_scene(
        r#"{ "id": "helix", "type": "curve", "var": "t",
             "expr": ["cos(t)", "sin(t)", "t"], "domain": [0, "2*pi"] }"#,
    ))
    .expect("読める");
    assert_eq!(scene.objects.len(), 1);
}

#[test]
fn 隠れた部分の線は_既定が点線で_破線と非表示を選べる() {
    let scene = parse_scene(&space_scene(
        r#"{ "id": "a", "type": "axis", "direction": "x", "range": [-1, 1] },
           { "id": "b", "type": "axis", "direction": "y", "range": [-1, 1],
             "style": { "hidden": "dashed" } },
           { "id": "c", "type": "axis", "direction": "z", "range": [-1, 1],
             "style": { "hidden": "none" } }"#,
    ))
    .expect("読める");
    let hidden: Vec<Hidden> = scene
        .objects
        .iter()
        .map(|object| match object {
            Object::Axis(axis) => axis.style.hidden,
            other => panic!("軸ではない: {other:?}"),
        })
        .collect();
    assert_eq!(hidden, [Hidden::Dotted, Hidden::Dashed, Hidden::None]);
}

#[test]
fn 隠れた部分の線の種類は_未知の名前を受けつけない() {
    let error = error_of(&space_scene(
        r#"{ "id": "a", "type": "axis", "direction": "x", "range": [-1, 1],
             "style": { "hidden": "wavy" } }"#,
    ));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)));
    assert_in_object(&error, "a");
}
