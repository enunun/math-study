//! 矢じりの形．
//!
//! `TikZ`の`Stealth`の寸法と，輪郭を求める式は，pgfのソース
//! (`tex/generic/pgf/libraries/pgflibraryarrows.meta.code.tex`の`Stealth`の宣言)から移した．
//! 輪郭は，塗りつぶして，同じ線幅の線で縁取る．縁の角は，とがらせる．

/// 長さの，線幅に依らない部分(pt)．
const BASE_LENGTH: f64 = 3.0;
/// 長さの，線幅に比例する部分の係数．
const LENGTH_PER_LINE_WIDTH: f64 = 4.5;
/// 幅と長さの比．
const WIDTH_RATIO: f64 = 0.75;
/// くぼみと長さの比．
const INSET_RATIO: f64 = 0.325;
/// 輪郭の線幅の上限を決める，(長さ − くぼみ)との比．
const LINE_WIDTH_CAP_RATIO: f64 = 0.25;

/// `TikZ`の`Stealth`の矢じり．寸法は，pt単位である．
#[derive(Debug, Clone, PartialEq)]
pub struct Stealth {
    /// 長さ．
    pub length: f64,
    /// 幅．
    pub width: f64,
    /// くぼみ．後ろの辺から，くぼみの点までの距離．
    pub inset: f64,
    /// 矢じりの輪郭の線幅．
    pub line_width: f64,
    /// 軸の線を止める位置．矢じりの後ろから測る．
    pub line_end: f64,
    outline: [[f64; 2]; 4],
}

/// 位置を決めた矢じり．
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// 矢じりの輪郭の4点(先端，上の後ろの角，くぼみ，下の後ろの角)．
    pub polygon: [[f64; 2]; 4],
    /// 軸の線を止める点．
    pub line_end: [f64; 2],
}

impl Stealth {
    /// 線幅(pt)に合わせた矢じりを作る．
    #[must_use]
    pub fn new(line_width: f64) -> Self {
        let length = LENGTH_PER_LINE_WIDTH.mul_add(line_width, BASE_LENGTH);
        let width = WIDTH_RATIO * length;
        let inset = INSET_RATIO * length;
        let outline_width = line_width.min(LINE_WIDTH_CAP_RATIO * (length - inset));

        // 先端の角の長さ．
        let front_miter = 0.5 * outline_width * (4.0 * (length / width).powi(2) + 1.0).sqrt();
        // 後ろの角の，軸の方向(back)と，幅の方向(top)の長さ．
        let half_width = width / 2.0;
        let angle_length = length.atan2(half_width);
        let angle_inset = inset.atan2(half_width);
        let half_angle = (angle_length - angle_inset) / 2.0;
        let along_bisector = 0.5 * outline_width / half_angle.tan();
        let bisector = angle_inset + half_angle;
        let back_miter = if inset == 0.0 {
            0.5 * outline_width
        } else {
            bisector.sin() * along_bisector
        };
        let top_miter = bisector.cos() * along_bisector;
        // くぼみの角の長さ．
        let inset_miter = 0.5 * outline_width * (4.0 * (inset / width).powi(2) + 1.0).sqrt();

        let inner_length = length - front_miter - back_miter;
        let inner_half_width = half_width - top_miter;
        Self {
            length,
            width,
            inset,
            line_width: outline_width,
            line_end: inset + inset_miter - 0.25 * outline_width,
            outline: [
                [inner_length + back_miter, 0.0],
                [back_miter, inner_half_width],
                [inset + inset_miter, 0.0],
                [back_miter, -inner_half_width],
            ],
        }
    }

    /// 矢じりの後ろを原点，進む向きをxとした，輪郭の4点(先端，上の後ろの角，くぼみ，下の後ろの角)．
    #[must_use]
    pub const fn outline(&self) -> [[f64; 2]; 4] {
        self.outline
    }

    /// 軸の端`end`に先端を置き，向き`direction`(単位ベクトル)に向けた矢じりを返す．
    /// `unit`は，1ptの，座標での大きさである．
    #[must_use]
    pub fn place(&self, end: [f64; 2], direction: [f64; 2], unit: f64) -> Placed {
        // 進む向きの左を，幅の正の向きとする．
        let normal = [-direction[1], direction[0]];
        let back = [
            end[0] - direction[0] * self.length * unit,
            end[1] - direction[1] * self.length * unit,
        ];
        let at = |along: f64, across: f64| {
            [
                back[0] + (direction[0] * along + normal[0] * across) * unit,
                back[1] + (direction[1] * along + normal[1] * across) * unit,
            ]
        };
        Placed {
            polygon: self.outline.map(|[along, across]| at(along, across)),
            line_end: at(self.line_end, 0.0),
        }
    }
}
