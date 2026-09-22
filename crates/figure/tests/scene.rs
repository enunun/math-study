//! シーンのJSONが，型にどう読まれるかを確かめる．最初の図のファイルも読む．

#![allow(
    clippy::expect_used,
    clippy::float_cmp,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use figure::scene::{Anchor, Arrow, Bound, Direction, Length, LengthUnit, Line, Object, Position};
use figure::{Scene, parse_scene};

const SINE_AND_SHIFTED_SINE: &str =
    include_str!("../../../site/src/figures/sine-and-shifted-sine.json");

/// 指定したオブジェクトだけを持つ，最小のシーン．
fn scene_with(objects: &str) -> Scene {
    let json = format!(
        r#"{{
            "version": "0.1.0",
            "description": "試験の図",
            "view": {{ "x": [-1, 1], "y": [-1, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
            "objects": [{objects}]
        }}"#
    );
    parse_scene(&json).expect("シーンを読める")
}

fn only_object(objects: &str) -> Object {
    let mut scene = scene_with(objects);
    assert_eq!(scene.objects.len(), 1);
    scene.objects.remove(0)
}

#[test]
fn 最初の図のファイルを読める() {
    let scene = parse_scene(SINE_AND_SHIFTED_SINE).expect("最初の図を読める");
    assert_eq!(scene.version.to_string(), "0.1.0");
    assert_eq!(
        scene.description,
        "y=sin x のグラフと，x軸の方向に平行移動した点線のグラフ"
    );
    let view = scene.view.as_plane().expect("平面の図");
    assert_eq!(view.x, [-7.0, 7.0]);
    assert_eq!(view.y, [-1.6, 1.8]);
    assert_eq!(
        view.unit.y,
        Length {
            value: 2.0,
            unit: LengthUnit::Cm
        }
    );
    let ids: Vec<&str> = scene.objects.iter().map(Object::id).collect();
    assert_eq!(
        ids,
        [
            "x_axis",
            "y_axis",
            "origin_label",
            "shift",
            "sine",
            "shifted_sine",
            "title"
        ]
    );
}

#[test]
fn 軸を読める() {
    let Object::Axis(axis) = only_object(
        r#"{ "id": "x_axis", "type": "axis", "direction": "x", "arrow": "none", "label": "x", "range": [0, 5] }"#,
    ) else {
        panic!("軸である");
    };
    assert_eq!(axis.direction, Direction::X);
    assert_eq!(axis.arrow, Arrow::None);
    assert_eq!(axis.label.as_deref(), Some("x"));
    assert_eq!(axis.range, Some([0.0, 5.0]));
}

#[test]
fn 軸の矢じりを省くとstealthになる() {
    let Object::Axis(axis) = only_object(r#"{ "id": "y_axis", "type": "axis", "direction": "y" }"#)
    else {
        panic!("軸である");
    };
    assert_eq!(axis.arrow, Arrow::Stealth);
    assert_eq!(axis.label, None);
    assert_eq!(axis.range, None);
}

#[test]
fn ラベルのアンカーを省くと中央になり指定した名前を読める() {
    let Object::Label(centered) =
        only_object(r#"{ "id": "a", "type": "label", "at": [0, 0], "tex": "O" }"#)
    else {
        panic!("ラベルである");
    };
    assert_eq!(centered.anchor, Anchor::Center);

    let Object::Label(placed) = only_object(
        r#"{ "id": "a", "type": "label", "at": [0, 0], "anchor": "south east", "tex": "O" }"#,
    ) else {
        panic!("ラベルである");
    };
    assert_eq!(placed.anchor, Anchor::SouthEast);
    assert_eq!(
        placed.at,
        Position::Coordinates(vec![Bound::Number(0.0), Bound::Number(0.0)])
    );
    assert_eq!(placed.tex, "O");
}

#[test]
fn グラフの線の種類を省くと実線になる() {
    let Object::Graph(graph) = only_object(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "sin(x)", "domain": [-7, 7] }"#,
    ) else {
        panic!("グラフである");
    };
    assert_eq!(graph.style.line, None);
    assert_eq!(graph.domain, [Bound::Number(-7.0), Bound::Number(7.0)]);
}

#[test]
fn 媒介変数表示の曲線を読める() {
    let Object::Curve(curve) = only_object(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["cos(t)", "sin(t)"],
             "domain": [0, "2*pi"], "style": { "line": "dashed" } }"#,
    ) else {
        panic!("曲線である");
    };
    assert_eq!(curve.expr, ["cos(t)", "sin(t)"]);
    assert_eq!(
        curve.domain,
        Some([Bound::Number(0.0), Bound::Expression("2*pi".to_owned())])
    );
    assert_eq!(curve.style.line, Some(Line::Dashed));
}

#[test]
fn 媒介変数を読める() {
    let Object::Parameter(parameter) =
        only_object(r#"{ "id": "shift", "type": "parameter", "value": -1.2 }"#)
    else {
        panic!("媒介変数である");
    };
    assert_eq!(parameter.value, -1.2);
}

#[test]
fn 長さは数と単位から読む() {
    let scene = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "x": [0, 1], "y": [0, 1], "unit": { "x": "2.5mm", "y": "10pt" } },
             "objects": [] }"#,
    )
    .expect("読める");
    let view = scene.view.as_plane().expect("平面の図");
    assert_eq!(
        view.unit.x,
        Length {
            value: 2.5,
            unit: LengthUnit::Mm
        }
    );
    assert_eq!(
        view.unit.y,
        Length {
            value: 10.0,
            unit: LengthUnit::Pt
        }
    );
}
