//! 楕円曲線(`elliptic_curve`)を確かめる．Weierstrassの標準形`y^2 = x^3 + a x + b`の実の点を描き，
//! 点`P`，`Q`を与えると，和`P + Q`の作図(`P`，`Q`を通る直線，3つ目の交点，x軸についての折り返し)を描く．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp
)]

use figure::figure::{Figure, Item};
use figure::{parse_scene, render};

fn plane_scene(range: f64, objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-{range}, {range}], "y": [-{range}, {range}], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn figure_of(range: f64, objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(range, objects)).expect("読める")).expect("描ける")
}

fn curve(a: &str, b: &str, extra: &str) -> String {
    format!(r#"{{ "id": "E", "type": "elliptic_curve", "a": {a}, "b": {b} {extra} }}"#)
}

fn paths(figure: &Figure) -> Vec<&Vec<[f64; 2]>> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Path(path) => Some(&path.points),
            _ => None,
        })
        .collect()
}

fn dots(figure: &Figure) -> Vec<[f64; 2]> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Dot(dot) => Some(dot.at),
            _ => None,
        })
        .collect()
}

fn labels(figure: &Figure) -> Vec<(String, [f64; 2])> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(label) => Some((label.tex.clone(), label.at)),
            _ => None,
        })
        .collect()
}

fn has_dot(figure: &Figure, target: [f64; 2]) -> bool {
    dots(figure)
        .iter()
        .any(|p| (p[0] - target[0]).abs() < 1e-9 && (p[1] - target[1]).abs() < 1e-9)
}

fn label_at(figure: &Figure, tex: &str) -> [f64; 2] {
    labels(figure)
        .into_iter()
        .find(|(text, _)| text == tex)
        .unwrap_or_else(|| panic!("{tex}のラベルがない：{:?}", labels(figure)))
        .1
}

#[test]
fn 曲線の点は方程式を満たす() {
    let figure = figure_of(3.0, &curve("-1", "1", ""));
    let all = paths(&figure);
    assert!(!all.is_empty());
    for points in all {
        // 見える範囲の縁で切った端は，線分の上の点なので，除く．
        for [x, y] in points
            .iter()
            .filter(|[x, y]| x.abs() < 3.0 && y.abs() < 3.0)
        {
            let residual = y * y - (x * x * x - x + 1.0);
            assert!(residual.abs() < 1e-9, "({x}, {y})：{residual}");
        }
    }
}

#[test]
fn 実根が3つなら閉じた卵形と無限に伸びる枝の2つに分かれる() {
    // y^2 = x^3 - x = (x + 1) x (x - 1)．卵形は-1 <= x <= 0，枝は1 <= x．
    let figure = figure_of(3.0, &curve("-1", "0", ""));
    let all = paths(&figure);
    let oval: Vec<_> = all
        .iter()
        .filter(|points| points.iter().all(|p| p[0] <= 1e-9))
        .collect();
    let branch: Vec<_> = all
        .iter()
        .filter(|points| points.iter().all(|p| p[0] >= 1.0 - 1e-9))
        .collect();
    assert_eq!(oval.len() + branch.len(), all.len(), "どちらかに入る");
    assert!(!oval.is_empty() && !branch.is_empty());
    // 卵形は閉じていて，x軸との交点(-1, 0)と(0, 0)を通る．
    let ring = oval[0];
    assert_eq!(oval.len(), 1);
    let first = ring.first().unwrap();
    let last = ring.last().unwrap();
    assert!((first[0] - last[0]).abs() < 1e-9 && (first[1] - last[1]).abs() < 1e-9);
    let min_x = ring.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let max_x = ring.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (min_x + 1.0).abs() < 1e-9 && max_x.abs() < 1e-9,
        "{min_x}，{max_x}"
    );
    // 枝は，根x = 1で縦に接し，上下に見える範囲の端まで伸びる．
    let ys: Vec<f64> = branch
        .iter()
        .flat_map(|points| points.iter().map(|p| p[1]))
        .collect();
    let top = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let bottom = ys.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        (top - 3.0).abs() < 1e-9 && (bottom + 3.0).abs() < 1e-9,
        "{top}，{bottom}"
    );
}

