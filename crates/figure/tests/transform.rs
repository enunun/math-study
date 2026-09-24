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

// ---- 座標軸などを除く，すべての図形の変換 ----

fn space_figure(objects: &str) -> Figure {
    render(&parse_scene(&space_scene(objects)).expect("読める")).expect("描画できる")
}

fn labels_at(figure: &Figure) -> Vec<[f64; 2]> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(label) => Some(label.at),
            _ => None,
        })
        .collect()
}

/// 色が赤の線の点．
fn red_points(figure: &Figure) -> Vec<[f64; 2]> {
    paths(figure)
        .iter()
        .filter(|path| format!("{:?}", path.stroke.color).contains("Red"))
        .flat_map(|path| path.points.clone())
        .collect()
}

/// 点`p`から，`a`と`b`を通る直線までの距離．
fn distance_to_line(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / dx.hypot(dy)
}

#[test]
fn ラベルの位置を変換できる() {
    let figure = plane_figure(
        r#"{ "id": "l", "type": "label", "at": [1, 0], "tex": "$A$",
             "transform": [{ "translate": [1, 1] }] }"#,
    );
    assert!(close(labels_at(&figure)[0], [2.0, 1.0]));
    let moved = space_figure(
        r#"{ "id": "l", "type": "label", "at": [0, 0, 0], "tex": "$A$",
             "transform": [{ "translate": [0, 0, 1] }] }"#,
    );
    let placed = space_figure(r#"{ "id": "l", "type": "label", "at": [0, 0, 1], "tex": "$A$" }"#);
    assert!(close(labels_at(&moved)[0], labels_at(&placed)[0]));
}

#[test]
fn テイラー展開の多項式のグラフを変換できる() {
    // y = x^2の2次の展開は，x^2そのものである．90度回すと，x = -y^2になる．
    let figure = plane_figure(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "x^2", "domain": [-2, 2],
             "style": { "line": "dotted" } },
           { "id": "t", "type": "taylor", "of": "f", "at": 0, "order": 2,
             "transform": [{ "rotate": 90 }] }"#,
    );
    let solid: Vec<[f64; 2]> = paths(&figure)
        .iter()
        .filter(|path| path.stroke.line == figure::scene::Line::Solid)
        .flat_map(|path| path.points.clone())
        .collect();
    assert!(solid.len() > 2);
    assert!(
        solid.iter().all(|p| (p[0] + p[1] * p[1]).abs() < 1e-6),
        "{solid:?}"
    );
}

#[test]
fn 接線を変換できる_写像で写すと曲がる() {
    let moved = plane_figure(
        r#"{ "id": "f", "type": "graph", "var": "x", "expr": "0", "domain": [-2, 2],
             "style": { "color": "blue" } },
           { "id": "t", "type": "tangent_line", "of": "f", "at": 0,
             "style": { "color": "red" }, "transform": [{ "translate": [0, 1] }] }"#,
    );
    let points = red_points(&moved);
    assert!(!points.is_empty());
    assert!(points.iter().all(|p| (p[1] - 1.0).abs() < 1e-9));
    let bent = plane_figure(
        r#"{ "id": "F", "type": "map", "vars": ["x", "y"], "expr": ["x", "y + x^2/4"] },
           { "id": "f", "type": "graph", "var": "x", "expr": "0", "domain": [-2, 2],
             "style": { "color": "blue" } },
           { "id": "t", "type": "tangent_line", "of": "f", "at": 0,
             "style": { "color": "red" }, "transform": [{ "map": "F" }] }"#,
    );
    let points = red_points(&bent);
    assert!(points.len() > 2);
    // 見える範囲の端で切った点は，弦の上にあるので，標本化の誤差の分だけずれる．
    assert!(
        points
            .iter()
            .all(|p| (p[1] - p[0] * p[0] / 4.0).abs() < 1e-2)
    );
}

fn region_fill(transform: &str, map: &str) -> Vec<[f64; 2]> {
    let figure = plane_figure(&format!(
        r#"{map}{{ "id": "f", "type": "graph", "var": "x", "expr": "1", "domain": [-3, 3] }},
           {{ "id": "r", "type": "region", "between": ["f"], "domain": [0, 1], "hatch": false,
             "fill": {{}}, "transform": {transform} }}"#
    ));
    figure
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fill(fill) => Some(fill.points.clone()),
            _ => None,
        })
        .expect("塗りがある")
}

