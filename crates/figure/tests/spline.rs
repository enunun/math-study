//! スプライン曲線(`curve`の`spline`)を確かめる．平面と空間，どちらでも使える．
//!
//! ベジエ曲線(制御点は曲線の形を決めるが，曲線自身は通らない)と違い，スプライン曲線は，
//! 与えた点をすべて，順に通る．

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

/// 平面のスプライン曲線．4点を順に通る．
const PLANE_SPLINE: &str = r#"{ "id": "c", "type": "curve",
    "spline": [[0, 0], [1, 2], [2, 0], [3, 1]] }"#;

/// 空間のスプライン曲線．
const SPACE_SPLINE: &str = r#"{ "id": "c", "type": "curve",
    "spline": [[0, 0, 0], [1, 2, 1], [2, 0, 0], [3, 1, 1]] }"#;

// ---- 読み込みと検査 ----

#[test]
fn スプライン曲線は_点だけで書け_var_expr_domainを持たない() {
    let scene = parse_scene(&plane_scene(PLANE_SPLINE)).expect("読める");
    let Object::Curve(curve) = &scene.objects[0] else {
        panic!("曲線である");
    };
    assert!(curve.var.is_none());
    assert!(curve.expr.is_empty());
    assert!(curve.domain.is_none());
    assert!(curve.bezier.is_none());
    let points = curve.spline.as_ref().expect("点がある");
    assert_eq!(points.len(), 4);
}

#[test]
fn 式とスプラインを同時に書けば断る() {
    let mixed = r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"],
        "domain": [0, 1], "spline": [[0, 0], [1, 1]] }"#;
    let error = plane_error_of(mixed);
    assert!(error.to_string().contains("spline"), "{error}");
}

#[test]
fn ベジエとスプラインを同時に書けば断る() {
    let mixed = r#"{ "id": "c", "type": "curve",
        "bezier": [[0, 0], [1, 1]], "spline": [[0, 0], [1, 1]] }"#;
    let error = plane_error_of(mixed);
    assert!(error.to_string().contains("spline"), "{error}");
}

#[test]
fn 点は2点未満なら断る() {
    let error = plane_error_of(r#"{ "id": "c", "type": "curve", "spline": [[0, 0]] }"#);
    assert!(error.to_string().contains('2'), "{error}");
}

#[test]
fn 点は平面では2個_空間では3個の座標で書く() {
    let wrong_dimension = r#"{ "id": "c", "type": "curve", "spline": [[0, 0, 0], [1, 1, 1]] }"#;
    let error = plane_error_of(wrong_dimension);
    assert!(error.to_string().contains("座標"), "{error}");

    let wrong_in_space = r#"{ "id": "c", "type": "curve", "spline": [[0, 0], [1, 1]] }"#;
    let error = space_error_of(wrong_in_space);
    assert!(error.to_string().contains("座標"), "{error}");
}

// ---- 描画 ----

#[test]
fn スプライン曲線の点を直接評価すると_与えた点をすべて_順に_ちょうど通る() {
    // 4点(t=0，1/3，2/3，1に対応)を，エンジンの関数で直接評価する．
    // 標本化(sample)は，画面での滑らかさに合わせて刻むので，途中の点をちょうど拾うとは限らない．
    let given: Vec<Vec<f64>> = vec![
        vec![0.0, 0.0],
        vec![1.0, 2.0],
        vec![2.0, 0.0],
        vec![3.0, 1.0],
    ];
    for (index, point) in given.iter().enumerate() {
        let t = f64::from(u32::try_from(index).unwrap_or(0))
            / f64::from(u32::try_from(given.len() - 1).unwrap_or(1));
        let found = figure::spline::catmull_rom_point(&given, t).expect("評価できる");
        assert!(
            (found[0] - point[0]).abs() < 1e-9 && (found[1] - point[1]).abs() < 1e-9,
            "t={t}：{found:?}が{point:?}と一致しない"
        );
    }
}

#[test]
fn 平面のスプライン曲線は_始点と終点をちょうど通る() {
    let figure = plane_figure_of(PLANE_SPLINE);
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
    // 1cm/単位なので，座標(0,0)は画面の原点，(3,1)は右へ3cm上へ1cmになる．
    assert!(
        (first[0] - 0.0).abs() < 1e-6 && (first[1] - 0.0).abs() < 1e-6,
        "{first:?}"
    );
    assert!(
        (last[0] - 3.0).abs() < 1e-6 && (last[1] - 1.0).abs() < 1e-6,
        "{last:?}"
    );
}

#[test]
fn 空間のスプライン曲線も描ける() {
    let figure = space_figure_of(SPACE_SPLINE);
    assert!(!paths(&figure).is_empty(), "線がある");
}

#[test]
fn 一直線上の点を順に通れば_スプライン曲線も同じ直線上を通る() {
    let straight = r#"{ "id": "c", "type": "curve",
        "spline": [[0, 0], [1, 0], [2, 0], [3, 0]] }"#;
    let figure = plane_figure_of(straight);
    for path in paths(&figure) {
        for point in &path.points {
            assert!(point[1].abs() < 1e-6, "y座標が0からずれている：{point:?}");
        }
    }
}
