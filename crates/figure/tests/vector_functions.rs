//! 点の式のベクトルの関数(`dot`，`cross`，`norm`)を確かめる．点の式は，点をベクトルのまま評価するので，
//! 内積，外積，長さが書ける．平面の図の外積は，数(z成分)である．点の式のほかでは使えない．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use figure::expr::ExprErrorKind;
use figure::figure::{Figure, Item};
use figure::{Error, ErrorKind, parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-5, 5], "y": [-5, 5], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

/// 空間の図(方位角60度，仰角20度)．
fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn dots(figure: &Figure) -> Vec<[f64; 2]> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Dot(dot) => Some(dot.at),
            _ => None,
        })
        .collect()
}

fn plane_dot(objects: &str) -> [f64; 2] {
    let figure = render(&parse_scene(&plane_scene(objects)).expect("読める")).expect("描ける");
    dots(&figure)[0]
}

fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

const B: &str = r#"{ "id": "B", "type": "point", "at": [3, 1] }"#;
const C: &str = r#"{ "id": "C", "type": "point", "at": [1, 3] }"#;

fn plane_error(at: &str) -> Error {
    parse_scene(&plane_scene(&format!(
        r#"{B}, {C}, {{ "id": "D", "type": "point", "at": "{at}" }}"#
    )))
    .expect_err("誤りになる")
}

#[test]
fn 内積と長さは数で_点の数倍に使える() {
    // B・C = 6，|B| = √10．
    let at = plane_dot(&format!(
        r#"{B}, {C}, {{ "id": "D", "type": "point", "at": "dot(B, C) / 12 * B", "dot": true }}"#
    ));
    assert!(close(at, [1.5, 0.5]), "{at:?}");
    let at = plane_dot(&format!(
        r#"{B}, {{ "id": "D", "type": "point", "at": "B / norm(B)", "dot": true }}"#
    ));
    let length = 10.0_f64.sqrt();
    assert!(close(at, [3.0 / length, 1.0 / length]), "{at:?}");
}

#[test]
fn 平面の外積は_z成分の数である() {
    // B × C = 3・3 - 1・1 = 8．
    let at = plane_dot(&format!(
        r#"{B}, {C}, {{ "id": "D", "type": "point", "at": "cross(B, C) / 8 * B", "dot": true }}"#
    ));
    assert!(close(at, [3.0, 1.0]), "{at:?}");
}

#[test]
fn ベクトルの関数の値は_数の関数にも入れられる() {
    // B・C = 6なので，sqrt(B・C - 2) = 2．点は見える範囲[-5, 5]^2に収める．
    let at = plane_dot(&format!(
        r#"{B}, {C}, {{ "id": "D", "type": "point", "at": "sqrt(dot(B, C) - 2) * C / 4", "dot": true }}"#
    ));
    assert!(close(at, [0.5, 1.5]), "{at:?}");
}

#[test]
fn 空間の外積はベクトルで_逆格子の基本ベクトルが書ける() {
    // b_1 = (a_2 × a_3) / (a_1・(a_2 × a_3))．a_1・b_1 = 1，a_2・b_1 = a_3・b_1 = 0を，あとの点の座標で確かめる．
    let scene = parse_scene(&space_scene(
        r#"{ "id": "A1", "type": "point", "at": [2, 0, 0] },
           { "id": "A2", "type": "point", "at": [1, 3, 0] },
           { "id": "A3", "type": "point", "at": [0.5, 1, 4] },
           { "id": "B1", "type": "point", "at": "cross(A2, A3) / dot(A1, cross(A2, A3))" },
           { "id": "P", "type": "point", "dot": true,
             "at": ["A1_x*B1_x + A1_y*B1_y + A1_z*B1_z",
                    "A2_x*B1_x + A2_y*B1_y + A2_z*B1_z",
                    "A3_x*B1_x + A3_y*B1_y + A3_z*B1_z"] }"#,
    ))
    .expect("読める");
    let figure = render(&scene).expect("描ける");
    // Pの座標(1, 0, 0)を投影した位置．
    let (a, e) = (60.0_f64.to_radians(), 20.0_f64.to_radians());
    let expected = [-a.sin(), -e.sin() * a.cos()];
    assert!(close(dots(&figure)[0], expected), "{:?}", dots(&figure));
}

#[test]
fn 点どうしの積は誤りで_内積と外積を案内する() {
    let error = plane_error("B * C");
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    let message = error.to_string();
    assert!(
        message.contains("dot") && message.contains("cross"),
        "{message}"
    );
}

#[test]
fn ベクトルの関数の引数は点でなければならない() {
    for at in ["dot(B, 1) * C", "norm(2) * C", "cross(1, B) * C"] {
        let error = plane_error(at);
        assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{at}: {error}");
        assert_eq!(error.object.as_deref(), Some("D"), "{at}");
    }
}

#[test]
fn ベクトルの関数の引数の数は決まっている() {
    for (at, name, expected) in [("dot(B) * C", "dot", 2), ("norm(B, C) * C", "norm", 1)] {
        let error = plane_error(at);
        let ErrorKind::Expression { error: inner, .. } = &error.kind else {
            panic!("式の誤りである: {error}");
        };
        assert_eq!(
            inner.kind,
            ExprErrorKind::ArgumentCount {
                name: name.to_owned(),
                expected,
                found: if expected == 2 { 1 } else { 2 },
            },
            "{at}"
        );
    }
}

#[test]
fn 値が数になる点の式は誤りになる() {
    let error = plane_error("dot(B, C)");
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert!(error.to_string().contains("点"), "{error}");
}

#[test]
fn ベクトルの関数は_点の式のほかでは使えない() {
    let error = parse_scene(&plane_scene(
        r#"{ "id": "g", "type": "graph", "var": "x", "expr": "dot(x, x)", "domain": [0, 1] }"#,
    ))
    .expect_err("誤りになる");
    assert!(error.to_string().contains("点の式"), "{error}");
}

#[test]
fn ベクトルの関数の名前は_媒介変数のidにできない() {
    for name in ["dot", "cross", "norm"] {
        let error = parse_scene(&plane_scene(&format!(
            r#"{{ "id": "{name}", "type": "parameter", "value": 1 }}"#
        )))
        .expect_err("誤りになる");
        assert_eq!(error.kind, ErrorKind::ReservedName(name.to_owned()));
    }
}
