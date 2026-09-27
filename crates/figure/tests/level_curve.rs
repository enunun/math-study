//! 等値線(`level_curve`)を確かめる．2つの変数の関数`level`が値`values`になる所を，写像`expr`で空間へ移した曲線である．
//! 4次元の曲面の超平面による切り口(時刻tの姿)や，曲面の等高線に使う．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Bound, Line, Object};
use figure::{Error, ErrorKind, parse_scene, render};

/// 真上から見る図．画面の座標は，(y, -x)ではなく，方位角90度なので(x, y)を回した位置になる．
fn top_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 90, "elevation": 90, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn error_of(objects: &str) -> Error {
    parse_scene(&top_scene(objects)).expect_err("誤りになる")
}

fn figure_of(objects: &str) -> Figure {
    render(&parse_scene(&top_scene(objects)).expect("読める")).expect("描画できる")
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

/// xy平面(z = height)を，そのまま写す等値線．`level`はxとyの式である．
fn planar(level: &str, values: &str, height: &str, extra: &str) -> String {
    format!(
        r#"{{ "id": "c", "type": "level_curve", "vars": ["x", "y"],
              "expr": ["x", "y", "{height}"], "domain": [[-3, 3], [-3, 3]],
              "level": "{level}", "values": {values} {extra} }}"#
    )
}

