//! 接線(`tangent_line`)と接平面(`tangent_plane`)を確かめる．
//!
//! 接線は，グラフ(`graph`)か曲線(`curve`)の`id`を`of`に，接する点を`at`に指定する．平面の図でだけ使える．
//! 接平面は，曲面(`surface`)の`id`を`of`に，接する点の変数の値を`at`に指定する．空間の図でだけ使える．
//! どちらも，数値微分(中心差分)で向きを求める．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::Object;
use figure::{Error, parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-4, 4], "y": [-4, 4], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn space_scene(objects: &str, azimuth: f64, elevation: f64) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": {azimuth}, "elevation": {elevation}, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn plane_error_of(objects: &str) -> Error {
    parse_scene(&plane_scene(objects)).expect_err("誤りになる")
}

fn space_error_of(objects: &str, azimuth: f64, elevation: f64) -> Error {
    parse_scene(&space_scene(objects, azimuth, elevation)).expect_err("誤りになる")
}

fn plane_figure_of(objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(objects)).expect("読める")).expect("描画できる")
}

fn space_figure_of(objects: &str, azimuth: f64, elevation: f64) -> Figure {
    render(&parse_scene(&space_scene(objects, azimuth, elevation)).expect("読める"))
        .expect("描画できる")
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

fn points(figure: &Figure) -> Vec<[f64; 2]> {
    paths(figure)
        .iter()
        .flat_map(|path| path.points.iter().copied())
        .collect()
}

/// $y=x^2$のグラフ．
const PARABOLA: &str = r#"{ "id": "f", "type": "graph", "var": "x", "expr": "x^2",
    "domain": [-3, 3] }"#;

/// 単位円の媒介変数表示．
const CIRCLE: &str = r#"{ "id": "c", "type": "curve", "var": "t",
    "expr": ["cos(t)", "sin(t)"], "domain": [0, "2*pi"] }"#;

// ---- 接線：読み込みと検査 ----

#[test]
fn 接線は_ofとatを持つ() {
    let objects =
        format!(r#"{PARABOLA}, {{ "id": "t", "type": "tangent_line", "of": "f", "at": 1 }}"#);
    let scene = parse_scene(&plane_scene(&objects)).expect("読める");
    let Object::TangentLine(tangent) = &scene.objects[1] else {
        panic!("接線である");
    };
    assert_eq!(tangent.of, "f");
}

#[test]
fn 存在しない対象を指せば断る() {
    let objects = r#"{ "id": "t", "type": "tangent_line", "of": "nothing", "at": 1 }"#;
    let error = plane_error_of(objects);
    assert!(error.to_string().contains("nothing"), "{error}");
}

#[test]
fn 空間の図では使えない() {
    let objects = r#"{ "id": "t", "type": "tangent_line", "of": "f", "at": 1 }"#;
    let error = space_error_of(objects, 60.0, 20.0);
    assert!(error.to_string().contains("空間"), "{error}");
}

// ---- 接線：描画 ----

#[test]
fn グラフの接線は_接点を通り_傾きが導関数に一致する() {
    // y=x^2のx=1における接線は，y=2x-1(傾き2，点(1,1)を通る)．
    let objects =
        format!(r#"{PARABOLA}, {{ "id": "t", "type": "tangent_line", "of": "f", "at": 1 }}"#);
    let figure = plane_figure_of(&objects);
    let tangent_points: Vec<[f64; 2]> = paths(&figure)
        .into_iter()
        .last()
        .expect("接線がある")
        .points
        .clone();
    assert!(tangent_points.len() >= 2, "2点以上ある");
    for [x, y] in tangent_points {
        // 1cm/単位なので，cmの座標がそのまま数学の座標である．
        assert!(
            (y - (2.0 * x - 1.0)).abs() < 1e-3,
            "({x}, {y})が直線上にない"
        );
    }
}

#[test]
fn 曲線の接線は_導関数が0でも_媒介変数の向きから求まる() {
    // 単位円のt=0における接線は，(1,0)を通る垂直な直線(x=1)である(dx/dt=0)．
    let objects =
        format!(r#"{CIRCLE}, {{ "id": "t", "type": "tangent_line", "of": "c", "at": 0 }}"#);
    let figure = plane_figure_of(&objects);
    let tangent_points: Vec<[f64; 2]> = paths(&figure)
        .into_iter()
        .last()
        .expect("接線がある")
        .points
        .clone();
    assert!(tangent_points.len() >= 2, "2点以上ある");
    for [x, _] in tangent_points {
        assert!((x - 1.0).abs() < 1e-3, "x座標が1からずれている：{x}");
    }
}

// ---- 接平面：読み込みと検査 ----

const PARABOLOID: &str = r#"{ "id": "s", "type": "surface", "vars": ["u", "v"],
    "expr": ["u", "v", "u^2 + v^2"], "domain": [[-2, 2], [-2, 2]] }"#;

#[test]
fn 接平面は_ofとatとsizeを持つ() {
    let objects = format!(
        r#"{PARABOLOID}, {{ "id": "p", "type": "tangent_plane", "of": "s",
            "at": [0, 0], "size": 1 }}"#
    );
    let scene = parse_scene(&space_scene(&objects, 60.0, 20.0)).expect("読める");
    let Object::TangentPlane(tangent) = &scene.objects[1] else {
        panic!("接平面である");
    };
    assert_eq!(tangent.of, "s");
}

