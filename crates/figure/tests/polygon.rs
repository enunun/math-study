//! 多角形(`polygon`)を確かめる．正多角形は辺の数と中心と半径で，ほかの多角形は頂点の並びで書き，
//! 辺(閉じた折れ線)と，塗った面(`fill`)を持つ．平面の図でだけ使える．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::{Error, parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-5, 5], "y": [-5, 5], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn figure_of(objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(objects)).expect("読める")).expect("描画できる")
}

fn error_of(objects: &str) -> Error {
    parse_scene(&plane_scene(objects)).expect_err("誤りになる")
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

#[test]
fn 正多角形は中心から半径の所に頂点を置き_底辺を水平にする() {
    let figure =
        figure_of(r#"{ "id": "p", "type": "polygon", "sides": 6, "center": [1, 1], "radius": 2 }"#);
    let edge = paths(&figure);
    assert_eq!(edge.len(), 1);
    let points = &edge[0].points;
    // 閉じた折れ線なので，最初の頂点に戻る．
    assert_eq!(points.len(), 7);
    assert_eq!(points.first(), points.last());
    for [x, y] in points {
        assert!(((x - 1.0).hypot(y - 1.0) - 2.0).abs() < 1e-9);
    }
    let lowest = points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let bottom = points
        .iter()
        .filter(|p| (p[1] - lowest).abs() < 1e-9)
        .count();
    // 最初と最後の点は同じ頂点なので，底辺の2頂点は，3回現れることもある．
    assert!(bottom >= 2);
}

#[test]
fn 頂点の並びで書けて_点の式を使える() {
    let figure = figure_of(
        r#"{ "id": "A", "type": "point", "at": [0, 0] },
           { "id": "B", "type": "point", "at": [3, 0] },
           { "id": "t", "type": "polygon", "vertices": ["A", "B", [0, 2]] }"#,
    );
    assert_eq!(
        paths(&figure)[0].points,
        [[0.0, 0.0], [3.0, 0.0], [0.0, 2.0], [0.0, 0.0]]
    );
}

#[test]
fn 面は辺の下に塗る() {
    let figure = figure_of(
        r#"{ "id": "p", "type": "polygon", "sides": 4, "center": [0, 0], "radius": 1,
             "fill": { "color": "blue", "opacity": 0.3 } }"#,
    );
    assert!(matches!(figure.items[0], Item::Fill(_)));
    assert!(matches!(figure.items[1], Item::Path(_)));
    let Item::Fill(fill) = &figure.items[0] else {
        panic!("塗りがない");
    };
    assert_eq!(fill.points.len(), 4);
    assert_eq!(fill.opacity, 0.3);
}

#[test]
fn 変換を施せる() {
    // 正方形を45度回すと，頂点が座標軸の上に来る．
    let figure = figure_of(
        r#"{ "id": "p", "type": "polygon", "sides": 4, "center": [0, 0], "radius": 1,
             "transform": [{ "rotate": 45 }] }"#,
    );
    for [x, y] in &paths(&figure)[0].points {
        assert!(x.abs() < 1e-9 || y.abs() < 1e-9);
    }
}

#[test]
fn 写像で写すと辺が曲がる() {
    let figure = figure_of(
        r#"{ "id": "F", "type": "map", "vars": ["x", "y"], "expr": ["x^2 - y^2", "2*x*y"] },
           { "id": "p", "type": "polygon", "sides": 4, "center": [1, 1], "radius": 0.5,
             "fill": {}, "transform": [{ "map": "F" }] }"#,
    );
    assert!(paths(&figure)[0].points.len() > 5);
    assert!(matches!(figure.items[0], Item::Fill(_)));
}

#[test]
fn 正多角形と頂点の並びは同時に書けない() {
    let error = error_of(
        r#"{ "id": "p", "type": "polygon", "sides": 3, "center": [0, 0], "radius": 1,
             "vertices": [[0, 0], [1, 0], [0, 1]] }"#,
    );
    assert!(error.to_string().contains("どちらか一方"));
    let error = error_of(r#"{ "id": "p", "type": "polygon" }"#);
    assert!(error.to_string().contains("どちらか一方"));
}

#[test]
fn 辺の数と頂点の数は3以上である() {
    let error =
        error_of(r#"{ "id": "p", "type": "polygon", "sides": 2, "center": [0, 0], "radius": 1 }"#);
    assert!(error.to_string().contains("sides"));
    let error = error_of(r#"{ "id": "p", "type": "polygon", "vertices": [[0, 0], [1, 0]] }"#);
    assert!(error.to_string().contains("vertices"));
    let error = error_of(r#"{ "id": "p", "type": "polygon", "sides": 3, "center": [0, 0] }"#);
    assert!(error.to_string().contains("radius"));
}

#[test]
fn 空間の図では使えない() {
    let scene = r#"{ "version": "0.1.0", "description": "試験の図",
        "view": { "azimuth": 60, "elevation": 20, "unit": "1cm" },
        "objects": [{ "id": "p", "type": "polygon", "sides": 3, "center": [0, 0], "radius": 1 }] }"#;
    assert!(parse_scene(scene).is_err());
}
