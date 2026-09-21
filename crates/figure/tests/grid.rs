//! 格子(`grid`)の読み込み，検査，描画を確かめる．格子は，見える範囲を，刻みごとの線で区切る．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Bound, Color, Line, Object};
use figure::{Error, ErrorKind, Scene, parse_scene, render};

fn plane_scene(unit: &str, objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-2, 3], "y": [-1, 2], "unit": {unit} }},
             "objects": [{objects}] }}"#
    )
}

const UNIT_1CM: &str = r#"{ "x": "1cm", "y": "1cm" }"#;

fn scene_of(objects: &str) -> Scene {
    parse_scene(&plane_scene(UNIT_1CM, objects)).expect("読める")
}

fn error_of(objects: &str) -> Error {
    parse_scene(&plane_scene(UNIT_1CM, objects)).expect_err("誤りになる")
}

fn figure_of(unit: &str, objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(unit, objects)).expect("読める")).expect("描画できる")
}

fn paths(figure: &Figure) -> Vec<&Path> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Path(path) => Some(path),
            Item::Label(_) => None,
        })
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn 格子は_x方向とy方向の刻みを数か式で持つ() {
    let scene = scene_of(
        r#"{ "id": "a", "type": "parameter", "value": 2 },
           { "id": "grid", "type": "grid", "x_step": 1, "y_step": "a/4" }"#,
    );
    let Object::Grid(grid) = &scene.objects[1] else {
        panic!("格子である");
    };
    assert_eq!(grid.x_step, Some(Bound::Number(1.0)));
    assert_eq!(grid.y_step, Some(Bound::Expression("a/4".to_owned())));
    // 線の種類を省くと，描くときに，点線になる．
    assert_eq!(grid.style.line, None);
    assert_eq!(scene.objects[1].type_name(), "grid");
}

