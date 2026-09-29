//! 結晶の回折図形(`diffraction`)を確かめる．逆格子点に，構造因子の絶対値に比例する半径(強さに比例する
//! 面積)の点を置く．3次元の結晶は，晶帯軸に垂直な逆格子の断面(ゼロ次のLaueゾーン)を描く．単位胞の数を
//! 与えると，有限の結晶の強さを画像にする．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    clippy::arithmetic_side_effects
)]

use figure::figure::{Figure, Item, RasterItem};
use figure::{parse_scene, render};

fn scene(range: f64, objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-{range}, {range}], "y": [-{range}, {range}], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn figure_of(range: f64, objects: &str) -> Figure {
    render(&parse_scene(&scene(range, objects)).expect("読める")).expect("描ける")
}

/// 点の位置と半径(pt)．
fn spots(figure: &Figure) -> Vec<([f64; 2], f64)> {
    figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Dot(dot) => Some((dot.at, dot.radius)),
            _ => None,
        })
        .collect()
}

fn spot_at(figure: &Figure, target: [f64; 2]) -> Option<f64> {
    spots(figure)
        .into_iter()
        .find(|(at, _)| (at[0] - target[0]).abs() < 1e-9 && (at[1] - target[1]).abs() < 1e-9)
        .map(|(_, radius)| radius)
}

#[test]
fn 正方格子の回折点は整数の点に同じ大きさで並ぶ() {
    let figure = figure_of(
        2.5,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]] }"#,
    );
    let all = spots(&figure);
    // -2から2の整数の組．
    assert_eq!(all.len(), 25, "{all:?}");
    let radius = all[0].1;
    assert!(all.iter().all(|(_, r)| (r - radius).abs() < 1e-12));
    assert!(spot_at(&figure, [2.0, -1.0]).is_some());
}

#[test]
fn 格子定数が2倍なら逆格子の間隔は半分である() {
    let figure = figure_of(
        1.2,
        r#"{ "id": "d", "type": "diffraction", "basis": [[2, 0], [0, 2]] }"#,
    );
    assert!(spot_at(&figure, [0.5, 0.5]).is_some());
    assert!(spot_at(&figure, [1.0, -0.5]).is_some());
}

#[test]
fn 斜交格子の逆格子は双対な基底で張られる() {
    // a1 = (1, 0)，a2 = (1, 2)．b1 = (1, -1/2)，b2 = (0, 1/2)．
    let figure = figure_of(
        3.0,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [1, 2]] }"#,
    );
    assert!(spot_at(&figure, [1.0, -0.5]).is_some());
    assert!(spot_at(&figure, [0.0, 0.5]).is_some());
    assert!(spot_at(&figure, [1.0, 0.0]).is_some());
    assert!(spot_at(&figure, [0.5, 0.0]).is_none());
}

#[test]
fn 面心の長方格子では_hとkの和が奇数の点が消える() {
    let figure = figure_of(
        2.5,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]],
             "atoms": [{ "position": [0, 0] }, { "position": [0.5, 0.5] }] }"#,
    );
    assert!(spot_at(&figure, [1.0, 1.0]).is_some());
    assert!(spot_at(&figure, [2.0, 0.0]).is_some());
    assert!(spot_at(&figure, [1.0, 0.0]).is_none());
    assert!(spot_at(&figure, [0.0, -1.0]).is_none());
}

#[test]
fn 体心立方格子の001晶帯ではhとkの和が偶数の点だけが残る() {
    let figure = figure_of(
        2.5,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
             "atoms": [{ "position": [0, 0, 0] }, { "position": [0.5, 0.5, 0.5] }], "zone": [0, 0, 1] }"#,
    );
    assert!(spot_at(&figure, [1.0, 1.0]).is_some());
    assert!(spot_at(&figure, [2.0, 0.0]).is_some());
    assert!(spot_at(&figure, [1.0, 0.0]).is_none());
    assert_eq!(spots(&figure).len(), 13);
}

#[test]
fn 面心立方格子の111晶帯は六角形の点の並びである() {
    let figure = figure_of(
        3.5,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
             "atoms": [{ "position": [0, 0, 0] }, { "position": [0.5, 0.5, 0] },
                       { "position": [0.5, 0, 0.5] }, { "position": [0, 0.5, 0.5] }],
             "zone": [1, 1, 1] }"#,
    );
    let mut distances: Vec<f64> = spots(&figure)
        .iter()
        .map(|(at, _)| at[0].hypot(at[1]))
        .filter(|distance| *distance > 1e-9)
        .collect();
    distances.sort_by(f64::total_cmp);
    // 最も近いのは{2 -2 0}の6つで，距離は√8．{1 1 -2}は消える．
    let nearest = distances[0];
    assert!((nearest - 8.0_f64.sqrt()).abs() < 1e-9, "{nearest}");
    assert_eq!(
        distances
            .iter()
            .filter(|d| (*d - nearest).abs() < 1e-9)
            .count(),
        6
    );
}

