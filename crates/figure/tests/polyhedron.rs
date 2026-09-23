//! 正多面体(`polyhedron`)．種類と中心と半径(中心から頂点までの距離)だけで書き，頂点と面はエンジンが
//! 決める．描くときは，同じ頂点と面を持つ複体(`complex`)と同じになる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    clippy::arithmetic_side_effects
)]

use std::collections::BTreeMap;

use figure::scene::{Complex, Object, Polyhedron};
use figure::tikz::export_tikz;
use figure::{ErrorKind, parse_scene};

fn space_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
             "objects": [{objects}] }}"#
    )
}

fn polyhedron_of(solid: &str, center: [f64; 3], radius: f64) -> Polyhedron {
    let [x, y, z] = center;
    let objects = format!(
        r#"{{ "id": "p", "type": "polyhedron", "solid": "{solid}",
              "center": [{x}, {y}, {z}], "radius": {radius} }}"#
    );
    let scene = parse_scene(&space_scene(&objects)).expect("読める");
    match scene.objects.into_iter().next() {
        Some(Object::Polyhedron(polyhedron)) => polyhedron,
        other => panic!("正多面体ではない：{other:?}"),
    }
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

/// 面の法線(最初の3頂点から求める)．頂点が外から見て反時計回りなら，外を向く．
fn normal(complex: &Complex, face: &[usize]) -> [f64; 3] {
    let [a, b, c] = [0, 1, 2].map(|k| complex.vertices[face[k]]);
    cross(sub(b, a), sub(c, a))
}

/// 向き付きの稜(始点，終点)ごとの，現れる回数．
fn directed_edges(complex: &Complex) -> BTreeMap<(usize, usize), usize> {
    let mut edges = BTreeMap::new();
    for face in &complex.faces {
        for k in 0..face.len() {
            *edges
                .entry((face[k], face[(k + 1) % face.len()]))
                .or_insert(0) += 1;
        }
    }
    edges
}

const SOLIDS: [(&str, usize, usize, usize); 5] = [
    ("tetrahedron", 4, 4, 3),
    ("cube", 8, 6, 4),
    ("octahedron", 6, 8, 3),
    ("dodecahedron", 20, 12, 5),
    ("icosahedron", 12, 20, 3),
];

const CENTER: [f64; 3] = [1.0, -2.0, 0.5];
const RADIUS: f64 = 1.5;
const TOLERANCE: f64 = 1e-9;

#[test]
fn 頂点と面の数は_正多面体の種類で決まる() {
    for (solid, vertices, faces, sides) in SOLIDS {
        let complex = polyhedron_of(solid, CENTER, RADIUS).to_complex();
        assert_eq!(complex.vertices.len(), vertices, "{solid}");
        assert_eq!(complex.faces.len(), faces, "{solid}");
        assert!(
            complex.faces.iter().all(|face| face.len() == sides),
            "{solid}"
        );
    }
}

#[test]
fn どの頂点も_中心から半径の距離にある() {
    for (solid, ..) in SOLIDS {
        let complex = polyhedron_of(solid, CENTER, RADIUS).to_complex();
        for vertex in &complex.vertices {
            assert!(
                (norm(sub(*vertex, CENTER)) - RADIUS).abs() < TOLERANCE,
                "{solid}"
            );
        }
    }
}

#[test]
fn 稜の長さはすべて等しく_オイラーの公式を満たす() {
    for (solid, vertices, faces, _) in SOLIDS {
        let complex = polyhedron_of(solid, CENTER, RADIUS).to_complex();
        let edges = directed_edges(&complex);
        // 向き付きの稜は，1本の稜につき2本ある．
        let edge_count = edges.len() / 2;
        assert_eq!(vertices + faces, edge_count + 2, "{solid}");
        let lengths: Vec<f64> = edges
            .keys()
            .map(|&(a, b)| norm(sub(complex.vertices[a], complex.vertices[b])))
            .collect();
        assert!(
            lengths
                .iter()
                .all(|length| (length - lengths[0]).abs() < TOLERANCE),
            "{solid}"
        );
    }
}

#[test]
fn どの稜も_向きが逆の2つの面にちょうど1回ずつ現れる() {
    for (solid, ..) in SOLIDS {
        let complex = polyhedron_of(solid, CENTER, RADIUS).to_complex();
        let edges = directed_edges(&complex);
        for (&(a, b), &count) in &edges {
            assert_eq!(count, 1, "{solid}：稜({a}，{b})が同じ向きで2回現れる");
            assert!(
                edges.contains_key(&(b, a)),
                "{solid}：稜({a}，{b})の逆向きがない"
            );
        }
    }
}

#[test]
fn 面は平らで_外から見て反時計回りに並ぶ() {
    for (solid, ..) in SOLIDS {
        let complex = polyhedron_of(solid, CENTER, RADIUS).to_complex();
        for face in &complex.faces {
            let n = normal(&complex, face);
            let first = complex.vertices[face[0]];
            // 平らである：どの頂点も，最初の3頂点が決める平面の上にある．
            for &index in face {
                let offset = dot(n, sub(complex.vertices[index], first)) / norm(n);
                assert!(offset.abs() < TOLERANCE, "{solid}：面{face:?}が平らでない");
            }
            // 外向きである：面の外側に，ほかの頂点がない(凸なので，すべて内側にある)．
            for vertex in &complex.vertices {
                assert!(
                    dot(n, sub(*vertex, first)) / norm(n) < TOLERANCE,
                    "{solid}：面{face:?}が内を向く"
                );
            }
        }
    }
}

#[test]
fn 描いた図は_同じ頂点と面の複体と同じになる() {
    for (solid, ..) in SOLIDS {
        let complex = polyhedron_of(solid, CENTER, RADIUS).to_complex();
        let polyhedron_scene = format!(
            r#"{{ "id": "p", "type": "polyhedron", "solid": "{solid}",
                  "center": [1, -2, 0.5], "radius": 1.5 }}"#
        );
        let complex_scene = format!(
            r#"{{ "id": "p", "type": "complex", "vertices": {}, "faces": {} }}"#,
            serde_json::to_string(&complex.vertices).unwrap(),
            serde_json::to_string(&complex.faces).unwrap(),
        );
        // JSONを経た座標は最後の桁がずれうるので，丸めて書き出したTikZで比べる．
        assert_eq!(
            export_tikz(&space_scene(&polyhedron_scene)).expect("描画できる"),
            export_tikz(&space_scene(&complex_scene)).expect("描画できる"),
            "{solid}"
        );
    }
}

#[test]
fn 正多面体は空間の図でだけ使える() {
    let scene = r#"{ "version": "0.1.0", "description": "試験の図",
        "view": { "x": [-2, 2], "y": [-2, 2], "unit": "1cm" },
        "objects": [{ "id": "p", "type": "polyhedron", "solid": "cube",
                      "center": [0, 0, 0], "radius": 1 }] }"#;
    let error = parse_scene(scene).expect_err("誤りになる");
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error:?}");
}

#[test]
fn 半径が0以下なら誤りになる() {
    for radius in ["0", "-1"] {
        let objects = format!(
            r#"{{ "id": "p", "type": "polyhedron", "solid": "cube",
                  "center": [0, 0, 0], "radius": {radius} }}"#
        );
        parse_scene(&space_scene(&objects)).expect_err("誤りになる");
    }
}

#[test]
fn 知らない種類の正多面体は誤りになる() {
    let objects = r#"{ "id": "p", "type": "polyhedron", "solid": "prism",
                       "center": [0, 0, 0], "radius": 1 }"#;
    parse_scene(&space_scene(objects)).expect_err("誤りになる");
}
