//! 陰線処理の書式を確かめる．オブジェクトごとに陰線処理をしない(`hidden: "visible"`)ことと，
//! 奥を通る線を交わる所で少し切る(`crossing_gap`)こと．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Hidden, Length, LengthUnit, Line, Object};
use figure::{parse_scene, render};

fn figure_of(json: &str) -> Figure {
    render(&parse_scene(json).expect("シーンを読める")).expect("描画できる")
}

/// 方位角0，仰角0の空間の図．画面の右がy，上がz，カメラへの向きがxになる．1単位は1cmである．
fn front_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 0, "elevation": 0, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn paths(figure: &Figure) -> Vec<&Path> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Path(path) => Some(path),
            Item::Label(_) | Item::Dot(_) | Item::Fill(_) => None,
        })
        .collect()
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// x = `depth`の平面にある，画面で横向きの線分(yが-1から1)．
fn horizontal(id: &str, depth: f64, style: &str) -> String {
    format!(
        r#"{{ "id": "{id}", "type": "curve", "var": "t",
              "expr": ["{depth}", "t", "0"], "domain": [-1, 1]{style} }}"#
    )
}

/// x = `depth`の平面にある，画面で縦向きの線分(zが-1から1)．
fn vertical(id: &str, depth: f64, style: &str) -> String {
    format!(
        r#"{{ "id": "{id}", "type": "curve", "var": "t",
              "expr": ["{depth}", "0", "t"], "domain": [-1, 1]{style} }}"#
    )
}

const BALL: &str = r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2 }"#;

// ---- 陰線処理をしない ----

#[test]
fn hiddenのvisibleを読み_書き出すと元に戻る() {
    let scene = parse_scene(&front_scene(&horizontal(
        "a",
        0.0,
        r#", "style": { "hidden": "visible" }"#,
    )))
    .expect("読める");
    let Object::Curve(curve) = &scene.objects[0] else {
        panic!("曲線である");
    };
    assert_eq!(curve.style.hidden, Hidden::Visible);
    let written = serde_json::to_value(&scene).expect("書き出せる");
    assert_eq!(written["objects"][0]["style"]["hidden"], "visible");
}

#[test]
fn hiddenがvisibleの線は_球に隠れても見える線のまま1本で描く() {
    let equator = r#"{ "id": "equator", "type": "curve", "var": "t",
        "expr": ["2*cos(t)", "2*sin(t)", "0"], "domain": [0, "2*pi"],
        "style": { "hidden": "visible", "line": "dashed" } }"#;
    let figure = figure_of(&front_scene(&format!("{equator}, {BALL}")));
    let all = paths(&figure);
    // 赤道の1本と，球の輪郭の1本．
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].stroke.line, Line::Dashed);
}

