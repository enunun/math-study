//! 流線(`field_line`)を確かめる．起点から，場の向き(単位ベクトル)に沿って積分した曲線である．場は，
//! ベクトル場(`vector_field`)と同じく，ベクトルの式か成分の式の並びで書く．場が有限でない所や0の所，
//! 向きが急に反転した所(点電荷を飛び越えたとき)で止まる．

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
use figure::{ErrorKind, parse_scene, render};

/// 見える範囲[-3, 3]^2，単位1cmの平面の図．
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

fn line(field: &str, seeds: &str, extra: &str) -> String {
    format!(r#"{{ "id": "L", "type": "field_line", "field": {field}, "seeds": {seeds} {extra} }}"#)
}

#[test]
fn 流線を読み_書き出して読み直すと同じになる() {
    let scene = parse_scene(&plane_scene(&line(r#""r""#, "[[1, 0]]", ""))).expect("読める");
    let Object::FieldLine(found) = &scene.objects[0] else {
        panic!("流線である");
    };
    assert_eq!(found.var, "r");
    assert_eq!(scene.objects[0].type_name(), "field_line");
    let written = serde_json::to_string(&scene).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 一様な場の流線は_起点を通る直線で_両側に長さだけ伸びる() {
    let figure = figure_of(&line("[1, 0]", "[[0, 1]]", r#", "length": 2"#));
    let lines = paths(&figure);
    assert_eq!(lines.len(), 1);
    let xs: Vec<f64> = lines[0].points.iter().map(|p| p[0]).collect();
    let low = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let high = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (low + 2.0).abs() < 1e-6 && (high - 2.0).abs() < 1e-6,
        "{low} {high}"
    );
    assert!(lines[0].points.iter().all(|p| (p[1] - 1.0).abs() < 1e-9));
}

#[test]
fn 回転の場の流線は_円になる() {
    // 前向きに2πだけ進むと，半径1の円を1周する．
    let figure = figure_of(&line(
        r#"["-r_y", "r_x"]"#,
        "[[1, 0]]",
        r#", "length": "2*pi", "direction": "forward""#,
    ));
    let points = &paths(&figure)[0].points;
    assert!(points.len() > 32, "滑らかに描くだけの点がある");
    for [x, y] in points {
        assert!(
            (x.hypot(*y) - 1.0).abs() < 1e-6,
            "({x}, {y})は半径1の円の上"
        );
    }
    let (first, last) = (points[0], points[points.len() - 1]);
    assert!(
        (first[0] - last[0]).hypot(first[1] - last[1]) < 1e-3,
        "1周して戻る"
    );
}

#[test]
fn 点電荷の電気力線は_電荷で止まり_外へは見える範囲の縁まで伸びる() {
    // 原点の正電荷．起点(0.5, 0)から，後ろ向きには原点へ向かい，原点の手前で止まる．前向きには，x軸に沿って
    // 見える範囲の縁(x = 3)まで伸びて切れる．
    let figure = figure_of(&format!(
        r#"{{ "id": "Q", "type": "point", "at": [0, 0] }},
           {}"#,
        line(
            r#""(r - Q) / norm(r - Q)^3""#,
            "[[0.5, 0]]",
            r#", "length": 10"#
        )
    ));
    let xs: Vec<f64> = paths(&figure)
        .iter()
        .flat_map(|path| path.points.iter().map(|p| p[0]))
        .collect();
    let low = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let high = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(low > -0.05 && low < 0.05, "電荷の近くで止まる：{low}");
    assert!((high - 3.0).abs() < 1e-9, "縁まで伸びる：{high}");
}

#[test]
fn 起点は点の座標の名前を使える() {
    let figure = figure_of(&format!(
        r#"{{ "id": "Q", "type": "point", "at": [1, 1] }},
           {}"#,
        line("[0, 1]", r#"[["Q_x", "Q_y"]]"#, r#", "length": 0.5"#)
    ));
    let points = &paths(&figure)[0].points;
    assert!(points.iter().all(|p| (p[0] - 1.0).abs() < 1e-9));
}

#[test]
fn 起点ごとに_流線を引く() {
    let figure = figure_of(&line(
        "[1, 0]",
        "[[0, -1], [0, 0], [0, 1]]",
        r#", "length": 1"#,
    ));
    assert_eq!(paths(&figure).len(), 3);
}

#[test]
fn 起点と長さと刻みの誤り() {
    let error_of = |objects: &str| parse_scene(&plane_scene(objects)).expect_err("誤りになる");
    let error = error_of(&line("[1, 0]", "[]", ""));
    assert!(error.to_string().contains("seeds"), "{error}");
    let error = error_of(&line("[1, 0]", "[[0, 0]]", r#", "length": 0"#));
    assert!(error.to_string().contains("length"), "{error}");
    let error = error_of(&line("[1, 0]", "[[0, 0]]", r#", "step": -1"#));
    assert!(error.to_string().contains("step"), "{error}");
    let error = error_of(&line(
        "[1, 0]",
        "[[0, 0]]",
        r#", "length": 1000, "step": 0.001"#,
    ));
    assert!(error.to_string().contains("点"), "{error}");
    let error = error_of(&line(r#""norm(r)""#, "[[1, 0]]", ""));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    let error = error_of(&line("[1, 0]", "[[0, 0, 0]]", ""));
    assert!(error.to_string().contains("seeds"), "{error}");
}

#[test]
fn 空間の流線は_らせんを描ける() {
    // 真上から見る空間の図．場(-y, x, 0.2)の流線は，半径1のらせんで，真上から見ると円になる．
    let json = r#"{ "version": "0.1.0", "description": "試験の図",
             "view": { "azimuth": 90, "elevation": 90, "unit": "1cm" },
             "objects": [{ "id": "L", "type": "field_line", "field": ["-r_y", "r_x", 0.2],
                           "seeds": [[1, 0, 0]], "length": 6, "direction": "forward" }] }"#;
    let figure = render(&parse_scene(json).expect("読める")).expect("描ける");
    let lines = paths(&figure);
    assert!(!lines.is_empty());
    for [x, y] in lines.iter().flat_map(|path| &path.points) {
        assert!((x.hypot(*y) - 1.0).abs() < 1e-6, "({x}, {y})");
    }
}