#[test]
fn 存在しない曲面を指せば断る() {
    let objects = r#"{ "id": "p", "type": "tangent_plane", "of": "nothing",
        "at": [0, 0], "size": 1 }"#;
    let error = space_error_of(objects, 60.0, 20.0);
    assert!(error.to_string().contains("nothing"), "{error}");
}

#[test]
fn 半径が0以下なら断る() {
    let objects = format!(
        r#"{PARABOLOID}, {{ "id": "p", "type": "tangent_plane", "of": "s",
            "at": [0, 0], "size": 0 }}"#
    );
    let error = space_error_of(&objects, 60.0, 20.0);
    assert!(error.to_string().contains("半径"), "{error}");
}

// ---- 接平面：描画 ----

#[test]
fn 放物面の頂点の接平面は_水平になる() {
    // z=u^2+v^2の(0,0)における接平面は，水平面z=0である．
    // 方位角0度・仰角0度(x軸の正の向きから見る)だと，画面の縦方向がちょうど世界のz軸になるので，
    // 水平な接平面は，画面上で，縦の位置(y)が0の水平線になる．
    // 曲面自身の輪郭とは区別できるように，曲面だけの図との差分を，接平面の点として取り出す．
    let without = space_figure_of(PARABOLOID, 0.0, 0.0);
    let objects = format!(
        r#"{PARABOLOID}, {{ "id": "p", "type": "tangent_plane", "of": "s",
            "at": [0, 0], "size": 1.5 }}"#
    );
    let with = space_figure_of(&objects, 0.0, 0.0);
    let tangent_points: Vec<[f64; 2]> = points(&with)
        .into_iter()
        .skip(points(&without).len())
        .collect();
    assert!(!tangent_points.is_empty(), "接平面の点がある");
    let extent = tangent_points
        .iter()
        .map(|[x, _]| x.abs())
        .fold(0.0_f64, f64::max);
    // sizeがcmでの半径になるので，画面の横方向の最大の広がりは，およそ1.5cmになる．
    assert!(
        (extent - 1.5).abs() < 1e-2,
        "接平面の広さがsizeに合わない：{extent}"
    );
    for [_, y] in tangent_points {
        assert!(y.abs() < 1e-2, "縦の位置が0からずれている(水平でない)：{y}");
    }
}

#[test]
fn 曲面でないオブジェクトを指せば断る() {
    let objects = r#"{ "id": "b", "type": "sphere", "center": [0, 0, 0], "radius": 1 },
        { "id": "p", "type": "tangent_plane", "of": "b", "at": [0, 0], "size": 1 }"#;
    let error = space_error_of(objects, 60.0, 20.0);
    assert!(error.to_string().contains('b'), "{error}");
}
