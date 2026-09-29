//! フーリエ級数の部分和(`fourier_series`)，1次元のフーリエ変換(`fourier_transform`)，2次元のフーリエ変換の
//! 強さ(`fourier_intensity`)を確かめる．値は，閉じた形が分かる関数(のこぎり波，矩形波，箱形，Gauss関数，
//! 円形と長方形の開口)で確かめる．

#![allow(
    clippy::as_conversions,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    clippy::arithmetic_side_effects
)]

use std::f64::consts::PI;

use figure::figure::{Figure, Item, RasterItem};
use figure::{parse_scene, render};

fn scene(x: [f64; 2], y: [f64; 2], objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [{}, {}], "y": [{}, {}], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#,
        x[0], x[1], y[0], y[1]
    )
}

fn figure_of(x: [f64; 2], y: [f64; 2], objects: &str) -> Figure {
    render(&parse_scene(&scene(x, y, objects)).expect("読める")).expect("描ける")
}

/// 折れ線の点．見える範囲の縁で切った端(線分の上の点)は除く．
fn curve_points(figure: &Figure, x: [f64; 2], y: [f64; 2]) -> Vec<[f64; 2]> {
    let inside = |p: &[f64; 2]| {
        p[0] > x[0] + 1e-9 && p[0] < x[1] - 1e-9 && p[1] > y[0] + 1e-9 && p[1] < y[1] - 1e-9
    };
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Path(path) => Some(
                path.points
                    .iter()
                    .copied()
                    .filter(inside)
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect()
}

/// のこぎり波`f(x) = x`(`-π < x < π`)の部分和．係数は`b_n = 2(-1)^{n+1}/n`．
fn sawtooth_sum(x: f64, terms: u32) -> f64 {
    (1..=terms)
        .map(|n| {
            let n = f64::from(n);
            let sign = if n % 2.0 == 1.0 { 1.0 } else { -1.0 };
            2.0 * sign * (n * x).sin() / n
        })
        .sum()
}

#[test]
fn のこぎり波の部分和を描く() {
    let (x, y) = ([-7.0, 7.0], [-4.0, 4.0]);
    let figure = figure_of(
        x,
        y,
        r#"{ "id": "s", "type": "fourier_series", "expr": "x", "period": ["-pi", "pi"], "terms": 5 }"#,
    );
    let points = curve_points(&figure, x, y);
    assert!(points.len() > 50);
    // 係数は，1周期の中点則(8192点)で求める．不連続な関数では，誤差が点の数の2乗に反比例して残る．
    for [px, py] in points {
        assert!(
            (py - sawtooth_sum(px, 5)).abs() < 1e-5,
            "({px}, {py})：{}",
            sawtooth_sum(px, 5)
        );
    }
}

#[test]
fn 項の数を媒介変数にできる_0項なら平均値の定数である() {
    let (x, y) = ([-3.0, 3.0], [-2.0, 2.0]);
    let figure = figure_of(
        x,
        y,
        r#"{ "id": "n", "type": "parameter", "value": 0 },
           { "id": "s", "type": "fourier_series", "expr": "x^2", "period": [-1, 1], "terms": "n" }"#,
    );
    // x^2の[-1, 1]での平均は1/3．
    for [_, py] in curve_points(&figure, x, y) {
        assert!((py - 1.0 / 3.0).abs() < 1e-6, "{py}");
    }
}

#[test]
fn 矩形波の振幅のスペクトルは奇数次だけに立つ() {
    // f(x) = x/|x|．b_n = 4/(nπ)(nが奇数)，偶数次は0．
    let (x, y) = ([-0.5, 8.0], [-0.5, 2.0]);
    let figure = figure_of(
        x,
        y,
        r#"{ "id": "s", "type": "fourier_series", "expr": "x / abs(x)", "period": ["-pi", "pi"], "terms": 7, "mode": "amplitude" }"#,
    );
    let tops: Vec<[f64; 2]> = figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Dot(dot) => Some(dot.at),
            _ => None,
        })
        .collect();
    assert_eq!(tops.len(), 8, "0次から7次まで：{tops:?}");
    for [n, height] in tops {
        let expected = if n.round() % 2.0 == 1.0 {
            4.0 / (n * PI)
        } else {
            0.0
        };
        assert!(
            (height - expected).abs() < 1e-4,
            "{n}：{height}，{expected}"
        );
    }
}

