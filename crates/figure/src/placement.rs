//! 数学の座標を，図の座標(cm)へ移し，見える範囲で切り取る手順．平面の図の，自分で描く要素を作る
//! オブジェクト(楕円曲線，フーリエ級数など)が共有する．

use crate::clip::clip_polyline;
use crate::figure::{Item, Path, Stroke};
use crate::sample::sample;
use crate::transform::Transform;

/// 数学の座標を，図の座標(cm)へ移し，見える範囲で切り取る手順．
pub struct Placement<'a> {
    /// オブジェクトの変換．
    pub transform: &'a Transform,
    /// 単位の長さ(cm)．図の座標は，変換を施した数学の座標にこれを掛けたものである．
    pub unit: [f64; 2],
    /// 見える範囲(cm)．
    pub window: [[f64; 2]; 2],
    /// 見える範囲を含む，原点を中心とする正方形の半分の幅(数学の座標)．
    pub extent: f64,
}

impl Placement<'_> {
    /// 数学の座標の点を，図の座標(cm)にする．変換が定まらなければ`None`．
    pub fn place(&self, point: [f64; 2]) -> Option<[f64; 2]> {
        match self.transform.apply(&point)?.as_slice() {
            [x, y] => Some([x * self.unit[0], y * self.unit[1]]),
            _ => None,
        }
    }

    /// 媒介変数`t`の曲線を標本化して，見える範囲で切り取った線．
    pub fn curve(
        &self,
        f: impl Fn(f64) -> [f64; 2],
        [start, end]: [f64; 2],
        stroke: Stroke,
    ) -> Vec<Item> {
        let [min, max] = self.window;
        sample(|t| self.place(f(t)), start, end)
            .iter()
            .flat_map(|points| clip_polyline(points, min, max))
            .map(|points| {
                Item::Path(Path {
                    points,
                    stroke,
                    arrow: None,
                })
            })
            .collect()
    }

    /// 図の座標の点が，見える範囲(縁を含む)にあるか．
    pub fn inside(&self, [x, y]: [f64; 2]) -> bool {
        let [min, max] = self.window;
        let epsilon = 1e-9;
        x >= min[0] - epsilon
            && x <= max[0] + epsilon
            && y >= min[1] - epsilon
            && y <= max[1] + epsilon
    }
}

/// 見える範囲を含む，原点を中心とする正方形の半分の幅．
#[must_use]
pub fn view_extent(x: [f64; 2], y: [f64; 2]) -> f64 {
    x.iter()
        .chain(&y)
        .fold(0.0_f64, |extent, value| extent.max(value.abs()))
}
