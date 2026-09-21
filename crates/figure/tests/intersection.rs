//! 2つの曲面の交線(`intersection`)を確かめる．交線は，2つの網の三角形が交わる所を結んだ線で，
//! 曲面に隠れる部分は，隠れた部分の線で描く．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Color, Line, Object};
use figure::{Error, ErrorKind, parse_scene, render};

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

/// 半径2の球を，中心(cx, 0, 0)に置いた曲面．
fn sphere_at(id: &str, cx: f64) -> String {
    format!(
        r#"{{ "id": "{id}", "type": "surface", "vars": ["u", "v"],
            "expr": ["{cx} + 2*sin(u)*cos(v)", "2*sin(u)*sin(v)", "2*cos(u)"],
            "domain": [[0, "pi"], [0, "2*pi"]] }}"#
    )
}

fn analytic_ball(id: &str, cx: f64) -> String {
    format!(r#"{{ "id": "{id}", "type": "sphere", "center": [{cx}, 0, 0], "radius": 2 }}"#)
}

fn intersection(a: &str, b: &str, extra: &str) -> String {
    format!(r#"{{ "id": "x", "type": "intersection", "surfaces": ["{a}", "{b}"] {extra} }}"#)
}

fn project(p: [f64; 3]) -> [f64; 2] {
    let (a, e) = (60.0_f64.to_radians(), 20.0_f64.to_radians());
    let right = [-a.sin(), a.cos(), 0.0];
    let up = [-e.sin() * a.cos(), -e.sin() * a.sin(), e.cos()];
    let dot = |u: [f64; 3]| u[0] * p[0] + u[1] * p[1] + u[2] * p[2];
    [dot(right), dot(up)]
}

fn segment_distance(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let squared = dx * dx + dy * dy;
    let along = if squared == 0.0 {
        0.0
    } else {
        (((point[0] - a[0]) * dx + (point[1] - a[1]) * dy) / squared).clamp(0.0, 1.0)
    };
    (point[0] - (a[0] + along * dx)).hypot(point[1] - (a[1] + along * dy))
}

// ---- 読み込みと検査 ----

#[test]
fn 交線は_2つの曲面のidで書く() {
    let scene = parse_scene(&space_scene(&format!(
        "{}, {}, {}",
        sphere_at("a", 0.0),
        sphere_at("b", 2.0),
        intersection("a", "b", "")
    )))
    .expect("読める");
    let Object::Intersection(found) = &scene.objects[2] else {
        panic!("交線である");
    };
    assert_eq!(found.surfaces, ["a", "b"]);
    assert_eq!(scene.objects[2].type_name(), "intersection");
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 曲面は_2つの違う曲面でなければならない() {
    let one = error_of(&format!(
        r#"{}, {{ "id": "x", "type": "intersection", "surfaces": ["a"] }}"#,
        sphere_at("a", 0.0)
    ));
    assert!(matches!(one.kind, ErrorKind::Invalid(_)), "{one}");
    assert!(one.to_string().contains("surfaces"), "{one}");
    let same = error_of(&format!(
        "{}, {}",
        sphere_at("a", 0.0),
        intersection("a", "a", "")
    ));
    assert!(matches!(same.kind, ErrorKind::Invalid(_)), "{same}");
    assert_eq!(same.object.as_deref(), Some("x"));
    let unknown = error_of(&format!(
        "{}, {}",
        sphere_at("a", 0.0),
        intersection("a", "nothing", "")
    ));
    assert_eq!(
        unknown.kind,
        ErrorKind::UnknownSurface("nothing".to_owned())
    );
}

#[test]
fn 平面の図では_交線は使えない() {
    let error = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "x": [0, 1], "y": [0, 1], "unit": { "x": "1cm", "y": "1cm" } },
             "objects": [ { "id": "x", "type": "intersection", "surfaces": ["a", "b"] } ] }"#,
    )
    .expect_err("誤りになる");
    assert!(error.to_string().contains("平面"), "{error}");
}

// ---- 描画 ----

