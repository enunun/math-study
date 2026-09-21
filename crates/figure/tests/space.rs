//! 空間の図の描画を確かめる．投影，座標軸，球の輪郭，隠れた線，空間の曲線を見る．
//!
//! 隠れているかどうかは，実装と別の方法(視線に沿って刻んで，球の内側に入るか調べる)で確かめる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, LabelItem, Path};
use figure::scene::{Anchor, Arrow, Line};
use figure::{parse_scene, render};

const SPHERE_WITH_AXES: &str = include_str!("../../../site/src/figures/sphere-with-axes.json");

/// 余白(cm)．`render.rs`の定数と同じ値である．
const MARGIN: f64 = 0.6;

type P3 = [f64; 3];

fn figure_of(json: &str) -> Figure {
    render(&parse_scene(json).expect("シーンを読める")).expect("描画できる")
}

fn space_scene(azimuth: f64, elevation: f64, unit: &str, objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "azimuth": {azimuth}, "elevation": {elevation}, "unit": "{unit}" }},
             "objects": [{objects}] }}"#
    )
}

fn axis(direction: &str, range: &str, extra: &str) -> String {
    format!(
        r#"{{ "id": "{direction}_axis", "type": "axis", "direction": "{direction}",
              "range": {range}, "label": "{direction}"{extra} }}"#
    )
}

const BALL: &str = r#"{ "id": "ball", "type": "sphere", "center": [0, 0, 0], "radius": 2 }"#;

fn paths(figure: &Figure) -> Vec<&Path> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Path(path) => Some(path),
            Item::Label(_) => None,
        })
        .collect()
}

/// 球の輪郭を除いた，曲線の線．球は，曲線より後に置いてある．
fn curve_paths(figure: &Figure) -> Vec<&Path> {
    let mut all = paths(figure);
    all.pop();
    all
}

fn labels(figure: &Figure) -> Vec<&LabelItem> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(label) => Some(label),
            Item::Path(_) => None,
        })
        .collect()
}

fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

fn close2(a: [f64; 2], b: [f64; 2], tolerance: f64) -> bool {
    close(a[0], b[0], tolerance) && close(a[1], b[1], tolerance)
}

// 仕様の式．カメラは原点から見て，方向(cos e cos a, cos e sin a, sin e)にある．

/// 見る点からカメラへ向かう単位ベクトル．
fn toward_camera(azimuth: f64, elevation: f64) -> P3 {
    let (a, e) = (azimuth.to_radians(), elevation.to_radians());
    [e.cos() * a.cos(), e.cos() * a.sin(), e.sin()]
}

/// 空間の点の，画面の位置(cm)．
fn project(point: P3, azimuth: f64, elevation: f64, unit: f64) -> [f64; 2] {
    let (a, e) = (azimuth.to_radians(), elevation.to_radians());
    let right = [-a.sin(), a.cos(), 0.0];
    let up = [-e.sin() * a.cos(), -e.sin() * a.sin(), e.cos()];
    [dot(point, right) * unit, dot(point, up) * unit]
}

fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// 点から，カメラへ向かう視線が，球に当たるか．視線に沿って刻んで，球の内側に入る点を探す．
fn hidden_by_ball(point: P3, azimuth: f64, elevation: f64, center: P3, radius: f64) -> bool {
    let toward = toward_camera(azimuth, elevation);
    let inside = |s: f64| {
        let p = [
            point[0] + s * toward[0] - center[0],
            point[1] + s * toward[1] - center[1],
            point[2] + s * toward[2] - center[2],
        ];
        dot(p, p) < radius * radius
    };
    // 点が球の内側にあるときも，隠れている．
    (0..=40_000).any(|step| inside(f64::from(step) * 0.001))
}

fn ball_hides(point: P3, azimuth: f64, elevation: f64) -> bool {
    hidden_by_ball(point, azimuth, elevation, [0.0; 3], 2.0)
}

