//! 曲線の標本化を確かめる．

#![allow(
    clippy::float_cmp,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used
)]

use figure::sample::sample;

const SY: f64 = 2.0;
const TOLERANCE_WITH_SLACK: f64 = 0.004;

/// 点から，線分までの距離．
fn distance_to_segment(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let squared = dx * dx + dy * dy;
    let along = if squared == 0.0 {
        0.0
    } else {
        (((point[0] - a[0]) * dx + (point[1] - a[1]) * dy) / squared).clamp(0.0, 1.0)
    };
    (point[0] - (a[0] + along * dx)).hypot(point[1] - (a[1] + along * dy))
}

/// 折れ線の各線分と，その間の曲線の点(横軸の値で決める)との，最大の距離．
fn max_distance(path: &[[f64; 2]], curve: impl Fn(f64) -> f64) -> (f64, f64) {
    let mut worst = (0.0, 0.0);
    for pair in path.windows(2) {
        for step in 1..8 {
            let x = pair[0][0] + (pair[1][0] - pair[0][0]) * f64::from(step) / 8.0;
            let distance = distance_to_segment([x, curve(x)], pair[0], pair[1]);
            if distance > worst.0 {
                worst = (distance, x);
            }
        }
    }
    worst
}

#[allow(clippy::unnecessary_wraps)]
fn sine(x: f64) -> Option<[f64; 2]> {
    Some([x, x.sin() * SY])
}

#[test]
fn 端の点を含み_曲線の上にある() {
    let paths = sample(sine, -7.0, 7.0);
    assert_eq!(paths.len(), 1);
    let path = &paths[0];
    assert_eq!(path.first().unwrap()[0], -7.0);
    assert_eq!(path.last().unwrap()[0], 7.0);
    for point in path {
        assert!((point[1] - point[0].sin() * SY).abs() < 1e-12);
    }
}

#[test]
fn 折れ線は曲線から許容の距離に収まる() {
    let path = &sample(sine, -7.0, 7.0)[0];
    let (distance, x) = max_distance(path, |x| x.sin() * SY);
    assert!(distance < TOLERANCE_WITH_SLACK, "距離 {distance} (x = {x})");
}

#[test]
fn 曲がりの大きいところに点を多くとる() {
    // 釣鐘形の曲線は，中央の近くで曲がりが大きく，裾は，ほぼ平らである．
    let path = &sample(|x| Some([x, 3.0 * (-x * x).exp()]), -4.0, 4.0)[0];
    let center = path.iter().filter(|p| p[0].abs() < 1.5).count();
    let tail = path.iter().filter(|p| p[0].abs() > 2.5).count();
    assert!(center > 3 * tail, "中央 {center}，裾 {tail}");
}

#[test]
fn 点の数は多すぎない() {
    let path = &sample(sine, -7.0, 7.0)[0];
    assert!(path.len() > 32);
    assert!(path.len() < 2000, "{}", path.len());
}

#[test]
fn 一周期を超える細かい波も取りこぼさない() {
    // 周期が，最初の区間の幅より短い波．
    let path = &sample(|x| Some([x, (20.0 * x).sin()]), 0.0, 3.0)[0];
    let (distance, x) = max_distance(path, |x| (20.0 * x).sin());
    assert!(distance < TOLERANCE_WITH_SLACK, "距離 {distance} (x = {x})");
}

#[test]
fn 定義域の外では線を切る() {
    // sqrt(x)は，x < 0で定義されない．
    let paths = sample(|x| (x >= 0.0).then_some([x, x.sqrt()]), -1.0, 1.0);
    assert_eq!(paths.len(), 1);
    let first = paths[0][0];
    assert!(first[0] >= 0.0 && first[0] < 0.01, "始まり {first:?}");
    assert_eq!(paths[0].last().unwrap()[0], 1.0);
}

#[test]
fn 途中に定義されない区間があると線が分かれる() {
    let paths = sample(|x| (x.abs() > 0.5).then_some([x, x]), -1.0, 1.0);
    assert_eq!(paths.len(), 2);
    assert!(paths[0].last().unwrap()[0] <= -0.5 + 0.01);
    assert!(paths[1][0][0] >= 0.5 - 0.01);
}

#[test]
fn 全体が定義されないと何も返さない() {
    assert!(sample(|_| None, 0.0, 1.0).is_empty());
}

#[test]
fn 大きすぎる値は線を切る() {
    // tan(x)は，x = pi/2の近くで，非常に大きくなる．
    let paths = sample(|x| Some([x, x.tan()]), 0.0, 3.0);
    for path in &paths {
        for point in path {
            assert!(point[1].abs() <= 300.0, "{point:?}");
        }
    }
    assert!(paths.len() >= 2);
}

#[test]
fn 同じ入力は同じ結果になる() {
    assert_eq!(sample(sine, -7.0, 7.0), sample(sine, -7.0, 7.0));
}

#[test]
fn 一点だけの線は捨てる() {
    let paths = sample(|x| (x == 0.0).then_some([x, 0.0]), 0.0, 1.0);
    assert!(paths.is_empty());
}
