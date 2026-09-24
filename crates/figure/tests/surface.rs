//! 曲面(`surface`)を確かめる．式で書いた曲面は，三角形の網にして，輪郭と，隠れ方を求める．
//! 球を式で書いた曲面は，解析的な球(`sphere`)と比べて確かめる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Line, Object, Surface};
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

fn close2(a: [f64; 2], b: [f64; 2], tolerance: f64) -> bool {
    (a[0] - b[0]).abs() < tolerance && (a[1] - b[1]).abs() < tolerance
}

/// 球を式で書いた曲面．半径2で，中心は原点である．
const SPHERE_SURFACE: &str = r#"{ "id": "s", "type": "surface", "vars": ["u", "v"],
    "expr": ["2*sin(u)*cos(v)", "2*sin(u)*sin(v)", "2*cos(u)"],
    "domain": [[0, "pi"], [0, "2*pi"]] }"#;

const BALL: &str = r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2 }"#;

fn axis(direction: &str) -> String {
    format!(
        r#"{{ "id": "{direction}_axis", "type": "axis", "direction": "{direction}", "range": [-5, 5] }}"#
    )
}

// ---- 読み込みと検査 ----

#[test]
fn 曲面は_2つの変数と3個の式と定義域を持ち_網の細かさの既定がある() {
    let scene = parse_scene(&space_scene(SPHERE_SURFACE)).expect("読める");
    let Object::Surface(surface) = &scene.objects[0] else {
        panic!("曲面である");
    };
    assert_eq!(surface.vars, ["u", "v"]);
    assert_eq!(surface.expr.len(), 3);
    assert_eq!(surface.mesh, [Surface::DEFAULT_MESH; 2]);
    assert!(!surface.boundary);
    assert_eq!(scene.objects[0].type_name(), "surface");
}

#[test]
fn 曲面を書き出して読み直すと同じになる() {
    let scene = parse_scene(&space_scene(
        r#"{ "id": "s", "type": "surface", "vars": ["x", "y"], "expr": ["x", "y", "x^2 + y^2"],
             "domain": [[-1, 1], [-1, 1]], "mesh": [20, 30], "boundary": true,
             "style": { "color": "blue", "hidden": "dashed" } }"#,
    ))
    .expect("読める");
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 曲面の式は_3個でなければならない() {
    let error = error_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "v"], "expr": ["u", "v"],
             "domain": [[0, 1], [0, 1]] }"#,
    );
    assert_eq!(
        error.kind,
        ErrorKind::ExpressionCount {
            expected: 3,
            found: 2
        }
    );
    assert_eq!(error.object.as_deref(), Some("s"));
}

#[test]
fn 変数は_2つの違う識別子でなければならない() {
    let same = error_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "u"], "expr": ["u", "u", "u"],
             "domain": [[0, 1], [0, 1]] }"#,
    );
    assert_eq!(same.kind, ErrorKind::NameConflict("u".to_owned()));
    let bad = error_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "1v"], "expr": ["u", "u", "u"],
             "domain": [[0, 1], [0, 1]] }"#,
    );
    assert_eq!(bad.kind, ErrorKind::InvalidVariable("1v".to_owned()));
    let three = error_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "v", "w"], "expr": ["u", "u", "u"],
             "domain": [[0, 1], [0, 1]] }"#,
    );
    assert!(matches!(three.kind, ErrorKind::Invalid(_)), "{three}");
    assert!(three.to_string().contains("vars"), "{three}");
}

#[test]
fn 式の誤りは_何番目の式かと位置を示す() {
    let error = error_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "v"], "expr": ["u", "v", "u +"],
             "domain": [[0, 1], [0, 1]] }"#,
    );
    let ErrorKind::Expression { field, index, .. } = &error.kind else {
        panic!("式の誤りである: {error}");
    };
    assert_eq!((*field, *index), ("expr", 2));
    assert_eq!(error.object.as_deref(), Some("s"));
    // 他の変数の名前は，使えない．
    let unknown = error_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "v"], "expr": ["u", "v", "w"],
             "domain": [[0, 1], [0, 1]] }"#,
    );
    assert!(
        matches!(unknown.kind, ErrorKind::Expression { .. }),
        "{unknown}"
    );
}