#[test]
fn 箱形の関数のフーリエ変換はsinc関数である() {
    // f = 1(-1 <= x <= 1)．F(k) = 2 sin k / k．
    let (x, y) = ([-15.0, 15.0], [-1.0, 3.0]);
    let figure = figure_of(
        x,
        y,
        r#"{ "id": "f", "type": "fourier_transform", "expr": "1", "support": [-1, 1], "part": "re" }"#,
    );
    let points = curve_points(&figure, x, y);
    assert!(points.len() > 50);
    for [k, value] in points {
        let expected = if k == 0.0 { 2.0 } else { 2.0 * k.sin() / k };
        assert!((value - expected).abs() < 1e-8, "{k}：{value}，{expected}");
    }
}

#[test]
fn gauss関数のフーリエ変換はgauss関数で_平行移動は位相になる() {
    let (x, y) = ([-6.0, 6.0], [-2.0, 2.0]);
    for (part, expected) in [
        (
            "abs",
            Box::new(|k: f64| PI.sqrt() * (-k * k / 4.0).exp()) as Box<dyn Fn(f64) -> f64>,
        ),
        (
            "re",
            Box::new(|k: f64| PI.sqrt() * (-k * k / 4.0).exp() * k.cos()),
        ),
        (
            "im",
            Box::new(|k: f64| -PI.sqrt() * (-k * k / 4.0).exp() * k.sin()),
        ),
        ("power", Box::new(|k: f64| PI * (-k * k / 2.0).exp())),
    ] {
        let figure = figure_of(
            x,
            y,
            &format!(
                r#"{{ "id": "f", "type": "fourier_transform", "expr": "exp(-(x - 1)^2)", "support": [-9, 11], "part": "{part}" }}"#
            ),
        );
        for [k, value] in curve_points(&figure, x, y) {
            assert!((value - expected(k)).abs() < 1e-8, "{part}，{k}：{value}");
        }
    }
}

#[test]
fn 変換の変数の範囲を書ける_複素数の式も使える() {
    let (x, y) = ([-10.0, 10.0], [-1.0, 3.0]);
    let figure = figure_of(
        x,
        y,
        r#"{ "id": "f", "type": "fourier_transform", "expr": "exp(i * x) * exp(-x^2)", "support": [-8, 8], "domain": [0, 3] }"#,
    );
    let points = curve_points(&figure, x, y);
    assert!(points.iter().all(|p| p[0] >= -1e-9 && p[0] <= 3.0 + 1e-9));
    // e^{ix}を掛けると，kの向きに1ずれる．
    for [k, value] in points {
        let expected = PI.sqrt() * (-(k - 1.0) * (k - 1.0) / 4.0).exp();
        assert!((value - expected).abs() < 1e-8, "{k}：{value}");
    }
}

fn raster(figure: &Figure) -> &RasterItem {
    figure
        .items
        .iter()
        .find_map(|item| match item {
            Item::Raster(raster) => Some(raster),
            _ => None,
        })
        .expect("画像がある")
}

/// 灰色(黒から白)の画素の明るさを，0から1にする．
fn level(raster: &RasterItem, column: usize, row: usize) -> f64 {
    f64::from(raster.pixels[row * raster.columns + column][0]) / 255.0
}

