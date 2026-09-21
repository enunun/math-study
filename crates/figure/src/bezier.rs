//! ベジエ曲面．制御点の網から，曲面の点を求める．
//!
//! 網の`i`行`j`列の点は，1つ目の変数`u`の方向にi番目，2つ目の変数`v`の方向にj番目の制御点である．
//! 曲面の点は，各行を`v`で，得た点の列を`u`で，de Casteljauの方法で内分して求める．
//! 内分は，重みが負にならないので，桁落ちが起きにくい．

/// 空間の点．
pub type Point3 = [f64; 3];

/// 2点の内分点．`t`が0なら`from`，1なら`to`である．
fn lerp(from: Point3, to: Point3, t: f64) -> Point3 {
    let [x0, y0, z0] = from;
    let [x1, y1, z1] = to;
    [x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, z0 + (z1 - z0) * t]
}

/// 点の列を，de Casteljauの方法で，パラメータ`t`の点にまとめる．点が1つもなければ，`None`を返す．
fn casteljau(points: &[Point3], t: f64) -> Option<Point3> {
    let mut level = points.to_vec();
    while level.len() > 1 {
        level = level
            .windows(2)
            .filter_map(|pair| match pair {
                [from, to] => Some(lerp(*from, *to, t)),
                _ => None,
            })
            .collect();
    }
    level.first().copied()
}

/// ベジエ曲面の点．`net`は，行が`u`の方向に並ぶ，制御点の網である．網が空か，行が空なら，`None`を返す．
#[must_use]
pub fn bezier_point(net: &[Vec<Point3>], u: f64, v: f64) -> Option<Point3> {
    let along_v: Option<Vec<Point3>> = net.iter().map(|row| casteljau(row, v)).collect();
    casteljau(&along_v?, u)
}
