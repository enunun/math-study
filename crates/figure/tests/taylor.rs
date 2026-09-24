//! テイラー展開の多項式(`taylor`)と，関数(`function`)を使うシーンを確かめる．

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
             "view": {{ "x": [-4, 4], "y": [-4, 4], "unit": {{ "x": "1cm", "y": "1cm" }} }},
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

/// 折れ線の点が，どれも`y = f(x)`の上にあるか．標本化した点は，多項式の値そのものである．
fn on_graph(paths: &[&Path], f: impl Fn(f64) -> f64) -> bool {
    paths
        .iter()
        .flat_map(|path| &path.points)
        .all(|[x, y]| (y - f(*x)).abs() < 1e-9)
}

#[test]
fn 正弦の3次のテイラー多項式を描く() {
    let figure = figure_of(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "sin(x)", "domain": [-3, 3] },
           { "id": "p", "type": "taylor", "of": "f", "at": 0, "order": 3, "domain": [-2, 2] }"#,
    );
    let polynomial = &paths(&figure)[1..];
    assert!(on_graph(polynomial, |x| x - x * x * x / 6.0));
    // 範囲は，書いた範囲である．
    let first = polynomial[0].points[0][0];
    assert!((first + 2.0).abs() < 1e-9);
}

#[test]
fn 範囲を省くとグラフの定義域に描く() {
    let figure = figure_of(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "exp(x)", "domain": [-1, 1] },
           { "id": "p", "type": "taylor", "of": "f", "at": 0, "order": 2 }"#,
    );
    let polynomial = paths(&figure)[1];
    assert_eq!(polynomial.points.first().unwrap()[0], -1.0);
    assert_eq!(polynomial.points.last().unwrap()[0], 1.0);
}

#[test]
fn 展開の中心は式で書け_グラフはあとに置いてもよい() {
    let figure = figure_of(
        r#"{ "id": "a", "type": "parameter", "value": 1 },
           { "id": "p", "type": "taylor", "of": "f", "at": "a", "order": 1 },
           { "id": "f", "type": "graph", "var": "x", "expr": "x^2", "domain": [-2, 2] }"#,
    );
    // x^2の，x = 1での接線は，y = 2x - 1である．
    let tangent = paths(&figure)[0];
    for [x, y] in &tangent.points {
        assert!((y - (2.0 * x - 1.0)).abs() < 1e-9);
    }
}

#[test]
fn 関数を使ったグラフも展開できる() {
    let figure = figure_of(
        r#"{ "id": "g", "type": "function", "vars": ["x"], "expr": "x^2" },
           { "id": "h", "type": "function", "vars": ["x"], "expr": "cos(g(x))" },
           { "id": "f", "type": "graph", "var": "x", "expr": "h(x)", "domain": [-1, 1] },
           { "id": "p", "type": "taylor", "of": "f", "at": 0, "order": 4 }"#,
    );
    // cos(x^2) = 1 - x^4/2 + ...
    let polynomial = &paths(&figure)[1..];
    assert!(on_graph(polynomial, |x| 1.0 - x.powi(4) / 2.0));
}

#[test]
fn 展開できない式と次数の上限() {
    let error = error_of(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "gamma(x)", "domain": [1, 3] },
           { "id": "p", "type": "taylor", "of": "f", "at": 2, "order": 3 }"#,
    );
    assert_eq!(error.object.as_deref(), Some("p"));
    assert!(error.to_string().contains("テイラー展開できない"));
    let error = error_of(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "x", "domain": [1, 3] },
           { "id": "p", "type": "taylor", "of": "f", "at": 2, "order": 31 }"#,
    );
    assert!(error.to_string().contains("order"));
    let error = error_of(r#"{ "id": "p", "type": "taylor", "of": "f", "at": 2, "order": 3 }"#);
    assert!(error.to_string().contains("グラフ「f」がない"));
}

#[test]
fn 関数は媒介変数を使え_名前は重ねられない() {
    let figure = figure_of(
        r#"{ "id": "k", "type": "parameter", "value": 2 },
           { "id": "f", "type": "function", "vars": ["x"], "expr": "k*x" },
           { "id": "g", "type": "graph", "var": "x", "expr": "f(x)", "domain": [0, 1] }"#,
    );
    let line = paths(&figure)[0];
    assert_eq!(line.points.last().unwrap(), &[1.0, 2.0]);
    let error = error_of(r#"{ "id": "sin", "type": "function", "vars": ["x"], "expr": "x" }"#);
    assert!(error.to_string().contains("関数か定数の名前"));
    let error = error_of(
        r#"{ "id": "f", "type": "function", "vars": ["x"], "expr": "g(x)" },
           { "id": "g", "type": "function", "vars": ["x"], "expr": "x" }"#,
    );
    // あとに定義する関数は，呼べない．
    assert_eq!(error.object.as_deref(), Some("f"));
}
