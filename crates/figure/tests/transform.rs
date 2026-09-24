//! 変換(`transform`)と像(`image`)を確かめる．手順は書いた順に施し，平行移動・回転・拡大縮小・
//! 対称移動・せん断・写像(`map`)を組み合わせられる．像は，先に置いたオブジェクトを変換した複製である．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::{Error, parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-5, 5], "y": [-5, 5], "unit": {{ "x": "1cm", "y": "1cm" }} }},
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

fn plane_figure(objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(objects)).expect("読める")).expect("描画できる")
}

fn plane_error(objects: &str) -> Error {
    parse_scene(&plane_scene(objects)).expect_err("誤りになる")
}

fn space_error(objects: &str) -> Error {
    parse_scene(&space_scene(objects)).expect_err("誤りになる")
}

/// 空間の図の線を，向きと順を問わない形(両端を丸めて並べ替えたものと，線の種類)で集める．
fn space_edges(objects: &str) -> Vec<(String, String)> {
    let figure = render(&parse_scene(&space_scene(objects)).expect("読める")).expect("描画できる");
    let mut edges: Vec<(String, String)> = paths(&figure)
        .iter()
        .map(|path| {
            let mut ends: Vec<String> = path
                .points
                .iter()
                .map(|[x, y]| format!("{x:.4},{y:.4}"))
                .collect();
            ends.sort();
            (ends.join(" "), format!("{:?}", path.stroke.line))
        })
        .collect();
    edges.sort();
    edges
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

fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

/// 点`p`を変換して，その座標を，あとの点の位置の式で読み出す．
fn moved_point(steps: &str) -> [f64; 2] {
    let figure = plane_figure(&format!(
        r#"{{ "id": "p", "type": "point", "at": [2, 0], "transform": {steps} }},
           {{ "id": "q", "type": "point", "at": ["p_x", "p_y"], "dot": true }}"#
    ));
    figure
        .items
        .iter()
        .find_map(|item| match item {
            Item::Dot(dot) => Some(dot.at),
            _ => None,
        })
        .expect("点の印がある")
}

#[test]
fn 点の変換は座標に施され_あとの式から読める() {
    assert!(close(moved_point(r#"[{"rotate": 90}]"#), [0.0, 2.0]));
    assert!(close(
        moved_point(r#"[{"translate": [1, -1]}]"#),
        [3.0, -1.0]
    ));
    assert!(close(moved_point(r#"[{"scale": 1.5}]"#), [3.0, 0.0]));
    assert!(close(moved_point(r#"[{"scale": [-1, 2]}]"#), [-2.0, 0.0]));
}

#[test]
fn 手順は書いた順に施す() {
    // 平行移動してから回すのと，回してから平行移動するのは，違う．
    assert!(close(
        moved_point(r#"[{"translate": [1, 0]}, {"rotate": 90}]"#),
        [0.0, 3.0]
    ));
    assert!(close(
        moved_point(r#"[{"rotate": 90}, {"translate": [1, 0]}]"#),
        [1.0, 2.0]
    ));
}

#[test]
fn 回転と拡大縮小と対称移動は_centerを動かさない() {
    assert!(close(
        moved_point(r#"[{"rotate": 180, "center": [1, 0]}]"#),
        [0.0, 0.0]
    ));
    assert!(close(
        moved_point(r#"[{"scale": 2, "center": [1, 0]}]"#),
        [3.0, 0.0]
    ));
    // 平面の対称移動は，鏡の直線の向きで書く．直線y = xに関して対称に移す．
    assert!(close(moved_point(r#"[{"reflect": [1, 1]}]"#), [0.0, 2.0]));
    // 点(1, 0)を通る，y軸に平行な直線に関して対称に移す．
    assert!(close(
        moved_point(r#"[{"reflect": [0, 1], "center": [1, 0]}]"#),
        [0.0, 0.0]
    ));
}

#[test]
fn 数は媒介変数を使う式で書ける() {
    let figure = plane_figure(
        r#"{ "id": "a", "type": "parameter", "value": 45 },
           { "id": "p", "type": "point", "at": [1, 0], "dot": true,
             "transform": [{ "rotate": "2*a" }] }"#,
    );
    let Item::Dot(dot) = &figure.items[0] else {
        panic!("点の印がない");
    };
    assert!(close(dot.at, [0.0, 1.0]));
}

#[test]
fn 写像で曲線を写せる() {
    // 単位円を，x方向に2倍する写像で写すと，(2, 0)を通る楕円になる．
    let figure = plane_figure(
        r#"{ "id": "F", "type": "map", "vars": ["x", "y"], "expr": ["2*x", "y"] },
           { "id": "c", "type": "curve", "var": "t", "expr": ["cos(t)", "sin(t)"],
             "domain": [0, "2*pi"], "transform": [{ "map": "F" }] }"#,
    );
    let points: Vec<[f64; 2]> = paths(&figure)
        .iter()
        .flat_map(|path| path.points.clone())
        .collect();
    assert!(points.iter().any(|p| close(*p, [2.0, 0.0])));
    assert!(points.iter().all(|p| p[0].abs() <= 2.0 + 1e-9));
}

#[test]
fn 写像で格子を写すと線が曲がる() {
    // 複素数の2乗(z^2)で，格子を写す．
    let figure = plane_figure(
        r#"{ "id": "F", "type": "map", "vars": ["x", "y"], "expr": ["x^2 - y^2", "2*x*y"] },
           { "id": "g", "type": "grid", "x_step": 1, "x_range": [0.5, 1.5], "y_range": [-1, 1],
             "transform": [{ "map": "F" }] }"#,
    );
    let lines = paths(&figure);
    assert!(!lines.is_empty());
    assert!(lines.iter().all(|path| path.points.len() > 2));
}

#[test]
fn 手順には操作を1つだけ書く() {
    let error = plane_error(
        r#"{ "id": "p", "type": "point", "at": [0, 0],
             "transform": [{ "rotate": 30, "translate": [1, 0] }] }"#,
    );
    assert!(error.to_string().contains("1つだけ"));
    let error = plane_error(r#"{ "id": "p", "type": "point", "at": [0, 0], "transform": [{}] }"#);
    assert!(error.to_string().contains("1つだけ"));
}

#[test]
fn 空間の回転には軸が要り_せん断は平面だけで使える() {
    let error = space_error(
        r#"{ "id": "p", "type": "point", "at": [1, 0, 0], "transform": [{ "rotate": 30 }] }"#,
    );
    assert!(error.to_string().contains("axis"));
    let error = space_error(
        r#"{ "id": "p", "type": "point", "at": [1, 0, 0], "transform": [{ "shear": [1, 0] }] }"#,
    );
    assert!(error.to_string().contains("平面の図でだけ"));
    let error = plane_error(
        r#"{ "id": "p", "type": "point", "at": [1, 0],
             "transform": [{ "rotate": 30, "axis": [0, 0, 1] }] }"#,
    );
    assert!(error.to_string().contains("axis"));
}

#[test]
fn 線分と正多面体には写像を使えない() {
    let error = plane_error(
        r#"{ "id": "F", "type": "map", "vars": ["x", "y"], "expr": ["x", "y"] },
           { "id": "A", "type": "point", "at": [0, 0] },
           { "id": "B", "type": "point", "at": [1, 0] },
           { "id": "s", "type": "segment", "from": "A", "to": "B", "transform": [{ "map": "F" }] }"#,
    );
    assert_eq!(error.object.as_deref(), Some("s"));
    let error = plane_error(
        r#"{ "id": "p", "type": "point", "at": [0, 0], "transform": [{ "map": "G" }] }"#,
    );
    assert!(error.to_string().contains("写像「G」がない"));
}

#[test]
fn 正多面体を回せる() {
    // z軸のまわりに90度回した立方体は，元の立方体と同じ形になる．
    let cube =
        r#"{ "id": "p", "type": "polyhedron", "solid": "cube", "center": [0, 0, 0], "radius": 1 }"#;
    let turned = r#"{ "id": "p", "type": "polyhedron", "solid": "cube", "center": [0, 0, 0], "radius": 1,
                      "transform": [{ "rotate": 90, "axis": [0, 0, 1] }] }"#;
    let tilted = r#"{ "id": "p", "type": "polyhedron", "solid": "cube", "center": [0, 0, 0], "radius": 1,
                      "transform": [{ "rotate": 30, "axis": [0, 0, 1] }] }"#;
    assert_eq!(space_edges(cube), space_edges(turned));
    assert_ne!(space_edges(cube), space_edges(tilted));
}

#[test]
fn 対称移動した正多面体は面の向きを保って隠れる() {
    // 立方体をxy平面に関して対称に移しても，同じ立方体である．面の向きが裏返れば，隠れ方が変わってしまう．
    let cube =
        r#"{ "id": "p", "type": "polyhedron", "solid": "cube", "center": [0, 0, 0], "radius": 1 }"#;
    let mirrored = r#"{ "id": "p", "type": "polyhedron", "solid": "cube", "center": [0, 0, 0], "radius": 1,
                        "transform": [{ "reflect": [0, 0, 1] }] }"#;
    assert_eq!(space_edges(cube), space_edges(mirrored));
}

#[test]
fn 像は元のオブジェクトを変換した複製である() {
    let figure = plane_figure(
        r#"{ "id": "A", "type": "point", "at": [1, 0], "dot": true },
           { "id": "B", "type": "image", "of": "A", "transform": [{ "rotate": 90 }],
             "label": "B" },
           { "id": "s", "type": "segment", "from": "A", "to": "B" }"#,
    );
    let segment = paths(&figure)[0];
    assert!(close(segment.points[0], [1.0, 0.0]));
    assert!(close(segment.points[1], [0.0, 1.0]));
    let labels: Vec<&str> = figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(label) => Some(label.tex.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(labels, ["$B$"]);
}

#[test]
fn 像のスタイルは書けば元の代わりに使う() {
    let figure = plane_figure(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "0"], "domain": [0, 1],
             "style": { "color": "red" } },
           { "id": "d", "type": "image", "of": "c", "transform": [{ "translate": [0, 1] }] },
           { "id": "e", "type": "image", "of": "c", "transform": [{ "translate": [0, 2] }],
             "style": { "color": "blue" } }"#,
    );
    let colors: Vec<_> = paths(&figure)
        .iter()
        .map(|path| path.stroke.color)
        .collect();
    assert_eq!(colors.len(), 3);
    assert_eq!(colors[1], colors[0]);
    assert_ne!(colors[2], colors[0]);
}

#[test]
fn 像の元は先に置いた変換できるオブジェクトである() {
    let error = plane_error(
        r#"{ "id": "B", "type": "image", "of": "A", "transform": [{ "rotate": 90 }] },
           { "id": "A", "type": "point", "at": [1, 0] }"#,
    );
    assert_eq!(error.object.as_deref(), Some("B"));
    let error = plane_error(
        r#"{ "id": "x", "type": "axis", "direction": "x" },
           { "id": "B", "type": "image", "of": "x", "transform": [{ "rotate": 90 }] }"#,
    );
    assert!(error.to_string().contains("変換できない"));
    let error = plane_error(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "0"], "domain": [0, 1] },
           { "id": "d", "type": "image", "of": "c", "transform": [{ "rotate": 90 }], "label": "d" }"#,
    );
    assert!(error.to_string().contains("点の像にだけ"));
}

#[test]
fn 変換したグラフには接線を引ける() {
    // y = x^2を原点のまわりに90度回すと，x = -y^2になる．x = 0での接線は，y軸に重なる．
    let figure = plane_figure(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "x^2", "domain": [-2, 2],
             "transform": [{ "rotate": 90 }] },
           { "id": "t", "type": "tangent_line", "of": "f", "at": 0 }"#,
    );
    let tangent = paths(&figure).last().copied().expect("接線がある");
    assert!(tangent.points.iter().all(|p| p[0].abs() < 1e-6));
}

#[test]
fn 変換したグラフは領域とテイラー展開に使えない() {
    let error = plane_error(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "x^2", "domain": [-2, 2],
             "transform": [{ "rotate": 90 }] },
           { "id": "r", "type": "region", "between": ["f"], "domain": [0, 1] }"#,
    );
    assert!(error.to_string().contains("変換"));
}