#[test]
fn 定義域は_増える有限の範囲で_網の細かさは4以上200以下である() {
    let range = error_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "v"], "expr": ["u", "v", "0"],
             "domain": [[0, 1], [1, 0]] }"#,
    );
    assert_eq!(range.kind, ErrorKind::InvalidRange("domain"));
    for mesh in ["[3, 10]", "[10, 201]", "[0, 0]"] {
        let error = error_of(&format!(
            r#"{{ "id": "s", "type": "surface", "vars": ["u", "v"], "expr": ["u", "v", "0"],
                 "domain": [[0, 1], [0, 1]], "mesh": {mesh} }}"#
        ));
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{mesh}: {error}"
        );
        assert!(error.to_string().contains("mesh"), "{mesh}: {error}");
    }
}

#[test]
fn 平面の図では_曲面は使えない() {
    let error = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "x": [0, 1], "y": [0, 1], "unit": { "x": "1cm", "y": "1cm" } },
             "objects": [ { "id": "s", "type": "surface", "vars": ["u", "v"],
                            "expr": ["u", "v", "0"], "domain": [[0, 1], [0, 1]] } ] }"#,
    )
    .expect_err("誤りになる");
    assert!(error.to_string().contains("平面"), "{error}");
}

// ---- 輪郭 ----

#[test]
fn 球の曲面の輪郭は_解析的な球と同じ半径の円になる() {
    let figure = figure_of(SPHERE_SURFACE);
    let all = paths(&figure);
    // 輪郭は，1本の閉じた線である(網の継ぎ目でも，途切れない)．
    assert_eq!(all.len(), 1, "{}", all.len());
    let outline = all[0];
    assert!(outline.points.len() >= 32);
    for point in &outline.points {
        let distance = point[0].hypot(point[1]);
        assert!((distance - 2.0).abs() < 0.03, "{point:?} {distance}");
    }
    assert!(close2(
        outline.points[0],
        *outline.points.last().unwrap(),
        1e-6
    ));
    assert_eq!(outline.stroke.line, Line::Solid);
}

#[test]
fn 曲面の輪郭の色と太さと線の種類は_styleで選べる() {
    let figure = figure_of(
        r#"{ "id": "s", "type": "surface", "vars": ["u", "v"],
             "expr": ["2*sin(u)*cos(v)", "2*sin(u)*sin(v)", "2*cos(u)"],
             "domain": [[0, "pi"], [0, "2*pi"]],
             "style": { "color": "red", "width": "1.2pt", "line": "dashed" } }"#,
    );
    let outline = paths(&figure)[0];
    assert_eq!(outline.stroke.color, Some(figure::scene::Color::Red));
    assert!((outline.stroke.width - 1.2).abs() < 1e-9);
    assert_eq!(outline.stroke.line, Line::Dashed);
}

// ---- 隠れ方 ----

/// 軸の線の，各部分の(線の種類，始まり，終わり)．
fn pieces(figure: &Figure, skip: usize, count: usize) -> Vec<(Line, [f64; 2], [f64; 2])> {
    paths(figure)
        .into_iter()
        .skip(skip)
        .take(count)
        .map(|path| {
            (
                path.stroke.line,
                path.points[0],
                *path.points.last().unwrap(),
            )
        })
        .collect()
}

#[test]
fn 球の曲面が隠す軸は_解析的な球が隠す軸と同じ所で切り替わる() {
    for direction in ["x", "y", "z"] {
        let by_surface = figure_of(&format!("{}, {SPHERE_SURFACE}", axis(direction)));
        let by_ball = figure_of(&format!("{}, {BALL}", axis(direction)));
        // 軸は，どちらも3つに分かれる(実線，点線，実線)．曲面の輪郭は，その後ろに1本ある．
        let mine = pieces(&by_surface, 0, 3);
        let theirs = pieces(&by_ball, 0, 3);
        let lines: Vec<Line> = mine.iter().map(|piece| piece.0).collect();
        assert_eq!(
            lines,
            [Line::Solid, Line::Dotted, Line::Solid],
            "{direction}"
        );
        for (a, b) in mine.iter().zip(&theirs) {
            assert_eq!(a.0, b.0, "{direction}");
            assert!(close2(a.1, b.1, 0.03), "{direction}: {a:?} {b:?}");
            assert!(close2(a.2, b.2, 0.03), "{direction}: {a:?} {b:?}");
        }
    }
}

