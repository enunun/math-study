//! フラクタル図形(`fractal`)を確かめる．平面の図でだけ使える．
//!
//! 反復関数系(IFS)：`base`(通る点の列)を，`transforms`(相似変換より自由な，並進・回転・拡大縮小・
//! せん断を組み合わせた変換の並び)で，`depth`回，再帰的に写す．深さ0は`base`そのもの，深さnは，
//! 深さn-1の図形全体に，それぞれの変換を施したものすべての集まりである．

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

fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn plane_error_of(objects: &str) -> Error {
    parse_scene(&plane_scene(objects)).expect_err("誤りになる")
}

fn space_error_of(objects: &str) -> Error {
    parse_scene(&space_scene(objects)).expect_err("誤りになる")
}

fn plane_figure_of(objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(objects)).expect("読める")).expect("描画できる")
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

fn has_point(path: &Path, point: [f64; 2]) -> bool {
    path.points
        .iter()
        .any(|[x, y]| (x - point[0]).abs() < 1e-6 && (y - point[1]).abs() < 1e-6)
}

/// 2点(0,0)-(1,0)の基本図形．
const SEGMENT: &str = r#""base": [[0, 0], [1, 0]]"#;

// ---- 読み込みと検査 ----

#[test]
fn フラクタルは_base_transforms_depthを持つ() {
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [[{{ "translate": [2, 0] }}]], "depth": 1 }}"#
    );
    let scene = parse_scene(&plane_scene(&objects)).expect("読める");
    let Object::Fractal(fractal) = &scene.objects[0] else {
        panic!("フラクタルである");
    };
    assert_eq!(fractal.base.len(), 2);
    assert_eq!(fractal.transforms.len(), 1);
    assert_eq!(fractal.depth, 1);
}

#[test]
fn 空間の図では使えない() {
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [[{{ "translate": [1, 0] }}]], "depth": 1 }}"#
    );
    let error = space_error_of(&objects);
    assert!(error.to_string().contains("空間"), "{error}");
}

#[test]
fn 基本図形の点が2点未満なら断る() {
    let objects = r#"{ "id": "f", "type": "fractal", "base": [[0, 0]],
        "transforms": [[{ "translate": [1, 0] }]], "depth": 1 }"#;
    let error = plane_error_of(objects);
    assert!(error.to_string().contains('2'), "{error}");
}

#[test]
fn 変換が1つもなければ断る() {
    let objects =
        format!(r#"{{ "id": "f", "type": "fractal", {SEGMENT}, "transforms": [], "depth": 1 }}"#);
    let error = plane_error_of(&objects);
    assert!(error.to_string().contains("transforms"), "{error}");
}

#[test]
fn 深さが大きすぎれば断る() {
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [[{{ "translate": [1, 0] }}]], "depth": 100 }}"#
    );
    let error = plane_error_of(&objects);
    assert!(error.to_string().contains("depth"), "{error}");
}

#[test]
fn 変換と深さの組み合わせで図形の数が多すぎれば断る() {
    // 3つの変換を10回反復すると3^10=59049個の図形になり，上限を超える．
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [
                [{{ "translate": [1, 0] }}], [{{ "translate": [2, 0] }}], [{{ "translate": [3, 0] }}]
            ], "depth": 10 }}"#
    );
    let error = plane_error_of(&objects);
    assert!(error.to_string().contains("フラクタル"), "{error}");
}

// ---- 描画 ----

#[test]
fn 深さ0では_基本図形をそのまま描く() {
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [[{{ "translate": [1, 0] }}]], "depth": 0 }}"#
    );
    let figure = plane_figure_of(&objects);
    let lines = paths(&figure);
    assert_eq!(lines.len(), 1, "線が1本");
    assert!(has_point(lines[0], [0.0, 0.0]));
    assert!(has_point(lines[0], [1.0, 0.0]));
}

#[test]
fn 並進だけの2つの変換は_深さ1で2つの離れた図形になる() {
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [[{{ "translate": [0, 0] }}], [{{ "translate": [2, 0] }}]], "depth": 1 }}"#
    );
    let figure = plane_figure_of(&objects);
    let lines = paths(&figure);
    assert_eq!(lines.len(), 2, "線が2本(離れた図形)");
    assert!(
        lines
            .iter()
            .any(|p| has_point(p, [2.0, 0.0]) && has_point(p, [3.0, 0.0]))
    );
    assert!(
        lines
            .iter()
            .any(|p| has_point(p, [0.0, 0.0]) && has_point(p, [1.0, 0.0]))
    );
}

#[test]
fn 拡大縮小のあとに並進すると_その順で施される() {
    // (1,0)を半分に縮め(0.5,0)，(2,0)だけ動かすと(2.5,0)になる．
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [[{{ "scale": [0.5, 0.5] }}, {{ "translate": [2, 0] }}]], "depth": 1 }}"#
    );
    let figure = plane_figure_of(&objects);
    let lines = paths(&figure);
    assert_eq!(lines.len(), 1);
    assert!(has_point(lines[0], [2.0, 0.0]), "{:?}", lines[0].points);
    assert!(has_point(lines[0], [2.5, 0.0]), "{:?}", lines[0].points);
}

#[test]
fn 回転は反時計回りに施される() {
    // (1,0)を90度回すと，およそ(0,1)になる．
    let objects = r#"{ "id": "f", "type": "fractal", "base": [[0, 0], [1, 0]],
        "transforms": [[{ "rotate": 90 }]], "depth": 1 }"#;
    let figure = plane_figure_of(objects);
    let lines = paths(&figure);
    assert_eq!(lines.len(), 1);
    assert!(has_point(lines[0], [0.0, 0.0]));
    assert!(has_point(lines[0], [0.0, 1.0]), "{:?}", lines[0].points);
}

#[test]
fn 深さを重ねると_図形の数が変換の数のべき乗になる() {
    let objects = format!(
        r#"{{ "id": "f", "type": "fractal", {SEGMENT},
            "transforms": [
                [{{ "scale": [0.4, 0.4] }}, {{ "translate": [0, 0] }}],
                [{{ "scale": [0.4, 0.4] }}, {{ "translate": [2, 0] }}]
            ], "depth": 2 }}"#
    );
    let figure = plane_figure_of(&objects);
    assert_eq!(paths(&figure).len(), 4, "2つの変換を深さ2で，4つの図形");
}

#[test]
fn closedにすると_最後の点から最初の点への線も引かれる() {
    let objects = r#"{ "id": "f", "type": "fractal",
        "base": [[0, 0], [1, 0], [0, 1]], "closed": true,
        "transforms": [[{ "translate": [0, 0] }]], "depth": 0 }"#;
    let figure = plane_figure_of(objects);
    let lines = paths(&figure);
    assert_eq!(lines.len(), 1);
    // 閉じているので，最初の点(0,0)が，並びの最後にも現れる．
    let last = lines[0].points.last().expect("点がある");
    assert!(
        (last[0] - 0.0).abs() < 1e-6 && (last[1] - 0.0).abs() < 1e-6,
        "{last:?}"
    );
}
