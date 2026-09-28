//! ベクトル場(`vector_field`)を確かめる．格子点ごとに場の式を評価し，矢印を置く．場は，ベクトルの式
//! (位置ベクトル`r`と点を使う，点の式と同じ形)か，成分の式の並びで書く．矢印の長さは，倍率を掛けるか
//! (`scaled`)，そろえるか(`normalized`)，上限で切る(`clamped`)．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Object, Position};
use figure::{Error, ErrorKind, parse_scene, render};

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

fn error_of(objects: &str) -> Error {
    parse_scene(&plane_scene(objects)).expect_err("誤りになる")
}

fn arrows(figure: &Figure) -> Vec<&Path> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Path(path) => Some(path),
            _ => None,
        })
        .collect()
}

/// 格子[-1, 1]^2，刻み1のベクトル場(9個の格子点)．
fn field(field: &str, extra: &str) -> String {
    format!(
        r#"{{ "id": "E", "type": "vector_field", "field": {field},
              "x_step": 1, "y_step": 1, "x_range": [-1, 1], "y_range": [-1, 1] {extra} }}"#
    )
}

fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

fn ends(path: &Path) -> ([f64; 2], [f64; 2]) {
    (path.points[0], path.points[path.points.len() - 1])
}

#[test]
fn ベクトル場を読み_書き出して読み直すと同じになる() {
    let scene = parse_scene(&plane_scene(&field(r#""r / norm(r)""#, ""))).expect("読める");
    let Object::VectorField(found) = &scene.objects[0] else {
        panic!("ベクトル場である");
    };
    assert_eq!(found.field, Position::Vector("r / norm(r)".to_owned()));
    assert_eq!(found.var, "r");
    assert_eq!(scene.objects[0].type_name(), "vector_field");
    let written = serde_json::to_string(&scene).expect("書き出せる");
    assert!(
        !written.contains(r#""var""#),
        "既定の名前は書き出さない：{written}"
    );
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 成分の式で書いた一様な場は_格子点を中心とする矢印になる() {
    let figure = figure_of(&field("[1, 0]", r#", "scale": 0.5"#));
    let lines = arrows(&figure);
    assert_eq!(lines.len(), 9);
    for line in &lines {
        let (start, end) = ends(line);
        assert!((end[0] - start[0] - 0.5).abs() < 1e-9 && (end[1] - start[1]).abs() < 1e-9);
        let middle = [
            f64::midpoint(start[0], end[0]),
            f64::midpoint(start[1], end[1]),
        ];
        assert!(
            close(middle, [middle[0].round(), middle[1].round()]),
            "中心が格子点：{middle:?}"
        );
        assert!(line.arrow.is_some(), "矢じりがある");
    }
}

#[test]
fn 矢印の根元を格子点に置ける() {
    // 回転の場(-y, x)．点(1, 0)の矢印は，(1, 0)から(1, 0.5)へ向かう．
    let figure = figure_of(&field(
        r#"["-r_y", "r_x"]"#,
        r#", "scale": 0.5, "pivot": "tail""#,
    ));
    let found = arrows(&figure)
        .into_iter()
        .map(ends)
        .find(|(start, _)| close(*start, [1.0, 0.0]))
        .expect("(1, 0)の矢印がある");
    assert!(close(found.1, [1.0, 0.5]), "{found:?}");
}

#[test]
fn ベクトルの式で書いた点電荷の場は_そろえた長さで外向きになり_電荷の位置は描かない() {
    let figure = figure_of(&format!(
        r#"{{ "id": "Q", "type": "point", "at": [0, 0] }},
           {}"#,
        field(
            r#""(r - Q) / norm(r - Q)^3""#,
            r#", "length": "normalized", "scale": 0.4, "pivot": "tail""#
        )
    ));
    let lines = arrows(&figure);
    assert_eq!(
        lines.len(),
        8,
        "電荷のある原点では値が有限でないので描かない"
    );
    for line in lines {
        let (start, end) = ends(line);
        let direction = [end[0] - start[0], end[1] - start[1]];
        assert!((direction[0].hypot(direction[1]) - 0.4).abs() < 1e-9);
        // 外向き：根元の位置ベクトルと同じ向き．
        let outward = start[0] * direction[0] + start[1] * direction[1];
        assert!(outward > 0.0, "{start:?} {direction:?}");
    }
}

#[test]
fn 上限で切ると_長い矢印だけが上限の長さになる() {
    // 場(x, 0)に倍率1を掛け，上限0.5で切る．x = ±1の矢印は長さ0.5，x = 0は長さ0なので描かない．
    let figure = figure_of(&field(
        r#"["r_x", 0]"#,
        r#", "length": "clamped", "scale": 1, "max_length": 0.5"#,
    ));
    let lines = arrows(&figure);
    assert_eq!(lines.len(), 6);
    for line in lines {
        let (start, end) = ends(line);
        assert!(((end[0] - start[0]).abs() - 0.5).abs() < 1e-9);
    }
}

#[test]
fn 点と媒介変数を使え_点を動かすと場も動く() {
    let at = |q: &str| {
        let figure = figure_of(&format!(
            r#"{{ "id": "k", "type": "parameter", "value": 2 }},
               {{ "id": "Q", "type": "point", "at": [{q}, 0] }},
               {}"#,
            field(r#""k * (r - Q)""#, r#", "pivot": "tail", "scale": 0.1"#)
        ));
        let (start, end) = arrows(&figure)
            .into_iter()
            .map(ends)
            .find(|(start, _)| close(*start, [1.0, 1.0]))
            .expect("(1, 1)の矢印がある");
        [end[0] - start[0], end[1] - start[1]]
    };
    // 0.1・2・((1, 1) - Q)．
    assert!(close(at("0"), [0.2, 0.2]));
    assert!(close(at("1"), [0.0, 0.2]));
}

#[test]
fn 見える範囲からはみ出す矢印は描かない() {
    let figure = figure_of(
        r#"{ "id": "E", "type": "vector_field", "field": [1, 0], "scale": 0.5,
             "x_step": 1, "y_step": 1, "x_range": [2, 3], "y_range": [0, 0] }"#,
    );
    // x = 2の矢印は[1.75, 2.25]，x = 3の矢印は[2.75, 3.25]で，後者ははみ出す．
    assert_eq!(arrows(&figure).len(), 1);
}

#[test]
fn 範囲を省くと_見える範囲から刻みの半分だけ内側の_刻みの倍数の格子点に置く() {
    // 見える範囲[-3, 3]から0.5内側の[-2.5, 2.5]にある，1の倍数-2から2まで．縁で矢印が切れない．
    let figure = figure_of(
        r#"{ "id": "E", "type": "vector_field", "field": [0, 1], "scale": 0.8,
             "x_step": 1, "y_step": 1 }"#,
    );
    let lines = arrows(&figure);
    assert_eq!(lines.len(), 25);
    let lowest = lines
        .iter()
        .map(|line| f64::midpoint(ends(line).0[1], ends(line).1[1]))
        .fold(f64::INFINITY, f64::min);
    assert!((lowest + 2.0).abs() < 1e-9, "{lowest}");
}

#[test]
fn 変換で動かせる() {
    let figure = figure_of(&field(
        "[1, 0]",
        r#", "scale": 0.5, "pivot": "tail", "transform": [{ "translate": [0.5, 0] }]"#,
    ));
    let starts: Vec<[f64; 2]> = arrows(&figure)
        .into_iter()
        .map(|path| ends(path).0)
        .collect();
    assert!(
        starts.iter().any(|start| close(*start, [0.5, 0.0])),
        "{starts:?}"
    );
}

#[test]
fn 場の式の値がベクトルでなければ誤りになる() {
    for source in [r#""norm(r)""#, r#""r * r""#, r#""sin(r)""#] {
        let error = error_of(&field(source, ""));
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{source}: {error}"
        );
        assert_eq!(error.object.as_deref(), Some("E"));
    }
}

#[test]
fn 成分の数は図の次元に合わせる() {
    let error = error_of(&field("[1, 0, 0]", ""));
    assert!(error.to_string().contains("field"), "{error}");
}

#[test]
fn 位置ベクトルの名前は_ほかの名前とぶつかれば誤りになる() {
    let error = error_of(&format!(
        r#"{{ "id": "p", "type": "parameter", "value": 1 }}, {}"#,
        field(r#""p""#, r#", "var": "p""#)
    ));
    assert_eq!(error.kind, ErrorKind::NameConflict("p".to_owned()));
}

#[test]
fn 刻みは正の数で_格子点の数には上限がある() {
    let error = error_of(&field("[1, 0]", "").replace(r#""x_step": 1"#, r#""x_step": 0"#));
    assert!(error.to_string().contains("x_step"), "{error}");
    let error = error_of(
        r#"{ "id": "E", "type": "vector_field", "field": [1, 0],
             "x_step": 0.001, "y_step": 0.001 }"#,
    );
    assert!(error.to_string().contains("格子点"), "{error}");
}

/// 空間の図(方位角60度，仰角20度)．
fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn space_figure(objects: &str) -> Figure {
    render(&parse_scene(&space_scene(objects)).expect("読める")).expect("描ける")
}

/// 空間の点を，画面(cm)へ投影する．
fn project(p: [f64; 3]) -> [f64; 2] {
    let (a, e) = (60.0_f64.to_radians(), 20.0_f64.to_radians());
    let right = [-a.sin(), a.cos(), 0.0];
    let up = [-e.sin() * a.cos(), -e.sin() * a.sin(), e.cos()];
    let dot = |u: [f64; 3]| u[0] * p[0] + u[1] * p[1] + u[2] * p[2];
    [dot(right), dot(up)]
}

/// 空間の格子[-1, 1]^3，刻み1のベクトル場(27個の格子点)．
fn space_field(field: &str, extra: &str) -> String {
    format!(
        r#"{{ "id": "E", "type": "vector_field", "field": {field},
              "x_step": 1, "y_step": 1, "z_step": 1,
              "x_range": [-1, 1], "y_range": [-1, 1], "z_range": [-1, 1] {extra} }}"#
    )
}

#[test]
fn 空間の一様な場は_格子点ごとに同じ向きの矢印になる() {
    let figure = space_figure(&space_field(
        "[0, 0, 1]",
        r#", "scale": 0.5, "pivot": "tail""#,
    ));
    let lines = arrows(&figure);
    assert_eq!(lines.len(), 27);
    let expected = project([0.0, 0.0, 0.5]);
    for line in lines {
        let (start, end) = ends(line);
        assert!(close([end[0] - start[0], end[1] - start[1]], expected));
        assert!(line.arrow.is_some());
    }
    // 原点の矢印は，原点から伸びる．
    assert!(
        arrows(&figure)
            .into_iter()
            .map(ends)
            .any(|(start, _)| close(start, [0.0, 0.0]))
    );
}

#[test]
fn 空間のベクトルの式は_3成分の位置ベクトルを使う() {
    // 原点の点電荷の場．原点では値が有限でないので，26本になる．
    let figure = space_figure(&format!(
        r#"{{ "id": "Q", "type": "point", "at": [0, 0, 0] }},
           {}"#,
        space_field(
            r#""(r - Q) / norm(r - Q)^3""#,
            r#", "length": "normalized", "scale": 0.3, "pivot": "tail""#
        )
    ));
    assert_eq!(arrows(&figure).len(), 26);
    // 成分の名前r_zも使える．z = 0の層の9点では長さ0なので描かない．
    let figure = space_figure(&space_field(r#"[0, 0, "r_z"]"#, ""));
    assert_eq!(arrows(&figure).len(), 18);
}

#[test]
fn 空間のベクトル場は_曲面に隠れる部分を隠れた線で描く() {
    // 大きな球の中心にある矢印は，すべて球に隠れる．
    let figure = space_figure(
        r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 3 },
           { "id": "E", "type": "vector_field", "field": [0, 0, 1], "scale": 0.5,
             "x_step": 1, "y_step": 1, "z_step": 1,
             "x_range": [0, 0], "y_range": [0, 0], "z_range": [0, 0] }"#,
    );
    let arrow = arrows(&figure)
        .into_iter()
        .find(|path| path.points.len() == 2 && path.arrow.is_none())
        .expect("矢印の線がある");
    assert_ne!(arrow.stroke.line, figure::scene::Line::Solid);
}

#[test]
fn 空間の図では_範囲を3つとも書き_成分は3個にする() {
    let error = parse_scene(&space_scene(
        r#"{ "id": "E", "type": "vector_field", "field": [0, 0, 1],
             "x_step": 1, "y_step": 1, "z_step": 1, "x_range": [-1, 1], "y_range": [-1, 1] }"#,
    ))
    .expect_err("誤りになる");
    assert!(error.to_string().contains("z_range"), "{error}");
    let error = parse_scene(&space_scene(&space_field("[0, 1]", ""))).expect_err("誤りになる");
    assert!(error.to_string().contains("field"), "{error}");
}

#[test]
fn 平面の図では_z方向の刻みと範囲は書けない() {
    let error = error_of(&field("[1, 0]", r#", "z_step": 1"#));
    assert!(error.to_string().contains("z_step"), "{error}");
}