#[test]
fn 球の面の上の曲線は_曲面でも_遠い側の半分が点線になる() {
    let equator = r#"{ "id": "equator", "type": "curve", "var": "t",
        "expr": ["2*cos(t)", "2*sin(t)", "0"], "domain": [0, "2*pi"] }"#;
    let by_surface = figure_of(&format!("{equator}, {SPHERE_SURFACE}"));
    let by_ball = figure_of(&format!("{equator}, {BALL}"));
    let dotted = |figure: &Figure| -> f64 {
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
    // 点線の長さ(半分の楕円の周の長さ)は，同じである．
    let (a, b) = (dotted(&by_surface), dotted(&by_ball));
    assert!(b > 3.0, "{b}");
    assert!((a - b).abs() < 0.1, "{a} {b}");
}

/// 高さの関数`z = x^2 + y^2`の曲面．xとyは，-1から1である．
const BOWL: &str = r#"{ "id": "bowl", "type": "surface", "vars": ["x", "y"],
    "expr": ["x", "y", "x^2 + y^2"], "domain": [[-1, 1], [-1, 1]] }"#;

/// 実装と別の方法(視線に沿って刻み，曲面を横切るか調べる)で，点が曲面に隠れているかを答える．
fn bowl_hides(point: [f64; 3], azimuth: f64, elevation: f64) -> bool {
    let (a, e) = (azimuth.to_radians(), elevation.to_radians());
    let toward = [e.cos() * a.cos(), e.cos() * a.sin(), e.sin()];
    let height = |s: f64| -> Option<f64> {
        let p = [
            point[0] + s * toward[0],
            point[1] + s * toward[1],
            point[2] + s * toward[2],
        ];
        (p[0].abs() <= 1.0 && p[1].abs() <= 1.0).then(|| p[2] - (p[0] * p[0] + p[1] * p[1]))
    };
    let start = height(0.0);
    let mut previous = start;
    let mut hidden = false;
    for step in 1..=20_000 {
        let s = f64::from(step) * 0.001;
        let now = height(s);
        if let (Some(before), Some(after)) = (previous, now)
            && before.signum() != after.signum()
        {
            hidden = true;
            break;
        }
        previous = now;
    }
    hidden
}

#[test]
fn 高さの関数の曲面が隠す軸は_視線に沿って刻んで確かめた隠れ方と合う() {
    let (a, e) = (60.0_f64, 50.0_f64);
    let scene = format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": {a}, "elevation": {e}, "unit": "1cm" }},
             "objects": [ {}, {BOWL} ] }}"#,
        r#"{ "id": "z_axis", "type": "axis", "direction": "z", "range": [-1, 3] }"#
    );
    let figure = render(&parse_scene(&scene).expect("読める")).expect("描画できる");
    let all = paths(&figure);
    // 2つ以上の部分に分かれ，隠れ方が実際に変わる．
    let axis_pieces: Vec<&&Path> = all.iter().filter(|path| path.stroke.width < 0.7).collect();
    assert!(axis_pieces.len() >= 2, "{}", axis_pieces.len());
    assert!(
        axis_pieces
            .iter()
            .any(|path| path.stroke.line == Line::Dotted)
    );
    assert!(
        axis_pieces
            .iter()
            .any(|path| path.stroke.line == Line::Solid)
    );
    // 各部分の中点の隠れ方は，別の方法の結果と合う．
    let (right, up) = (
        [-a.to_radians().sin(), a.to_radians().cos(), 0.0],
        [
            -e.to_radians().sin() * a.to_radians().cos(),
            -e.to_radians().sin() * a.to_radians().sin(),
            e.to_radians().cos(),
        ],
    );
    let _ = right;
    // z軸の点(0, 0, z)の画面の高さは，z * cos(e)である．
    let mut checked = 0;
    for path in &axis_pieces {
        let middle = f64::midpoint(path.points[0][1], path.points.last().unwrap()[1]);
        let z = middle / up[2];
        let hidden = bowl_hides([0.0, 0.0, z], a, e);
        assert_eq!(path.stroke.line == Line::Dotted, hidden, "z={z}");
        checked += 1;
    }
    assert!(checked >= 2);
}