#[test]
fn 和の作図は3つ目の交点とその折り返しを描く() {
    // y^2 = x^3 + 17．P = (-2, 3)，Q = (-1, 4)の和は(4, -9)，3つ目の交点は(4, 9)．
    let figure = figure_of(10.0, &curve("0", "17", r#", "p": -2, "q": -1"#));
    for point in [[-2.0, 3.0], [-1.0, 4.0], [4.0, 9.0], [4.0, -9.0]] {
        assert!(has_dot(&figure, point), "{point:?}：{:?}", dots(&figure));
    }
    assert_eq!(label_at(&figure, "$P$"), [-2.0, 3.0]);
    assert_eq!(label_at(&figure, "$Q$"), [-1.0, 4.0]);
    assert_eq!(label_at(&figure, "$-(P+Q)$"), [4.0, 9.0]);
    assert_eq!(label_at(&figure, "$P+Q$"), [4.0, -9.0]);
}

#[test]
fn 下の枝の点も選べる() {
    // P = (-2, -3)，Q = (-1, 4)．傾き7，x3 = 49 + 3 = 52 は範囲の外でも，点の座標は正しく求める．
    // 見える範囲の外の点は描かないので，ここでは範囲を広げる．
    let figure = figure_of(
        400.0,
        &curve("0", "17", r#", "p": -2, "p_lower": true, "q": -1"#),
    );
    // y3 = 7 (-2 - 52) + 3 = -375．
    assert!(has_dot(&figure, [-2.0, -3.0]));
    assert!(has_dot(&figure, [52.0, -375.0]), "{:?}", dots(&figure));
    assert!(has_dot(&figure, [52.0, 375.0]));
}

#[test]
fn qを省くと接線で2倍の点を作図する() {
    // P = (-2, 3)の2倍は(8, -23)．
    let figure = figure_of(30.0, &curve("0", "17", r#", "p": -2"#));
    assert!(has_dot(&figure, [8.0, -23.0]), "{:?}", dots(&figure));
    assert!(has_dot(&figure, [8.0, 23.0]));
    assert_eq!(label_at(&figure, "$2P$"), [8.0, -23.0]);
    assert_eq!(label_at(&figure, "$-2P$"), [8.0, 23.0]);
}

#[test]
fn 作図の直線は3つの点を通る() {
    let figure = figure_of(10.0, &curve("0", "17", r#", "p": -2, "q": -1"#));
    // 傾き1の直線y = x + 5．
    let on_line = paths(&figure).into_iter().any(|points| {
        points.len() >= 2 && points.iter().all(|[x, y]| (y - (x + 5.0)).abs() < 1e-9)
    });
    assert!(on_line);
}

#[test]
fn 和が無限遠点になるときは縦の直線だけを描く() {
    // P = (-2, 3)，Q = (-2, -3)．
    let figure = figure_of(
        10.0,
        &curve("0", "17", r#", "p": -2, "q": -2, "q_lower": true"#),
    );
    assert_eq!(dots(&figure).len(), 2);
    let vertical = paths(&figure)
        .into_iter()
        .any(|points| points.iter().all(|[x, _]| (x + 2.0).abs() < 1e-12));
    assert!(vertical);
}

#[test]
fn 倍数の点を並べられる() {
    // y^2 = x^3 + 17，P = (-2, 3)．2P = (8, -23)，3P = P + 2P．
    let figure = figure_of(
        1000.0,
        &curve(
            "0",
            "17",
            r#", "p": -2, "multiples": 3, "construction": false"#,
        ),
    );
    assert!(has_dot(&figure, [-2.0, 3.0]));
    assert!(has_dot(&figure, [8.0, -23.0]));
    // 3P：傾き(-23 - 3) / (8 + 2) = -2.6，x = 6.76 + 2 - 8 = 0.76，y = -2.6 (-2 - 0.76) - 3 = 4.176．
    let three = label_at(&figure, "$3P$");
    assert!(
        (three[0] - 0.76).abs() < 1e-9 && (three[1] - 4.176).abs() < 1e-9,
        "{three:?}"
    );
    assert_eq!(dots(&figure).len(), 3);
}

#[test]
fn 名前を消せる() {
    let figure = figure_of(
        10.0,
        &curve("0", "17", r#", "p": -2, "q": -1, "labels": false"#),
    );
    assert!(labels(&figure).is_empty());
}

#[test]
fn 媒介変数を使える() {
    let figure = figure_of(
        10.0,
        &format!(
            r#"{{ "id": "t", "type": "parameter", "value": -2 }}, {}"#,
            curve("0", "\"17\"", r#", "p": "t", "q": "t + 1""#)
        ),
    );
    assert!(has_dot(&figure, [4.0, -9.0]));
}

#[test]
fn 曲線上にない点は誤りになる() {
    // x = -3 では x^3 + 17 < 0．
    let scene = plane_scene(10.0, &curve("0", "17", r#", "p": -3"#));
    let error = parse_scene(&scene).expect_err("誤りになる");
    assert!(error.to_string().contains('p'), "{error}");
}

#[test]
fn 空間の図では使えない() {
    let error = parse_scene(&format!(
        r#"{{ "version": "0.1.0", "description": "a",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{}] }}"#,
        curve("0", "1", "")
    ))
    .expect_err("誤りになる");
    assert!(error.to_string().contains("elliptic_curve"), "{error}");
}
