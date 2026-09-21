//! `TikZ`の`Stealth`の矢じりの形を確かめる．
//!
//! 寸法は，pgfのソース(`pgflibraryarrows.meta.code.tex`)の式から，手で求めた値と，
//! 「矢じりの輪郭を，線幅の半分だけ外へ広げると，長さL，幅Wの箱にほぼ収まる」という設計の意図で確かめる．
//! pgfは，名目の形(長さ，幅，くぼみ)で角の長さを求めるので，実際の角は，線幅の1割ほどはみ出す．

#![allow(
    clippy::arithmetic_side_effects,
    clippy::float_cmp,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use figure::arrow::Stealth;

const EPSILON: f64 = 1e-9;

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < EPSILON
}

#[test]
fn 既定の線幅の寸法はソースの式どおりである() {
    let stealth = Stealth::new(0.4);
    // L = 3pt + 4.5 * 0.4pt，W = 0.75 * L，I = 0.325 * L．
    assert!(close(stealth.length, 4.8));
    assert!(close(stealth.width, 3.6));
    assert!(close(stealth.inset, 1.56));
    assert!(close(stealth.line_width, 0.4));
}

#[test]
fn 線幅が太いと長さは線幅に比例して伸びる() {
    let stealth = Stealth::new(1.2);
    assert!(close(stealth.length, 3.0 + 4.5 * 1.2));
    assert!(close(stealth.width, 0.75 * stealth.length));
    assert!(close(stealth.inset, 0.325 * stealth.length));
}

#[test]
fn 輪郭の線幅は長さとくぼみの差の4分の1を超えない() {
    // 線幅が十分に太いと，上限に当たる．L = 3 + 4.5 * 20 = 93，I = 0.325 * 93．
    let stealth = Stealth::new(20.0);
    let cap = 0.25 * (stealth.length - stealth.inset);
    assert!(close(stealth.line_width, cap));
    assert!(stealth.line_width < 20.0);
}

#[test]
fn 輪郭は軸に対して上下に対称である() {
    let [tip, upper, inset, lower] = Stealth::new(0.4).outline();
    assert!(close(tip[1], 0.0));
    assert!(close(inset[1], 0.0));
    assert!(close(upper[0], lower[0]));
    assert!(close(upper[1], -lower[1]));
    assert!(upper[1] > 0.0);
}

/// 多角形を，辺の法線の向きに`half`だけ広げた頂点(角は，とがらせる)．反時計回りの向きで渡す．
fn mitered_offset(points: &[[f64; 2]], half: f64) -> Vec<[f64; 2]> {
    let count = points.len();
    let normal = |from: [f64; 2], to: [f64; 2]| {
        let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
        let length = dx.hypot(dy);
        [dy / length, -dx / length]
    };
    (0..count)
        .map(|index| {
            let previous = points[(index + count - 1) % count];
            let current = points[index];
            let next = points[(index + 1) % count];
            let n1 = normal(previous, current);
            let n2 = normal(current, next);
            let scale = half / (1.0 + n1[0] * n2[0] + n1[1] * n2[1]);
            [
                current[0] + (n1[0] + n2[0]) * scale,
                current[1] + (n1[1] + n2[1]) * scale,
            ]
        })
        .collect()
}

fn signed_area(points: &[[f64; 2]]) -> f64 {
    let count = points.len();
    (0..count)
        .map(|index| {
            let (a, b) = (points[index], points[(index + 1) % count]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        / 2.0
}

#[test]
fn 輪郭を線幅の半分だけ広げると_長さと幅の箱にほぼ収まる() {
    for line_width in [0.4, 0.6, 0.8, 1.2] {
        let stealth = Stealth::new(line_width);
        let mut outline = stealth.outline().to_vec();
        if signed_area(&outline) < 0.0 {
            outline.reverse();
        }
        let outer = mitered_offset(&outline, stealth.line_width / 2.0);
        let min_x = outer.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let max_x = outer.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let max_y = outer.iter().map(|p| p[1].abs()).fold(0.0, f64::max);
        let tolerance = 0.1 * stealth.line_width;
        assert!(min_x.abs() < tolerance, "lw={line_width}: 後ろの端 {min_x}");
        assert!(
            (max_x - stealth.length).abs() < tolerance,
            "lw={line_width}: 先端 {max_x}"
        );
        assert!(
            (max_y - stealth.width / 2.0).abs() < tolerance,
            "lw={line_width}: 幅 {max_y}"
        );
    }
}

#[test]
fn 軸の線は_くぼみの内側で止まり_先端の外へ出ない() {
    let stealth = Stealth::new(0.4);
    let inset_outer = stealth.outline()[2][0];
    // 線は，くぼみの点より前で止まる(矢じりの中に隠れる)．先端よりは後ろである．
    assert!(stealth.line_end < inset_outer + 1e-9 + stealth.line_width);
    assert!(stealth.line_end > 0.0);
    assert!(stealth.line_end < stealth.length);
}

#[test]
fn 位置を決めると_先端が軸の端の手前に来て_向きに従って回る() {
    let stealth = Stealth::new(0.4);
    let unit = 0.1;
    let placed = stealth.place([10.0, 0.0], [1.0, 0.0], unit);
    // 矢じりの後ろは，軸の端から，長さの分だけ戻った点である．
    let back_x = 10.0 - stealth.length * unit;
    assert!(close(placed.line_end[0], back_x + stealth.line_end * unit));
    assert!(close(placed.line_end[1], 0.0));
    assert!(close(
        placed.polygon[0][0],
        back_x + stealth.outline()[0][0] * unit
    ));

    // y軸の向きに向けると，x方向の成分が，y方向へ回る．
    let up = stealth.place([0.0, 10.0], [0.0, 1.0], unit);
    assert!(close(up.line_end[0], 0.0));
    assert!(close(up.line_end[1], back_x + stealth.line_end * unit));
    // 上の後ろの角は，向きの左側(-x側)に来る．
    assert!(up.polygon[1][0] < 0.0);
    assert!(close(up.polygon[1][0], -stealth.outline()[1][1] * unit));
}