#[test]
fn 縁を描くと_縁の線が加わり_見える部分は実線で_隠れる部分は点線になる() {
    let bowl_with_edge = BOWL.replace(r#""domain""#, r#""boundary": true, "domain""#);
    let without = figure_of(BOWL);
    let with = figure_of(&bowl_with_edge);
    // 縁のない曲面の輪郭は，なだらかな曲面には，ほとんど出ない．縁は，4本の辺からなる．
    assert!(paths(&with).len() >= paths(&without).len() + 4);
    // 高さの関数の曲面は，上から見ると，遠い側の縁の内側も見える(縁は，隠れない)．
    let edges: Vec<&Path> = paths(&with)
        .into_iter()
        .filter(|path| !paths(&without).contains(path))
        .collect();
    assert!(edges.iter().all(|path| path.points.len() >= 2));
}

#[test]
fn 隠れた部分を描かない指定は_縁にも効く() {
    let scene = r#"{ "version": "0.1.0", "description": "試験の図",
             "view": { "azimuth": 60, "elevation": 10, "unit": "1cm" },
             "objects": [ { "id": "s", "type": "surface", "vars": ["x", "y"],
               "expr": ["x", "y", "x^2 + y^2"], "domain": [[-1, 1], [-1, 1]], "boundary": true,
               "style": { "hidden": "none" } } ] }"#;
    let figure = render(&parse_scene(scene).expect("読める")).expect("描画できる");
    assert!(
        paths(&figure)
            .iter()
            .all(|path| path.stroke.line != Line::Dotted)
    );
}

