//! Wigner-Seitz胞(`wigner_seitz`)を確かめる．2つの基本ベクトルで張る格子の，原点(中心)にいちばん近い点の
//! 集まりである．原点と近くの格子点を結ぶ線分の垂直二等分線で切った半平面を重ねた，凸多角形になる．
//! 逆格子の基本ベクトルを渡せば，第1Brillouinゾーンである．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item};
use figure::{ErrorKind, parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-3, 3], "y": [-3, 3], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn figure_of(objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(objects)).expect("読める")).expect("描ける")
}

/// 閉じた辺の折れ線の頂点(始めに戻る点は除き，重なる点は1つにする)．
fn vertices(figure: &Figure) -> Vec<[f64; 2]> {
    let path = figure
        .items
        .iter()
        .find_map(|item| match item {
            Item::Path(path) => Some(path),
            _ => None,
        })
        .expect("辺がある");
    let mut points = path.points.clone();
    points.pop();
    points
}

fn cell(basis: &str, extra: &str) -> String {
    format!(r#"{{ "id": "w", "type": "wigner_seitz", "basis": {basis} {extra} }}"#)
}

fn contains(points: &[[f64; 2]], target: [f64; 2]) -> bool {
    points
        .iter()
        .any(|p| (p[0] - target[0]).abs() < 1e-9 && (p[1] - target[1]).abs() < 1e-9)
}

#[test]
fn 正方格子のwigner_seitz胞は_原点を中心とする正方形である() {
    let points = vertices(&figure_of(&cell("[[1, 0], [0, 1]]", "")));
    assert_eq!(points.len(), 4, "{points:?}");
    for corner in [[0.5, 0.5], [-0.5, 0.5], [-0.5, -0.5], [0.5, -0.5]] {
        assert!(contains(&points, corner), "{corner:?}：{points:?}");
    }
}

#[test]
fn 六方格子のwigner_seitz胞は_正六角形である() {
    // a_1 = (1, 0)，a_2 = (1/2, √3/2)．頂点は原点から1/√3の距離にある．
    let points = vertices(&figure_of(&cell("[[1, 0], [0.5, \"sqrt(3)/2\"]]", "")));
    assert_eq!(points.len(), 6, "{points:?}");
    for [x, y] in &points {
        assert!(
            (x.hypot(*y) - 1.0 / 3.0_f64.sqrt()).abs() < 1e-9,
            "({x}, {y})"
        );
    }
}

#[test]
fn 斜交格子のwigner_seitz胞は_面積が単位胞と等しい六角形である() {
    // a_1 = (1, 0)，a_2 = (0.3, 1.2)．単位胞の面積は1.2である．
    let points = vertices(&figure_of(&cell("[[1, 0], [0.3, 1.2]]", "")));
    assert_eq!(points.len(), 6, "{points:?}");
    let area: f64 = (0..points.len())
        .map(|k| {
            let [x0, y0] = points[k];
            let [x1, y1] = points[(k + 1) % points.len()];
            x0 * y1 - x1 * y0
        })
        .sum::<f64>()
        / 2.0;
    assert!((area.abs() - 1.2).abs() < 1e-9, "{area}");
}

#[test]
fn 中心と塗りと媒介変数を使える() {
    let figure = figure_of(&format!(
        r#"{{ "id": "a", "type": "parameter", "value": 2 }},
           {}"#,
        cell(
            r#"[["a", 0], [0, "a"]]"#,
            r#", "center": [1, 0], "fill": { "color": "blue", "opacity": 0.2 }"#
        )
    ));
    let points = vertices(&figure);
    assert!(
        contains(&points, [2.0, 1.0]) && contains(&points, [0.0, -1.0]),
        "{points:?}"
    );
    assert!(
        figure
            .items
            .iter()
            .any(|item| matches!(item, Item::Fill(_)))
    );
}

#[test]
fn 基本ベクトルが平行なら誤りになる() {
    let error = parse_scene(&plane_scene(&cell("[[1, 0], [2, 0]]", ""))).expect_err("誤りになる");
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert!(error.to_string().contains("basis"), "{error}");
}

#[test]
fn 空間の図ではまだ使えない() {
    let error = parse_scene(&format!(
        r#"{{ "version": "0.1.0", "description": "a",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{}] }}"#,
        cell("[[1, 0], [0, 1]]", "")
    ))
    .expect_err("誤りになる");
    assert!(error.to_string().contains("wigner_seitz"), "{error}");
}

#[test]
fn 射影がちょうど半分になる基底でも_止まって胞を描く() {
    // a_2の，a_1への射影がちょうど1/2になる．簡約が同じ組を行き来しないことを確かめる．
    for basis in [
        "[[1, 0], [0.5, 1]]",
        "[[1, 0], [-0.5, 1]]",
        "[[2, 0], [1, 1]]",
        "[[1, 0], [1.5, 1]]",
    ] {
        let points = vertices(&figure_of(&cell(basis, "")));
        assert!(points.len() >= 4, "{basis}：{points:?}");
    }
}
