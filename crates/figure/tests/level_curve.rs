//! 等値線(`level_curve`)を確かめる．2つの変数の関数`level`が値`values`になる所を，写像`expr`で空間へ移した曲線である．
//! 4次元の曲面の超平面による切り口(時刻tの姿)や，曲面の等高線に使う．平面の図では，写像`expr`を平面への
//! 2つの式にして，陰関数の曲線や，斜交座標で書いた直線の族を描く．

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
fn 値は_1つ以上要る() {
    let error = error_of(&planar("x^2 + y^2", "[]", "0", ""));
    assert!(error.to_string().contains("values"), "{error}");
    assert_eq!(error.object.as_deref(), Some("c"));
}

#[test]
fn 空間の図の写像は3つの式で_変数は2つの名前で書く() {
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

/// 平面の図．x，yとも-3から3まで，1の長さを`unit`にする．
fn plane_scene(objects: &str, unit: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-3, 3], "y": [-3, 3], "unit": {{ "x": "{unit}", "y": "{unit}" }} }},
             "objects": [{objects}] }}"#
    )
}

fn plane_figure_of(objects: &str, unit: &str) -> Figure {
    render(&parse_scene(&plane_scene(objects, unit)).expect("読める")).expect("描画できる")
}

/// 平面を，そのまま写す等値線(陰関数の曲線)．
fn implicit(level: &str, values: &str, extra: &str) -> String {
    format!(
        r#"{{ "id": "c", "type": "level_curve", "vars": ["x", "y"],
              "expr": ["x", "y"], "domain": [[-4, 4], [-4, 4]],
              "level": "{level}", "values": {values} {extra} }}"#
    )
}

#[test]
fn 平面の図では_陰関数の曲線を描く() {
    let figure = plane_figure_of(&implicit("x^2 + y^2", "[1, 4]", ""), "1cm");
    let lines = paths(&figure);
    assert_eq!(lines.len(), 2, "値ごとに閉じた1本の線");
    for (line, radius) in lines.iter().zip([1.0, 2.0]) {
        assert!(line.points.len() > 16, "曲がりに合わせて点がある");
        for [x, y] in &line.points {
            assert!(
                (x.hypot(*y) - radius).abs() < 4e-3,
                "({x}, {y})は半径{radius}の円の上"
            );
        }
        assert_eq!(line.stroke.line, Line::Solid);
    }
}

#[test]
fn 平面の等値線は_単位の長さで拡大する() {
    let figure = plane_figure_of(&implicit("x^2 + y^2", "[1]", ""), "2cm");
    for [x, y] in &paths(&figure)[0].points {
        assert!(
            (x.hypot(*y) - 2.0).abs() < 8e-3,
            "({x}, {y})は半径2cmの円の上"
        );
    }
}

#[test]
fn 平面の等値線は_見える範囲で切り取る() {
    // 半径4の円は，見える範囲[-3, 3]の角だけに掛かる．
    let figure = plane_figure_of(&implicit("x^2 + y^2", "[16]", ""), "1cm");
    let lines = paths(&figure);
    assert!(!lines.is_empty());
    for [x, y] in lines.iter().flat_map(|line| &line.points) {
        assert!(
            x.abs() <= 3.0 + 1e-9 && y.abs() <= 3.0 + 1e-9,
            "({x}, {y})は範囲の中"
        );
    }
}

#[test]
fn 平面の写像で_斜交座標の直線の族を描く() {
    // 基本ベクトル(1, 0)と(1/2, 1)の格子の座標(u, v)で，u = nの線は，x - y/2 = nの直線である．
    let figure = plane_figure_of(
        r#"{ "id": "c", "type": "level_curve", "vars": ["u", "v"],
             "expr": ["u + v/2", "v"], "domain": [[-6, 6], [-4, 4]],
             "level": "u", "values": [-1, 0, 1] }"#,
        "1cm",
    );
    let lines = paths(&figure);
    assert_eq!(lines.len(), 3);
    for (line, n) in lines.iter().zip([-1.0, 0.0, 1.0]) {
        for [x, y] in &line.points {
            assert!(
                (x - y / 2.0 - n).abs() < 1e-6,
                "({x}, {y})はx - y/2 = {n}の上"
            );
        }
        let (first, last) = (line.points[0], line.points[line.points.len() - 1]);
        assert!(
            (first[1] - last[1]).abs() > 5.9,
            "見える範囲の上下をまたぐ: {first:?} {last:?}"
        );
    }
}

#[test]
fn 平面の等値線を変換で動かせる() {
    let figure = plane_figure_of(
        &implicit(
            "x^2 + y^2",
            "[1]",
            r#", "transform": [{ "translate": [1, 0] }]"#,
        ),
        "1cm",
    );
    for [x, y] in &paths(&figure)[0].points {
        assert!(
            ((x - 1.0).hypot(*y) - 1.0).abs() < 4e-3,
            "({x}, {y})は中心(1, 0)の円の上"
        );
    }
}

#[test]
fn 平面の図の写像は2つの式で書く() {
    let error = parse_scene(&plane_scene(
        r#"{ "id": "c", "type": "level_curve", "vars": ["x", "y"], "expr": ["x", "y", "0"],
             "domain": [[0, 1], [0, 1]], "level": "x", "values": [0.5] }"#,
        "1cm",
    ))
    .expect_err("誤りになる");
    assert!(
        matches!(
            error.kind,
            ErrorKind::ExpressionCount {
                expected: 2,
                found: 3
            }
        ),
        "{error}"
    );
}
