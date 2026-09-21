//! シーンのデータの型．

use std::fmt;
use std::str::FromStr;

use semver::Version;
use serde::{Deserialize, Serialize};

/// 図のシーン．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// シーンを書いたアプリケーションの版．
    pub version: Version,
    /// 図の説明．SVGの代替テキストになる．
    pub description: String,
    /// 見える範囲と，1単位の実寸．
    pub view: View,
    /// 図を作るオブジェクト．
    pub objects: Vec<Object>,
}

/// 見える範囲と，1単位の実寸．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    /// x方向の範囲．
    pub x: [f64; 2],
    /// y方向の範囲．
    pub y: [f64; 2],
    /// 1単位の実寸．
    pub unit: Unit,
}

/// 各方向の，1単位の実寸．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    /// x方向．
    pub x: Length,
    /// y方向．
    pub y: Length,
}

/// 1ptの長さ(cm)．`TeX`のptで，1インチの72.27分の1である．
pub const CM_PER_PT: f64 = 2.54 / 72.27;

/// 長さの単位．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnit {
    /// センチメートル．
    Cm,
    /// ミリメートル．
    Mm,
    /// ポイント．
    Pt,
}

/// 単位つきの長さ．JSONでは，`"2.5mm"`のような文字列で書く．
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Length {
    /// 数値．
    pub value: f64,
    /// 単位．
    pub unit: LengthUnit,
}

/// 図を作るオブジェクト．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Object {
    /// 座標軸．
    Axis(Axis),
    /// 数式を置くラベル．
    Label(Label),
    /// 式から名前で参照する数．
    Parameter(Parameter),
    /// 関数のグラフ．
    Graph(Graph),
    /// 媒介変数表示の曲線．
    Curve(Curve),
}

/// 軸の向き．
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// x軸．
    X,
    /// y軸．
    Y,
}

/// 矢じりの形．
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrow {
    /// `TikZ`の`Stealth`．
    #[default]
    Stealth,
    /// 矢じりなし．
    None,
}

/// 座標軸．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Axis {
    /// 識別子．
    pub id: String,
    /// 向き．
    pub direction: Direction,
    /// 矢じり．
    #[serde(default)]
    pub arrow: Arrow,
    /// 軸の名前の式．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 軸を引く範囲．なければ，見える範囲．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<[f64; 2]>,
}

/// ラベルの位置の基準．TikZのアンカーと同じ名前を使う．
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    /// 中央．
    #[default]
    #[serde(rename = "center")]
    Center,
    /// 北．
    #[serde(rename = "north")]
    North,
    /// 南．
    #[serde(rename = "south")]
    South,
    /// 東．
    #[serde(rename = "east")]
    East,
    /// 西．
    #[serde(rename = "west")]
    West,
    /// 北東．
    #[serde(rename = "north east")]
    NorthEast,
    /// 北西．
    #[serde(rename = "north west")]
    NorthWest,
    /// 南東．
    #[serde(rename = "south east")]
    SouthEast,
    /// 南西．
    #[serde(rename = "south west")]
    SouthWest,
}

/// 数式を置くラベル．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Label {
    /// 識別子．
    pub id: String,
    /// 置く位置(数学の座標)．
    pub at: [f64; 2],
    /// 位置の基準．
    #[serde(default)]
    pub anchor: Anchor,
    /// `TeX`の文字列．
    pub tex: String,
}

/// 式から名前で参照する数．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    /// 識別子．式の中の名前になる．
    pub id: String,
    /// 値．
    pub value: f64,
}

/// 定義域の端．数か式．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Bound {
    /// 数．
    Number(f64),
    /// 式(`2*pi`など)．
    Expression(String),
}

/// 線の種類．
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Line {
    /// 実線．
    #[default]
    Solid,
    /// 点線．
    Dotted,
    /// 破線．
    Dashed,
}

/// 線のスタイル．
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// 線の種類．
    pub line: Line,
}

/// 関数のグラフ．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    /// 識別子．
    pub id: String,
    /// 変数の名前．
    pub var: String,
    /// 変数の式．
    pub expr: String,
    /// 定義域．
    pub domain: [Bound; 2],
    /// スタイル．
    #[serde(default)]
    pub style: Style,
}

/// 媒介変数表示の曲線．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Curve {
    /// 識別子．
    pub id: String,
    /// 媒介変数の名前．
    pub var: String,
    /// 各座標の式．
    pub expr: Vec<String>,
    /// 媒介変数の範囲．
    pub domain: [Bound; 2],
    /// スタイル．
    #[serde(default)]
    pub style: Style,
}

impl Object {
    /// オブジェクトの識別子．
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Axis(o) => &o.id,
            Self::Label(o) => &o.id,
            Self::Parameter(o) => &o.id,
            Self::Graph(o) => &o.id,
            Self::Curve(o) => &o.id,
        }
    }

    /// オブジェクトの種類の名前．JSONの`type`と同じである．
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Axis(_) => "axis",
            Self::Label(_) => "label",
            Self::Parameter(_) => "parameter",
            Self::Graph(_) => "graph",
            Self::Curve(_) => "curve",
        }
    }
}

impl Length {
    /// cmに直した長さ．
    #[must_use]
    pub fn to_cm(self) -> f64 {
        match self.unit {
            LengthUnit::Cm => self.value,
            LengthUnit::Mm => self.value / 10.0,
            LengthUnit::Pt => self.value * CM_PER_PT,
        }
    }
}

impl LengthUnit {
    const fn suffix(self) -> &'static str {
        match self {
            Self::Cm => "cm",
            Self::Mm => "mm",
            Self::Pt => "pt",
        }
    }
}

impl FromStr for Length {
    type Err = String;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("長さ「{source}」は，数と単位(cm，mm，pt)で書く．");
        let (number, unit) = [LengthUnit::Cm, LengthUnit::Mm, LengthUnit::Pt]
            .into_iter()
            .find_map(|unit| Some((source.strip_suffix(unit.suffix())?, unit)))
            .ok_or_else(invalid)?;
        let value: f64 = number.parse().map_err(|_| invalid())?;
        if value.is_finite() && value > 0.0 {
            Ok(Self { value, unit })
        } else {
            Err(invalid())
        }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.value, self.unit.suffix())
    }
}

impl TryFrom<String> for Length {
    type Error = String;

    fn try_from(source: String) -> Result<Self, Self::Error> {
        source.parse()
    }
}

impl From<Length> for String {
    fn from(length: Length) -> Self {
        length.to_string()
    }
}
