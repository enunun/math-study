//! 色で値を表す図(`heatmap`)と，複素関数の色塗り(`domain_coloring`)を確かめる．どちらも，点ごとに
//! 色を決めた画像(`Item::Raster`)になる．SVGにはPNGの画像として埋め込み，TikZには，粗い升目の
//! 塗りつぶしとして書き出す．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, RasterItem};
use figure::tikz::to_tikz;
use figure::{parse_scene, render};

fn plane_scene(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-2, 2], "y": [-1, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn figure_of(objects: &str) -> Figure {
    render(&parse_scene(&plane_scene(objects)).expect("読める")).expect("描ける")
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

/// 画像の画素．行は上から，列は左から数える．
fn pixel(raster: &RasterItem, column: usize, row: usize) -> [u8; 4] {
    raster.pixels[row * raster.columns + column]
}

fn brightness([r, g, b, _]: [u8; 4]) -> u32 {
    u32::from(r) + u32::from(g) + u32::from(b)
}

fn heatmap(extra: &str) -> String {
    format!(r#"{{ "id": "h", "type": "heatmap", "expr": "x" {extra} }}"#)
}

#[test]
fn 画像は見える範囲に置かれ_解像度は長い辺の画素の数である() {
    let figure = figure_of(&heatmap(r#", "resolution": 100"#));
    let image = raster(&figure);
    assert_eq!(image.min, [-2.0, -1.0]);
    assert_eq!(image.max, [2.0, 1.0]);
    assert_eq!((image.columns, image.rows), (100, 50));
    assert_eq!(image.pixels.len(), 100 * 50);
    assert!(
        image.href.starts_with("data:image/png;base64,"),
        "{}",
        &image.href[..40]
    );
}

#[test]
fn 範囲を書くと_その範囲に画像を置く() {
    let figure = figure_of(&heatmap(r#", "domain": [[0, 1], [-0.5, 3]]"#));
    let image = raster(&figure);
    // 見える範囲の外は切り取る．
    assert_eq!(image.min, [0.0, -0.5]);
    assert_eq!(image.max, [1.0, 1.0]);
}

#[test]
fn 灰色の色の対応では_値が大きいほど明るい() {
    let figure = figure_of(&heatmap(r#", "colormap": "gray", "resolution": 40"#));
    let image = raster(&figure);
    let left = pixel(image, 0, 10);
    let right = pixel(image, image.columns - 1, 10);
    assert!(brightness(left) < 30, "{left:?}");
    assert!(brightness(right) > 3 * 225, "{right:?}");
    // 左から右へ，明るさは増える．
    let row: Vec<u32> = (0..image.columns)
        .map(|c| brightness(pixel(image, c, 5)))
        .collect();
    assert!(row.windows(2).all(|pair| pair[0] <= pair[1]), "{row:?}");
    // 反転した灰色は逆になる．
    let inverse = figure_of(&heatmap(
        r#", "colormap": "gray_inverse", "resolution": 40"#,
    ));
    let image = raster(&inverse);
    assert!(brightness(pixel(image, 0, 10)) > 3 * 225);
}

#[test]
fn 値の範囲を書くと_外の値は端の色になる() {
    let figure = figure_of(&heatmap(
        r#", "colormap": "gray", "range": [-0.5, 0.5], "resolution": 40"#,
    ));
    let image = raster(&figure);
    assert_eq!(pixel(image, 0, 3), pixel(image, 2, 3));
    assert_eq!(pixel(image, 0, 3)[..3], [0, 0, 0]);
    assert_eq!(pixel(image, image.columns - 1, 3)[..3], [255, 255, 255]);
}

#[test]
fn 対数の目盛では_正でない値は透明になる() {
    let figure = figure_of(&heatmap(r#", "scale": "log", "resolution": 40"#));
    let image = raster(&figure);
    assert_eq!(pixel(image, 0, 3)[3], 0);
    assert_eq!(pixel(image, image.columns - 1, 3)[3], 255);
}

#[test]
fn 値のない点は透明になる() {
    let figure =
        figure_of(r#"{ "id": "h", "type": "heatmap", "expr": "sqrt(x)", "resolution": 40 }"#);
    let image = raster(&figure);
    assert_eq!(pixel(image, 0, 3)[3], 0);
    assert_eq!(pixel(image, image.columns - 1, 3)[3], 255);
}

#[test]
fn 変数の名前と媒介変数と複素数を使える() {
    let figure = figure_of(
        r#"{ "id": "c", "type": "parameter", "value": 2 },
           { "id": "h", "type": "heatmap", "vars": ["u", "v"], "expr": "abs(u + i*v) * c", "resolution": 40 }"#,
    );
    let image = raster(&figure);
    // 中央(原点の近く)が最も暗い(viridisは，小さい値が暗い)．
    let center = brightness(pixel(image, image.columns / 2, image.rows / 2));
    let corner = brightness(pixel(image, 0, 0));
    assert!(center < corner, "{center}，{corner}");
}

#[test]
fn 複素関数の色塗りは偏角を色相で表す() {
    let figure = figure_of(
        r#"{ "id": "d", "type": "domain_coloring", "expr": "z", "shading": "none", "resolution": 80 }"#,
    );
    let image = raster(&figure);
    // 正の実軸(右端，中央の行)は赤，負の実軸(左端)は青緑．
    let [r, g, b, a] = pixel(image, image.columns - 1, image.rows / 2);
    assert!(r > 200 && g < 80 && b < 80 && a == 255, "{r} {g} {b} {a}");
    let [r, g, b, _] = pixel(image, 0, image.rows / 2);
    assert!(r < 80 && g > 150 && b > 150, "{r} {g} {b}");
    // 上の虚軸(偏角π/2)は黄緑．
    let [r, g, b, _] = pixel(image, image.columns / 2, 0);
    assert!(g > 200 && b < 80 && r > 60, "{r} {g} {b}");
}

#[test]
fn 絶対値の陰影は明るさを変える() {
    let plain = figure_of(
        r#"{ "id": "d", "type": "domain_coloring", "expr": "z", "shading": "none", "resolution": 80 }"#,
    );
    let shaded =
        figure_of(r#"{ "id": "d", "type": "domain_coloring", "expr": "z", "resolution": 80 }"#);
    assert_ne!(raster(&plain).pixels, raster(&shaded).pixels);
}

#[test]
fn 値が有限でない点は透明になる() {
    // 画素の中心(±0.5, ±0.5)は極を踏まないので，どれも色がある．
    let figure = figure_of(
        r#"{ "id": "d", "type": "domain_coloring", "var": "w", "expr": "1 / w", "domain": [[-1, 1], [-1, 1]], "resolution": 2 }"#,
    );
    assert!(raster(&figure).pixels.iter().all(|p| p[3] == 255));
    // 0で割った値は，どの画素でも有限でない．
    let figure = figure_of(
        r#"{ "id": "d", "type": "domain_coloring", "var": "w", "expr": "1 / (0 * w)", "resolution": 8 }"#,
    );
    assert!(raster(&figure).pixels.iter().all(|p| p[3] == 0));
}

#[test]
fn tikzでは粗い升目の塗りつぶしになる() {
    let figure = figure_of(&heatmap(r#", "tikz_resolution": 8"#));
    let tikz = to_tikz(&figure);
    let fills = tikz
        .lines()
        .filter(|line| line.contains("rectangle"))
        .count();
    // 8 × 4の升目．同じ色が続く升目は1つにまとめるので，それ以下である．
    assert!(fills > 0 && fills <= 32, "{fills}");
    assert!(tikz.contains("rgb,255:red,"), "{tikz}");
}

#[test]
fn 空間の図では使えない() {
    for object in [
        r#"{ "id": "h", "type": "heatmap", "expr": "x" }"#,
        r#"{ "id": "d", "type": "domain_coloring", "expr": "z" }"#,
    ] {
        let error = parse_scene(&format!(
            r#"{{ "version": "0.1.0", "description": "a",
                 "view": {{ "azimuth": 60, "elevation": 20, "unit": "1cm" }},
                 "objects": [{object}] }}"#
        ))
        .expect_err("誤りになる");
        assert!(error.to_string().contains("平面"), "{error}");
    }
}

#[test]
fn 解像度の上限を超えると誤りになる() {
    let error = parse_scene(&plane_scene(&heatmap(r#", "resolution": 100000"#))).expect_err("誤り");
    assert!(error.to_string().contains("resolution"), "{error}");
}
