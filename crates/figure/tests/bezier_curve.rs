//! ベジエ曲線(`curve`の`bezier`)を確かめる．平面と空間，どちらでも使える．

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
             "view": {{ "x": [-1, 4], "y": [-1, 4], "unit": {{ "x": "1cm", "y": "1cm" }} }},
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

fn space_figure_of(objects: &str) -> Figure {
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

/// 平面のベジエ曲線．3点の制御点による，2次のベジエ曲線である．
const PLANE_BEZIER: &str = r#"{ "id": "c", "type": "curve",
    "bezier": [[0, 0], [1, 2], [2, 0]] }"#;

/// 空間のベジエ曲線．
const SPACE_BEZIER: &str = r#"{ "id": "c", "type": "curve",
    "bezier": [[0, 0, 0], [1, 2, 1], [2, 0, 0]] }"#;

// ---- 読み込みと検査 ----

#[test]
fn ベジエ曲線は_制御点だけで書け_var_expr_domainを持たない() {
    let scene = parse_scene(&plane_scene(PLANE_BEZIER)).expect("読める");
    let Object::Curve(curve) = &scene.objects[0] else {
        panic!("曲線である");
    };
    assert!(curve.var.is_none());
    assert!(curve.expr.is_empty());
    assert!(curve.domain.is_none());
    let net = curve.bezier.as_ref().expect("制御点がある");
    assert_eq!(net.len(), 3);
}

#[test]
fn 式とベジエを同時に書けば断る() {
    let mixed = r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"],
        "domain": [0, 1], "bezier": [[0, 0], [1, 1]] }"#;
    let error = plane_error_of(mixed);
    assert!(error.to_string().contains("bezier"), "{error}");
}

#[test]
fn 式もベジエも持たなければ断る() {
    let error = plane_error_of(r#"{ "id": "c", "type": "curve" }"#);
    assert!(error.to_string().contains("bezier"), "{error}");
}

#[test]
fn 制御点は2点未満なら断る() {
    let error = plane_error_of(r#"{ "id": "c", "type": "curve", "bezier": [[0, 0]] }"#);
    assert!(error.to_string().contains('2'), "{error}");
}

#[test]
fn 制御点は平面では2個_空間では3個の座標で書く() {
    let wrong_dimension = r#"{ "id": "c", "type": "curve", "bezier": [[0, 0, 0], [1, 1, 1]] }"#;
    let error = plane_error_of(wrong_dimension);
    assert!(error.to_string().contains("座標"), "{error}");

    let wrong_in_space = r#"{ "id": "c", "type": "curve", "bezier": [[0, 0], [1, 1]] }"#;
    let error = space_error_of(wrong_in_space);
    assert!(error.to_string().contains("座標"), "{error}");
}

#[test]
fn 書き出すと_ベジエ曲線にはvar_expr_domainの項目がない() {
    let scene = parse_scene(&plane_scene(PLANE_BEZIER)).expect("読める");
    let json = serde_json::to_string(&scene).expect("書き出せる");
    assert!(!json.contains("\"var\""));
    assert!(!json.contains("\"expr\""));
    assert!(!json.contains("\"domain\""));
}

// ---- 描画 ----

#[test]
fn 平面のベジエ曲線は_始点と終点をちょうど通る() {
    let figure = plane_figure_of(PLANE_BEZIER);
    let lines = paths(&figure);
    assert!(!lines.is_empty(), "線がある");
    let first = lines
        .first()
        .expect("最初の線がある")
        .points
        .first()
        .expect("最初の点がある");
    let last = lines
        .last()
        .expect("最後の線がある")
        .points
        .last()
        .expect("最後の点がある");
    // 1cm/単位なので，座標(0,0)は画面の原点，(2,0)は右へ2cmになる．
    assert!(
        (first[0] - 0.0).abs() < 1e-6 && (first[1] - 0.0).abs() < 1e-6,
        "{first:?}"
    );
    assert!(
        (last[0] - 2.0).abs() < 1e-6 && (last[1] - 0.0).abs() < 1e-6,
        "{last:?}"
    );
}

#[test]
fn 空間のベジエ曲線も描ける() {
    let figure = space_figure_of(SPACE_BEZIER);
    assert!(!paths(&figure).is_empty(), "線がある");
}

#[test]
fn 直線状の3制御点は_中点も同じ直線上を通る() {
    // (0,0)，(1,0)，(2,0)は一直線なので，ベジエ曲線も同じ直線(y=0)になる．
    let straight = r#"{ "id": "c", "type": "curve", "bezier": [[0, 0], [1, 0], [2, 0]] }"#;
    let figure = plane_figure_of(straight);
    for path in paths(&figure) {
        for point in &path.points {
            assert!(point[1].abs() < 1e-6, "y座標が0からずれている：{point:?}");
        }
    }
}