#[test]
fn 等値線を読み_書き出して読み直すと同じになる() {
    let scene = parse_scene(&top_scene(&planar("x^2 + y^2", r#"[1, "t"]"#, "0", "")))
        .expect_err("tがないので誤りになる");
    assert!(
        matches!(scene.kind, ErrorKind::Expression { .. }),
        "{scene}"
    );
    let parsed = parse_scene(&top_scene(&format!(
        r#"{{ "id": "t", "type": "parameter", "value": 2 }}, {}"#,
        planar("x^2 + y^2", r#"[1, "t"]"#, "0", "")
    )))
    .expect("読める");
    let Object::LevelCurve(curve) = &parsed.objects[1] else {
        panic!("等値線である");
    };
    assert_eq!(curve.level, "x^2 + y^2");
    assert_eq!(
        curve.values,
        [Bound::Number(1.0), Bound::Expression("t".to_owned())]
    );
    assert_eq!(parsed.objects[1].type_name(), "level_curve");
    let written = serde_json::to_string(&parsed).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), parsed);
}

#[test]
fn 円の等値線は_その半径の円になる() {
    let figure = figure_of(&planar("x^2 + y^2", "[1]", "0", ""));
    let lines = paths(&figure);
    assert_eq!(lines.len(), 1, "閉じた1本の線");
    let line = lines[0];
    assert!(line.points.len() > 16, "曲がりに合わせて点がある");
    for [x, y] in &line.points {
        assert!(
            (x.hypot(*y) - 1.0).abs() < 2e-3,
            "({x}, {y})は半径1の円の上"
        );
    }
    // 閉じている．
    let (first, last) = (line.points[0], line.points[line.points.len() - 1]);
    assert!((first[0] - last[0]).hypot(first[1] - last[1]) < 1e-6);
    assert_eq!(line.stroke.line, Line::Solid);
}

#[test]
fn 値を並べると_値ごとに線を引く() {
    let figure = figure_of(&planar("x^2 + y^2", "[1, 4]", "0", ""));
    let lines = paths(&figure);
    assert_eq!(lines.len(), 2);
    for (line, radius) in lines.iter().zip([1.0, 2.0]) {
        for [x, y] in &line.points {
            assert!((x.hypot(*y) - radius).abs() < 4e-3);
        }
    }
}

#[test]
fn 値に届かない等値線は_何も描かない() {
    let figure = figure_of(&planar("x^2 + y^2", "[-1]", "0", ""));
    assert!(paths(&figure).is_empty());
}

#[test]
fn 値と式は_媒介変数を使える() {
    let figure = figure_of(&format!(
        r#"{{ "id": "t", "type": "parameter", "value": 1.5, "range": [0, 2] }},
           {}"#,
        planar("x^2 + y^2", r#"["t^2"]"#, "t", "")
    ));
    let lines = paths(&figure);
    assert_eq!(lines.len(), 1);
    for [x, y] in &lines[0].points {
        assert!((x.hypot(*y) - 1.5).abs() < 4e-3);
    }
}

#[test]
fn 等値線は_不透明な曲面に隠れる() {
    // 半径1の球の下(z = -2)の，半径0.5の円は，真上から見ると，すべて球に隠れる．
    let figure = figure_of(&format!(
        r#"{{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 1 }},
           {}"#,
        planar("x^2 + y^2", "[0.25]", "-2", "")
    ));
    let lines: Vec<&Path> = paths(&figure)
        .into_iter()
        .filter(|path| path.points.iter().all(|[x, y]| x.hypot(*y) < 0.6))
        .collect();
    assert!(!lines.is_empty());
    assert!(lines.iter().all(|path| path.stroke.line == Line::Dotted));
}

#[test]
fn 等値線自身は_ほかの線を隠さない() {
    // 等値線の写像が作る面は，描かないので，下にある軸を隠さない．
    let figure = figure_of(&format!(
        r#"{{ "id": "x_axis", "type": "axis", "direction": "x", "range": [-1, 1] }},
           {}"#,
        planar("x^2 + y^2", "[1]", "1", "")
    ));
    let axis = paths(&figure)[0];
    assert_eq!(axis.stroke.line, Line::Solid);
    assert_eq!(paths(&figure).len(), 2);
}

#[test]
fn 変換で動かせる() {
    let figure = figure_of(&planar(
        "x^2 + y^2",
        "[1]",
        "0",
        r#", "transform": [{ "translate": [2, 0, 0] }]"#,
    ));
    let line = paths(&figure)[0];
    let xs: Vec<f64> = line.points.iter().map(|p| p[0]).collect();
    let ys: Vec<f64> = line.points.iter().map(|p| p[1]).collect();
    let center = |values: &[f64]| {
        let low = values.iter().copied().fold(f64::INFINITY, f64::min);
        let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        f64::midpoint(low, high)
    };
    // 真上から見た画面では，x方向の平行移動は，画面の縦か横の平行移動になる．
    let moved = (center(&xs).abs() - 2.0).abs() < 1e-2 || (center(&ys).abs() - 2.0).abs() < 1e-2;
    assert!(moved, "{} {}", center(&xs), center(&ys));
}

#[test]
fn 平面の図では使えない() {
    let error = parse_scene(&format!(
        r#"{{ "version": "0.1.0", "description": "a",
             "view": {{ "x": [-1, 1], "y": [-1, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{}] }}"#,
        planar("x^2 + y^2", "[1]", "0", "")
    ))
    .expect_err("誤りになる");
    assert!(error.to_string().contains("level_curve"), "{error}");
}

#[test]
fn 値は_1つ以上要る() {
    let error = error_of(&planar("x^2 + y^2", "[]", "0", ""));
    assert!(error.to_string().contains("values"), "{error}");
    assert_eq!(error.object.as_deref(), Some("c"));
}

#[test]
fn 写像は3つの式で_変数は2つの名前で書く() {
    let error = error_of(
        r#"{ "id": "c", "type": "level_curve", "vars": ["x", "y"], "expr": ["x", "y"],
             "domain": [[0, 1], [0, 1]], "level": "x", "values": [0.5] }"#,
    );
    assert!(
        matches!(
            error.kind,
            ErrorKind::ExpressionCount {
                expected: 3,
                found: 2
            }
        ),
        "{error}"
    );
    let error = error_of(
        r#"{ "id": "c", "type": "level_curve", "vars": ["x"], "expr": ["x", "x", "x"],
             "domain": [[0, 1], [0, 1]], "level": "x", "values": [0.5] }"#,
    );
    assert!(error.to_string().contains("vars"), "{error}");
}

#[test]
fn 関数の式の誤りは_項目の名前を示す() {
    let error = error_of(&planar("x^2 +", "[1]", "0", ""));
    let ErrorKind::Expression { field, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!(*field, "level");
    let error = error_of(&planar("x", r#"["1/"]"#, "0", ""));
    let ErrorKind::Expression { field, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!(*field, "values");
}

#[test]
fn 未知の項目は誤りになる() {
    let error = error_of(&planar("x", "[0]", "0", r#", "value": 1"#));
    assert!(error.to_string().contains("value"), "{error}");
}
