//! シーンから，描画の中間表現ができることを確かめる．最初の図の各要素の，位置と種類を見る．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::figure::{Figure, Item, LabelItem, Path};
use figure::scene::{Anchor, Arrow, Line};
use figure::{parse_scene, render};

const SINE_AND_SHIFTED_SINE: &str =
    include_str!("../../../site/src/figures/sine-and-shifted-sine.json");

/// 余白(cm)．`render.rs`の定数と同じ値である．
const MARGIN: f64 = 0.6;
const PT_CM: f64 = 2.54 / 72.27;

fn figure_of(json: &str) -> Figure {
    render(&parse_scene(json).expect("シーンを読める")).expect("描画できる")
}

fn scene_with(unit: &str, objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-2, 3], "y": [-1, 2], "unit": {unit} }},
             "objects": [{objects}] }}"#
    )
}

const UNIT_1CM: &str = r#"{ "x": "1cm", "y": "1cm" }"#;

fn path(item: &Item) -> &Path {
    match item {
        Item::Path(path) => path,
        Item::Label(_) | Item::Dot(_) => panic!("折れ線ではない: {item:?}"),
    }
}

fn label(item: &Item) -> &LabelItem {
    match item {
        Item::Label(label) => label,
        Item::Path(_) | Item::Dot(_) => panic!("ラベルではない: {item:?}"),
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn 最初の図は_要素が描く順に並ぶ() {
    let figure = figure_of(SINE_AND_SHIFTED_SINE);
    let kinds: Vec<&str> = figure
        .items
        .iter()
        .map(|item| match item {
            Item::Path(_) => "path",
            Item::Label(_) => "label",
            Item::Dot(_) => "dot",
        })
        .collect();
    // x軸，xのラベル，y軸，yのラベル，原点のラベル，sin x，平行移動したsin x，表題．
    assert_eq!(
        kinds,
        [
            "path", "label", "path", "label", "label", "path", "path", "label"
        ]
    );
    assert_eq!(
        figure.description,
        "y=sin x のグラフと，x軸の方向に平行移動した点線のグラフ"
    );
}

#[test]
fn x軸は範囲の左右の端を結び_先端に矢じりと名前を持つ() {
    let figure = figure_of(SINE_AND_SHIFTED_SINE);
    let axis = path(&figure.items[0]);
    // 1単位は，x方向に1cmである．
    assert_eq!(axis.points, [[-7.0, 0.0], [7.0, 0.0]]);
    assert_eq!(axis.stroke.line, Line::Solid);
    assert!(close(axis.stroke.width, 0.6));
    let arrow = axis.arrow.as_ref().expect("矢じりがある");
    assert_eq!(arrow.kind, Arrow::Stealth);
    // 先端は，軸の端にあり，線は，矢じりの手前で止まる．
    let tip = arrow.polygon[0][0];
    assert!(tip < 7.0 && 7.0 - tip < 0.05, "先端 {tip}");
    assert!(arrow.line_end[0] < tip);
    assert!(close(arrow.line_end[1], 0.0));

    let name = label(&figure.items[1]);
    assert_eq!(name.at, [7.0, 0.0]);
    assert_eq!(name.anchor, Anchor::West);
    assert_eq!(name.tex, "$x$");
}

#[test]
fn y軸は上向きで_縦の単位の大きさに従う() {
    let figure = figure_of(SINE_AND_SHIFTED_SINE);
    let axis = path(&figure.items[2]);
    // 縦の1単位は，2cmである．範囲は，-1.6から1.8である．
    assert!(close(axis.points[0][1], -3.2));
    assert!(close(axis.points[1][1], 3.6));
    assert!(close(axis.points[0][0], 0.0));
    let arrow = axis.arrow.as_ref().expect("矢じりがある");
    assert!(arrow.line_end[1] < 3.6);
    assert!(close(arrow.line_end[0], 0.0));

    let name = label(&figure.items[3]);
    assert_eq!(name.at, [0.0, 3.6]);
    assert_eq!(name.anchor, Anchor::South);
    assert_eq!(name.tex, "$y$");
}

#[test]
fn ラベルは指定した位置と向きに置く() {
    let figure = figure_of(SINE_AND_SHIFTED_SINE);
    let origin = label(&figure.items[4]);
    assert_eq!((origin.at, origin.anchor), ([0.0, 0.0], Anchor::NorthWest));
    assert_eq!(origin.tex, "O");
    let title = label(&figure.items[7]);
    assert_eq!((title.at, title.anchor), ([1.0, 4.0], Anchor::West));
    assert_eq!(title.tex, r"Graph of $y=\sin x$");
}

#[test]
fn グラフは定義域を標本化した折れ線で_線の種類に従う() {
    let figure = figure_of(SINE_AND_SHIFTED_SINE);
    let solid = path(&figure.items[5]);
    assert_eq!(solid.stroke.line, Line::Solid);
    assert!(close(solid.stroke.width, 0.8));
    assert!(solid.arrow.is_none());
    let first = solid.points[0];
    assert!(close(first[0], -7.0));
    assert!(close(first[1], (-7.0_f64).sin() * 2.0));
    for point in &solid.points {
        assert!(close(point[1], point[0].sin() * 2.0));
    }
    assert!(close(solid.points.last().unwrap()[0], 7.0));

    let dotted = path(&figure.items[6]);
    assert_eq!(dotted.stroke.line, Line::Dotted);
}

#[test]
fn 媒介変数の値が式に入る() {
    let figure = figure_of(SINE_AND_SHIFTED_SINE);
    let shifted = path(&figure.items[6]);
    // shift = -1.2．
    for point in &shifted.points {
        assert!(close(point[1], (point[0] + 1.2).sin() * 2.0));
    }
}

#[test]
fn 媒介変数表示の曲線は式の値を実寸にして結ぶ() {
    let json = plane_scene(
        r#"{ "x": [-3, 3], "y": [-3, 3], "unit": { "x": "1cm", "y": "1cm" } }"#,
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["2*cos(t)", "2*sin(t)"],
             "domain": [0, "2*pi"], "style": { "line": "dashed" } }"#,
    );
    let figure = figure_of(&json);
    let circle = path(&figure.items[0]);
    assert_eq!(circle.stroke.line, Line::Dashed);
    for point in &circle.points {
        assert!(close(point[0].hypot(point[1]), 2.0));
    }
    let first = circle.points[0];
    let last = circle.points.last().unwrap();
    assert!(close(first[0], last[0]) && close(first[1], last[1]));
}

#[test]
fn 矢じりのない軸は線だけを引く() {
    let json = scene_with(
        UNIT_1CM,
        r#"{ "id": "x", "type": "axis", "direction": "x", "arrow": "none" }"#,
    );
    let figure = figure_of(&json);
    assert_eq!(figure.items.len(), 1);
    assert!(path(&figure.items[0]).arrow.is_none());
}

#[test]
fn 軸の範囲を指定できる() {
    let json = scene_with(
        UNIT_1CM,
        r#"{ "id": "y", "type": "axis", "direction": "y", "range": [0, 1.5], "label": "y" }"#,
    );
    let figure = figure_of(&json);
    let axis = path(&figure.items[0]);
    assert_eq!(axis.points, [[0.0, 0.0], [0.0, 1.5]]);
    assert_eq!(label(&figure.items[1]).at, [0.0, 1.5]);
}

#[test]
fn 長さの単位はcmに直す() {
    let json = scene_with(
        r#"{ "x": "10mm", "y": "72.27pt" }"#,
        r#"{ "id": "y", "type": "axis", "direction": "y", "arrow": "none" },
           { "id": "x", "type": "axis", "direction": "x", "arrow": "none" }"#,
    );
    let figure = figure_of(&json);
    // yの範囲は，-1から2で，1単位は72.27pt = 2.54cmである．
    let y_axis = path(&figure.items[0]);
    assert!(close(y_axis.points[0][1], -2.54));
    assert!(close(y_axis.points[1][1], 5.08));
    let x_axis = path(&figure.items[1]);
    assert!(close(x_axis.points[0][0], -2.0));
    assert!(close(x_axis.points[1][0], 3.0));
    assert!(close(PT_CM * 72.27, 2.54));
}

#[test]
fn 描く範囲は見える範囲に余白を足したものである() {
    let figure = figure_of(SINE_AND_SHIFTED_SINE);
    assert!(close(figure.bounds.min[0], -7.0 - MARGIN));
    assert!(close(figure.bounds.max[0], 7.0 + MARGIN));
    assert!(close(figure.bounds.min[1], -3.2 - MARGIN));
    assert!(close(figure.bounds.max[1], 3.6 + MARGIN));
}

#[test]
fn 媒介変数は何も描かない() {
    let json = scene_with(
        UNIT_1CM,
        r#"{ "id": "a", "type": "parameter", "value": 1 }"#,
    );
    assert!(figure_of(&json).items.is_empty());
}

#[test]
fn 中間表現は_kindごとに型の名前を持つjsonになる() {
    let value = serde_json::to_value(figure_of(SINE_AND_SHIFTED_SINE)).expect("書き出せる");
    assert_eq!(value["items"][0]["type"], "path");
    assert_eq!(value["items"][0]["arrow"]["kind"], "stealth");
    assert_eq!(value["items"][1]["type"], "label");
    assert_eq!(value["items"][1]["anchor"], "west");
    assert_eq!(value["items"][4]["anchor"], "north west");
    assert_eq!(value["items"][6]["stroke"]["line"], "dotted");
    assert!(value["items"][5]["arrow"].is_null());
    assert_eq!(value["bounds"]["min"][0], -7.6);
}

fn plane_scene(view: &str, objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {view}, "objects": [{objects}] }}"#
    )
}

