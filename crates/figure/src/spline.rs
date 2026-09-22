//! スプライン曲線．Catmull-Romの方法で，与えた点をすべて，順に通る滑らかな曲線を作る．
//!
//! ベジエ曲線(`bezier.rs`)と違い，制御点自身が曲線の上にある．両端の外側の点(仮想の`P_{-1}`と
//! `P_n`)は，それぞれ最初と最後の点を重ねて作る(一様Catmull-Rom，鎖線でない既定の形)．

/// 4点の重みつき和．次元(要素数)は揃っている前提．
fn combine(coefficients: [f64; 4], points: [&[f64]; 4]) -> Vec<f64> {
    let dimension = points[0].len();
    (0..dimension)
        .map(|index| {
            coefficients
                .iter()
                .zip(&points)
                .map(|(c, p)| c * p.get(index).copied().unwrap_or(0.0))
                .sum()
        })
        .collect()
}

/// スプライン曲線の点．`points`は，曲線が順に通る点の列で，各点の次元(平面なら2，空間なら3)は
/// 揃っている前提である．`t`は0から1で，曲線全体に対する位置(0が最初の点，1が最後の点)である．
/// 点が2つ未満なら，`None`を返す．
#[must_use]
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn catmull_rom_point(points: &[Vec<f64>], t: f64) -> Option<Vec<f64>> {
    let segments = points.len().saturating_sub(1);
    if segments == 0 {
        return None;
    }
    let scaled = t.clamp(0.0, 1.0) * f64::from(u32::try_from(segments).unwrap_or(u32::MAX));
    // 0以上segments未満の整数である(tを0から1に収めてあるので)．
    let index = (scaled.floor() as usize).min(segments.saturating_sub(1));
    let local = scaled - f64::from(u32::try_from(index).unwrap_or(u32::MAX));
    let at = |offset: isize| -> &[f64] {
        let position = index.saturating_add_signed(offset).min(segments);
        points.get(position).map_or(&[], Vec::as_slice)
    };
    let (t2, t3) = (local * local, local * local * local);
    // Catmull-Romの基底(一様，P1からP2への区間)．t=0でP1，t=1でP2に一致する．
    let coefficients = [
        -0.5 * local + t2 - 0.5 * t3,
        1.0 - 2.5 * t2 + 1.5 * t3,
        0.5 * local + 2.0 * t2 - 1.5 * t3,
        -0.5 * t2 + 0.5 * t3,
    ];
    Some(combine(coefficients, [at(-1), at(0), at(1), at(2)]))
}
