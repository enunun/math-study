//! 曲面の平面による切り口(`cut`)を確かめる．切り口は，曲面の網の上で，平面の式の値が0になる所を結んだ線で，
//! 曲面に隠れる部分は，隠れた部分の線で描く．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Color, Line, Object};
use figure::{Error, ErrorKind, parse_scene, render};

fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
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

const SPHERE: &str = r#"{ "id": "s", "type": "surface", "vars": ["u", "v"],
    "expr": ["2*sin(u)*cos(v)", "2*sin(u)*sin(v)", "2*cos(u)"],
    "domain": [[0, "pi"], [0, "2*pi"]] }"#;

/// 頂点が原点の円錐．高さ2まで．
const CONE: &str = r#"{ "id": "cone", "type": "surface", "vars": ["r", "t"],
    "expr": ["r*cos(t)", "r*sin(t)", "r"], "domain": [[0, 2], [0, "2*pi"]] }"#;

fn cut(surface: &str, normal: &str, offset: &str, extra: &str) -> String {
    format!(
        r#"{{ "id": "c", "type": "cut", "surface": "{surface}", "normal": {normal},
              "offset": {offset} {extra} }}"#
    )
}

fn project(p: [f64; 3]) -> [f64; 2] {
    let (a, e) = (60.0_f64.to_radians(), 20.0_f64.to_radians());
    let right = [-a.sin(), a.cos(), 0.0];
    let up = [-e.sin() * a.cos(), -e.sin() * a.sin(), e.cos()];
    let dot = |u: [f64; 3]| u[0] * p[0] + u[1] * p[1] + u[2] * p[2];
    [dot(right), dot(up)]
}

/// 円(中心(0, 0, height)，半径radius)の点の，画面の位置．
fn circle_points(radius: f64, height: f64) -> Vec<[f64; 2]> {
    (0..4000)
        .map(|k| {
            let t = f64::from(k) / 4000.0 * std::f64::consts::TAU;
            project([radius * t.cos(), radius * t.sin(), height])
        })
        .collect()
}

fn near_any(point: [f64; 2], set: &[[f64; 2]], tolerance: f64) -> bool {
    set.iter()
        .any(|q| (point[0] - q[0]).hypot(point[1] - q[1]) < tolerance)
}

// ---- 読み込みと検査 ----

#[test]
fn 切り口は_曲面のidと平面の法線と定数で書く() {
    let scene = parse_scene(&space_scene(&format!(
        r#"{{ "id": "k", "type": "parameter", "value": 1 }}, {SPHERE},
           {}"#,
        cut("s", "[0, 0, 1]", r#""k / 2""#, "")
    )))
    .expect("読める");
    let Object::Cut(cut) = &scene.objects[2] else {
        panic!("切り口である");
    };
    assert_eq!(cut.surface, "s");
    assert_eq!(scene.objects[2].type_name(), "cut");
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 存在しない曲面を指すと_誤りになる() {
    let error = error_of(&cut("nothing", "[0, 0, 1]", "0", ""));
    assert_eq!(error.kind, ErrorKind::UnknownSurface("nothing".to_owned()));
    assert_eq!(error.object.as_deref(), Some("c"));
    assert_eq!(error.kind.code(), "unknown_surface");
    // 曲面でないオブジェクトは，指せない．
    let ball = error_of(&format!(
        r#"{{ "id": "b", "type": "sphere", "center": [0, 0, 0], "radius": 1 }}, {}"#,
        cut("b", "[0, 0, 1]", "0", "")
    ));
    assert_eq!(ball.kind, ErrorKind::UnknownSurface("b".to_owned()));
}

#[test]
fn 法線は_3個の式で_0でない有限のベクトルでなければならない() {
    let two = error_of(&format!("{SPHERE}, {}", cut("s", "[0, 1]", "0", "")));
    assert!(matches!(two.kind, ErrorKind::Invalid(_)), "{two}");
    assert!(two.to_string().contains("normal"), "{two}");
    let zero = error_of(&format!("{SPHERE}, {}", cut("s", "[0, 0, 0]", "0", "")));
    assert!(matches!(zero.kind, ErrorKind::Invalid(_)), "{zero}");
    assert_eq!(zero.object.as_deref(), Some("c"));
    // 式の誤りは，項目の名前と番号を示す．
    let bad = error_of(&format!(
        "{SPHERE}, {}",
        cut("s", r#"[0, 0, "1 +"]"#, "0", "")
    ));
    let ErrorKind::Expression { field, index, .. } = &bad.kind else {
        panic!("式の誤りである: {bad}");
    };
    assert_eq!((*field, *index), ("normal", 2));
}

#[test]
fn 平面の図では_切り口は使えない() {
    let error = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "x": [0, 1], "y": [0, 1], "unit": { "x": "1cm", "y": "1cm" } },
             "objects": [ { "id": "c", "type": "cut", "surface": "s", "normal": [0, 0, 1], "offset": 0 } ] }"#,
    )
    .expect_err("誤りになる");
    assert!(error.to_string().contains("平面"), "{error}");
}

// ---- 描画 ----

