//! ベジエ曲面と，ベジエ曲線．制御点から，曲面や曲線の点を求める．
//!
//! 曲面の網の`i`行`j`列の点は，1つ目の変数`u`の方向にi番目，2つ目の変数`v`の方向にj番目の制御点である．
//! 曲面の点は，各行を`v`で，得た点の列を`u`で，de Casteljauの方法で内分して求める．
//! 曲線の点は，制御点の列を，媒介変数`t`で，同じ方法で内分して求める(`bezier_curve_point`)．
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

/// 点(座標の並び)どうしの内分点．`t`が0なら`from`，1なら`to`である．次元(要素数)は揃っている前提．
fn lerp_point(from: &[f64], to: &[f64], t: f64) -> Vec<f64> {
    from.iter().zip(to).map(|(a, b)| a + (b - a) * t).collect()
}

/// ベジエ曲線の点．`points`は制御点の列で，各点の次元(平面なら2，空間なら3)は揃っている前提である．
/// 曲面(`bezier_point`)と同じde Casteljauの方法だが，行が1本(媒介変数が1つ)である．
/// 点が1つもなければ，`None`を返す．
#[must_use]
pub fn bezier_curve_point(points: &[Vec<f64>], t: f64) -> Option<Vec<f64>> {
    let mut level = points.to_vec();
    while level.len() > 1 {
        level = level
            .windows(2)
            .filter_map(|pair| match pair {
                [from, to] => Some(lerp_point(from, to, t)),
                _ => None,
            })
            .collect();
    }
    level.into_iter().next()
}