// ---- 投影と座標軸 ----

#[test]
fn 方位角0_仰角0では_y軸は右向き_z軸は上向きに描く() {
    let scene = space_scene(
        0.0,
        0.0,
        "1cm",
        &format!("{}, {}", axis("y", "[-1, 2]", ""), axis("z", "[-1, 2]", "")),
    );
    let figure = figure_of(&scene);
    let axes = paths(&figure);
    assert_eq!(axes.len(), 2);
    assert!(close2(axes[0].points[0], [-1.0, 0.0], 1e-9));
    assert!(close2(*axes[0].points.last().unwrap(), [2.0, 0.0], 1e-9));
    assert!(close2(axes[1].points[0], [0.0, -1.0], 1e-9));
    assert!(close2(*axes[1].points.last().unwrap(), [0.0, 2.0], 1e-9));
}

#[test]
fn 方位角90_仰角0では_x軸は左向きに描く() {
    let figure = figure_of(&space_scene(90.0, 0.0, "1cm", &axis("x", "[0, 2]", "")));
    let axes = paths(&figure);
    assert!(close2(*axes[0].points.last().unwrap(), [-2.0, 0.0], 1e-9));
}

#[test]
fn 仰角90では_真上から見て_x軸は下向き_y軸は右向きになる() {
    let figure = figure_of(&space_scene(
        0.0,
        90.0,
        "1cm",
        &format!("{}, {}", axis("x", "[0, 2]", ""), axis("y", "[0, 2]", "")),
    ));
    let axes = paths(&figure);
    assert!(close2(*axes[0].points.last().unwrap(), [0.0, -2.0], 1e-9));
    assert!(close2(*axes[1].points.last().unwrap(), [2.0, 0.0], 1e-9));
}

#[test]
fn 一般の向きの投影は_式の通りで_1単位の実寸を掛ける() {
    let figure = figure_of(&space_scene(
        60.0,
        20.0,
        "0.5cm",
        &format!(
            "{}, {}, {}",
            axis("x", "[-5, 5]", ""),
            axis("y", "[-5, 5]", ""),
            axis("z", "[-5, 5]", "")
        ),
    ));
    let all = paths(&figure);
    // 球がないので，各軸は1本の線である．
    assert_eq!(all.len(), 3);
    for (path, end) in all
        .iter()
        .zip([[5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, 0.0, 5.0]])
    {
        assert!(close2(
            *path.points.last().unwrap(),
            project(end, 60.0, 20.0, 0.5),
            1e-9
        ));
        let start = end.map(|c| -c);
        assert!(close2(
            path.points[0],
            project(start, 60.0, 20.0, 0.5),
            1e-9
        ));
    }
}