#[test]
fn 陰線処理はオブジェクトごとに選び_既定では行う() {
    let figure = figure_of(&front_scene(&format!(
        "{}, {}, {BALL}",
        horizontal("seen", 0.0, r#", "style": { "hidden": "visible" }"#),
        vertical("tested", 0.0, ""),
    )));
    let all = paths(&figure);
    let lines: Vec<Line> = all.iter().map(|path| path.stroke.line).collect();
    // 1本目は処理しないので実線のまま．2本目は球に隠れて点線になる．
    assert_eq!(lines, [Line::Solid, Line::Dotted, Line::Solid]);
}

#[test]
fn hiddenがvisibleの点は_球に隠れても印を描く() {
    let point = |style: &str| {
        figure_of(&front_scene(&format!(
            r#"{{ "id": "P", "type": "point", "at": [-1, 0, 0], "dot": true{style} }}, {BALL}"#
        )))
    };
    let dots = |figure: &Figure| {
        figure
            .items
            .iter()
            .filter(|item| matches!(item, Item::Dot(_)))
            .count()
    };
    assert_eq!(dots(&point("")), 0);
    assert_eq!(dots(&point(r#", "style": { "hidden": "visible" }"#)), 1);
}

// ---- 交わる所で奥の線を切る ----

#[test]
fn crossing_gapを読み_書き出すと元に戻る() {
    let scene = parse_scene(&front_scene(&horizontal(
        "a",
        0.0,
        r#", "style": { "crossing_gap": "2mm" }"#,
    )))
    .expect("読める");
    let Object::Curve(curve) = &scene.objects[0] else {
        panic!("曲線である");
    };
    assert_eq!(
        curve.style.crossing_gap,
        Some(Length {
            value: 2.0,
            unit: LengthUnit::Mm
        })
    );
    let written = serde_json::to_value(&scene).expect("書き出せる");
    assert_eq!(written["objects"][0]["style"]["crossing_gap"], "2mm");
}

#[test]
fn 奥を通る線は_手前の線と交わる所で_crossing_gapの長さだけ切れる() {
    let figure = figure_of(&front_scene(&format!(
        "{}, {}",
        horizontal("back", 0.0, r#", "style": { "crossing_gap": "2mm" }"#),
        vertical("front", 1.0, ""),
    )));
    let all = paths(&figure);
    assert_eq!(all.len(), 3, "奥の線が2本に分かれ，手前の線は1本のまま");
    let (left, right) = (all[0], all[1]);
    assert!(distance(left.points[0], [-1.0, 0.0]) < 1e-9);
    assert!(distance(*left.points.last().unwrap(), [-0.1, 0.0]) < 1e-6);
    assert!(distance(right.points[0], [0.1, 0.0]) < 1e-6);
    assert!(distance(*right.points.last().unwrap(), [1.0, 0.0]) < 1e-9);
    assert_eq!(left.stroke, right.stroke);
}

#[test]
fn 手前を通る線は_crossing_gapがあっても切れない() {
    let figure = figure_of(&front_scene(&format!(
        "{}, {}",
        horizontal("front", 1.0, r#", "style": { "crossing_gap": "2mm" }"#),
        vertical("back", 0.0, ""),
    )));
    assert_eq!(paths(&figure).len(), 2);
}

#[test]
fn 空間で交わる線どうしは_crossing_gapがあっても切れない() {
    let figure = figure_of(&front_scene(&format!(
        "{}, {}",
        horizontal("a", 0.0, r#", "style": { "crossing_gap": "2mm" }"#),
        vertical("b", 0.0, r#", "style": { "crossing_gap": "2mm" }"#),
    )));
    assert_eq!(paths(&figure).len(), 2);
}

#[test]
fn 自分自身の手前を通る曲線も_奥の側が切れる() {
    // t = 0(奥，x = 0)とt = pi(手前，x = pi)で，画面の原点を通る．
    let curve = r#"{ "id": "loop", "type": "curve", "var": "t",
        "expr": ["t", "sin(t)", "sin(2*t)"], "domain": [-1, 4],
        "style": { "crossing_gap": "2mm" } }"#;
    let figure = figure_of(&front_scene(curve));
    let all = paths(&figure);
    assert_eq!(all.len(), 2);
    let end = *all[0].points.last().unwrap();
    let start = all[1].points[0];
    // 線に沿って前後に1mmずつ切るので，原点からの距離はほぼ1mmである．交点は，曲線を近似した
    // 折れ線どうしの交点なので，原点から少しずれる．
    for point in [end, start] {
        let gap = distance(point, [0.0, 0.0]);
        assert!((gap - 0.1).abs() < 0.01, "{gap}");
    }
    // 切れ目は，t = 0の側にある(画面で右上がりの向き)．
    assert!(end[0] < 0.0 && end[1] < 0.0);
    assert!(start[0] > 0.0 && start[1] > 0.0);
}

#[test]
fn 切れた線の矢じりは_終わりの部分に残る() {
    let figure = figure_of(&front_scene(
        r#"{ "id": "A", "type": "point", "at": [0, -1, 0] },
           { "id": "B", "type": "point", "at": [0, 1, 0] },
           { "id": "v", "type": "vector", "from": "A", "to": "B",
             "style": { "crossing_gap": "2mm" } },
           { "id": "front", "type": "curve", "var": "t",
             "expr": ["1", "0", "t"], "domain": [-1, 1] }"#,
    ));
    let all = paths(&figure);
    assert_eq!(all.len(), 3);
    assert!(all[0].arrow.is_none());
    assert!(all[1].arrow.is_some());
}

#[test]
fn 平面の図では_crossing_gapは何もしない() {
    let figure = figure_of(
        r#"{ "version": "0.1.0", "description": "試験の図",
             "view": { "x": [-2, 2], "y": [-2, 2], "unit": { "x": "1cm", "y": "1cm" } },
             "objects": [
               { "id": "a", "type": "curve", "var": "t", "expr": ["t", "0"], "domain": [-1, 1],
                 "style": { "crossing_gap": "2mm" } },
               { "id": "b", "type": "curve", "var": "t", "expr": ["0", "t"], "domain": [-1, 1] }
             ] }"#,
    );
    assert_eq!(paths(&figure).len(), 2);
}