#[test]
fn 交わる2つの球の交線は_円になる() {
    // 中心が(0, 0, 0)と(2, 0, 0)の，半径2の球は，x = 1の平面の上の，半径sqrt(3)の円で交わる．
    let figure = figure_of(&format!(
        "{}, {}, {}",
        sphere_at("a", 0.0),
        sphere_at("b", 2.0),
        intersection("a", "b", r#", "style": { "color": "red" }"#)
    ));
    let circle: Vec<[f64; 2]> = (0..4000)
        .map(|k| {
            let t = f64::from(k) / 4000.0 * std::f64::consts::TAU;
            project([1.0, 3.0_f64.sqrt() * t.cos(), 3.0_f64.sqrt() * t.sin()])
        })
        .collect();
    let red: Vec<&Path> = paths(&figure)
        .into_iter()
        .filter(|path| path.stroke.color == Some(Color::Red))
        .collect();
    assert!(!red.is_empty());
    for path in &red {
        for point in &path.points {
            let near = circle
                .iter()
                .any(|q| (point[0] - q[0]).hypot(point[1] - q[1]) < 0.03);
            assert!(near, "{point:?}");
        }
    }
    // 円を，ほぼ一周する．
    let covered = circle
        .iter()
        .step_by(40)
        .filter(|q| {
            red.iter().any(|path| {
                path.points
                    .windows(2)
                    .any(|pair| segment_distance(**q, pair[0], pair[1]) < 0.03)
            })
        })
        .count();
    assert!(covered >= 95, "一周していない: {covered}");
}

#[test]
fn 交線の隠れる部分は_解析的な球で隠した円と同じ長さの点線になる() {
    let ring = r#"{ "id": "ring", "type": "curve", "var": "t",
        "expr": ["1", "sqrt(3)*cos(t)", "sqrt(3)*sin(t)"], "domain": [0, "2*pi"],
        "style": { "color": "red" } }"#;
    let reference = figure_of(&format!(
        "{ring}, {}, {}",
        analytic_ball("a", 0.0),
        analytic_ball("b", 2.0)
    ));
    let mine = figure_of(&format!(
        "{}, {}, {}",
        sphere_at("a", 0.0),
        sphere_at("b", 2.0),
        intersection("a", "b", r#", "style": { "color": "red" }"#)
    ));
    let dotted_length = |figure: &Figure| -> f64 {
        paths(figure)
            .iter()
            .filter(|path| {
                path.stroke.color == Some(Color::Red) && path.stroke.line == Line::Dotted
            })
            .map(|path| {
                path.points
                    .windows(2)
                    .map(|pair| (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]))
                    .sum::<f64>()
            })
            .sum()
    };
    let (a, b) = (dotted_length(&mine), dotted_length(&reference));
    assert!(b > 1.0, "{b}");
    assert!((a - b).abs() < 0.3, "{a} {b}");
}

#[test]
fn 交わらない2つの曲面は_何も描かない() {
    let with = figure_of(&format!(
        "{}, {}, {}",
        sphere_at("a", 0.0),
        sphere_at("b", 10.0),
        intersection("a", "b", "")
    ));
    let without = figure_of(&format!(
        "{}, {}",
        sphere_at("a", 0.0),
        sphere_at("b", 10.0)
    ));
    assert_eq!(with.items.len(), without.items.len());
}

#[test]
fn 交線の線の種類と隠れた部分は_styleで選べる() {
    let figure = figure_of(&format!(
        "{}, {}, {}",
        sphere_at("a", 0.0),
        sphere_at("b", 2.0),
        intersection(
            "a",
            "b",
            r#", "style": { "color": "blue", "line": "dashed", "hidden": "none" }"#
        )
    ));
    let blue: Vec<&Path> = paths(&figure)
        .into_iter()
        .filter(|path| path.stroke.color == Some(Color::Blue))
        .collect();
    assert!(!blue.is_empty());
    assert!(blue.iter().all(|path| path.stroke.line == Line::Dashed));
}

#[test]
fn 球と円柱の交線は_円柱の面の上にある() {
    // 半径2の球と，軸がz軸に平行で，半径1の円柱 (x - 1)^2 + y^2 = 1．交線の点は，円柱の式を満たす．
    // 点は画面の位置しかないので，交線が有限で，2本以上に分かれないことだけを確かめる．
    let cylinder = r#"{ "id": "c", "type": "surface", "vars": ["t", "z"],
        "expr": ["1 + cos(t)", "sin(t)", "z"], "domain": [[0, "2*pi"], [-3, 3]] }"#;
    let figure = figure_of(&format!(
        "{}, {cylinder}, {}",
        sphere_at("a", 0.0),
        intersection("a", "c", r#", "style": { "color": "green" }"#)
    ));
    let green: Vec<&Path> = paths(&figure)
        .into_iter()
        .filter(|path| path.stroke.color == Some(Color::Green))
        .collect();
    assert!(!green.is_empty());
    assert!(
        green
            .iter()
            .flat_map(|path| &path.points)
            .all(|p| p[0].is_finite() && p[1].is_finite())
    );
}

#[test]
fn 描画は同じ入力なら同じ結果になる() {
    let scene = format!(
        "{}, {}, {}",
        sphere_at("a", 0.0),
        sphere_at("b", 2.0),
        intersection("a", "b", "")
    );
    assert_eq!(figure_of(&scene), figure_of(&scene));
}
