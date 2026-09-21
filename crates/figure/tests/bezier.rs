//! ベジエ曲面を確かめる．制御点の網で書いた曲面は，同じ形を式で書いた曲面と，同じ図になる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Line, Object};
use figure::{Error, ErrorKind, parse_scene, render};

fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn error_of(objects: &str) -> Error {
    parse_scene(&space_scene(objects)).expect_err("誤りになる")
}

fn figure_of(objects: &str) -> Figure {
    render(&parse_scene(&space_scene(objects)).expect("読める")).expect("描画できる")
}

fn paths(figure: &Figure) -> Vec<&Path> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Path(path) => Some(path),
            _ => None,
        })
        .collect()
}

/// u方向に3点，v方向に2点の網．xはu，yはvに比例し，zはuについて放物線になる．
const BEZIER: &str = r#"{ "id": "b", "type": "surface", "boundary": true,
    "bezier": [[[0, 0, 0], [0, 2, 0]], [[1, 0, 2], [1, 2, 2]], [[2, 0, 0], [2, 2, 0]]] }"#;

/// 上と同じ形を，式で書いた曲面．
const FORMULA: &str = r#"{ "id": "b", "type": "surface", "boundary": true,
    "vars": ["u", "v"], "expr": ["2*u", "2*v", "4*u*(1-u)"], "domain": [[0, 1], [0, 1]] }"#;

fn same_paths(a: &Figure, b: &Figure) {
    let (a, b) = (paths(a), paths(b));
    assert!(!a.is_empty(), "線がある");
    assert_eq!(a.len(), b.len(), "線の数が同じである");
    for (left, right) in a.iter().zip(&b) {
        assert_eq!(left.points.len(), right.points.len(), "点の数が同じである");
        for (p, q) in left.points.iter().zip(&right.points) {
            assert!(
                (p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9,
                "点が重なる：{p:?}と{q:?}"
            );
        }
    }
}

// ---- 読み込みと検査 ----

#[test]
fn ベジエ曲面は_制御点の網だけで書け_変数と式と定義域を持たない() {
    let scene = parse_scene(&space_scene(BEZIER)).expect("読める");
    let Object::Surface(surface) = &scene.objects[0] else {
        panic!("曲面である");
    };
    assert!(surface.vars.is_empty());
    assert!(surface.expr.is_empty());
    let net = surface.bezier.as_ref().expect("網がある");
    assert_eq!(net.len(), 3);
    assert_eq!(net[0].len(), 2);
}

#[test]
fn ベジエ曲面を書き出して読み直すと同じになる() {
    let scene = parse_scene(&space_scene(BEZIER)).expect("読める");
    let json = serde_json::to_string(&scene).expect("書き出せる");
    assert_eq!(parse_scene(&json).expect("読み直せる"), scene);
}

#[test]
fn 制御点の座標は_式でも書ける() {
    let scene = parse_scene(&space_scene(
        r#"{ "id": "b", "type": "surface",
             "bezier": [[[0, 0, 0], [0, 1, "1+1"]], [[1, 0, 0], [1, 1, 0]]] }"#,
    ));
    assert!(scene.is_ok());
}

#[test]
fn 網の行の長さが揃わないと_誤りになる() {
    let error = error_of(
        r#"{ "id": "b", "type": "surface",
             "bezier": [[[0, 0, 0], [0, 1, 0]], [[1, 0, 0]]] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error:?}");
    assert_eq!(error.object.as_deref(), Some("b"));
}

#[test]
fn 網は_各方向に2点以上が要る() {
    let error = error_of(r#"{ "id": "b", "type": "surface", "bezier": [[[0, 0, 0], [1, 0, 0]]] }"#);
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error:?}");
}

#[test]
fn 制御点は_3つの座標を持つ() {
    let error = error_of(
        r#"{ "id": "b", "type": "surface",
             "bezier": [[[0, 0], [0, 1]], [[1, 0], [1, 1]]] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error:?}");
}

#[test]
fn ベジエ曲面と式の曲面は_同時に書けない() {
    let error = error_of(
        r#"{ "id": "b", "type": "surface", "vars": ["u", "v"], "expr": ["u", "v", "0"],
             "domain": [[0, 1], [0, 1]],
             "bezier": [[[0, 0, 0], [0, 1, 0]], [[1, 0, 0], [1, 1, 0]]] }"#,
    );
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error:?}");
}

#[test]
fn ベジエ曲面でも_式の曲面でもないものは_誤りになる() {
    let error = error_of(r#"{ "id": "b", "type": "surface" }"#);
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error:?}");
}

#[test]
fn 制御点の式に誤りがあると_式の誤りになる() {
    let error = error_of(
        r#"{ "id": "b", "type": "surface",
             "bezier": [[[0, 0, 0], [0, 1, "1+"]], [[1, 0, 0], [1, 1, 0]]] }"#,
    );
    assert!(
        matches!(error.kind, ErrorKind::Expression { .. }),
        "{error:?}"
    );
}

// ---- 形 ----

#[test]
fn ベジエ曲面は_同じ形を式で書いた曲面と_同じ図になる() {
    same_paths(&figure_of(BEZIER), &figure_of(FORMULA));
}

#[test]
fn ベジエ曲面の切り口は_式の曲面の切り口と同じになる() {
    let cut = r#", { "id": "c", "type": "cut", "surface": "b", "normal": [0, 0, 1], "offset": 1 }"#;
    same_paths(
        &figure_of(&format!("{BEZIER}{cut}")),
        &figure_of(&format!("{FORMULA}{cut}")),
    );
}

#[test]
fn ベジエ曲面は_後ろの線を隠す() {
    // 広い平らな網の下を通る軸は，網に隠れて，実線の部分がなくなる．
    let flat = r#"{ "id": "b", "type": "surface",
        "bezier": [[[-6, -6, 0], [-6, 6, 0]], [[6, -6, 0], [6, 6, 0]]] }"#;
    let axis = r#"{ "id": "z", "type": "axis", "direction": "z", "range": [-2, -1] }"#;
    let visible = |figure: &Figure| {
        paths(figure)
            .iter()
            .filter(|path| path.stroke.line == Line::Solid)
            .count()
    };
    assert!(visible(&figure_of(axis)) > 0, "単独なら，実線が見える");
    assert_eq!(
        visible(&figure_of(&format!("{flat}, {axis}"))),
        0,
        "網の下では，実線が消える"
    );
}
