//! 描画の中間表現．座標の単位はcm，向きは数学と同じ(yは上向き)，線幅と破線の寸法の単位はptである．
//! `TikZ`と`SVG`は，どちらも，この表現から書き出す．

use serde::Serialize;

use crate::scene::{Anchor, Arrow, Line};

/// 描く範囲(cm)．ラベルの余白を含む．
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bounds {
    /// 左下．
    pub min: [f64; 2],
    /// 右上．
    pub max: [f64; 2],
}

/// 線のスタイル．
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Stroke {
    /// 線の種類．
    pub line: Line,
    /// 線幅(pt)．
    pub width: f64,
}

/// 線の端の矢じり．`TikZ`は，`kind`だけを使い，形を自分で決める．`SVG`は，`polygon`を塗り，
/// 線を`line_end`で止める．
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrowHead {
    /// 矢じりの形．
    pub kind: Arrow,
    /// 輪郭の4点(cm)．
    pub polygon: [[f64; 2]; 4],
    /// 輪郭の線幅(pt)．
    pub line_width: f64,
    /// 軸の線を止める点(cm)．
    pub line_end: [f64; 2],
}

/// 折れ線．
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Path {
    /// 点(cm)．矢じりがあるときも，先端の位置まで続く．
    pub points: Vec<[f64; 2]>,
    /// 線のスタイル．
    pub stroke: Stroke,
    /// 終わりの矢じり．
    pub arrow: Option<ArrowHead>,
}

/// ラベル．
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LabelItem {
    /// 置く位置(cm)．
    pub at: [f64; 2],
    /// ラベルの箱の，位置に合わせる部分．`TikZ`の`anchor`と同じ意味である．
    pub anchor: Anchor,
    /// `$…$`で数式を含められる，`TeX`の文字列．
    pub tex: String,
}

/// 描く要素．
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Item {
    /// 折れ線．
    Path(Path),
    /// ラベル．
    Label(LabelItem),
}

/// 描画の中間表現．
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Figure {
    /// 図の説明．
    pub description: String,
    /// 描く範囲．
    pub bounds: Bounds,
    /// 描く要素．描く順に並ぶ．
    pub items: Vec<Item>,
}
