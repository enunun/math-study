//! 数値微分(中心差分)．接線と接平面の向きを求めるために使う．
//!
//! 刻みは，`refine.rs`の磨き込みと同じ考え方で，パラメータの範囲に対する割合にする．

/// 数値微分の刻み(パラメータの範囲に対する割合)．
const STEP_FRACTION: f64 = 1e-6;
/// 刻みの下限．範囲の幅が0に近いときの床にする．
const MIN_STEP: f64 = 1e-9;

/// 範囲の幅から，中心差分の刻みを決める．
fn step(domain_width: f64) -> f64 {
    (domain_width.abs() * STEP_FRACTION).max(MIN_STEP)
}

/// 座標の並びを返す`f`の，`t`における中心差分．曲線の向きや，曲面の偏微分を求めるために使う．
/// `f`が，`t + h`か`t - h`で`None`を返せば，`None`を返す．
#[must_use]
pub fn central_difference_point<const N: usize>(
    f: impl Fn(f64) -> Option<[f64; N]>,
    t: f64,
    domain_width: f64,
) -> Option<[f64; N]> {
    let h = step(domain_width);
    let (forward, backward) = (f(t + h)?, f(t - h)?);
    let mut result = [0.0; N];
    for (slot, (high, low)) in result.iter_mut().zip(forward.iter().zip(&backward)) {
        *slot = (high - low) / (2.0 * h);
    }
    Some(result)
}
