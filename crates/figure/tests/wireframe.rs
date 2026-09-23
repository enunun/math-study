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

/// 平らな正方形(z = 0，x，yとも-2から2)．ほかに隠すものがないので，断面の1本が1つの線になる．
const FLAT: &str = r#"{ "id": "f", "type": "surface", "vars": ["u", "v"],
    "expr": ["u", "v", "0"], "domain": [[-2, 2], [-2, 2]] }"#;

fn flat_wireframe_lines(step: &str) -> usize {
    let objects = with_field(
        FLAT,
        &format!(r#""wireframe": {{ "color": "blue" }}, "wireframe_step": {step}"#),
    );
    colored_paths(&figure_of(&objects), Color::Blue)
}

#[test]
fn ワイヤーフレームの断面は_刻みの整数倍の所に_定義域の内側だけ引く() {
    // u一定は-1，0，1の3本，v一定は-1.5から1.5まで0.5おきの7本．定義域の端(±2)は縁なので引かない．
    assert_eq!(flat_wireframe_lines("[1, 0.5]"), 10);
    // 4の整数倍で-2と2の間にあるのは0だけなので，u一定・v一定とも1本ずつ．
    assert_eq!(flat_wireframe_lines("[4, 4]"), 2);
}

#[test]
fn ワイヤーフレームの刻みは_u方向とv方向で別々に効く() {
    // u一定は0の1本，v一定は-1，0，1の3本．
    assert_eq!(flat_wireframe_lines("[4, 1]"), 4);
    assert_eq!(flat_wireframe_lines("[1, 4]"), 4);
}

#[test]
fn ワイヤーフレームの刻みは式でも書ける() {
    // 刻み2/3なら，u一定・v一定とも-4/3，-2/3，0，2/3，4/3の5本ずつ．
    assert_eq!(flat_wireframe_lines(r#"["2/3", "2/3"]"#), 10);
}

#[test]
fn ワイヤーフレームの刻みを書かなければ_定義域の幅の4分の1になる() {
    let objects = with_field(FLAT, r#""wireframe": { "color": "blue" }"#);
    // 刻み1なので，u一定・v一定とも-1，0，1の3本ずつ．
    assert_eq!(colored_paths(&figure_of(&objects), Color::Blue), 6);
}

#[test]
fn ワイヤーフレームの刻みが0以下か_断面が多すぎれば誤りになる() {
    error_of(&with_field(
        PARABOLOID,
        r#""wireframe": {}, "wireframe_step": [0, 1]"#,
    ));
    error_of(&with_field(
        PARABOLOID,
        r#""wireframe": {}, "wireframe_step": [1, -1]"#,
    ));
    // 式で書いた刻みは，評価してから確かめる．
    let error = error_of(&with_field(
        PARABOLOID,
        r#""wireframe": {}, "wireframe_step": ["1/1000", 1]"#,
    ));
    assert!(error.to_string().contains("多すぎる"), "{error}");
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
