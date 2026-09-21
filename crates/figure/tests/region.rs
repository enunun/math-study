//! 領域(`region`)を確かめる．2つのグラフ(か，グラフとx軸)の間を，定義域の中で，斜線で埋める．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic
)]

use figure::figure::{Figure, Item, Path};
use figure::scene::{Color, Length, LengthUnit, Line, Object};
use figure::{Error, ErrorKind, Scene, parse_scene, render};

fn plane_scene(unit: &str, objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-4, 4], "y": [-2, 2], "unit": {{ "x": "{unit}", "y": "{unit}" }} }},
             "objects": [{objects}] }}"#
    )
}

fn scene_of(objects: &str) -> Scene {
    parse_scene(&plane_scene("1cm", objects)).expect("読める")
}

fn error_of(objects: &str) -> Error {
    parse_scene(&plane_scene("1cm", objects)).expect_err("誤りになる")
}

fn figure_of(unit: &str, objects: &str) -> Result<Figure, Error> {
    render(&parse_scene(&plane_scene(unit, objects)).expect("読める"))
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

const SINE: &str =
    r#"{ "id": "sine", "type": "graph", "var": "x", "expr": "sin(x)", "domain": [-4, 4] }"#;
const COSINE: &str =
    r#"{ "id": "cosine", "type": "graph", "var": "x", "expr": "cos(x)", "domain": [-4, 4] }"#;

#[test]
fn 領域は_2つのグラフと定義域を持ち_斜線の角度と間隔の既定がある() {
    let scene = scene_of(&format!(
        r#"{SINE}, {COSINE},
           {{ "id": "r", "type": "region", "between": ["sine", "cosine"],
              "domain": ["-3*pi/4", "pi/4"] }}"#
    ));
    let Object::Region(region) = &scene.objects[2] else {
        panic!("領域である");
    };
    assert_eq!(region.between, ["sine", "cosine"]);
    assert_eq!(region.angle, 45.0);
    assert_eq!(
        region.gap,
        Length {
            value: 3.0,
            unit: LengthUnit::Mm
        }
    );
    assert_eq!(scene.objects[2].type_name(), "region");
}

#[test]
fn 領域を書き出して読み直すと同じになる() {
    let scene = scene_of(&format!(
        r#"{SINE},
           {{ "id": "r", "type": "region", "between": ["sine"], "domain": [0, 3],
              "angle": 30, "gap": "2mm", "style": {{ "color": "blue" }} }}"#
    ));
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    assert_eq!(parse_scene(&written).expect("読み直せる"), scene);
}

#[test]
fn 領域が指すグラフは_1つか2つである() {
    for between in ["[]", r#"["sine", "sine", "sine"]"#] {
        let error = error_of(&format!(
            r#"{SINE}, {{ "id": "r", "type": "region", "between": {between}, "domain": [0, 1] }}"#
        ));
        assert!(
            matches!(error.kind, ErrorKind::Invalid(_)),
            "{between}: {error}"
        );
        assert_eq!(error.object.as_deref(), Some("r"));
        assert!(error.to_string().contains("between"), "{error}");
    }
}

#[test]
fn 存在しないグラフを指すと_誤りになる() {
    let error = error_of(&format!(
        r#"{SINE}, {{ "id": "r", "type": "region", "between": ["sine", "nothing"], "domain": [0, 1] }}"#
    ));
    assert_eq!(error.kind, ErrorKind::UnknownGraph("nothing".to_owned()));
    assert_eq!(error.object.as_deref(), Some("r"));
    assert_eq!(error.kind.code(), "unknown_graph");
    // グラフでないオブジェクトは，指せない．
    let point = error_of(&format!(
        r#"{SINE}, {{ "id": "P", "type": "point", "at": [0, 0] }},
           {{ "id": "r", "type": "region", "between": ["sine", "P"], "domain": [0, 1] }}"#
    ));
    assert_eq!(point.kind, ErrorKind::UnknownGraph("P".to_owned()));
}

#[test]
fn 定義域は_増える有限の範囲で_式も書ける() {
    let error = error_of(&format!(
        r#"{SINE}, {{ "id": "r", "type": "region", "between": ["sine"], "domain": [2, 1] }}"#
    ));
    assert_eq!(error.kind, ErrorKind::InvalidRange("domain"));
    assert_eq!(error.object.as_deref(), Some("r"));
}

#[test]
fn 斜線の角度と間隔は_有限で正である() {
    let angle = error_of(&format!(
        r#"{SINE}, {{ "id": "r", "type": "region", "between": ["sine"], "domain": [0, 1],
                     "angle": "steep" }}"#
    ));
    assert!(matches!(angle.kind, ErrorKind::Invalid(_)), "{angle}");
    let gap = error_of(&format!(
        r#"{SINE}, {{ "id": "r", "type": "region", "between": ["sine"], "domain": [0, 1],
                     "gap": "0mm" }}"#
    ));
    assert!(matches!(gap.kind, ErrorKind::Invalid(_)), "{gap}");
}

