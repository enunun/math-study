//! 点(`point`)，ベクトル(`vector`)，線分(`segment`)の読み込みと検査を確かめる．
//! 点の座標は，あとのオブジェクトの式から，`<id>_x`と`<id>_y`の名前で参照できる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::scene::{Anchor, Arrow, Bound, Object, Position};
use figure::{Error, ErrorKind, Scene, parse_scene};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-5, 5], "y": [-5, 5], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn scene_of(objects: &str) -> Scene {
    parse_scene(&plane_scene(objects)).expect("読める")
}

fn error_of(objects: &str) -> Error {
    parse_scene(&plane_scene(objects)).expect_err("誤りになる")
}

const A: &str = r#"{ "id": "A", "type": "point", "at": [0, 0] }"#;
const B: &str = r#"{ "id": "B", "type": "point", "at": [3, 1] }"#;

#[test]
fn 点は_座標を数か式で持ち_名前と点の印を任意で持つ() {
    let scene = scene_of(
        r#"{ "id": "A", "type": "point", "at": [1, "2*pi"], "label": "A", "anchor": "north east", "dot": true,
             "style": { "color": "red" } },
           { "id": "B", "type": "point", "at": [0, 0] }"#,
    );
    let Object::Point(a) = &scene.objects[0] else {
        panic!("点である");
    };
    assert_eq!(
        a.at,
        Position::Coordinates(vec![
            Bound::Number(1.0),
            Bound::Expression("2*pi".to_owned())
        ])
    );
    assert_eq!(a.label.as_deref(), Some("A"));
    assert_eq!(a.anchor, Some(Anchor::NorthEast));
    assert!(a.dot);
    let Object::Point(b) = &scene.objects[1] else {
        panic!("点である");
    };
    assert_eq!((b.label.clone(), b.anchor, b.dot), (None, None, false));
}