const VIEW_2X3: &str = r#"{ "x": [-2, 2], "y": [-1, 3], "unit": { "x": "1cm", "y": "1cm" } }"#;

#[test]
fn グラフは見える範囲で切り取り_外へ出る部分を描かない() {
    let parabola =
        r#"{ "id": "p", "type": "graph", "var": "x", "expr": "x^2", "domain": [-3, 3] }"#;
    let figure = figure_of(&plane_scene(VIEW_2X3, parabola));
    assert_eq!(figure.items.len(), 1);
    let curve = path(&figure.items[0]);
    // y = x^2 は，y = 3 の所(x = ±sqrt(3))で，範囲の上の辺に出る．
    for point in &curve.points {
        assert!(
            point[0] >= -2.0 - 1e-9 && point[0] <= 2.0 + 1e-9,
            "{point:?}"
        );
        assert!(
            point[1] >= -1.0 - 1e-9 && point[1] <= 3.0 + 1e-9,
            "{point:?}"
        );
    }
    let (first, last) = (curve.points[0], *curve.points.last().unwrap());
    assert!(
        close(first[1], 3.0) && close(last[1], 3.0),
        "{first:?} {last:?}"
    );
    // 折れ線の弦で切るので，曲線との差は，標本化の許容(0.003cm)程度である．
    assert!((first[0] + 3.0_f64.sqrt()).abs() < 0.01 && (last[0] - 3.0_f64.sqrt()).abs() < 0.01);
}