#[test]
fn 空間の図では_領域は使えない() {
    let error = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "azimuth": 0, "elevation": 0, "unit": "1cm" },
             "objects": [ { "id": "r", "type": "region", "between": ["g"], "domain": [0, 1] } ] }"#,
    )
    .expect_err("誤りになる");
    assert!(error.to_string().contains("空間"), "{error}");
}

// ---- 描画 ----

/// 領域の斜線だけを集める．領域は，グラフの後ろに置く．
fn hatch_of(figure: &Figure, graphs: usize) -> Vec<&Path> {
    paths(figure).into_iter().skip(graphs).collect()
}

#[test]
fn 二つのグラフの間の斜線は_すべて領域の内側にあり_間隔と角度に従う() {
    // 10cmを1単位とし，y = x^2とy = xの間(0から1)を，3mm間隔の45度の斜線で埋める．
    let figure = figure_of(
        "10cm",
        r#"{ "id": "lower", "type": "graph", "var": "x", "expr": "x^2", "domain": [0, 1] },
           { "id": "upper", "type": "graph", "var": "x", "expr": "x", "domain": [0, 1] },
           { "id": "r", "type": "region", "between": ["lower", "upper"], "domain": [0, 1] }"#,
    )
    .expect("描画できる");
    let lines = hatch_of(&figure, 2);
    assert!(lines.len() >= 5, "{}", lines.len());
    let (sin, cos) = 45.0_f64.to_radians().sin_cos();
    let mut offsets = Vec::new();
    for line in &lines {
        assert_eq!(line.points.len(), 2);
        assert!(line.arrow.is_none());
        let [a, b] = [line.points[0], line.points[1]];
        // 向きは，45度である．
        assert!(((b[0] - a[0]) * sin - (b[1] - a[1]) * cos).abs() < 1e-9);
        // 中点は，領域の内側にある(x^2 <= y <= x，単位10cmで直す)．
        let (x, y) = ((a[0] + b[0]) / 20.0, (a[1] + b[1]) / 20.0);
        assert!(y >= x * x - 0.002 && y <= x + 0.002, "({x}, {y})");
        offsets.push(-sin * a[0] + cos * a[1]);
    }
    offsets.sort_by(f64::total_cmp);
    for pair in offsets.windows(2) {
        // 隣り合う斜線の間隔は，3mm(0.3cm)の整数倍である．
        let steps = (pair[1] - pair[0]) / 0.3;
        assert!(
            (steps - steps.round()).abs() < 1e-9 && steps > 0.5,
            "{offsets:?}"
        );
    }
}

#[test]
fn グラフとx軸の間の斜線を引ける() {
    // y = sin xと，x軸の間(0からpi)を，水平な線で埋める．
    let figure = figure_of(
        "1cm",
        r#"{ "id": "s", "type": "graph", "var": "x", "expr": "sin(x)", "domain": [0, "pi"] },
           { "id": "r", "type": "region", "between": ["s"], "domain": [0, "pi"],
             "angle": 0, "gap": "5mm" }"#,
    )
    .expect("描画できる");
    let lines = hatch_of(&figure, 1);
    // 高さ0.5cmの線は，sin x = 0.5の2点(x = 0.5236と2.618)の間に引かれる．
    let at_half: Vec<&&Path> = lines
        .iter()
        .filter(|line| (line.points[0][1] - 0.5).abs() < 1e-9)
        .collect();
    assert_eq!(at_half.len(), 1);
    let mut xs = [at_half[0].points[0][0], at_half[0].points[1][0]];
    xs.sort_by(f64::total_cmp);
    assert!(
        (xs[0] - std::f64::consts::FRAC_PI_6).abs() < 0.01
            && (xs[1] - 5.0 * std::f64::consts::FRAC_PI_6).abs() < 0.01,
        "{xs:?}"
    );
}

#[test]
fn 交わる2つのグラフの間は_どちらの区間も斜線で埋まる() {
    let figure = figure_of(
        "1cm",
        &format!(
            r#"{SINE}, {COSINE},
               {{ "id": "r", "type": "region", "between": ["sine", "cosine"],
                  "domain": [-3.5, 3.5], "gap": "2mm" }}"#
        ),
    )
    .expect("描画できる");
    let lines = hatch_of(&figure, 2);
    assert!(lines.len() > 20, "{}", lines.len());
    for line in &lines {
        // 線の中点は，その所での，sinとcosの間にある．
        let mid = [
            f64::midpoint(line.points[0][0], line.points[1][0]),
            f64::midpoint(line.points[0][1], line.points[1][1]),
        ];
        let (low, high) = (
            mid[0].sin().min(mid[0].cos()),
            mid[0].sin().max(mid[0].cos()),
        );
        assert!(mid[1] >= low - 0.01 && mid[1] <= high + 0.01, "{mid:?}");
    }
}