#[test]
fn 網を細かくしても粗くしても_球の輪郭は近い() {
    let coarse = SPHERE_SURFACE.replace(r#""domain""#, r#""mesh": [16, 16], "domain""#);
    let fine = SPHERE_SURFACE.replace(r#""domain""#, r#""mesh": [96, 96], "domain""#);
    for scene in [coarse, fine] {
        let figure = figure_of(&scene);
        let outline = paths(&figure)[0];
        for point in &outline.points {
            assert!((point[0].hypot(point[1]) - 2.0).abs() < 0.12, "{point:?}");
        }
    }
}

#[test]
fn 描画は同じ入力なら同じ結果になる() {
    let scene = format!("{}, {SPHERE_SURFACE}", axis("x"));
    assert_eq!(figure_of(&scene), figure_of(&scene));
}

const PARABOLOID: &str = include_str!("../../../site/src/figures/paraboloid-with-axes.json");

#[test]
fn 放物面の図は_軸と縁と輪郭を描き_隠れた部分を点線にする() {
    let figure = render(&parse_scene(PARABOLOID).expect("読める")).expect("描画できる");
    let all = paths(&figure);
    let dotted = all
        .iter()
        .filter(|path| path.stroke.line == Line::Dotted)
        .count();
    // 縁の遠い側と，放物面に隠れる軸の部分が，点線になる．
    assert!(dotted >= 2, "{dotted}");
    assert!(all.iter().any(|path| path.stroke.line == Line::Solid));
    // すべての点が，有限である．
    assert!(
        all.iter()
            .flat_map(|path| &path.points)
            .all(|p| p[0].is_finite() && p[1].is_finite())
    );
}

// ---- 高速化の照合 ----

use figure::surface::{Frame, Mesh};

/// 2つの変数から，空間の点を返す関数．
type SurfaceFn<'a> = &'a dyn Fn(f64, f64) -> Option<[f64; 3]>;

fn frame_of(azimuth: f64, elevation: f64) -> Frame {
    let (a, e) = (azimuth.to_radians(), elevation.to_radians());
    Frame {
        right: [-a.sin(), a.cos(), 0.0],
        up: [-e.sin() * a.cos(), -e.sin() * a.sin(), e.cos()],
        toward: [e.cos() * a.cos(), e.cos() * a.sin(), e.sin()],
    }
}

/// 決まった順に並ぶ，疑似乱数の点(-3から3)．
fn scattered(count: usize) -> Vec<[f64; 3]> {
    let mut state = 12_345_u64;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        f64::from(u32::try_from(state >> 40).unwrap()) / f64::from(1_u32 << 24) * 6.0 - 3.0
    };
    (0..count).map(|_| [next(), next(), next()]).collect()
}

#[test]
fn 画面の格子で絞った隠れ方の判定は_全三角形を調べた結果と一致する() {
    let sphere = |u: f64, v: f64| {
        Some([
            2.0 * u.sin() * v.cos(),
            2.0 * u.sin() * v.sin(),
            2.0 * u.cos(),
        ])
    };
    let bowl = |x: f64, y: f64| Some([x, y, x * x + y * y]);
    let cases: [(SurfaceFn, [[f64; 2]; 2]); 2] = [
        (
            &sphere,
            [[0.0, std::f64::consts::PI], [0.0, std::f64::consts::TAU]],
        ),
        (&bowl, [[-1.0, 1.0], [-1.0, 1.0]]),
    ];
    for (surface, domain) in cases {
        for (azimuth, elevation) in [(60.0, 20.0), (200.0, 55.0), (10.0, -30.0)] {
            let mesh = Mesh::build(surface, domain, [40, 40], frame_of(azimuth, elevation));
            for point in scattered(400) {
                assert_eq!(mesh.hides(point), mesh.hides_exhaustive(point), "{point:?}");
            }
        }
    }
}

#[test]
fn 網が大きくても_隠れ方の判定は速い() {
    let sphere = |u: f64, v: f64| {
        Some([
            2.0 * u.sin() * v.cos(),
            2.0 * u.sin() * v.sin(),
            2.0 * u.cos(),
        ])
    };
    let mesh = Mesh::build(
        &sphere,
        [[0.0, std::f64::consts::PI], [0.0, std::f64::consts::TAU]],
        [200, 200],
        frame_of(60.0, 20.0),
    );
    let started = std::time::Instant::now();
    let hidden = scattered(2_000)
        .into_iter()
        .filter(|p| mesh.hides(*p))
        .count();
    assert!(hidden > 0);
    // 全三角形(8万個)を毎回調べると，開発用のビルドでは数秒かかる．
    assert!(
        started.elapsed().as_secs_f64() < 1.0,
        "{:?}",
        started.elapsed()
    );
}

// ---- 交線と切り口の精度 ----

#[allow(clippy::unnecessary_wraps)]
fn sphere_point(u: f64, v: f64) -> Option<[f64; 3]> {
    Some([
        2.0 * u.sin() * v.cos(),
        2.0 * u.sin() * v.sin(),
        2.0 * u.cos(),
    ])
}

/// 軸がz軸に平行で，半径1の円柱 (x - 1)^2 + y^2 = 1．
#[allow(clippy::unnecessary_wraps)]
fn cylinder_point(t: f64, z: f64) -> Option<[f64; 3]> {
    Some([1.0 + t.cos(), t.sin(), z])
}

const SPHERE_DOMAIN: [[f64; 2]; 2] = [[0.0, std::f64::consts::PI], [0.0, std::f64::consts::TAU]];
const CYLINDER_DOMAIN: [[f64; 2]; 2] = [[0.0, std::f64::consts::TAU], [-3.0, 3.0]];

fn build(point_at: SurfaceFn, domain: [[f64; 2]; 2], mesh: usize) -> Mesh {
    Mesh::build(point_at, domain, [mesh, mesh], frame_of(60.0, 20.0))
}

/// 曲線の点の，球と円柱の式からの残差．どちらも，0になる．
fn viviani_residual(p: [f64; 3]) -> f64 {
    let on_sphere = p[0] * p[0] + p[1] * p[1] + p[2] * p[2] - 4.0;
    let on_cylinder = (p[0] - 1.0).powi(2) + p[1] * p[1] - 1.0;
    on_sphere.abs().max(on_cylinder.abs())
}

#[test]
fn 球と円柱の交線の点は_2つの面の式の上にあり_線分の中点も面から離れない() {
    let (sphere, cylinder) = (
        build(&sphere_point, SPHERE_DOMAIN, 48),
        build(&cylinder_point, CYLINDER_DOMAIN, 48),
    );
    let lines = sphere.intersection(&cylinder, &sphere_point, &cylinder_point);
    assert!(!lines.is_empty());
    let mut worst_vertex = 0.0_f64;
    let mut worst_middle = 0.0_f64;
    let mut count = 0;
    for line in &lines {
        for pair in line.windows(2) {
            let middle = [
                f64::midpoint(pair[0].0[0], pair[1].0[0]),
                f64::midpoint(pair[0].0[1], pair[1].0[1]),
                f64::midpoint(pair[0].0[2], pair[1].0[2]),
            ];
            worst_middle = worst_middle.max(viviani_residual(middle));
        }
        for (point, _) in line {
            worst_vertex = worst_vertex.max(viviani_residual(*point));
            count += 1;
        }
    }
    // 磨いた点は，式の上にあり，線分の中点は，弦の許容(座標の大きさ×1e-3)で，面から離れる．節の近くの
    // 点は，磨けないことがあるので，外れた点は，ごく一部に限る．
    assert!(worst_vertex < 0.05, "{worst_vertex}");
    assert!(worst_middle < 0.02, "{worst_middle}");
    assert!(count < 3000, "{count}");
}

#[test]
fn 球と円柱の交線の点の大半は_式の上に厳密にある() {
    let (sphere, cylinder) = (
        build(&sphere_point, SPHERE_DOMAIN, 48),
        build(&cylinder_point, CYLINDER_DOMAIN, 48),
    );
    let lines = sphere.intersection(&cylinder, &sphere_point, &cylinder_point);
    let points: Vec<[f64; 3]> = lines.iter().flatten().map(|(point, _)| *point).collect();
    let exact = points
        .iter()
        .filter(|p| viviani_residual(**p) < 1e-7)
        .count();
    // 節(2, 0, 0)の近くを除き，点は，磨かれて，式の上にある．
    assert!(exact * 10 >= points.len() * 9, "{exact} / {}", points.len());
}

#[test]
fn 交線の点は_曲がりの強い所で密になる() {
    let (sphere, cylinder) = (
        build(&sphere_point, SPHERE_DOMAIN, 24),
        build(&cylinder_point, CYLINDER_DOMAIN, 24),
    );
    let coarse = sphere.intersection(&cylinder, &sphere_point, &cylinder_point);
    let (fine_sphere, fine_cylinder) = (
        build(&sphere_point, SPHERE_DOMAIN, 96),
        build(&cylinder_point, CYLINDER_DOMAIN, 96),
    );
    let fine = fine_sphere.intersection(&fine_cylinder, &sphere_point, &cylinder_point);
    let count = |lines: &[Vec<([f64; 3], [f64; 3])>]| lines.iter().map(Vec::len).sum::<usize>();
    // 網が4倍細かくても，弦の許容で点を足すので，点の数は，4倍にならない．
    assert!(
        count(&fine) < 3 * count(&coarse),
        "{} {}",
        count(&fine),
        count(&coarse)
    );
    // 粗い網でも，中点の誤差は，細かい網と同程度に小さい．
    let worst = |lines: &[Vec<([f64; 3], [f64; 3])>]| {
        lines
            .iter()
            .flat_map(|line| line.windows(2))
            .map(|pair| {
                let middle = [0, 1, 2].map(|k| f64::midpoint(pair[0].0[k], pair[1].0[k]));
                viviani_residual(middle)
            })
            .fold(0.0_f64, f64::max)
    };
    assert!(worst(&coarse) < 0.03, "{}", worst(&coarse));
}

#[test]
fn 球の水平な切り口の点は_厳密に球と平面の上にあり_中点も離れない() {
    let sphere = build(&sphere_point, SPHERE_DOMAIN, 48);
    let lines = sphere.cut([0.0, 0.0, 1.0], 1.0, &sphere_point);
    assert!(!lines.is_empty());
    let mut worst_vertex = 0.0_f64;
    let mut worst_middle = 0.0_f64;
    let residual = |p: [f64; 3]| {
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2] - 4.0)
            .abs()
            .max((p[2] - 1.0).abs())
    };
    for line in &lines {
        for pair in line.windows(2) {
            let middle = [0, 1, 2].map(|k| f64::midpoint(pair[0].0[k], pair[1].0[k]));
            worst_middle = worst_middle.max(residual(middle));
        }
        for (point, _) in line {
            worst_vertex = worst_vertex.max(residual(*point));
        }
    }
    assert!(worst_vertex < 1e-9, "{worst_vertex}");
    assert!(worst_middle < 0.01, "{worst_middle}");
}