#[test]
fn 岩塩型の構造では_点の半径の比が構造因子の比になる() {
    // 陽イオン(f = 3)と陰イオン(f = 1)．F(111) = 4(3 - 1)，F(002) = 4(3 + 1)．晶帯[1 -1 0]には，どちらも入る．
    let atoms = r#"[{ "position": [0, 0, 0], "factor": 3 }, { "position": [0.5, 0.5, 0], "factor": 3 },
                    { "position": [0.5, 0, 0.5], "factor": 3 }, { "position": [0, 0.5, 0.5], "factor": 3 },
                    { "position": [0.5, 0, 0], "factor": 1 }, { "position": [0, 0.5, 0], "factor": 1 },
                    { "position": [0, 0, 0.5], "factor": 1 }, { "position": [0.5, 0.5, 0.5], "factor": 1 }]"#;
    let figure = figure_of(
        2.5,
        &format!(
            r#"{{ "id": "d", "type": "diffraction", "basis": [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
                 "atoms": {atoms}, "zone": [1, -1, 0], "labels": true }}"#
        ),
    );
    let label_at = |tex: &str| {
        figure
            .items
            .iter()
            .find_map(|item| match item {
                Item::Label(label) if label.tex == tex => Some(label.at),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{tex}がない"))
    };
    let radius_near = |at: [f64; 2]| {
        spots(&figure)
            .into_iter()
            .min_by(|a, b| {
                let da = (a.0[0] - at[0]).hypot(a.0[1] - at[1]);
                let db = (b.0[0] - at[0]).hypot(b.0[1] - at[1]);
                da.total_cmp(&db)
            })
            .unwrap()
            .1
    };
    let odd = radius_near(label_at("$(111)$"));
    let even = radius_near(label_at("$(002)$"));
    assert!((even / odd - 2.0).abs() < 1e-9, "{even}，{odd}");
}

#[test]
fn 原子散乱因子を式で書ける() {
    // f = exp(-q^2)．(1 0)の点はq = 1，(2 0)はq = 2で，半径の比はexp(-4)/exp(-1)．
    let figure = figure_of(
        2.5,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]],
             "atoms": [{ "position": [0, 0], "factor": "exp(-q^2)" }] }"#,
    );
    let one = spot_at(&figure, [1.0, 0.0]).unwrap();
    let two = spot_at(&figure, [2.0, 0.0]).unwrap();
    assert!((two / one - (-3.0_f64).exp()).abs() < 1e-9);
}

#[test]
fn 名前の負の指数には上線を付ける() {
    let figure = figure_of(
        1.5,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]], "labels": true }"#,
    );
    let texts: Vec<String> = figure
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Label(label) => Some(label.tex.clone()),
            _ => None,
        })
        .collect();
    assert!(texts.contains(&r"$(1\bar{1})$".to_owned()), "{texts:?}");
    assert!(texts.contains(&"$(00)$".to_owned()), "{texts:?}");
}

#[test]
fn 半径と点の大きさを指定できる() {
    let figure = figure_of(
        3.0,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]], "radius": 1.5, "spot": "6pt" }"#,
    );
    // |G| <= 1.5：(0,0)，(±1,0)，(0,±1)，(±1,±1)．
    assert_eq!(spots(&figure).len(), 9);
    assert!(spots(&figure).iter().all(|(_, r)| (r - 6.0).abs() < 1e-9));
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

#[test]
fn 有限の結晶の強さは画像になり_逆格子点で最も強い() {
    let figure = figure_of(
        1.5,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]], "cells": [4, 4],
             "resolution": 60, "colormap": "gray", "range": [0, 1] }"#,
    );
    let image = raster(&figure);
    assert!(spots(&figure).is_empty());
    let at = |column: usize, row: usize| {
        f64::from(image.pixels[row * image.columns + column][0]) / 255.0
    };
    // 画素の幅は0.05．逆格子点(0, 0)と(1, 0)の近く(画素の中心は0.025ずれる)は明るく，その間は暗い．
    let near_origin = at(30, 29);
    let near_next = at(50, 29);
    let between = at(40, 29);
    assert!(
        near_origin > 0.9 && near_next > 0.9,
        "{near_origin}，{near_next}"
    );
    assert!(between < 0.05, "{between}");
}

#[test]
fn 誤りを報告する() {
    for object in [
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [2, 0]] }"#,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0, 0], [0, 1, 0]] }"#,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0, 0], [0, 1, 0], [0, 0, 1]], "zone": [0, 0, 0] }"#,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]], "atoms": [{ "position": [0, 0, 0] }] }"#,
        r#"{ "id": "d", "type": "diffraction", "basis": [[1, 0], [0, 1]], "cells": [0, 3] }"#,
    ] {
        assert!(parse_scene(&scene(2.0, object)).is_err(), "{object}");
    }
}
