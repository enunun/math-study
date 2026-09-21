//! 折れ線を，長方形で切り取ることを確かめる．外に出た線は捨て，境目には，長方形の辺の上の点を置く．

#![allow(
    clippy::float_cmp,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::arithmetic_side_effects
)]

use figure::clip::clip_polyline;

const MIN: [f64; 2] = [0.0, 0.0];
const MAX: [f64; 2] = [4.0, 2.0];

fn clip(points: &[[f64; 2]]) -> Vec<Vec<[f64; 2]>> {
    clip_polyline(points, MIN, MAX)
}

fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

#[test]
fn 全体が中にある線は_そのまま残る() {
    let line = [[0.5, 0.5], [1.0, 1.5], [3.0, 1.0]];
    assert_eq!(clip(&line), vec![line.to_vec()]);
}

#[test]
fn 辺の上の点は中にあるとして残す() {
    let line = [[0.0, 0.0], [4.0, 2.0]];
    assert_eq!(clip(&line), vec![line.to_vec()]);
}

#[test]
fn 全体が外にある線は_捨てる() {
    assert!(clip(&[[5.0, 0.5], [6.0, 1.5]]).is_empty());
    assert!(clip(&[[-1.0, 3.0], [5.0, 3.0]]).is_empty());
    // 長方形を見かけ上またぐが，角の外側を通る線．
    assert!(clip(&[[-1.0, 1.5], [1.5, 3.0]]).is_empty());
}

#[test]
fn 外へ出る線は_辺の上の点で止める() {
    let paths = clip(&[[1.0, 1.0], [6.0, 1.0]]);
    assert_eq!(paths.len(), 1);
    assert!(close(paths[0][0], [1.0, 1.0]));
    assert!(close(paths[0][1], [4.0, 1.0]));
    assert_eq!(paths[0].len(), 2);
}

#[test]
fn 外から入る線は_辺の上の点から始める() {
    let paths = clip(&[[-2.0, 0.0], [2.0, 2.0]]);
    // (-2, 0)から(2, 2)へ．x=0で，y=1に入る．
    assert_eq!(paths.len(), 1);
    assert!(close(paths[0][0], [0.0, 1.0]));
    assert!(close(paths[0][1], [2.0, 2.0]));
}

#[test]
fn 長方形を突き抜ける線は_両端を辺の上で止める() {
    let paths = clip(&[[-1.0, 1.0], [5.0, 1.0]]);
    assert_eq!(paths.len(), 1);
    assert!(close(paths[0][0], [0.0, 1.0]));
    assert!(close(paths[0][1], [4.0, 1.0]));
}

#[test]
fn 出て戻る線は_2本に分かれる() {
    // 中(1, 1)から，上へ外に出て(2, 5)，中(3, 1)に戻る．
    let paths = clip(&[[1.0, 1.0], [2.0, 5.0], [3.0, 1.0]]);
    assert_eq!(paths.len(), 2);
    assert!(close(paths[0][0], [1.0, 1.0]));
    assert!(close(*paths[0].last().unwrap(), [1.25, 2.0]));
    assert!(close(paths[1][0], [2.75, 2.0]));
    assert!(close(*paths[1].last().unwrap(), [3.0, 1.0]));
}

#[test]
fn 切った後の点は_すべて長方形の中にある() {
    let line: Vec<[f64; 2]> = (0..200)
        .map(|step| {
            let x = f64::from(step) / 10.0 - 5.0;
            [x, x * x - 3.0]
        })
        .collect();
    let paths = clip(&line);
    assert!(!paths.is_empty());
    for point in paths.iter().flatten() {
        assert!(point[0] >= MIN[0] && point[0] <= MAX[0], "{point:?}");
        assert!(point[1] >= MIN[1] && point[1] <= MAX[1], "{point:?}");
    }
}

#[test]
fn 辺に触れるだけの線は_長さのない線を作らない() {
    // 角(4, 2)に触れて，外へ戻る．
    assert!(clip(&[[5.0, 3.0], [4.0, 2.0], [5.0, 1.0]]).is_empty());
}

#[test]
fn 点が1つ以下の線は_何も残さない() {
    assert!(clip(&[]).is_empty());
    assert!(clip(&[[1.0, 1.0]]).is_empty());
}

#[test]
fn 辺に沿う線は_残る() {
    let paths = clip(&[[-1.0, 2.0], [5.0, 2.0]]);
    assert_eq!(paths.len(), 1);
    assert!(close(paths[0][0], [0.0, 2.0]));
    assert!(close(paths[0][1], [4.0, 2.0]));
}

// ---- 多角形の切り取り ----

use figure::clip::clip_polygon;

fn area(points: &[[f64; 2]]) -> f64 {
    let mut sum = 0.0;
    for (index, point) in points.iter().enumerate() {
        let next = points[(index + 1) % points.len()];
        sum += point[0] * next[1] - next[0] * point[1];
    }
    sum.abs() / 2.0
}

#[test]
fn 全体が中にある多角形は_そのまま残る() {
    let square = [[1.0, 0.5], [3.0, 0.5], [3.0, 1.5], [1.0, 1.5]];
    assert_eq!(clip_polygon(&square, MIN, MAX), square.to_vec());
}

#[test]
fn 全体が外にある多角形は_捨てる() {
    let far = [[5.0, 3.0], [6.0, 3.0], [6.0, 4.0]];
    assert!(clip_polygon(&far, MIN, MAX).is_empty());
}

#[test]
fn はみ出す多角形は_長方形との共通部分になる() {
    // 長方形(0, 0)から(4, 2)を，x: 2から6，y: -1から1のはみ出す正方形で切る．共通部分は，x: 2から4，y: 0から1．
    let box_ = [[2.0, -1.0], [6.0, -1.0], [6.0, 1.0], [2.0, 1.0]];
    let clipped = clip_polygon(&box_, MIN, MAX);
    assert!((area(&clipped) - 2.0).abs() < 1e-9, "{clipped:?}");
    for point in &clipped {
        assert!(point[0] >= 2.0 - 1e-9 && point[0] <= 4.0 + 1e-9);
        assert!(point[1] >= -1e-9 && point[1] <= 1.0 + 1e-9);
    }
}

#[test]
fn 長方形を覆う多角形は_長方形になる() {
    let big = [[-1.0, -1.0], [9.0, -1.0], [9.0, 5.0], [-1.0, 5.0]];
    let clipped = clip_polygon(&big, MIN, MAX);
    assert!((area(&clipped) - 8.0).abs() < 1e-9, "{clipped:?}");
}

#[test]
fn 三角形の切り取りは_面積で確かめる() {
    // 頂点(0, 0)，(8, 0)，(0, 4)の三角形を，x <= 4，y <= 2の長方形で切る．
    // 三角形は，斜辺が(8, 0)から(0, 4)で，長方形の中は，面積 8 - 0 = 8 のうち…直接求める．
    let triangle = [[0.0, 0.0], [8.0, 0.0], [0.0, 4.0]];
    let clipped = clip_polygon(&triangle, MIN, MAX);
    // 長方形は4 x 2で，斜辺 y = 4 - x/2 は，x = 4で y = 2，x = 0で y = 4．長方形の中はすべて三角形の内側である．
    assert!((area(&clipped) - 8.0).abs() < 1e-9, "{clipped:?}");
}

#[test]
fn 頂点が3つに満たない多角形は_何も残さない() {
    assert!(clip_polygon(&[], MIN, MAX).is_empty());
    assert!(clip_polygon(&[[1.0, 1.0], [2.0, 1.0]], MIN, MAX).is_empty());
}