#[test]
fn 範囲の外にだけあるグラフは_何も描かない() {
    let far = r#"{ "id": "p", "type": "graph", "var": "x", "expr": "x + 10", "domain": [-1, 1] }"#;
    assert!(figure_of(&plane_scene(VIEW_2X3, far)).items.is_empty());
}

#[test]
fn tanのように何度も範囲を出入りするグラフは_範囲の中の線に分かれる() {
    let tan = r#"{ "id": "t", "type": "graph", "var": "x", "expr": "tan(x)", "domain": [-2, 2] }"#;
    let figure = figure_of(&plane_scene(VIEW_2X3, tan));
    let curves: Vec<&Path> = figure.items.iter().map(path).collect();
    assert!(curves.len() >= 2, "{}", curves.len());
    for curve in curves {
        for point in &curve.points {
            assert!(
                point[1] >= -1.0 - 1e-9 && point[1] <= 3.0 + 1e-9,
                "{point:?}"
            );
        }
    }
}

#[test]
fn 媒介変数表示の曲線も_見える範囲で切り取り_線の種類を保つ() {
    // 見える範囲(x: -2から2，y: -1から3)の四方に，はみ出す円．
    let circle = r#"{ "id": "c", "type": "curve", "var": "t",
        "expr": ["2.5*cos(t)", "1+2.5*sin(t)"],
        "domain": [0, "2*pi"], "style": { "line": "dotted" } }"#;
    let figure = figure_of(&plane_scene(VIEW_2X3, circle));
    assert!(figure.items.len() >= 2);
    for item in &figure.items {
        let curve = path(item);
        assert_eq!(curve.stroke.line, Line::Dotted);
        for point in &curve.points {
            assert!(
                point[0].abs() <= 2.0 + 1e-9 && point[1] >= -1.0 - 1e-9 && point[1] <= 3.0 + 1e-9
            );
        }
    }
}

