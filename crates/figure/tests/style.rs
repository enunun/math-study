//! 線のスタイル(`style`の`line`，`color`，`width`)を確かめる．読み込み，検査，中間表現，TikZの出力を見る．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::arrow::Stealth;
use figure::figure::{Figure, Item, Path};
use figure::scene::{CM_PER_PT, Color, Length, LengthUnit, Line, Object};
use figure::tikz::export_tikz;
use figure::{Error, ErrorKind, parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-4, 4], "y": [-2, 2], "unit": {{ "x": "1cm", "y": "1cm" }} }},
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

fn figure_of(json: &str) -> Figure {
    render(&parse_scene(json).expect("読める")).expect("描画できる")
}

fn error_of(json: &str) -> Error {
    parse_scene(json).expect_err("誤りになる")
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

const GRAPH: &str = r#""type": "graph", "var": "x", "expr": "x", "domain": [-1, 1]"#;

#[test]
fn styleは_線の種類と色と太さを持ち_省くと何も指定しない() {
    let scene = parse_scene(&plane_scene(&format!(
        r#"{{ "id": "a", {GRAPH}, "style": {{ "line": "dashed", "color": "red", "width": "1.2pt" }} }},
           {{ "id": "b", {GRAPH} }}"#
    )))
    .expect("読める");
    let (Object::Graph(a), Object::Graph(b)) = (&scene.objects[0], &scene.objects[1]) else {
        panic!("グラフである");
    };
    assert_eq!(a.style.line, Some(Line::Dashed));
    assert_eq!(a.style.color, Some(Color::Red));
    assert_eq!(
        a.style.width,
        Some(Length {
            value: 1.2,
            unit: LengthUnit::Pt
        })
    );
    assert_eq!(
        (b.style.line, b.style.color, b.style.width),
        (None, None, None)
    );
}

#[test]
fn 色は_決まった名前から選ぶ() {
    for (name, color) in [
        ("gray", Color::Gray),
        ("red", Color::Red),
        ("blue", Color::Blue),
        ("green", Color::Green),
        ("orange", Color::Orange),
        ("purple", Color::Purple),
    ] {
        let scene = parse_scene(&plane_scene(&format!(
            r#"{{ "id": "a", {GRAPH}, "style": {{ "color": "{name}" }} }}"#
        )))
        .expect("読める");
        let Object::Graph(graph) = &scene.objects[0] else {
            panic!("グラフである");
        };
        assert_eq!(graph.style.color, Some(color), "{name}");
    }
    let error = error_of(&plane_scene(&format!(
        r#"{{ "id": "a", {GRAPH}, "style": {{ "color": "pink" }} }}"#
    )));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert_eq!(error.object.as_deref(), Some("a"));
    assert!(error.to_string().contains("pink"), "{error}");
}

#[test]
fn 線の太さは_0より大きく10pt以下でなければならない() {
    for width in ["0pt", "-1pt", "11pt", "5mm", "abc"] {
        let error = error_of(&plane_scene(&format!(
            r#"{{ "id": "a", {GRAPH}, "style": {{ "width": "{width}" }} }}"#
        )));
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{width}: {error}"
        );
        assert_eq!(error.object.as_deref(), Some("a"), "{width}");
    }
    let ok = parse_scene(&plane_scene(&format!(
        r#"{{ "id": "a", {GRAPH}, "style": {{ "width": "10pt" }} }}"#
    )));
    assert!(ok.is_ok(), "{ok:?}");
}

