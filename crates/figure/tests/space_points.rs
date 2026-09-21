//! 空間の図の，点(`point`)，ベクトル(`vector`)，線分(`segment`)を確かめる．座標は3個で，球や曲面に隠れる部分は
//! 隠れた部分の線で描く．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, LabelItem, Path};
use figure::scene::{Anchor, Arrow, Line};
use figure::{Error, ErrorKind, parse_scene, render};

fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-5, 5], "y": [-5, 5], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn figure_of(objects: &str) -> Figure {
    render(&parse_scene(&space_scene(objects)).expect("読める")).expect("描画できる")
}

fn error_of(json: &str) -> Error {
    parse_scene(json).expect_err("誤りになる")
}

fn project(p: [f64; 3]) -> [f64; 2] {
    let (a, e) = (60.0_f64.to_radians(), 20.0_f64.to_radians());
    let right = [-a.sin(), a.cos(), 0.0];
    let up = [-e.sin() * a.cos(), -e.sin() * a.sin(), e.cos()];
    let dot = |u: [f64; 3]| u[0] * p[0] + u[1] * p[1] + u[2] * p[2];
    [dot(right), dot(up)]
}

fn close2(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
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

fn labels(figure: &Figure) -> Vec<&LabelItem> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(label) => Some(label),
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

const BALL: &str = r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2 }"#;