#[test]
fn 軸には_既定で矢じりが付き_noneなら付かない() {
    let figure = figure_of(&space_scene(
        60.0,
        20.0,
        "1cm",
        &format!(
            "{}, {}",
            axis("x", "[-1, 1]", ""),
            axis("y", "[-1, 1]", r#", "arrow": "none""#)
        ),
    ));
    let all = paths(&figure);
    assert_eq!(
        all[0].arrow.as_ref().map(|arrow| arrow.kind),
        Some(Arrow::Stealth)
    );
    assert!(all[1].arrow.is_none());
}

#[test]
fn 軸の名前は_正の端に置き_軸の向きの先に箱が延びるアンカーを使う() {
    let figure = figure_of(&space_scene(
        60.0,
        20.0,
        "1cm",
        &format!(
            "{}, {}, {}",
            axis("x", "[-5, 5]", ""),
            axis("y", "[-5, 5]", ""),
            axis("z", "[-5, 5]", "")
        ),
    ));
    let names = labels(&figure);
    let texts: Vec<&str> = names.iter().map(|label| label.tex.as_str()).collect();
    assert_eq!(texts, ["$x$", "$y$", "$z$"]);
    assert!(close2(
        names[0].at,
        project([5.0, 0.0, 0.0], 60.0, 20.0, 1.0),
        1e-9
    ));
    // x軸は左へ，y軸は右下へ，z軸は上へ向かう．
    assert_eq!(names[0].anchor, Anchor::East);
    assert_eq!(names[1].anchor, Anchor::NorthWest);
    assert_eq!(names[2].anchor, Anchor::South);
}

#[test]
fn 描く範囲は_描いた要素の外枠に余白を足したものである() {
    let figure = figure_of(&space_scene(
        0.0,
        0.0,
        "1cm",
        &format!("{}, {}", axis("y", "[-1, 3]", ""), axis("z", "[-2, 1]", "")),
    ));
    // 名前の位置(y軸の右端(3, 0)と，z軸の上端(0, 1))は，軸の端と同じで，矢じりは端で止まる．
    assert!(
        close(figure.bounds.min[0], -1.0 - MARGIN, 1e-9),
        "{:?}",
        figure.bounds
    );
    assert!(
        close(figure.bounds.max[0], 3.0 + MARGIN, 1e-6),
        "{:?}",
        figure.bounds
    );
    assert!(
        close(figure.bounds.min[1], -2.0 - MARGIN, 1e-9),
        "{:?}",
        figure.bounds
    );
    assert!(
        close(figure.bounds.max[1], 1.0 + MARGIN, 1e-6),
        "{:?}",
        figure.bounds
    );
}

// ---- 球の輪郭 ----

#[test]
fn 球の輪郭は_中心の投影を中心とする_半径の円になる() {
    let scene = space_scene(
        60.0,
        20.0,
        "0.5cm",
        r#"{ "id": "ball", "type": "sphere", "center": [1, 2, 3], "radius": 2 }"#,
    );
    let figure = figure_of(&scene);
    let all = paths(&figure);
    assert_eq!(all.len(), 1);
    let outline = all[0];
    let center = project([1.0, 2.0, 3.0], 60.0, 20.0, 0.5);
    assert!(outline.points.len() >= 32);
    for point in &outline.points {
        let distance = (point[0] - center[0]).hypot(point[1] - center[1]);
        assert!(close(distance, 1.0, 1e-9), "{point:?}");
    }
    // 閉じている．
    assert_eq!(outline.points[0], *outline.points.last().unwrap());
    assert_eq!(outline.stroke.line, Line::Solid);
    assert!(outline.arrow.is_none());
}

#[test]
fn 球の輪郭は_弦との誤差が小さい() {
    let figure = figure_of(&space_scene(0.0, 0.0, "1cm", BALL));
    let outline = paths(&figure)[0];
    // 半径2cmの円を折れ線にしたときの，弦と円の最大の隔たり．
    for pair in outline.points.windows(2) {
        let middle = [
            f64::midpoint(pair[0][0], pair[1][0]),
            f64::midpoint(pair[0][1], pair[1][1]),
        ];
        let gap = 2.0 - middle[0].hypot(middle[1]);
        assert!(gap < 0.003, "{gap}");
    }
}

// ---- 隠れた線 ----

/// 軸の線の，各部分の(線の種類，始まりの点，終わりの点)．
fn axis_pieces(figure: &Figure, count: usize) -> Vec<(Line, [f64; 2], [f64; 2])> {
    paths(figure)
        .into_iter()
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
fn x軸は_球の裏から現れる所で実線になり_球の表の面で実線に戻る() {
    let (a, e) = (60.0_f64, 20.0_f64);
    let figure = figure_of(&space_scene(
        a,
        e,
        "1cm",
        &format!("{}, {BALL}", axis("x", "[-5, 5]", "")),
    ));
    let pieces = axis_pieces(&figure, 3);
    let lines: Vec<Line> = pieces.iter().map(|piece| piece.0).collect();
    assert_eq!(lines, [Line::Solid, Line::Dotted, Line::Solid]);
    // 球の裏では，視線が球にかすめる所で，線が隠れる．
    let toward = toward_camera(a, e);
    let behind = -2.0 / toward[1].hypot(toward[2]);
    assert!(
        close2(pieces[0].2, project([behind, 0.0, 0.0], a, e, 1.0), 1e-6),
        "{pieces:?}"
    );
    assert!(close2(pieces[1].1, pieces[0].2, 1e-12));
    // 球の表側では，軸が球の面に出る所(x=2)で，線が現れる．
    assert!(
        close2(pieces[1].2, project([2.0, 0.0, 0.0], a, e, 1.0), 1e-6),
        "{pieces:?}"
    );
    assert!(close2(pieces[2].1, pieces[1].2, 1e-12));
}

#[test]
fn z軸は_球の面に出る所で実線に変わる() {
    let (a, e) = (60.0_f64, 20.0_f64);
    let figure = figure_of(&space_scene(
        a,
        e,
        "1cm",
        &format!("{}, {BALL}", axis("z", "[-5, 5]", "")),
    ));
    let pieces = axis_pieces(&figure, 3);
    let lines: Vec<Line> = pieces.iter().map(|piece| piece.0).collect();
    assert_eq!(lines, [Line::Solid, Line::Dotted, Line::Solid]);
    // 上では，球の面(z=2)で線が現れる．
    assert!(
        close2(pieces[1].2, project([0.0, 0.0, 2.0], a, e, 1.0), 1e-6),
        "{pieces:?}"
    );
    // 下では，視線が球にかすめる所で，線が現れる．
    let behind = -2.0 / e.to_radians().cos();
    assert!(
        close2(pieces[0].2, project([0.0, 0.0, behind], a, e, 1.0), 1e-6),
        "{pieces:?}"
    );
}

#[test]
fn 各部分の線の種類は_視線に沿って刻んで確かめた隠れ方と合う() {
    let (a, e) = (60.0_f64, 20.0_f64);
    let figure = figure_of(&space_scene(
        a,
        e,
        "1cm",
        &format!(
            "{}, {}, {}, {BALL}",
            axis("x", "[-5, 5]", ""),
            axis("y", "[-5, 5]", ""),
            axis("z", "[-5, 5]", "")
        ),
    ));
    let all = paths(&figure);
    // 軸ごとに，線の中の点を，軸の上のパラメータに戻して，確かめる．
    let axes = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut checked = 0;
    for path in &all {
        let Some(direction) = axes.iter().find(|direction| {
            let a0 = project(**direction, a, e, 1.0);
            let point = path.points[0];
            let s = (point[0] * a0[0] + point[1] * a0[1]) / (a0[0] * a0[0] + a0[1] * a0[1]);
            close2(point, [a0[0] * s, a0[1] * s], 1e-6) && !close2(a0, [0.0, 0.0], 1e-9)
        }) else {
            continue;
        };
        let a0 = project(*direction, a, e, 1.0);
        for pair in path.points.windows(2) {
            let middle = [
                f64::midpoint(pair[0][0], pair[1][0]),
                f64::midpoint(pair[0][1], pair[1][1]),
            ];
            let s = (middle[0] * a0[0] + middle[1] * a0[1]) / (a0[0] * a0[0] + a0[1] * a0[1]);
            let point = direction.map(|c| c * s);
            let hidden = ball_hides(point, a, e);
            assert_eq!(
                path.stroke.line == Line::Dotted,
                hidden,
                "{direction:?} s={s} {:?}",
                path.stroke
            );
            checked += 1;
        }
    }
    assert!(checked >= 9, "{checked}");
}

#[test]
fn 矢じりは_最後の部分が実線のときだけ付く() {
    let figure = figure_of(&space_scene(
        60.0,
        20.0,
        "1cm",
        &format!("{}, {BALL}", axis("x", "[-5, 5]", "")),
    ));
    let pieces = paths(&figure);
    assert!(pieces[0].arrow.is_none());
    assert!(pieces[1].arrow.is_none());
    let tip = pieces[2].arrow.as_ref().expect("先端の矢じり");
    assert_eq!(tip.kind, Arrow::Stealth);
    // 球の内側だけにある軸は，先端も隠れているので，矢じりを付けない．
    let inside = figure_of(&space_scene(
        60.0,
        20.0,
        "1cm",
        &format!("{}, {BALL}", axis("z", "[-1, 1]", "")),
    ));
    let pieces = paths(&inside);
    assert_eq!(pieces[0].stroke.line, Line::Dotted);
    assert!(pieces[0].arrow.is_none());
}

#[test]
fn 隠れた線の種類は_styleのhiddenで選べる() {
    let make = |style: &str| {
        figure_of(&space_scene(
            60.0,
            20.0,
            "1cm",
            &format!("{}, {BALL}", axis("x", "[-5, 5]", style)),
        ))
    };
    let dashed = make(r#", "style": { "hidden": "dashed" }"#);
    let lines: Vec<Line> = paths(&dashed).iter().map(|path| path.stroke.line).collect();
    assert_eq!(lines, [Line::Solid, Line::Dashed, Line::Solid, Line::Solid]);
    // 描かないときは，隠れた部分がなくなる．軸は，2本に分かれて残る．
    let none = make(r#", "style": { "hidden": "none" }"#);
    let lines: Vec<Line> = paths(&none).iter().map(|path| path.stroke.line).collect();
    assert_eq!(lines, [Line::Solid, Line::Solid, Line::Solid]);
    assert!(paths(&none)[0].arrow.is_none());
    assert!(paths(&none)[1].arrow.is_some());
}

#[test]
fn 最初の空間の図は_3本の軸と球の輪郭と3つの名前からできる() {
    let figure = figure_of(SPHERE_WITH_AXES);
    let names: Vec<&str> = labels(&figure)
        .iter()
        .map(|label| label.tex.as_str())
        .collect();
    assert_eq!(names, ["$x$", "$y$", "$z$"]);
    // 各軸は，球に隠れる部分で3つに分かれ，球の輪郭が1本．
    let lines: Vec<Line> = paths(&figure).iter().map(|path| path.stroke.line).collect();
    let dotted = lines.iter().filter(|line| **line == Line::Dotted).count();
    assert_eq!(dotted, 3);
    assert_eq!(lines.len(), 3 * 3 + 1);
}

// ---- 空間の曲線 ----

fn equator() -> String {
    r#"{ "id": "equator", "type": "curve", "var": "t",
         "expr": ["2*cos(t)", "2*sin(t)", "0"], "domain": [0, "2*pi"] }"#
        .to_owned()
}

#[test]
fn 球の面の上の曲線は_遠い側の半分が点線になる() {
    let (a, e) = (60.0_f64, 20.0_f64);
    let figure = figure_of(&space_scene(a, e, "1cm", &format!("{}, {BALL}", equator())));
    let curve = curve_paths(&figure);
    let lines: Vec<Line> = curve.iter().map(|path| path.stroke.line).collect();
    assert_eq!(
        lines.iter().filter(|line| **line == Line::Dotted).count(),
        1,
        "{lines:?}"
    );
    // 見える半分は，パラメータの始まり(t=0)をまたぐので，2つに分かれる．
    assert_eq!(
        lines.iter().filter(|line| **line == Line::Solid).count(),
        2,
        "{lines:?}"
    );
    // 実線と点線は，球の輪郭の左右の端で切り替わる．
    let hidden = curve
        .iter()
        .find(|path| path.stroke.line == Line::Dotted)
        .unwrap();
    let ends = [hidden.points[0], *hidden.points.last().unwrap()];
    let mut xs = [ends[0][0], ends[1][0]];
    xs.sort_by(f64::total_cmp);
    assert!(
        close(xs[0], -2.0, 1e-4) && close(xs[1], 2.0, 1e-4),
        "{ends:?}"
    );
    // 点は，曲線の上にある．
    for path in &curve {
        for point in &path.points {
            let on_curve = (0..20_000).any(|step| {
                let t = f64::from(step) / 20_000.0 * std::f64::consts::TAU;
                close2(
                    *point,
                    project([2.0 * t.cos(), 2.0 * t.sin(), 0.0], a, e, 1.0),
                    2e-3,
                )
            });
            assert!(on_curve, "{point:?}");
        }
    }
}

#[test]
fn 曲線の実線と点線の長さは_視線に沿って刻んで確かめた隠れ方と合う() {
    let (a, e) = (60.0_f64, 20.0_f64);
    let figure = figure_of(&space_scene(a, e, "1cm", &format!("{}, {BALL}", equator())));
    for path in curve_paths(&figure) {
        // 曲線の点を，パラメータに戻し，赤道の点の隠れ方と比べる．
        let middle = path.points[path.points.len() / 2];
        let t = (0..20_000)
            .map(|step| f64::from(step) / 20_000.0 * std::f64::consts::TAU)
            .min_by(|s, t| {
                let d = |u: &f64| {
                    let q = project([2.0 * u.cos(), 2.0 * u.sin(), 0.0], a, e, 1.0);
                    (q[0] - middle[0]).hypot(q[1] - middle[1])
                };
                d(s).total_cmp(&d(t))
            })
            .unwrap();
        let hidden = ball_hides([2.0 * t.cos(), 2.0 * t.sin(), 0.0], a, e);
        // 面の上の点は，境界の誤差で，見え方が食い違うことがあるので，表と裏の中央付近で比べる．
        let facing = (t - a.to_radians()).cos();
        if facing.abs() > 0.3 {
            assert_eq!(path.stroke.line == Line::Dotted, hidden, "t={t}");
        }
    }
}

#[test]
fn 空間の曲線の線の種類と_隠れた部分の種類は_それぞれ選べる() {
    let curve = r#"{ "id": "equator", "type": "curve", "var": "t",
        "expr": ["2*cos(t)", "2*sin(t)", "0"], "domain": [0, "2*pi"],
        "style": { "line": "dashed", "hidden": "none" } }"#;
    let figure = figure_of(&space_scene(60.0, 20.0, "1cm", &format!("{curve}, {BALL}")));
    let lines: Vec<Line> = curve_paths(&figure)
        .iter()
        .map(|path| path.stroke.line)
        .collect();
    // 描かない部分を除き，パラメータの始まりで分かれた，見える半分が残る．
    assert_eq!(lines, [Line::Dashed, Line::Dashed]);
}

#[test]
fn 球のない図では_空間の曲線は隠れない() {
    let helix = r#"{ "id": "helix", "type": "curve", "var": "t",
        "expr": ["cos(t)", "sin(t)", "t/5"], "domain": [0, "4*pi"] }"#;
    let figure = figure_of(&space_scene(60.0, 20.0, "1cm", helix));
    let all = paths(&figure);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].stroke.line, Line::Solid);
    assert!(all[0].points.len() > 50);
    assert!(close2(
        all[0].points[0],
        project([1.0, 0.0, 0.0], 60.0, 20.0, 1.0),
        1e-9
    ));
}

#[test]
fn 式の誤りは_空間の曲線でも_項目と位置を示す() {
    let error = parse_scene(&space_scene(
        60.0,
        20.0,
        "1cm",
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t", "2t"], "domain": [0, 1] }"#,
    ))
    .expect_err("誤りになる");
    assert_eq!(error.object.as_deref(), Some("c"));
    assert!(
        matches!(error.kind, figure::ErrorKind::Expression { index: 2, .. }),
        "{error}"
    );
}
