//! 複体(`complex`)．頂点と面(平らな多角形)でできた，空間の図形．稜は面から自動的に決まり，
//! 稜に隣接する2つの面の法線がどちらもカメラを向いていなければ，隠れる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::Line;
use figure::{Error, ErrorKind, parse_scene, render};

fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 225, "elevation": -35, "unit": "1cm" }},
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

/// 正4面体．頂点0(1,1,1)が，方位角225度・仰角-35度のカメラから見て，最も奥にある．
/// 面は，外向きの法線になるように，頂点を反時計回りに並べる．
const TETRAHEDRON: &str = r#"{ "id": "t", "type": "complex",
    "vertices": [[1, 1, 1], [1, -1, -1], [-1, 1, -1], [-1, -1, 1]],
    "faces": [[1, 3, 2], [0, 2, 3], [0, 3, 1], [0, 1, 2]] }"#;

#[test]
fn 複体は空間の図でだけ使える() {
    let scene = format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-2, 2], "y": [-2, 2], "unit": "1cm" }},
             "objects": [{TETRAHEDRON}] }}"#
    );
    let error = parse_scene(&scene).expect_err("誤りになる");
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error:?}");
}

#[test]
fn 頂点が3個未満だと誤りになる() {
    error_of(
        r#"{ "id": "t", "type": "complex", "vertices": [[0,0,0],[1,0,0]], "faces": [[0,1]] }"#,
    );
}

#[test]
fn 面の頂点は3個以上要る() {
    error_of(
        r#"{ "id": "t", "type": "complex",
             "vertices": [[0,0,0],[1,0,0],[0,1,0]], "faces": [[0,1]] }"#,
    );
}

#[test]
fn 面が指す頂点の番号は範囲の中でなければならない() {
    error_of(
        r#"{ "id": "t", "type": "complex",
             "vertices": [[0,0,0],[1,0,0],[0,1,0]], "faces": [[0,1,9]] }"#,
    );
}

#[test]
fn 正4面体は6本の稜を持つ() {
    let figure = figure_of(TETRAHEDRON);
    assert_eq!(paths(&figure).len(), 6);
}

#[test]
fn 手前の3本の稜は実線_奥の頂点につながる3本は隠れた線になる() {
    let figure = figure_of(TETRAHEDRON);
    let by_line = |line: Line| {
        paths(&figure)
            .into_iter()
            .filter(|p| p.stroke.line == line)
            .count()
    };
    assert_eq!(by_line(Line::Solid), 3, "手前の三角形(1,2,3)の3本の稜");
    assert_eq!(by_line(Line::Dotted), 3, "奥の頂点0につながる3本の稜");
}