fn point(id: &str, at: &str, extra: &str) -> String {
    format!(r#"{{ "id": "{id}", "type": "point", "at": {at} {extra} }}"#)
}

// ---- 読み込みと検査 ----

#[test]
fn 空間の点は_3個の座標か点の式で書く() {
    let ok = parse_scene(&space_scene(&format!(
        "{}, {}, {}",
        point("A", "[1, 0, 0]", ""),
        point("B", "[0, 1, 0]", ""),
        point("C", r#""A + B""#, "")
    )));
    assert!(ok.is_ok(), "{ok:?}");
    let two = error_of(&space_scene(&point("A", "[1, 0]", "")));
    assert!(matches!(two.kind, ErrorKind::Invalid(_)), "{two}");
    assert!(two.to_string().contains("3個"), "{two}");
    assert_eq!(two.object.as_deref(), Some("A"));
    let three = error_of(&plane_scene(&point("A", "[1, 0, 0]", "")));
    assert!(matches!(three.kind, ErrorKind::Invalid(_)), "{three}");
    assert!(three.to_string().contains("2個"), "{three}");
}

#[test]
fn 空間の点の座標は_zも名前で参照でき_平面の図ではzを参照できない() {
    let ok = parse_scene(&space_scene(&format!(
        r#"{}, {{ "id": "l", "type": "label", "at": ["A_x", "A_y", "A_z + 1"], "tex": "L" }}"#,
        point("A", "[1, 2, 3]", "")
    )));
    assert!(ok.is_ok(), "{ok:?}");
    let plane = error_of(&plane_scene(&format!(
        r#"{}, {{ "id": "l", "type": "label", "at": ["A_x", "A_z"], "tex": "L" }}"#,
        point("A", "[1, 2]", "")
    )));
    assert!(
        matches!(plane.kind, ErrorKind::Expression { .. }),
        "{plane}"
    );
}

#[test]
fn 空間の点の式は_3成分で計算する() {
    let figure = figure_of(&format!(
        "{}, {}, {}, {}",
        point("A", "[1, 0, 0]", ""),
        point("B", "[0, 2, 0]", ""),
        point("C", "[0, 0, 3]", ""),
        point("D", r#""A + B + C - 2 * A / 2""#, r#", "dot": true"#)
    ));
    // A + B + C - A = (0, 2, 3)．
    assert!(close2(dots(&figure)[0], project([0.0, 2.0, 3.0])));
}

#[test]
fn ベクトルと線分の端は_点のidで指し_存在しない点は誤りになる() {
    let error = error_of(&space_scene(&format!(
        r#"{}, {{ "id": "v", "type": "vector", "from": "A", "to": "Z" }}"#,
        point("A", "[0, 0, 0]", "")
    )));
    assert_eq!(error.kind, ErrorKind::UnknownPoint("Z".to_owned()));
}

// ---- 描画 ----

#[test]
fn 点の印と名前は_投影した位置に置く() {
    let figure = figure_of(&point(
        "P",
        "[1, 2, 3]",
        r#", "dot": true, "label": "P", "style": { "color": "red" }"#,
    ));
    let expected = project([1.0, 2.0, 3.0]);
    assert!(close2(dots(&figure)[0], expected));
    let name = labels(&figure)[0];
    assert!(close2(name.at, expected));
    assert_eq!((name.tex.as_str(), name.anchor), ("$P$", Anchor::SouthWest));
}

#[test]
fn 球の裏にある点の印は_描かず_名前は描く() {
    // 視線に沿って，球の裏(原点の奥)の点．カメラへの向きは，方位角60度，仰角20度である．
    let (a, e) = (60.0_f64.to_radians(), 20.0_f64.to_radians());
    let toward = [e.cos() * a.cos(), e.cos() * a.sin(), e.sin()];
    let behind = format!(
        "[{}, {}, {}]",
        -toward[0] * 3.0,
        -toward[1] * 3.0,
        -toward[2] * 3.0
    );
    let front = format!(
        "[{}, {}, {}]",
        toward[0] * 3.0,
        toward[1] * 3.0,
        toward[2] * 3.0
    );
    let figure = figure_of(&format!(
        "{}, {}, {BALL}",
        point("Far", &behind, r#", "dot": true, "label": "F""#),
        point("Near", &front, r#", "dot": true, "label": "N""#)
    ));
    let marks = dots(&figure);
    assert_eq!(marks.len(), 1);
    assert!(close2(
        marks[0],
        project([toward[0] * 3.0, toward[1] * 3.0, toward[2] * 3.0])
    ));
    assert_eq!(labels(&figure).len(), 2);
}

#[test]
fn 球を貫くベクトルは_球に隠れる部分が点線で_矢じりは見える先端に付く() {
    let figure = figure_of(&format!(
        "{}, {}, {}, {BALL}",
        point("A", "[-5, 0, 0]", ""),
        point("B", "[5, 0, 0]", ""),
        r#"{ "id": "v", "type": "vector", "from": "A", "to": "B" }"#
    ));
    let all = paths(&figure);
    // ベクトルは3つに分かれ(実線，点線，実線)，球の輪郭が1本．
    let lines: Vec<Line> = all.iter().map(|path| path.stroke.line).collect();
    assert_eq!(lines, [Line::Solid, Line::Dotted, Line::Solid, Line::Solid]);
    assert!(all[0].arrow.is_none() && all[1].arrow.is_none());
    let arrow = all[2].arrow.as_ref().expect("矢じりがある");
    assert_eq!(arrow.kind, Arrow::Stealth);
    let tip = arrow.polygon[0];
    assert!(
        (tip[0] - project([5.0, 0.0, 0.0])[0]).hypot(tip[1] - project([5.0, 0.0, 0.0])[1]) < 0.05
    );
}

#[test]
fn 空間の線分は_矢じりがなく_長さのない線分は描かない() {
    let figure = figure_of(&format!(
        "{}, {}, {}, {}",
        point("A", "[0, 0, 0]", ""),
        point("B", "[1, 1, 1]", ""),
        r#"{ "id": "s", "type": "segment", "from": "A", "to": "B", "style": { "line": "dashed" } }"#,
        r#"{ "id": "z", "type": "segment", "from": "A", "to": "A" }"#
    ));
    let all = paths(&figure);
    assert_eq!(all.len(), 1);
    assert!(all[0].arrow.is_none());
    assert_eq!(all[0].stroke.line, Line::Dashed);
    assert!(close2(all[0].points[0], project([0.0; 3])));
    assert!(close2(*all[0].points.last().unwrap(), project([1.0; 3])));
}

#[test]
fn 隠れた部分の線の種類は_線分でも選べる() {
    let make = |style: &str| {
        let segment = format!(
            r#"{{ "id": "s", "type": "segment", "from": "A", "to": "B", "style": {style} }}"#
        );
        figure_of(&format!(
            "{}, {}, {segment}, {BALL}",
            point("A", "[-5, 0, 0]", ""),
            point("B", "[5, 0, 0]", "")
        ))
    };
    let dashed = make(r#"{ "hidden": "dashed" }"#);
    assert!(
        paths(&dashed)
            .iter()
            .any(|path| path.stroke.line == Line::Dashed)
    );
    let none = make(r#"{ "hidden": "none" }"#);
    assert!(
        paths(&none)
            .iter()
            .all(|path| path.stroke.line == Line::Solid)
    );
}

#[test]
fn 曲面に隠れる点と線分も_同じように扱う() {
    let bowl = r#"{ "id": "bowl", "type": "surface", "vars": ["x", "y"],
        "expr": ["x", "y", "x^2 + y^2"], "domain": [[-1, 1], [-1, 1]] }"#;
    // お椀の真下の点は，上から見ると，お椀に隠れる．
    let figure = render(
        &parse_scene(&format!(
            r#"{{ "version": "0.1.0", "description": "a",
                 "view": {{ "azimuth": 60, "elevation": 85, "unit": "1cm" }},
                 "objects": [ {}, {}, {bowl} ] }}"#,
            point("Under", "[0, 0, -1]", r#", "dot": true"#),
            point("Above", "[0, 0, 3]", r#", "dot": true"#)
        ))
        .expect("読める"),
    )
    .expect("描画できる");
    assert_eq!(dots(&figure).len(), 1);
}

#[test]
fn 描く範囲は_点と線分と名前を含む() {
    let figure = figure_of(&format!(
        "{}, {}",
        point("A", "[0, 0, 0]", r#", "dot": true"#),
        point("B", "[4, 0, 0]", r#", "dot": true, "label": "B""#)
    ));
    let far = project([4.0, 0.0, 0.0]);
    assert!(figure.bounds.min[0] < far[0].min(0.0));
    assert!(figure.bounds.max[0] > far[0].max(0.0));
}