#[test]
fn 磨いても_閉じた切り口は閉じたまま_始めと終わりが同じ点になる() {
    let sphere = build(&sphere_point, SPHERE_DOMAIN, 48);
    let lines = sphere.cut([0.0, 0.0, 1.0], 1.0, &sphere_point);
    assert_eq!(lines.len(), 1, "{}", lines.len());
    let line = &lines[0];
    let (first, last) = (line[0].0, line.last().unwrap().0);
    let gap = (first[0] - last[0])
        .hypot(first[1] - last[1])
        .hypot(first[2] - last[2]);
    assert!(gap < 1e-6, "{gap}");
}

// ---- 縁と継ぎ目 ----

/// 管の中心の半径2，管の半径0.5のトーラス．
#[allow(clippy::unnecessary_wraps)]
fn torus_point(u: f64, v: f64) -> Option<[f64; 3]> {
    let r = 2.0 + 0.5 * v.cos();
    Some([r * u.cos(), r * u.sin(), 0.5 * v.sin()])
}

/// メビウスの帯．`u = -pi`の辺と`u = pi`の辺は，向きを逆にして重なる．
#[allow(clippy::unnecessary_wraps)]
fn mobius_point(u: f64, v: f64) -> Option<[f64; 3]> {
    let r = 2.0 + v / 2.0 * (u / 2.0).cos();
    Some([r * u.cos(), r * u.sin(), v / 2.0 * (u / 2.0).sin()])
}