#[test]
fn 領域を変換できる_写像で写すとx軸の辺も曲がる() {
    let moved = region_fill(r#"[{ "translate": [0, 1] }]"#, "");
    assert!(
        moved
            .iter()
            .all(|p| p[1] >= 1.0 - 1e-9 && p[1] <= 2.0 + 1e-9)
    );
    assert!(moved.iter().any(|p| (p[1] - 1.0).abs() < 1e-9));
    // x軸の辺(y = 0)は，写像でy = x^2に写る．辺の途中の点も，その曲線の上にある．
    let bent = region_fill(
        r#"[{ "map": "F" }]"#,
        r#"{ "id": "F", "type": "map", "vars": ["x", "y"], "expr": ["x", "y + x^2"] },"#,
    );
    assert!(
        bent.iter()
            .any(|p| p[0] > 0.2 && p[0] < 0.8 && (p[1] - p[0] * p[0]).abs() < 1e-6),
        "{bent:?}"
    );
}

#[test]
fn 球を変換できる() {
    // 半径1の球を2倍にして，z方向に3動かす．輪郭は，(0, 0, 3)の投影を中心とする，半径2の円に近い．
    let figure = space_figure(
        r#"{ "id": "s", "type": "sphere", "center": [0, 0, 0], "radius": 1, "wireframe": {},
             "transform": [{ "scale": 2 }, { "translate": [0, 0, 3] }] },
           { "id": "c", "type": "label", "at": [0, 0, 3], "tex": "$C$" }"#,
    );
    let center = labels_at(&figure)[0];
    let solid: Vec<[f64; 2]> = paths(&figure)
        .iter()
        .filter(|path| path.stroke.line == figure::scene::Line::Solid)
        .flat_map(|path| path.points.clone())
        .collect();
    assert!(!solid.is_empty());
    for p in &solid {
        let r = (p[0] - center[0]).hypot(p[1] - center[1]);
        assert!((r - 2.0).abs() < 0.05, "{r}");
    }
    // ワイヤーフレーム(点線)も描く．
    assert!(
        paths(&figure)
            .iter()
            .any(|path| path.stroke.line == figure::scene::Line::Dotted)
    );
}

/// xy平面の正方形の曲面．
const FLOOR: &str = r#"{ "id": "floor", "type": "surface", "vars": ["x", "y"],
    "expr": ["x", "y", "0"], "domain": [[-2, 2], [-2, 2]] }"#;
/// 直線x = 0.3，z = 1の上の2点を結ぶ曲線．変換した線が，この直線の上にあるかを比べる．
const GUIDE: &str = r#"{ "id": "guide", "type": "curve", "var": "t", "expr": ["0.3", "t", "1"],
    "domain": [-2, 2], "style": { "color": "blue" } }"#;

fn guide_ends(figure: &Figure) -> ([f64; 2], [f64; 2]) {
    let guide = paths(figure)
        .into_iter()
        .find(|path| format!("{:?}", path.stroke.color).contains("Blue"))
        .expect("目安の線がある");
    (guide.points[0], *guide.points.last().unwrap())
}

#[test]
fn 切り口を変換できる() {
    let figure = space_figure(&format!(
        r#"{FLOOR}, {GUIDE},
           {{ "id": "c", "type": "cut", "surface": "floor", "normal": [1, 0, 0], "offset": 0.3,
             "style": {{ "color": "red" }}, "transform": [{{ "translate": [0, 0, 1] }}] }}"#
    ));
    let (a, b) = guide_ends(&figure);
    let points = red_points(&figure);
    assert!(!points.is_empty());
    assert!(points.iter().all(|p| distance_to_line(*p, a, b) < 1e-6));
}

#[test]
fn 交線を変換できる() {
    let figure = space_figure(&format!(
        r#"{FLOOR}, {GUIDE},
           {{ "id": "wall", "type": "surface", "vars": ["y", "z"], "expr": ["0.3", "y", "z"],
             "domain": [[-2, 2], [-1, 1]] }},
           {{ "id": "c", "type": "intersection", "surfaces": ["floor", "wall"],
             "style": {{ "color": "red" }}, "transform": [{{ "translate": [0, 0, 1] }}] }}"#
    ));
    let (a, b) = guide_ends(&figure);
    let points = red_points(&figure);
    assert!(!points.is_empty());
    assert!(points.iter().all(|p| distance_to_line(*p, a, b) < 1e-6));
}

#[test]
fn 接平面を変換できる() {
    let tangent = |surface_z: &str, transform: &str| {
        let figure = space_figure(&format!(
            r#"{{ "id": "s", "type": "surface", "vars": ["x", "y"], "expr": ["x", "y", "{surface_z}"],
                 "domain": [[-2, 2], [-2, 2]] }},
               {{ "id": "t", "type": "tangent_plane", "of": "s", "at": [0, 0], "size": 1,
                 "style": {{ "color": "red" }}, "transform": {transform} }}"#
        ));
        let mut ends: Vec<String> = paths(&figure)
            .iter()
            .filter(|path| format!("{:?}", path.stroke.color).contains("Red"))
            .flat_map(|path| [path.points[0], *path.points.last().unwrap()])
            .map(|[x, y]| format!("{x:.6},{y:.6}"))
            .collect();
        ends.sort();
        ends.dedup();
        ends
    };
    let moved = tangent("0", r#"[{ "translate": [0, 0, 1] }]"#);
    assert_eq!(moved.len(), 4);
    assert_eq!(moved, tangent("1", "[]"));
}

#[test]
fn 新しく変換できる種類も像の元にできる() {
    let figure = plane_figure(
        r#"{ "id": "l", "type": "label", "at": [1, 0], "tex": "$A$" },
           { "id": "m", "type": "image", "of": "l", "transform": [{ "rotate": 90 }] }"#,
    );
    let at = labels_at(&figure);
    assert!(close(at[0], [1.0, 0.0]));
    assert!(close(at[1], [0.0, 1.0]));
}