#[test]
fn 軸は_範囲を指定すれば見える範囲の外にも延ばせる() {
    let axis = r#"{ "id": "x_axis", "type": "axis", "direction": "x", "range": [-5, 5] }"#;
    let figure = figure_of(&plane_scene(VIEW_2X3, axis));
    assert_eq!(path(&figure.items[0]).points, [[-5.0, 0.0], [5.0, 0.0]]);
}

// ---- 目盛 ----

/// 目盛の半分の長さ(cm)．`render.rs`の定数(3pt)と同じ値である．
const TICK_HALF: f64 = 3.0 * PT_CM;

fn axes_with_ticks(x_extra: &str, y_extra: &str) -> String {
    plane_scene(
        r#"{ "x": [-4, 4], "y": [-2, 2], "unit": { "x": "1cm", "y": "2cm" } }"#,
        &format!(
            r#"{{ "id": "a", "type": "parameter", "value": 2 }},
               {{ "id": "x_axis", "type": "axis", "direction": "x" {x_extra} }},
               {{ "id": "y_axis", "type": "axis", "direction": "y" {y_extra} }}"#
        ),
    )
}

#[test]
fn x軸の目盛は_軸に直角な短い線で_名前は下に置く() {
    let figure = figure_of(&axes_with_ticks(
        r#", "label": "x", "ticks": [ { "at": 1, "label": "1" }, { "at": -2 } ]"#,
        "",
    ));
    // 軸，1の目盛，1の名前，-2の目盛，軸の名前，y軸．
    let kinds: Vec<&str> = figure
        .items
        .iter()
        .map(|item| match item {
            Item::Path(_) => "path",
            Item::Label(_) => "label",
            Item::Dot(_) => "dot",
        })
        .collect();
    assert_eq!(kinds, ["path", "path", "label", "path", "label", "path"]);
    let tick = path(&figure.items[1]);
    assert!(close(tick.points[0][0], 1.0) && close(tick.points[1][0], 1.0));
    assert!(close(tick.points[0][1], -TICK_HALF) && close(tick.points[1][1], TICK_HALF));
    assert_eq!(tick.stroke.line, Line::Solid);
    assert!(close(tick.stroke.width, 0.6));
    assert!(tick.arrow.is_none());
    let name = label(&figure.items[2]);
    assert!(close(name.at[0], 1.0) && close(name.at[1], -TICK_HALF));
    assert_eq!((name.anchor, name.tex.as_str()), (Anchor::North, "$1$"));
    // 名前のない目盛は，線だけである．
    let bare = path(&figure.items[3]);
    assert!(close(bare.points[0][0], -2.0));
    assert_eq!(label(&figure.items[4]).tex, "$x$");
}

