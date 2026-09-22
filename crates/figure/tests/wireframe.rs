//! 曲面と球のワイヤーフレーム(`wireframe`)と，ベジエ曲面の制御点の網(`control_net`)を確かめる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::Color;
use figure::{Error, parse_scene, render};

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

fn colored_paths(figure: &Figure, color: Color) -> usize {
    paths(figure)
        .into_iter()
        .filter(|path| path.stroke.color == Some(color))
        .count()
}

/// 放物面．`u`と`v`が，そのままx，yになる．末尾に`}`が1つだけなので，そこへ項目を足せる．
const PARABOLOID: &str = r#"{ "id": "s", "type": "surface", "vars": ["u", "v"],
    "expr": ["u", "v", "u^2 + v^2"], "domain": [[-2, 2], [-2, 2]] }"#;

/// ベジエ曲面．制御点は配列(`[`，`]`)で書くので，こちらも末尾に`}`が1つだけである．
const BEZIER: &str = r#"{ "id": "b", "type": "surface",
    "bezier": [[[0,0,0],[0,2,0],[0,4,0]],[[2,0,0],[2,2,1],[2,4,0]],[[4,0,0],[4,2,0],[4,4,0]]] }"#;

const BALL: &str = r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2 }"#;

fn with_field(objects: &str, field: &str) -> String {
    objects.replace('}', &format!(", {field} }}"))
}

#[test]
fn ワイヤーフレームを指定しない曲面には_断面の線が増えない() {
    let without = figure_of(PARABOLOID).items.len();
    let with = figure_of(&with_field(PARABOLOID, r#""wireframe": {}"#))
        .items
        .len();
    assert!(with > without);
}

#[test]
fn 式で書いた曲面のワイヤーフレームは_指定した色で描かれる() {
    let plain = figure_of(PARABOLOID);
    let colored = figure_of(&with_field(
        PARABOLOID,
        r#""wireframe": { "color": "blue" }"#,
    ));
    assert_eq!(colored_paths(&plain, Color::Blue), 0);
    assert!(colored_paths(&colored, Color::Blue) > 0);
}

#[test]
fn ベジエ曲面にも_式の曲面と同じようにワイヤーフレームを引ける() {
    let without = figure_of(BEZIER).items.len();
    let with = figure_of(&with_field(BEZIER, r#""wireframe": {}"#))
        .items
        .len();
    assert!(with > without);
}

#[test]
fn 球にもワイヤーフレーム_経線と緯線_を引ける() {
    let without = figure_of(BALL).items.len();
    let with = figure_of(&with_field(BALL, r#""wireframe": {}"#))
        .items
        .len();
    assert!(with > without);
}

#[test]
fn ベジエ曲面の制御点の網は_行と列を結ぶ線分になる() {
    let without = figure_of(BEZIER).items.len();
    let with = figure_of(&with_field(BEZIER, r#""control_net": {}"#))
        .items
        .len();
    assert!(with > without);
}

#[test]
fn 制御点の網は_ベジエ曲面にだけ使える() {
    let error = error_of(&with_field(PARABOLOID, r#""control_net": {}"#));
    assert!(error.to_string().contains("ベジエ"));
}

#[test]
fn ワイヤーフレームのスタイルも_ほかのスタイルと同じ太さの上限を守る() {
    let error = error_of(&with_field(
        PARABOLOID,
        r#""wireframe": { "width": "20pt" }"#,
    ));
    assert!(error.to_string().contains("太さ") || error.to_string().contains("width"));
}

#[test]
fn 書き出すと_ワイヤーフレームを指定しない曲面には_wireframeの項目がない() {
    let scene = parse_scene(&space_scene(PARABOLOID)).expect("読める");
    let json = serde_json::to_string(&scene).expect("書き出せる");
    assert!(!json.contains("wireframe"));
    assert!(!json.contains("control_net"));
}