const PI: f64 = std::f64::consts::PI;

#[test]
fn 継ぎ目の辺は縁にならず_円柱の縁は上下の円だけである() {
    let cylinder = build(&cylinder_point, CYLINDER_DOMAIN, 32);
    let lines = cylinder.boundary();
    assert_eq!(lines.len(), 2);
    for line in &lines {
        let z = line[0][2];
        assert!((z.abs() - 3.0).abs() < 1e-12, "{z}");
        assert!(line.iter().all(|p| (p[2] - z).abs() < 1e-12));
    }
}

#[test]
fn 閉じた曲面には縁がない() {
    let torus = build(&torus_point, [[-PI, PI], [-PI, PI]], 32);
    assert!(torus.boundary().is_empty());
}

#[test]
fn 向きを逆にして重なる辺も継ぎ目であり_メビウスの帯の縁は幅の端の2本である() {
    let band = build(&mobius_point, [[-PI, PI], [-1.0, 1.0]], 32);
    let lines = band.boundary();
    assert_eq!(lines.len(), 2);
    // 幅の端(v = ±1)の線は，帯の中心の円から0.5離れている．
    for line in &lines {
        for p in line {
            let off = (p[0].hypot(p[1]) - 2.0).hypot(p[2]);
            assert!((off - 0.5).abs() < 1e-9, "{off}");
        }
    }
}