#[test]
fn y軸の目盛は_縦の単位の大きさに従い_名前は左に置く() {
    let figure = figure_of(&axes_with_ticks(
        "",
        r#", "ticks": [ { "at": 1, "label": "1" } ]"#,
    ));
    // x軸，y軸，1の目盛，1の名前．
    let tick = path(&figure.items[2]);
    // 縦の1単位は，2cmである．
    assert!(close(tick.points[0][1], 2.0) && close(tick.points[1][1], 2.0));
    assert!(close(tick.points[0][0], -TICK_HALF) && close(tick.points[1][0], TICK_HALF));
    let name = label(&figure.items[3]);
    assert!(close(name.at[0], -TICK_HALF) && close(name.at[1], 2.0));
    assert_eq!(name.anchor, Anchor::East);
}

#[test]
fn 目盛の位置の式は_媒介変数と定数を使える() {
    let figure = figure_of(&axes_with_ticks(r#", "ticks": [ { "at": "a*pi/2" } ]"#, ""));
    let tick = path(&figure.items[1]);
    assert!(close(tick.points[0][0], std::f64::consts::PI));
}

#[test]
fn 目盛を付けても_軸の線と矢じりは変わらない() {
    let plain = figure_of(&axes_with_ticks("", ""));
    let ticked = figure_of(&axes_with_ticks(r#", "ticks": [ { "at": 1 } ]"#, ""));
    assert_eq!(plain.items[0], ticked.items[0]);
    assert_eq!(plain.bounds, ticked.bounds);
}

const SINE_WITH_TICKS: &str = include_str!("../../../site/src/figures/sine-with-ticks.json");

#[test]
fn 目盛つきの図は_軸ごとに目盛の線と名前を持つ() {
    let figure = figure_of(SINE_WITH_TICKS);
    let paths = figure
        .items
        .iter()
        .filter(|item| matches!(item, Item::Path(_)))
        .count();
    let names: Vec<&str> = figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(name) => Some(name.tex.as_str()),
            Item::Path(_) | Item::Dot(_) => None,
        })
        .collect();
    // x軸と目盛4本，y軸と目盛2本，sin x．
    assert_eq!(paths, 1 + 4 + 1 + 2 + 1);
    assert_eq!(
        names,
        [
            "$-2\\pi$", "$-\\pi$", "$\\pi$", "$2\\pi$", "$x$", "$1$", "$-1$", "$y$", "O"
        ]
    );
}

#[test]
fn 目盛の名前は_anchorを指定すると_線の端のその向きに置く() {
    let figure = figure_of(&axes_with_ticks(
        r#", "ticks": [ { "at": 1, "label": "1", "anchor": "north west" },
                        { "at": 2, "label": "2", "anchor": "south" } ]"#,
        r#", "ticks": [ { "at": 1, "label": "1", "anchor": "west" } ]"#,
    ));
    let names: Vec<&LabelItem> = figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(name) => Some(name),
            Item::Path(_) | Item::Dot(_) => None,
        })
        .collect();
    assert_eq!(names.len(), 3);
    // 位置は，anchorに関わらず，線の端(軸から3pt離れた所)である．
    assert!(close(names[0].at[0], 1.0) && close(names[0].at[1], -TICK_HALF));
    assert_eq!(names[0].anchor, Anchor::NorthWest);
    assert_eq!(names[1].anchor, Anchor::South);
    assert!(close(names[2].at[0], -TICK_HALF) && close(names[2].at[1], 2.0));
    assert_eq!(names[2].anchor, Anchor::West);
}
