//! ベクトルの式で書いた曲線を確かめる．曲線の`expr`は，成分の式の並びのほか，点の式と同じベクトルの式
//! (媒介変数は数，先に置いた点はベクトル)の文字列で書ける．例：中心C，直交する単位ベクトルU，Vの円は
//! `"C + cos(t)*U + sin(t)*V"`．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Bound, CurveExpr, Object};
use figure::{ErrorKind, parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-3, 3], "y": [-3, 3], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
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

const FRAME: &str = r#"{ "id": "C", "type": "point", "at": [1, 1] },
    { "id": "U", "type": "point", "at": [1, 0] },
    { "id": "V", "type": "point", "at": [0, 1] }"#;

fn circle(extra: &str) -> String {
    format!(
        r#"{FRAME},
           {{ "id": "c", "type": "curve", "var": "t", "expr": "C + cos(t)*U + sin(t)*V",
              "domain": [0, "2*pi"] }} {extra}"#
    )
}

#[test]
fn ベクトルの式の曲線を読み_書き出して読み直すと同じになる() {
    let scene = parse_scene(&plane_scene(&circle(""))).expect("読める");
    let Object::Curve(curve) = &scene.objects[3] else {
        panic!("曲線である");
    };
    assert_eq!(
        curve.expr,
        CurveExpr::Vector("C + cos(t)*U + sin(t)*V".to_owned())
    );
    let written = serde_json::to_string(&scene).expect("書き出せる");
    assert!(
        written.contains(r#""expr":"C + cos(t)*U + sin(t)*V""#),
        "{written}"
    );
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn ベクトルの式の円は_中心と半径のとおりに描く() {
    let figure = render(&parse_scene(&plane_scene(&circle(""))).expect("読める")).expect("描ける");
    let points = &paths(&figure)[0].points;
    assert!(points.len() > 16);
    for [x, y] in points {
        assert!(((x - 1.0).hypot(y - 1.0) - 1.0).abs() < 1e-9, "({x}, {y})");
    }
}

#[test]
fn ベクトルの式の曲線にも_接線を引ける() {
    // t = 0の点(2, 1)で，円の接線はx = 2の縦の直線である．
    let figure = render(
        &parse_scene(&plane_scene(&circle(
            r#", { "id": "l", "type": "tangent_line", "of": "c", "at": 0 }"#,
        )))
        .expect("読める"),
    )
    .expect("描ける");
    let tangent = paths(&figure)[1];
    assert!(
        tangent.points.iter().all(|[x, _]| (x - 2.0).abs() < 1e-6),
        "{:?}",
        tangent.points
    );
}

#[test]
fn 空間の曲線もベクトルの式で書ける() {
    // 真上から見ると，z = 1の面の半径1の円は，半径1の円に見える．
    let json = r#"{ "version": "0.1.0", "description": "試験の図",
             "view": { "azimuth": 90, "elevation": 90, "unit": "1cm" },
             "objects": [
               { "id": "C", "type": "point", "at": [0, 0, 1] },
               { "id": "U", "type": "point", "at": [1, 0, 0] },
               { "id": "V", "type": "point", "at": [0, 1, 0] },
               { "id": "c", "type": "curve", "var": "t", "expr": "C + cos(t)*U + sin(t)*V",
                 "domain": [0, "2*pi"] }] }"#;
    let figure = render(&parse_scene(json).expect("読める")).expect("描ける");
    let points: Vec<[f64; 2]> = paths(&figure)
        .iter()
        .flat_map(|path| path.points.clone())
        .collect();
    assert!(points.len() > 16);
    for [x, y] in points {
        assert!((x.hypot(y) - 1.0).abs() < 1e-9, "({x}, {y})");
    }
}

#[test]
fn 値が点にならない式は_誤りになる() {
    let error = parse_scene(&plane_scene(&format!(
        r#"{FRAME}, {{ "id": "c", "type": "curve", "var": "t", "expr": "norm(C) * t",
              "domain": [0, 1] }}"#
    )))
    .expect_err("誤りになる");
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert_eq!(error.object.as_deref(), Some("c"));
    let error = parse_scene(&plane_scene(&format!(
        r#"{FRAME}, {{ "id": "c", "type": "curve", "var": "t", "expr": "C * U",
              "domain": [0, 1] }}"#
    )))
    .expect_err("誤りになる");
    assert!(error.to_string().contains("dot"), "{error}");
}

#[test]
fn 成分の並びで書く曲線は_これまでどおりである() {
    let scene = parse_scene(&plane_scene(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t^2"], "domain": [0, 1] }"#,
    ))
    .expect("読める");
    let Object::Curve(curve) = &scene.objects[0] else {
        panic!("曲線である");
    };
    assert_eq!(
        curve.expr,
        CurveExpr::Components(vec![
            Bound::Expression("t".to_owned()),
            Bound::Expression("t^2".to_owned())
        ])
    );
}