#[test]
fn 長方形の開口の強さはsinc関数の2乗の積である() {
    // 開口[-1, 1] × [-0.5, 0.5]．強さは(sin kx / kx)^2 (sin(ky/2) / (ky/2))^2に比例する．
    let (x, y) = ([-8.0, 8.0], [-8.0, 8.0]);
    let figure = figure_of(
        x,
        y,
        r#"{ "id": "a", "type": "fourier_intensity", "expr": "1", "support": [[-1, 1], [-0.5, 0.5]],
             "colormap": "gray", "range": [0, 1], "resolution": 64 }"#,
    );
    let image = raster(&figure);
    assert_eq!((image.columns, image.rows), (64, 64));
    let sinc = |t: f64| if t == 0.0 { 1.0 } else { t.sin() / t };
    for (column, row) in [(32_u8, 32_u8), (40, 32), (32, 20), (45, 50), (10, 12)] {
        let kx = -8.0 + (f64::from(column) + 0.5) * 0.25;
        let ky = 8.0 - (f64::from(row) + 0.5) * 0.25;
        let expected = (sinc(kx) * sinc(ky / 2.0)).powi(2);
        // 画素の中心の強さを，画像全体の最大で割った値．中心の画素は(0.125, -0.125)なので，最大は1よりわずかに小さい．
        let peak = (sinc(0.125) * sinc(0.0625)).powi(2);
        let actual = level(image, usize::from(column), usize::from(row));
        assert!(
            (actual - expected / peak).abs() < 3.0 / 255.0,
            "({kx}, {ky})：{actual}，{expected}"
        );
    }
}

#[test]
fn 円形の開口の強さはairyの図形である() {
    // 半径1の円．強さは(2 J1(k) / k)^2に比例し，最初の暗い環はk = 3.8317にある．
    let (x, y) = ([-6.0, 6.0], [-6.0, 6.0]);
    let figure = figure_of(
        x,
        y,
        r#"{ "id": "a", "type": "fourier_intensity", "expr": "(1 - x^2 - y^2) / abs(1 - x^2 - y^2) / 2 + 1/2",
             "support": [[-1, 1], [-1, 1]], "samples": 256, "colormap": "gray", "range": [0, 1], "resolution": 120 }"#,
    );
    let image = raster(&figure);
    let airy = |k: f64| (2.0 * puruspe::Jn(1, k) / k).powi(2);
    let center = level(image, 60, 60);
    assert!(center > 0.99, "{center}");
    // 行60の中心は，ky = -0.05．列ごとのkxで比べる．
    for column in [60_u8, 70, 80, 90, 100, 110] {
        let kx = -6.0 + (f64::from(column) + 0.5) * 0.1;
        let k = kx.hypot(0.05);
        let expected = airy(k) / airy(0.05f64.hypot(0.05));
        let actual = level(image, usize::from(column), 60);
        assert!(
            (actual - expected).abs() < 0.02,
            "k = {k}：{actual}，{expected}"
        );
    }
}

#[test]
fn 振幅と対数の目盛を選べる() {
    let (x, y) = ([-8.0, 8.0], [-8.0, 8.0]);
    for extra in [r#", "quantity": "amplitude""#, r#", "scale": "log""#] {
        let figure = figure_of(
            x,
            y,
            &format!(
                r#"{{ "id": "a", "type": "fourier_intensity", "expr": "1", "support": [[-1, 1], [-1, 1]], "resolution": 32 {extra} }}"#
            ),
        );
        assert_eq!(raster(&figure).pixels.len(), 32 * 32);
    }
}

#[test]
fn 範囲の誤りと上限() {
    for object in [
        r#"{ "id": "s", "type": "fourier_series", "expr": "x", "period": [1, 1], "terms": 3 }"#,
        r#"{ "id": "s", "type": "fourier_series", "expr": "x", "period": [0, 1], "terms": 100000 }"#,
        r#"{ "id": "s", "type": "fourier_series", "expr": "x", "period": [0, 1], "terms": -1 }"#,
        r#"{ "id": "f", "type": "fourier_transform", "expr": "1", "support": [2, 1] }"#,
        r#"{ "id": "a", "type": "fourier_intensity", "expr": "1", "support": [[-1, 1], [-1, 1]], "samples": 100000 }"#,
    ] {
        let result = parse_scene(&scene([-1.0, 1.0], [-1.0, 1.0], object));
        assert!(result.is_err(), "{object}");
    }
}