#[test]
fn 球の水平な切り口は_円で_画面では中心と半径の合う楕円になる() {
    let figure = figure_of(&format!("{SPHERE}, {}", cut("s", "[0, 0, 1]", "1", "")));
    // 切り口の円は，高さ1で，半径sqrt(3)である．曲面の輪郭は，別に1本ある．
    let expected = circle_points(3.0_f64.sqrt(), 1.0);
    let cuts: Vec<&Path> = paths(&figure).into_iter().skip(1).collect();
    assert!(!cuts.is_empty());
    for path in &cuts {
        for point in &path.points {
            assert!(near_any(*point, &expected, 0.02), "{point:?}");
        }
    }
    // 切り口は，円を(遠い側の点線も含めて)ほぼ一周する．円の点から，折れ線までの距離を見る．
    let covered = expected
        .iter()
        .step_by(40)
        .filter(|q| {
            cuts.iter().any(|path| {
                path.points
                    .windows(2)
                    .any(|pair| segment_distance(**q, pair[0], pair[1]) < 0.03)
            })
        })
        .count();
    assert!(covered >= 95, "一周していない: {covered}");
}

/// 点から，線分までの距離．
fn segment_distance(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let squared = dx * dx + dy * dy;
    let along = if squared == 0.0 {
        0.0
    } else {
        (((point[0] - a[0]) * dx + (point[1] - a[1]) * dy) / squared).clamp(0.0, 1.0)
    };
    (point[0] - (a[0] + along * dx)).hypot(point[1] - (a[1] + along * dy))
}

#[test]
fn 切り口の遠い側は_解析的な球の上の円と同じ長さの点線になる() {
    let circle = r#"{ "id": "ring", "type": "curve", "var": "t",
        "expr": ["sqrt(3)*cos(t)", "sqrt(3)*sin(t)", "1"], "domain": [0, "2*pi"] }"#;
    let ball = r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2 }"#;
    let reference = figure_of(&format!("{circle}, {ball}"));
    let mine = figure_of(&format!("{SPHERE}, {}", cut("s", "[0, 0, 1]", "1", "")));
    let dotted_length = |figure: &Figure| -> f64 {
        paths(figure)
            .iter()
            .filter(|path| path.stroke.line == Line::Dotted)
            .map(|path| {
                path.points
                    .windows(2)
                    .map(|pair| (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]))
                    .sum::<f64>()
            })
            .sum()
    };
    let (a, b) = (dotted_length(&mine), dotted_length(&reference));
    assert!(b > 2.0, "{b}");
    assert!((a - b).abs() < 0.15, "{a} {b}");
}

#[test]
fn 円錐の水平な切り口は_円になる() {
    let figure = figure_of(&format!("{CONE}, {}", cut("cone", "[0, 0, 1]", "1", "")));
    let expected = circle_points(1.0, 1.0);
    for path in paths(&figure).iter().skip(1) {
        for point in &path.points {
            assert!(near_any(*point, &expected, 0.02), "{point:?}");
        }
    }
}

#[test]
fn 平面が曲面と交わらなければ_何も描かない() {
    let with = figure_of(&format!("{SPHERE}, {}", cut("s", "[0, 0, 1]", "5", "")));
    let without = figure_of(SPHERE);
    assert_eq!(with.items.len(), without.items.len());
}

#[test]
fn 傾いた平面の切り口は_平面の上にある() {
    // 平面 x + y + z = 1 と球の交わりの円．点は，画面の位置から空間へ戻せないので，
    // 中心の投影と，半径の範囲だけを確かめる．
    let figure = figure_of(&format!("{SPHERE}, {}", cut("s", "[1, 1, 1]", "1", "")));
    let radius = (4.0 - 1.0 / 3.0_f64).sqrt();
    let center = project([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
    for path in paths(&figure).iter().skip(1) {
        for point in &path.points {
            let distance = (point[0] - center[0]).hypot(point[1] - center[1]);
            // 画面の楕円の，半径は，最小0，最大radiusである．
            assert!(distance <= radius + 0.02, "{point:?} {distance}");
        }
    }
    assert!(paths(&figure).len() >= 2);
}

#[test]
fn 切り口の色と太さと線の種類と隠れた部分は_styleで選べる() {
    let styled = figure_of(&format!(
        "{SPHERE}, {}",
        cut(
            "s",
            "[0, 0, 1]",
            "1",
            r#", "style": { "color": "red", "width": "1.2pt", "line": "dashed", "hidden": "none" }"#
        )
    ));
    let cuts: Vec<&Path> = paths(&styled).into_iter().skip(1).collect();
    assert!(!cuts.is_empty());
    for path in cuts {
        assert_eq!(path.stroke.color, Some(Color::Red));
        assert!((path.stroke.width - 1.2).abs() < 1e-9);
        // 隠れた部分は描かず，見える部分だけが，指定した線の種類になる．
        assert_eq!(path.stroke.line, Line::Dashed);
    }
}

#[test]
fn 切り口の位置は_媒介変数で動かせる() {
    let make = |value: f64| {
        figure_of(&format!(
            r#"{{ "id": "h", "type": "parameter", "value": {value} }}, {SPHERE},
               {}"#,
            cut("s", "[0, 0, 1]", r#""h""#, "")
        ))
    };
    let low = make(0.5);
    let high = make(1.5);
    let top = |figure: &Figure| {
        paths(figure)
            .iter()
            .skip(1)
            .flat_map(|path| path.points.iter().map(|p| p[1]))
            .fold(f64::NEG_INFINITY, f64::max)
    };
    assert!(top(&high) > top(&low) + 0.5);
}