#[test]
fn 点を書き出して読み直すと同じになり_省いた項目は書き出さない() {
    let scene = scene_of(B);
    let written = serde_json::to_string(&scene).expect("書き出せる");
    assert!(
        !written.contains("dot") && !written.contains("label"),
        "{written}"
    );
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 点の座標は_2個でなければならない() {
    let error = error_of(r#"{ "id": "A", "type": "point", "at": [1] }"#);
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert_eq!(error.object.as_deref(), Some("A"));
}

#[test]
fn 点の名前は_空にできない() {
    let error = error_of(r#"{ "id": "A", "type": "point", "at": [0, 0], "label": "" }"#);
    assert_eq!(error.kind, ErrorKind::EmptyText("label"));
}

#[test]
fn ベクトルと線分は_始点と終点を点のidで持つ() {
    let scene = scene_of(&format!(
        r#"{A}, {B},
           {{ "id": "v", "type": "vector", "from": "A", "to": "B", "arrow": "none" }},
           {{ "id": "w", "type": "vector", "from": "B", "to": "A" }},
           {{ "id": "s", "type": "segment", "from": "A", "to": "B",
              "style": {{ "line": "dashed" }} }}"#
    ));
    let (Object::Vector(v), Object::Vector(w), Object::Segment(s)) =
        (&scene.objects[2], &scene.objects[3], &scene.objects[4])
    else {
        panic!("ベクトルと線分である");
    };
    assert_eq!(
        (v.from.as_str(), v.to.as_str(), v.arrow),
        ("A", "B", Arrow::None)
    );
    // 矢じりの既定は，stealthである．
    assert_eq!(w.arrow, Arrow::Stealth);
    assert_eq!((s.from.as_str(), s.to.as_str()), ("A", "B"));
    assert_eq!(scene.objects[2].type_name(), "vector");
    assert_eq!(scene.objects[4].type_name(), "segment");
}

#[test]
fn 存在しない点を指すと_誤りになり_名前と項目を示す() {
    let error = error_of(&format!(
        r#"{A}, {{ "id": "v", "type": "vector", "from": "A", "to": "Z" }}"#
    ));
    assert_eq!(error.kind, ErrorKind::UnknownPoint("Z".to_owned()));
    assert_eq!(error.object.as_deref(), Some("v"));
    assert!(error.to_string().contains('Z'), "{error}");
    assert_eq!(error.kind.code(), "unknown_point");
}

#[test]
fn 点でないオブジェクトを指すと_誤りになる() {
    let error = error_of(&format!(
        r#"{A}, {{ "id": "p", "type": "parameter", "value": 1 }},
           {{ "id": "v", "type": "segment", "from": "A", "to": "p" }}"#
    ));
    assert_eq!(error.kind, ErrorKind::UnknownPoint("p".to_owned()));
}

#[test]
fn 点は_あとで置いた点を指す線分にも使える() {
    // 線分は，点より前に置いても，点の座標が決まったあとで描く．
    let scene = parse_scene(&plane_scene(&format!(
        r#"{{ "id": "s", "type": "segment", "from": "A", "to": "B" }}, {A}, {B}"#
    )));
    assert!(scene.is_ok(), "{scene:?}");
}

#[test]
fn 点の座標は_あとのオブジェクトの式から_idのxとyで参照できる() {
    let ok = parse_scene(&plane_scene(&format!(
        r#"{A}, {B},
           {{ "id": "D", "type": "point", "at": ["B_x + A_x", "B_y - A_y"] }},
           {{ "id": "g", "type": "graph", "var": "x", "expr": "B_y * x", "domain": [0, "B_x"] }},
           {{ "id": "t", "type": "label", "at": ["(A_x + B_x) / 2", "(A_y + B_y) / 2"], "tex": "M" }}"#
    )));
    assert!(ok.is_ok(), "{ok:?}");
}

#[test]
fn 点の座標は_置く順であとの点しか参照できない() {
    // 自分の座標も，あとの点の座標も参照できない．
    let own = error_of(r#"{ "id": "A", "type": "point", "at": ["A_x", 0] }"#);
    assert!(matches!(own.kind, ErrorKind::Expression { .. }), "{own}");
    assert_eq!(own.object.as_deref(), Some("A"));
    let later = error_of(&format!(
        r#"{{ "id": "D", "type": "point", "at": ["B_x", 0] }}, {B}"#
    ));
    assert!(
        matches!(later.kind, ErrorKind::Expression { .. }),
        "{later}"
    );
    assert_eq!(later.object.as_deref(), Some("D"));
}

#[test]
fn 点の座標の式の誤りは_座標の番号と式の中の位置を示す() {
    let error = error_of(r#"{ "id": "A", "type": "point", "at": [0, "1 +"] }"#);
    let ErrorKind::Expression { field, index, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!((*field, *index), ("at", 1));
}

#[test]
fn 点の座標の名前が_媒介変数や変数の名前とぶつかると_誤りになる() {
    let error = error_of(&format!(
        r#"{{ "id": "A_x", "type": "parameter", "value": 1 }}, {A}"#
    ));
    assert_eq!(error.kind, ErrorKind::NameConflict("A_x".to_owned()));
    assert_eq!(error.object.as_deref(), Some("A"));
    let variable = error_of(&format!(
        r#"{A}, {{ "id": "g", "type": "graph", "var": "A_x", "expr": "A_x", "domain": [0, 1] }}"#
    ));
    assert_eq!(variable.kind, ErrorKind::NameConflict("A_x".to_owned()));
}

#[test]
fn ラベルの位置には_式が書け_点の座標を使える() {
    let scene = scene_of(&format!(
        r#"{A}, {B},
           {{ "id": "m", "type": "label", "at": ["(A_x + B_x)/2", 1], "tex": "M" }}"#
    ));
    let Object::Label(label) = &scene.objects[2] else {
        panic!("ラベルである");
    };
    assert_eq!(
        label.at,
        Position::Coordinates(vec![
            Bound::Expression("(A_x + B_x)/2".to_owned()),
            Bound::Number(1.0)
        ])
    );
}

#[test]
fn 空間の図では_点とベクトルと線分はまだ使えない() {
    for object in [
        r#"{ "id": "A", "type": "point", "at": [0, 0] }"#,
        r#"{ "id": "s", "type": "segment", "from": "A", "to": "B" }"#,
        r#"{ "id": "v", "type": "vector", "from": "A", "to": "B" }"#,
    ] {
        let error = parse_scene(&format!(
            r#"{{ "version": "0.1.0", "description": "a",
                 "view": {{ "azimuth": 0, "elevation": 0, "unit": "1cm" }},
                 "objects": [ {object} ] }}"#
        ))
        .expect_err("誤りになる");
        assert!(error.to_string().contains("空間"), "{error}");
    }
}

#[test]
fn 未知の項目は誤りになる() {
    let error = error_of(&format!(
        r#"{A}, {B}, {{ "id": "v", "type": "vector", "from": "A", "to": "B", "head": "big" }}"#
    ));
    assert!(error.to_string().contains("head"), "{error}");
}

// ---- 描画 ----

use figure::figure::{DotItem, Figure, Item, LabelItem, Path};
use figure::scene::{Color, Line};
use figure::{render, tikz::export_tikz};

/// 横が2cm，縦が1cmの単位の図を描く．
fn figure_of(objects: &str) -> Figure {
    let json = format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-5, 5], "y": [-5, 5], "unit": {{ "x": "2cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    );
    render(&parse_scene(&json).expect("読める")).expect("描画できる")
}

fn dots(figure: &Figure) -> Vec<&DotItem> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Dot(dot) => Some(dot),
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

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn 点の印は_単位の実寸に従った位置に置き_色を持つ() {
    let figure = figure_of(
        r#"{ "id": "P", "type": "point", "at": [1, 2], "dot": true, "style": { "color": "red" } },
           { "id": "Q", "type": "point", "at": [3, 3] }"#,
    );
    // 印を描くのは，dotがtrueの点だけである．
    let all = dots(&figure);
    assert_eq!(all.len(), 1);
    assert!(close(all[0].at[0], 2.0) && close(all[0].at[1], 2.0));
    assert!(close(all[0].radius, 2.0));
    assert_eq!(all[0].color, Some(Color::Red));
}

#[test]
fn 点の名前は_式として置き_既定は点の右上である() {
    let figure = figure_of(
        r#"{ "id": "A", "type": "point", "at": [1, 1], "label": "A" },
           { "id": "B", "type": "point", "at": [2, 0], "label": "\\vec{b}", "anchor": "north" }"#,
    );
    let all = labels(&figure);
    assert_eq!(all.len(), 2);
    assert_eq!(
        (all[0].tex.as_str(), all[0].anchor),
        ("$A$", Anchor::SouthWest)
    );
    assert!(close(all[0].at[0], 2.0) && close(all[0].at[1], 1.0));
    assert_eq!(
        (all[1].tex.as_str(), all[1].anchor),
        (r"$\vec{b}$", Anchor::North)
    );
}

#[test]
fn 印のあとに名前が続き_点は描く順に並ぶ() {
    let figure =
        figure_of(r#"{ "id": "A", "type": "point", "at": [0, 0], "dot": true, "label": "A" }"#);
    assert!(matches!(
        figure.items.as_slice(),
        [Item::Dot(_), Item::Label(_)]
    ));
}

#[test]
fn ベクトルは_始点から終点への線で_終点に矢じりを持つ() {
    let figure = figure_of(&format!(
        r#"{A}, {B},
           {{ "id": "v", "type": "vector", "from": "A", "to": "B" }}"#
    ));
    let all = paths(&figure);
    assert_eq!(all.len(), 1);
    // A(0, 0)からB(3, 1)へ．横の単位は2cmである．
    assert_eq!(all[0].points, [[0.0, 0.0], [6.0, 1.0]]);
    assert_eq!(all[0].stroke.line, Line::Solid);
    assert!(close(all[0].stroke.width, 0.8));
    let arrow = all[0].arrow.as_ref().expect("矢じりがある");
    assert_eq!(arrow.kind, Arrow::Stealth);
    // 矢じりの先端は，終点にほぼ一致し，線は，矢じりの手前で止まる．
    let tip = arrow.polygon[0];
    assert!((tip[0] - 6.0).hypot(tip[1] - 1.0) < 0.05, "{tip:?}");
    assert!((arrow.line_end[0] - 6.0).hypot(arrow.line_end[1] - 1.0) > 0.05);
}

#[test]
fn 矢じりなしのベクトルと線分は_矢じりのない線になる() {
    let figure = figure_of(&format!(
        r#"{A}, {B},
           {{ "id": "v", "type": "vector", "from": "A", "to": "B", "arrow": "none" }},
           {{ "id": "s", "type": "segment", "from": "B", "to": "A",
              "style": {{ "line": "dashed", "color": "blue", "width": "0.5pt" }} }}"#
    ));
    let all = paths(&figure);
    assert!(all[0].arrow.is_none());
    assert!(all[1].arrow.is_none());
    assert_eq!(all[1].points, [[6.0, 1.0], [0.0, 0.0]]);
    assert_eq!(all[1].stroke.line, Line::Dashed);
    assert_eq!(all[1].stroke.color, Some(Color::Blue));
    assert!(close(all[1].stroke.width, 0.5));
}

#[test]
fn ベクトルの色と太さは_矢じりにも及ぶ() {
    let figure = figure_of(&format!(
        r#"{A}, {B},
           {{ "id": "v", "type": "vector", "from": "A", "to": "B",
              "style": {{ "color": "red", "width": "1.2pt" }} }}"#
    ));
    let vector = paths(&figure)[0];
    assert_eq!(vector.stroke.color, Some(Color::Red));
    assert!(close(vector.stroke.width, 1.2));
    let thin = figure_of(&format!(
        r#"{A}, {B}, {{ "id": "v", "type": "vector", "from": "A", "to": "B" }}"#
    ));
    let big = vector.arrow.as_ref().unwrap().polygon;
    let small = paths(&thin)[0].arrow.as_ref().unwrap().polygon;
    let length = |polygon: [[f64; 2]; 4]| {
        (polygon[0][0] - polygon[1][0]).hypot(polygon[0][1] - polygon[1][1])
    };
    assert!(length(big) > length(small));
}

#[test]
fn 長さのないベクトルと線分は_何も描かない() {
    let figure = figure_of(&format!(
        r#"{A},
           {{ "id": "v", "type": "vector", "from": "A", "to": "A" }},
           {{ "id": "s", "type": "segment", "from": "A", "to": "A" }}"#
    ));
    assert!(figure.items.is_empty());
}

#[test]
fn 点を式で決めると_ベクトルの和の図ができる() {
    // D = B + C - A．
    let figure = figure_of(
        r#"{ "id": "A", "type": "point", "at": [0, 0] },
           { "id": "B", "type": "point", "at": [3, 1] },
           { "id": "C", "type": "point", "at": [1, 3] },
           { "id": "D", "type": "point", "at": ["B_x + C_x - A_x", "B_y + C_y - A_y"],
             "dot": true, "label": "D" },
           { "id": "sum", "type": "vector", "from": "A", "to": "D" },
           { "id": "m", "type": "label", "at": ["(A_x + D_x) / 2", "(A_y + D_y) / 2"], "tex": "M" }"#,
    );
    // Dは(4, 4)で，横の単位が2cmなので，(8, 4)cmである．
    let dot = dots(&figure)[0];
    assert!(close(dot.at[0], 8.0) && close(dot.at[1], 4.0));
    assert_eq!(paths(&figure)[0].points, [[0.0, 0.0], [8.0, 4.0]]);
    // ラベルは，AとDの中点(2, 2)，つまり(4, 2)cmにある．
    let middle = labels(&figure)[1];
    assert!(close(middle.at[0], 4.0) && close(middle.at[1], 2.0));
}

#[test]
fn 線分は_点より前に置いても描かれ_描く順はシーンの順である() {
    let figure = figure_of(&format!(
        r#"{{ "id": "s", "type": "segment", "from": "A", "to": "B" }}, {A}, {B}"#
    ));
    assert_eq!(paths(&figure).len(), 1);
}

#[test]
fn 点の印はtikzの塗りつぶした丸になり_線の色と矢じりは引き継がれる() {
    let json = plane_scene(&format!(
        r#"{A}, {B},
           {{ "id": "D", "type": "point", "at": [1, 1], "dot": true, "style": {{ "color": "red" }} }},
           {{ "id": "E", "type": "point", "at": [-1, 1], "dot": true }},
           {{ "id": "v", "type": "vector", "from": "A", "to": "B", "style": {{ "color": "blue" }} }}"#
    ));
    let output = export_tikz(&json).expect("出力できる");
    assert!(
        output.contains("\\fill[red] (1,1) circle (2pt);"),
        "{output}"
    );
    assert!(output.contains("\\fill (-1,1) circle (2pt);"), "{output}");
    assert!(
        output.contains("\\draw[line width=0.8pt, blue, -{Stealth}] (0,0) -- (3,1);"),
        "{output}"
    );
}

const VECTOR_ADDITION: &str = include_str!("../../../site/src/figures/vector-addition.json");

#[test]
fn ベクトルの和の図は_対角線のベクトルが_dの印に届く() {
    let figure = render(&parse_scene(VECTOR_ADDITION).expect("読める")).expect("描画できる");
    let lines = paths(&figure);
    // 破線の辺2本と，ベクトル3本．
    assert_eq!(lines.len(), 5);
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.stroke.line == Line::Dashed)
            .count(),
        2
    );
    assert_eq!(lines.iter().filter(|line| line.arrow.is_some()).count(), 3);
    // D = B + C - A = (5.6, 5.6)．
    let dot = dots(&figure)[0];
    let sum = lines[4];
    assert!(close(dot.at[0], 5.6) && close(dot.at[1], 5.6));
    assert!(close(sum.points[1][0], dot.at[0]) && close(sum.points[1][1], dot.at[1]));
    assert_eq!(labels(&figure).len(), 4);
}

// ---- 点の式(位置を，点の和と差，数倍で書く) ----

#[test]
fn 位置は_座標の並びのほか_点の式の文字列でも書ける() {
    let scene = scene_of(&format!(
        r#"{A}, {B},
           {{ "id": "M", "type": "point", "at": "(A + B) / 2" }},
           {{ "id": "t", "type": "label", "at": "A - B", "tex": "T" }}"#
    ));
    let Object::Point(m) = &scene.objects[2] else {
        panic!("点である");
    };
    assert_eq!(m.at, Position::Vector("(A + B) / 2".to_owned()));
    let Object::Label(label) = &scene.objects[3] else {
        panic!("ラベルである");
    };
    assert_eq!(label.at, Position::Vector("A - B".to_owned()));
    let written = serde_json::to_string(&scene).expect("書き出せる");
    assert!(written.contains(r#""at":"(A + B) / 2""#), "{written}");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 点の式は_点の和と差と数倍を成分ごとに計算する() {
    // A(0, 0)，B(3, 1)，C(1, 3)，媒介変数k = 0.5．横の単位2cm，縦の単位1cmの図で，位置を見る．
    let figure = figure_of(
        r#"{ "id": "k", "type": "parameter", "value": 0.5 },
           { "id": "A", "type": "point", "at": [0, 0] },
           { "id": "B", "type": "point", "at": [3, 1] },
           { "id": "C", "type": "point", "at": [1, 3] },
           { "id": "D", "type": "point", "at": "B + C - A", "dot": true },
           { "id": "E", "type": "point", "at": "A + k * (B - A)", "dot": true },
           { "id": "F", "type": "point", "at": "-C", "dot": true },
           { "id": "G", "type": "point", "at": "2 * B / 4 + C * k", "dot": true },
           { "id": "m", "type": "label", "at": "(A + D) / 2", "tex": "M" }"#,
    );
    let at: Vec<[f64; 2]> = dots(&figure).iter().map(|dot| dot.at).collect();
    let expected = [
        [8.0, 4.0],   // B + C - A = (4, 4)．横は2cm単位．
        [3.0, 0.5],   // A + 0.5(B - A) = (1.5, 0.5)
        [-2.0, -3.0], // -C = (-1, -3)
        [4.0, 2.0],   // 2B/4 + 0.5C = (1.5 + 0.5, 0.5 + 1.5) = (2, 2)
    ];
    for (got, want) in at.iter().zip(expected) {
        assert!(
            close(got[0], want[0]) && close(got[1], want[1]),
            "{got:?} {want:?}"
        );
    }
    // ラベルの位置は，AとDの中点(2, 2)，つまり(4, 2)cmである．
    let middle = labels(&figure)[0];
    assert!(close(middle.at[0], 4.0) && close(middle.at[1], 2.0));
}

#[test]
fn 点の式は_座標の名前と混ぜて使える() {
    let ok = parse_scene(&plane_scene(&format!(
        r#"{A}, {B}, {{ "id": "D", "type": "point", "at": "A + B" }},
           {{ "id": "g", "type": "graph", "var": "x", "expr": "D_y * x", "domain": [0, 1] }}"#
    )));
    assert!(ok.is_ok(), "{ok:?}");
}

#[test]
fn 点の式でない式は_誤りになる() {
    // 点どうしの積，点に数を足す，点を関数に入れる，点で割る，点のべき，点を含まない式．
    for source in ["A * B", "A + 1", "sin(A)", "1 / A", "A ^ 2", "1 + 2", "k"] {
        let error = error_of(&format!(
            r#"{{ "id": "k", "type": "parameter", "value": 1 }}, {A}, {B},
               {{ "id": "D", "type": "point", "at": "{source}" }}"#
        ));
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{source}: {error}"
        );
        assert_eq!(error.object.as_deref(), Some("D"), "{source}");
        assert!(error.to_string().contains("点"), "{source}: {error}");
    }
}

#[test]
fn 点の式の名前が未知か_あとに置いた点なら_式の誤りになる() {
    let unknown = error_of(&format!(
        r#"{A}, {{ "id": "D", "type": "point", "at": "A + Z" }}"#
    ));
    let ErrorKind::Expression { field, index, .. } = &unknown.kind else {
        panic!("式の誤りである: {unknown}");
    };
    assert_eq!((*field, *index), ("at", 0));
    // 自分も，あとの点も，参照できない．
    let later = error_of(&format!(
        r#"{{ "id": "D", "type": "point", "at": "A" }}, {A}"#
    ));
    assert!(
        matches!(later.kind, ErrorKind::Expression { .. }),
        "{later}"
    );
    let own = error_of(r#"{ "id": "D", "type": "point", "at": "D" }"#);
    assert!(matches!(own.kind, ErrorKind::Expression { .. }), "{own}");
}

#[test]
fn 点の式の構文の誤りは_式の中の位置を示す() {
    let error = error_of(&format!(
        r#"{A}, {{ "id": "D", "type": "point", "at": "A + " }}"#
    ));
    let ErrorKind::Expression { error: inner, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!(inner.kind, figure::expr::ExprErrorKind::UnexpectedEnd);
}

#[test]
fn 点のidは_関数や定数の名前にできない() {
    let error = error_of(r#"{ "id": "e", "type": "point", "at": [0, 0] }"#);
    assert_eq!(error.kind, ErrorKind::ReservedName("e".to_owned()));
    assert_eq!(error.object.as_deref(), Some("e"));
}

#[test]
fn 空間の図では_位置を点の式で書けない() {
    let error = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "azimuth": 0, "elevation": 0, "unit": "1cm" },
             "objects": [ { "id": "t", "type": "label", "at": "A", "tex": "T" } ] }"#,
    )
    .expect_err("誤りになる");
    assert!(error.to_string().contains("空間"), "{error}");
}