#[test]
fn 斜線は_見える範囲で切り取る() {
    // 定義域が，見える範囲(x: -4から4)より広い．
    let figure = figure_of(
        "1cm",
        r#"{ "id": "g", "type": "graph", "var": "x", "expr": "x", "domain": [-9, 9] },
           { "id": "r", "type": "region", "between": ["g"], "domain": [-9, 9], "angle": 90, "gap": "3mm" }"#,
    )
    .expect("描画できる");
    for line in hatch_of(&figure, 1) {
        for p in &line.points {
            assert!(
                p[0].abs() <= 4.0 + 1e-9 && p[1].abs() <= 2.0 + 1e-9,
                "{p:?}"
            );
        }
    }
}

#[test]
fn 斜線の線の種類と色と太さは_styleで選べ_既定は細い実線である() {
    let plain = figure_of(
        "1cm",
        &format!(
            r#"{SINE}, {{ "id": "r", "type": "region", "between": ["sine"], "domain": [0, 3] }}"#
        ),
    )
    .expect("描画できる");
    let stroke = hatch_of(&plain, 1)[0].stroke;
    assert_eq!((stroke.line, stroke.color), (Line::Solid, None));
    assert!((stroke.width - 0.4).abs() < 1e-9);
    let styled = figure_of(
        "1cm",
        &format!(
            r#"{SINE}, {{ "id": "r", "type": "region", "between": ["sine"], "domain": [0, 3],
                         "style": {{ "color": "red", "width": "1pt", "line": "dotted" }} }}"#
        ),
    )
    .expect("描画できる");
    let stroke = hatch_of(&styled, 1)[0].stroke;
    assert_eq!(
        (stroke.line, stroke.color),
        (Line::Dotted, Some(Color::Red))
    );
    assert!((stroke.width - 1.0).abs() < 1e-9);
}

#[test]
fn 領域の定義域で途切れるグラフは_描画の誤りになる() {
    // tan xは，pi/2で途切れる．
    let error = figure_of(
        "1cm",
        r#"{ "id": "t", "type": "graph", "var": "x", "expr": "tan(x)", "domain": [0, 3] },
           { "id": "r", "type": "region", "between": ["t"], "domain": [0, 3] }"#,
    )
    .expect_err("誤りになる");
    assert_eq!(error.object.as_deref(), Some("r"));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert!(error.to_string().contains("途切れ"), "{error}");
}

#[test]
fn 領域は_グラフの定義域の外へは延ばせない() {
    let error = error_of(
        r#"{ "id": "g", "type": "graph", "var": "x", "expr": "x", "domain": [0, 1] },
           { "id": "r", "type": "region", "between": ["g"], "domain": [0, 2] }"#,
    );
    assert_eq!(error.object.as_deref(), Some("r"));
    assert!(matches!(error.kind, ErrorKind::Invalid(_)), "{error}");
    assert!(error.to_string().contains("定義域"), "{error}");
}

#[test]
fn 領域は_描く範囲を変えない() {
    let with = figure_of(
        "1cm",
        &format!(
            r#"{SINE}, {{ "id": "r", "type": "region", "between": ["sine"], "domain": [0, 3] }}"#
        ),
    )
    .expect("描画できる");
    let without = figure_of("1cm", SINE).expect("描画できる");
    assert_eq!(with.bounds, without.bounds);
}

const SINE_COSINE_REGION: &str = include_str!("../../../site/src/figures/sine-cosine-region.json");

#[test]
fn sinとcosの間の領域の図は_交点の間だけを斜線で埋める() {
    let figure = render(&parse_scene(SINE_COSINE_REGION).expect("読める")).expect("描画できる");
    // 軸2本，グラフ2本のあとに，斜線が並ぶ．
    let lines = hatch_of(&figure, 4);
    assert!(lines.len() >= 4, "{}", lines.len());
    let (low, high) = (
        -3.0 * std::f64::consts::FRAC_PI_4,
        std::f64::consts::FRAC_PI_4,
    );
    for line in &lines {
        for point in &line.points {
            // 斜線は，交点の間(x: -3pi/4からpi/4，横の単位1cm)にあり，2つのグラフの間にある．
            assert!(
                point[0] >= low - 0.01 && point[0] <= high + 0.01,
                "{point:?}"
            );
            let y = point[1] / 1.5;
            let (bottom, top) = (
                point[0].sin().min(point[0].cos()),
                point[0].sin().max(point[0].cos()),
            );
            assert!(y >= bottom - 0.01 && y <= top + 0.01, "{point:?}");
        }
    }
}