#[test]
fn styleを書き出して読み直すと同じになり_省いた項目は書き出さない() {
    let scene = parse_scene(&plane_scene(&format!(
        r#"{{ "id": "a", {GRAPH}, "style": {{ "color": "blue" }} }}"#
    )))
    .expect("読める");
    let written = serde_json::to_string(&scene).expect("書き出せる");
    assert!(written.contains(r#""color":"blue""#), "{written}");
    assert!(!written.contains("width"), "{written}");
    assert!(!written.contains(r#""line""#), "{written}");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn グラフの色と太さは_中間表現の線に入り_省くと既定である() {
    let figure = figure_of(&plane_scene(&format!(
        r#"{{ "id": "a", {GRAPH}, "style": {{ "color": "red", "width": "1.2pt", "line": "dotted" }} }},
           {{ "id": "b", {GRAPH} }}"#
    )));
    let lines = paths(&figure);
    assert_eq!(lines[0].stroke.color, Some(Color::Red));
    assert!(close(lines[0].stroke.width, 1.2));
    assert_eq!(lines[0].stroke.line, Line::Dotted);
    // 省くと，実線で，0.8pt，色の指定なしである．
    assert_eq!(lines[1].stroke.color, None);
    assert!(close(lines[1].stroke.width, 0.8));
    assert_eq!(lines[1].stroke.line, Line::Solid);
}

#[test]
fn 太さは_ptのほか_cmとmmでも書け_ptに直して持つ() {
    let figure = figure_of(&plane_scene(&format!(
        r#"{{ "id": "a", {GRAPH}, "style": {{ "width": "0.5mm" }} }}"#
    )));
    let expected = 0.05 / CM_PER_PT;
    assert!(close(paths(&figure)[0].stroke.width, expected));
}

#[test]
fn 曲線の色と太さも_中間表現に入る() {
    let figure = figure_of(&plane_scene(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["cos(t)", "sin(t)"],
             "domain": [0, "2*pi"], "style": { "color": "green", "width": "1pt" } }"#,
    ));
    let curve = paths(&figure)[0];
    assert_eq!(curve.stroke.color, Some(Color::Green));
    assert!(close(curve.stroke.width, 1.0));
}

#[test]
fn 軸の色と太さは_線と矢じりと目盛に及び_矢じりは太さに合わせて大きくなる() {
    let figure = figure_of(&plane_scene(
        r#"{ "id": "x", "type": "axis", "direction": "x",
             "style": { "color": "blue", "width": "1.2pt" },
             "ticks": [ { "at": 1, "label": "1" } ] }"#,
    ));
    let lines = paths(&figure);
    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert_eq!(line.stroke.color, Some(Color::Blue));
        assert!(close(line.stroke.width, 1.2));
    }
    // 矢じりの大きさは，線幅で決まる(TikZの規則)．
    let arrow = lines[0].arrow.as_ref().expect("矢じりがある");
    let stealth = Stealth::new(1.2);
    assert!(close(arrow.line_width, stealth.line_width));
    let length = arrow.polygon[0][0] - arrow.polygon[1][0];
    let plain = figure_of(&plane_scene(
        r#"{ "id": "x", "type": "axis", "direction": "x" }"#,
    ));
    let plain_arrow = paths(&plain)[0].arrow.clone().expect("矢じりがある");
    let plain_length = plain_arrow.polygon[0][0] - plain_arrow.polygon[1][0];
    assert!(length > plain_length * 1.2, "{length} {plain_length}");
}

#[test]
fn 軸のstyleの線の種類は_軸の線に使う() {
    let figure = figure_of(&plane_scene(
        r#"{ "id": "x", "type": "axis", "direction": "x", "style": { "line": "dashed" } }"#,
    ));
    assert_eq!(paths(&figure)[0].stroke.line, Line::Dashed);
}

#[test]
fn 格子の線の種類の既定は点線で_色と太さも選べる() {
    let default = figure_of(&plane_scene(
        r#"{ "id": "g", "type": "grid", "x_step": 2 }"#,
    ));
    let stroke = paths(&default)[0].stroke;
    assert_eq!((stroke.line, stroke.color), (Line::Dotted, None));
    assert!(close(stroke.width, 0.3));
    let styled = figure_of(&plane_scene(
        r#"{ "id": "g", "type": "grid", "x_step": 2,
             "style": { "line": "solid", "color": "gray", "width": "0.5pt" } }"#,
    ));
    let stroke = paths(&styled)[0].stroke;
    assert_eq!(
        (stroke.line, stroke.color),
        (Line::Solid, Some(Color::Gray))
    );
    assert!(close(stroke.width, 0.5));
}

#[test]
fn 空間の図の球と軸と曲線にも_色と太さが効く() {
    let figure = figure_of(&space_scene(
        r#"{ "id": "x", "type": "axis", "direction": "x", "range": [-1, 1],
             "style": { "color": "orange", "width": "1pt" } },
           { "id": "c", "type": "curve", "var": "t", "expr": ["cos(t)", "sin(t)", "0"],
             "domain": [0, "2*pi"], "style": { "color": "purple" } },
           { "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2,
             "style": { "color": "red", "width": "1.5pt" } }"#,
    ));
    let lines = paths(&figure);
    let axis = lines[0];
    assert_eq!(axis.stroke.color, Some(Color::Orange));
    assert!(close(axis.stroke.width, 1.0));
    let curve = lines[1];
    assert_eq!(curve.stroke.color, Some(Color::Purple));
    let ball = lines.last().unwrap();
    assert_eq!(ball.stroke.color, Some(Color::Red));
    assert!(close(ball.stroke.width, 1.5));
}

#[test]
fn 隠れた部分の線も_色と太さを保つ() {
    let figure = figure_of(&space_scene(
        r#"{ "id": "x", "type": "axis", "direction": "x", "range": [-5, 5],
             "style": { "color": "blue", "width": "1pt" } },
           { "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2 }"#,
    ));
    let pieces = paths(&figure);
    assert!(pieces.iter().any(|piece| piece.stroke.line == Line::Dotted));
    for piece in &pieces[..pieces.len() - 1] {
        assert_eq!(piece.stroke.color, Some(Color::Blue));
        assert!(close(piece.stroke.width, 1.0));
    }
}

#[test]
fn tikzの線には_太さの次に色の名前が付く() {
    let json = plane_scene(&format!(
        r#"{{ "id": "a", {GRAPH}, "style": {{ "color": "red", "width": "1.2pt", "line": "dotted" }} }},
           {{ "id": "b", {GRAPH}, "style": {{ "color": "green" }} }},
           {{ "id": "c", {GRAPH}, "style": {{ "color": "purple" }} }}"#
    ));
    let output = export_tikz(&json).expect("出力できる");
    assert!(
        output.contains("\\draw[line width=1.2pt, red, dotted]"),
        "{output}"
    );
    assert!(
        output.contains("\\draw[line width=0.8pt, green!50!black]"),
        "{output}"
    );
    assert!(
        output.contains("\\draw[line width=0.8pt, violet]"),
        "{output}"
    );
}

#[test]
fn tikzの矢じりは_線の色を引き継ぐ() {
    let json = plane_scene(
        r#"{ "id": "x", "type": "axis", "direction": "x", "style": { "color": "blue" } }"#,
    );
    let output = export_tikz(&json).expect("出力できる");
    assert!(
        output.contains("\\draw[line width=0.6pt, blue, -{Stealth}]"),
        "{output}"
    );
}
