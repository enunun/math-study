//! 多角形の内側を，平行な斜線で埋める関数を確かめる．内外の判定は，偶奇規則である．

#![allow(
    clippy::float_cmp,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used
)]

use figure::hatch::hatch_lines;

type Segment = [[f64; 2]; 2];

fn square(size: f64) -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [size, 0.0], [size, size], [0.0, size]]
}

fn length(segment: &Segment) -> f64 {
    (segment[1][0] - segment[0][0]).hypot(segment[1][1] - segment[0][1])
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// 線の位置を，向きの法線方向の座標で表す．
fn offset(segment: &Segment, angle_deg: f64) -> f64 {
    let (sin, cos) = angle_deg.to_radians().sin_cos();
    -sin * segment[0][0] + cos * segment[0][1]
}

#[test]
fn 水平な線は_正方形を刻みごとに横切る() {
    let lines = hatch_lines(&square(2.0), 0.0, 0.5);
    // 高さ0，0.5，1，1.5，2の線のうち，半開区間の規則で，0，0.5，1，1.5の4本になる．
    let heights: Vec<f64> = lines.iter().map(|line| line[0][1]).collect();
    assert_eq!(heights.len(), 4, "{heights:?}");
    for line in &lines {
        assert!(close(line[0][1], line[1][1]));
        assert!(close(length(line), 2.0), "{line:?}");
    }
}

#[test]
fn 線の間隔は_刻みに等しく_原点から数えた倍数の位置にある() {
    let lines = hatch_lines(&square(3.0), 45.0, 0.4);
    let mut offsets: Vec<f64> = lines.iter().map(|line| offset(line, 45.0)).collect();
    offsets.sort_by(f64::total_cmp);
    for pair in offsets.windows(2) {
        assert!(close(pair[1] - pair[0], 0.4), "{offsets:?}");
    }
    for value in &offsets {
        let steps = value / 0.4;
        assert!((steps - steps.round()).abs() < 1e-9, "{value}");
    }
}

#[test]
fn 斜線は_指定した角度に向く() {
    for angle in [30.0_f64, 45.0, 90.0, 135.0] {
        let lines = hatch_lines(&square(2.0), angle, 0.3);
        assert!(!lines.is_empty(), "{angle}");
        let (sin, cos) = angle.to_radians().sin_cos();
        for line in &lines {
            let d = [line[1][0] - line[0][0], line[1][1] - line[0][1]];
            // 向きは，指定した角度の直線に平行である(向きの正負は問わない)．
            assert!((d[0] * sin - d[1] * cos).abs() < 1e-9, "{angle}: {line:?}");
        }
    }
}

#[test]
fn 線の端は_多角形の辺の上にある() {
    let lines = hatch_lines(&square(2.0), 45.0, 0.3);
    for line in &lines {
        for end in line {
            let on_edge = close(end[0], 0.0)
                || close(end[0], 2.0)
                || close(end[1], 0.0)
                || close(end[1], 2.0);
            assert!(on_edge, "{end:?}");
        }
    }
}

#[test]
fn 凹んだ多角形では_1本の線が複数の区間に分かれる() {
    // U字形．真ん中が抜けている．
    let u = vec![
        [0.0, 0.0],
        [3.0, 0.0],
        [3.0, 3.0],
        [2.0, 3.0],
        [2.0, 1.0],
        [1.0, 1.0],
        [1.0, 3.0],
        [0.0, 3.0],
    ];
    let lines = hatch_lines(&u, 0.0, 0.5);
    // 高さ2の線は，左の腕(0から1)と右の腕(2から3)の2区間である．
    let at_two: Vec<&Segment> = lines.iter().filter(|line| close(line[0][1], 2.0)).collect();
    assert_eq!(at_two.len(), 2);
    let mut xs: Vec<f64> = at_two
        .iter()
        .map(|line| line[0][0].min(line[1][0]))
        .collect();
    xs.sort_by(f64::total_cmp);
    assert!(close(xs[0], 0.0) && close(xs[1], 2.0));
    // 高さ0.5の線は，底を通り，1区間である．
    assert_eq!(
        lines.iter().filter(|line| close(line[0][1], 0.5)).count(),
        1
    );
}

#[test]
fn 自分と交わる多角形は_偶奇規則で内側を決める() {
    // 蝶ネクタイ形(8の字)．交点(1, 1)の左右に，2つの三角形ができる．
    let bow = vec![[0.0, 0.0], [2.0, 2.0], [2.0, 0.0], [0.0, 2.0]];
    let lines = hatch_lines(&bow, 0.0, 0.5);
    // 高さ0.5と1.5の線は，左の三角形と右の三角形を，1区間ずつ横切る．交点の高さ1の線は，2区間が，交点(1, 1)で接する．
    let count = |y: f64| lines.iter().filter(|line| close(line[0][1], y)).count();
    assert_eq!((count(0.5), count(1.5)), (2, 2));
    assert_eq!(count(1.0), 2);
}

#[test]
fn 長さのない線は_引かない() {
    let lines = hatch_lines(&square(1.0), 0.0, 0.5);
    assert!(lines.iter().all(|line| length(line) > 1e-9));
    // 頂点だけに触れる線は，半開区間の規則で，どちらか一方の側にだけ数える．
    let triangle = vec![[0.0, 0.0], [2.0, 0.0], [1.0, 1.0]];
    let lines = hatch_lines(&triangle, 0.0, 1.0);
    assert!(lines.iter().all(|line| length(line) > 1e-9), "{lines:?}");
}

#[test]
fn 頂点が2つ以下の多角形や_刻みが正でない場合は_何も引かない() {
    assert!(hatch_lines(&[], 0.0, 0.5).is_empty());
    assert!(hatch_lines(&[[0.0, 0.0], [1.0, 1.0]], 0.0, 0.5).is_empty());
    assert!(hatch_lines(&square(1.0), 0.0, 0.0).is_empty());
    assert!(hatch_lines(&square(1.0), 0.0, -1.0).is_empty());
    assert!(hatch_lines(&square(1.0), f64::NAN, 0.5).is_empty());
}

#[test]
fn 同じ入力は同じ結果になり_多角形の始点や回る向きを変えても線の集まりは同じである() {
    let a = hatch_lines(&square(2.0), 30.0, 0.35);
    assert_eq!(a, hatch_lines(&square(2.0), 30.0, 0.35));
    let mut shifted = square(2.0);
    shifted.rotate_left(1);
    let mut reversed = square(2.0);
    reversed.reverse();
    for other in [shifted, reversed] {
        let b = hatch_lines(&other, 30.0, 0.35);
        assert_eq!(a.len(), b.len());
        let key = |lines: &[Segment]| {
            let mut keys: Vec<i64> = lines
                .iter()
                .map(|line| {
                    let value = (offset(line, 30.0) * 1e6).round();
                    format!("{value}").parse::<i64>().unwrap()
                })
                .collect();
            keys.sort_unstable();
            keys
        };
        assert_eq!(key(&a), key(&b));
    }
}