#[test]
fn 格子は_書き出して読み直すと同じになる() {
    let scene =
        scene_of(r#"{ "id": "grid", "type": "grid", "x_step": 1, "style": { "line": "solid" } }"#);
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
    // 省いた刻みは，書き出さない．
    assert!(!written.contains("y_step"), "{written}");
}

#[test]
fn 刻みは_少なくとも片方が必要である() {
    let error = error_of(r#"{ "id": "grid", "type": "grid" }"#);
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert_eq!(error.object.as_deref(), Some("grid"));
    assert!(error.to_string().contains("x_step"), "{error}");
}

#[test]
fn 刻みは_正の有限の数でなければならない() {
    for step in ["0", "-1", r#""0""#, r#""-a""#] {
        let error = error_of(&format!(
            r#"{{ "id": "a", "type": "parameter", "value": 1 }},
               {{ "id": "grid", "type": "grid", "x_step": {step} }}"#
        ));
        assert_eq!(error.object.as_deref(), Some("grid"), "{step}: {error}");
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{step}: {error}"
        );
        assert!(error.to_string().contains("x_step"), "{step}: {error}");
    }
}

#[test]
fn 刻みの式の誤りは_項目の名前と位置を示す() {
    let error = error_of(r#"{ "id": "grid", "type": "grid", "y_step": "1/" }"#);
    let ErrorKind::Expression { field, index, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!((*field, *index), ("y_step", 0));
    assert_eq!(error.object.as_deref(), Some("grid"));
}

#[test]
fn 線が多すぎる格子は_誤りになる() {
    // 見える範囲(幅5)を，0.001刻みにすると，5000本になる．
    let error = error_of(r#"{ "id": "grid", "type": "grid", "x_step": 0.001 }"#);
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert!(error.to_string().contains("刻み"), "{error}");
}

#[test]
fn 空間の図では_格子は使えない() {
    let error = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "azimuth": 0, "elevation": 0, "unit": "1cm" },
             "objects": [ { "id": "grid", "type": "grid", "x_step": 1 } ] }"#,
    )
    .expect_err("誤りになる");
    assert!(error.to_string().contains("空間"), "{error}");
    assert_eq!(error.object.as_deref(), Some("grid"));
}

#[test]
fn 未知の項目は誤りになる() {
    let error = error_of(r#"{ "id": "grid", "type": "grid", "x_step": 1, "colour": "gray" }"#);
    assert!(error.to_string().contains("colour"), "{error}");
}

#[test]
fn x方向の刻みは_見える範囲の縦の線を引く() {
    // 見える範囲は，x: -2から3，y: -1から2．
    let figure = figure_of(UNIT_1CM, r#"{ "id": "grid", "type": "grid", "x_step": 1 }"#);
    let lines = paths(&figure);
    let xs: Vec<f64> = lines.iter().map(|line| line.points[0][0]).collect();
    assert_eq!(xs, [-2.0, -1.0, 0.0, 1.0, 2.0, 3.0]);
    for line in &lines {
        assert_eq!(line.points.len(), 2);
        assert!(close(line.points[0][0], line.points[1][0]));
        assert!(close(line.points[0][1], -1.0) && close(line.points[1][1], 2.0));
        assert_eq!(line.stroke.line, Line::Dotted);
        assert!(close(line.stroke.width, 0.3));
        assert!(line.arrow.is_none());
    }
}

#[test]
fn y方向の刻みは_見える範囲の横の線を引く() {
    let figure = figure_of(UNIT_1CM, r#"{ "id": "grid", "type": "grid", "y_step": 1 }"#);
    let lines = paths(&figure);
    let ys: Vec<f64> = lines.iter().map(|line| line.points[0][1]).collect();
    assert_eq!(ys, [-1.0, 0.0, 1.0, 2.0]);
    for line in &lines {
        assert!(close(line.points[0][0], -2.0) && close(line.points[1][0], 3.0));
    }
}

#[test]
fn 刻みは_原点から数えた倍数の位置に線を引く() {
    // x: -2から3を1.5刻みにすると，-1.5，0，1.5，3である．
    let figure = figure_of(
        UNIT_1CM,
        r#"{ "id": "grid", "type": "grid", "x_step": 1.5 }"#,
    );
    let xs: Vec<f64> = paths(&figure)
        .iter()
        .map(|line| line.points[0][0])
        .collect();
    assert_eq!(xs.len(), 4);
    for (x, expected) in xs.iter().zip([-1.5, 0.0, 1.5, 3.0]) {
        assert!(close(*x, expected), "{xs:?}");
    }
}

#[test]
fn 線の位置は_単位の実寸に従い_両方の刻みを続けて引く() {
    let figure = figure_of(
        r#"{ "x": "2cm", "y": "0.5cm" }"#,
        r#"{ "id": "grid", "type": "grid", "x_step": 1, "y_step": 1, "style": { "line": "solid" } }"#,
    );
    let lines = paths(&figure);
    // 縦の線6本の後に，横の線4本．
    assert_eq!(lines.len(), 10);
    assert!(close(lines[1].points[0][0], -2.0));
    assert!(close(lines[0].points[0][1], -0.5) && close(lines[0].points[1][1], 1.0));
    assert!(close(lines[6].points[0][1], -0.5));
    assert!(close(lines[6].points[0][0], -4.0) && close(lines[6].points[1][0], 6.0));
    assert_eq!(lines[0].stroke.line, Line::Solid);
}

#[test]
fn 刻みの式は_媒介変数を使える() {
    let figure = figure_of(
        UNIT_1CM,
        r#"{ "id": "s", "type": "parameter", "value": 0.5 },
           { "id": "grid", "type": "grid", "y_step": "s*2" }"#,
    );
    assert_eq!(paths(&figure).len(), 4);
}

#[test]
fn 格子は_描く範囲を変えず_軸より先に置けば軸が上に描かれる() {
    let with = figure_of(
        UNIT_1CM,
        r#"{ "id": "grid", "type": "grid", "x_step": 1, "y_step": 1 },
           { "id": "x", "type": "axis", "direction": "x" }"#,
    );
    let without = figure_of(
        UNIT_1CM,
        r#"{ "id": "x", "type": "axis", "direction": "x" }"#,
    );
    assert_eq!(with.bounds, without.bounds);
    let last = with.items.last().expect("要素がある");
    assert_eq!(Some(last), without.items.first());
}

const PARABOLA_ON_GRID: &str = include_str!("../../../site/src/figures/parabola-on-grid.json");

#[test]
fn 格子の上の放物線の図は_格子14本の点線と_軸と2本の曲線からできる() {
    let figure = render(&parse_scene(PARABOLA_ON_GRID).expect("読める")).expect("描画できる");
    let lines = paths(&figure);
    // x: -3から3の縦7本，y: -1から5の横7本，軸2本，放物線と接線．
    assert_eq!(lines.len(), 7 + 7 + 2 + 2);
    let dotted = lines
        .iter()
        .filter(|line| line.stroke.line == Line::Dotted)
        .count();
    assert_eq!(dotted, 14);
    assert!(
        lines[..14]
            .iter()
            .all(|line| line.stroke.color == Some(Color::Gray))
    );
    // 放物線は，範囲(y = 5)で切れている．
    let parabola = lines[16];
    assert!(parabola.points.iter().all(|point| point[1] <= 5.0));
    assert!(close(parabola.points[0][1], 5.0));
    assert_eq!(parabola.stroke.color, Some(Color::Blue));
    assert!(close(parabola.stroke.width, 1.2));
    let tangent = lines[17];
    assert_eq!(
        (tangent.stroke.line, tangent.stroke.color),
        (Line::Dashed, Some(Color::Red))
    );
}
