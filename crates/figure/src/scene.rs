//! シーンのデータの型．

use std::fmt;
use std::str::FromStr;

use semver::Version;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

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

/// 図の見え方．平面の図は，見える範囲と1単位の実寸で，空間の図は，見る向きと1単位の実寸で決める．
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum View {
    /// 平面の図．
    Plane(PlaneView),
    /// 空間の図．
    Space(SpaceView),
}

/// 平面の図の，見える範囲と，1単位の実寸．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaneView {
    /// x方向の範囲．
    pub x: [f64; 2],
    /// y方向の範囲．
    pub y: [f64; 2],
    /// 1単位の実寸．
    pub unit: Unit,
}

/// 空間の図の，見る向きと，1単位の実寸．平行投影で，見える範囲は，描いたものから決まる．
///
/// カメラは，原点から`(cos e cos a, cos e sin a, sin e)`の向き(`a`は方位角，`e`は仰角)にあり，原点を見る．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpaceView {
    /// 方位角(度)．z軸のまわりに，x軸からy軸の向きに測る．
    pub azimuth: f64,
    /// 仰角(度)．xy平面から上向きに測る．
    pub elevation: f64,
    /// 3方向共通の，1単位の実寸．
    pub unit: Length,
}

impl<'de> Deserialize<'de> for View {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let has = |names: [&str; 2]| names.iter().any(|name| value.get(name).is_some());
        match (has(["x", "y"]), has(["azimuth", "elevation"])) {
            (true, false) => serde_json::from_value(value)
                .map(Self::Plane)
                .map_err(D::Error::custom),
            (false, true) => serde_json::from_value(value)
                .map(Self::Space)
                .map_err(D::Error::custom),
            _ => Err(D::Error::custom(
                "`view`は，平面の図(`x`，`y`，`unit`)か，空間の図(`azimuth`，`elevation`，`unit`)のどちらかの形で書く．",
            )),
        }
    }
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
    /// 球．空間の図でだけ使える．
    Sphere(Sphere),
    /// 格子．平面の図でだけ使える．
    Grid(Grid),
    /// 点．平面の図でだけ使える．座標は，あとの式から`<id>_x`と`<id>_y`で参照できる．
    Point(Point),
    /// 向きのある線分．平面の図でだけ使える．
    Vector(Vector),
    /// 線分．平面の図でだけ使える．
    Segment(Segment),
    /// 領域．斜線で埋める．平面の図でだけ使える．
    Region(Region),
}

/// 軸の向き．
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// x軸．
    X,
    /// y軸．
    Y,
    /// z軸．空間の図でだけ使える．
    Z,
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
    /// 軸を引く範囲．平面の図では，なければ見える範囲．空間の図では，必要である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<[f64; 2]>,
    /// スタイル．空間の図で，隠れた部分の線に使う．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 目盛．軸に直角な短い線と，任意の名前を，位置ごとに置く．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ticks: Vec<Tick>,
}

/// 軸の目盛．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tick {
    /// 軸の上の位置．数か，媒介変数と定数を使える式．
    pub at: Bound,
    /// 目盛の名前の式．なければ，線だけを引く．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 名前の箱の，線の端に合わせる部分．なければ，x軸では`north`，y軸では`east`である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
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
    /// 置く位置(数学の座標)．平面の図では2個，空間の図では3個の，数か式で書く．
    pub at: Vec<Bound>,
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

/// 不透明な面に隠れた部分の線．
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hidden {
    /// 点線．
    #[default]
    Dotted,
    /// 破線．
    Dashed,
    /// 描かない．
    None,
}

/// 線の色．決まった名前から選ぶ．色を省くと，文字の色になる．
///
/// 名前は，明るい背景と暗い背景の両方で見やすい色に，`SVG`で置き換える．`TikZ`では，`xcolor`の名前にする．
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    /// 灰色．
    Gray,
    /// 赤．
    Red,
    /// 青．
    Blue,
    /// 緑．
    Green,
    /// 橙．
    Orange,
    /// 紫．
    Purple,
}

/// 線の太さの上限(pt)．
pub const MAX_WIDTH_PT: f64 = 10.0;

/// 線のスタイル．省いた項目は，オブジェクトの種類ごとの既定になる(線の種類は，格子が点線，ほかは実線)．
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Style {
    /// 線の種類．
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<Line>,
    /// 隠れた部分の線．
    #[serde(skip_serializing_if = "Hidden::is_default")]
    pub hidden: Hidden,
    /// 線の色．
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// 線の太さ．なければ，オブジェクトの種類ごとの既定である．
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<Length>,
}

impl Style {
    /// すべて省いた，既定のスタイルか．書き出しで省くために使う．
    #[must_use]
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// 線の太さ(pt)．なければ`None`．
    #[must_use]
    pub fn width_pt(&self) -> Option<f64> {
        self.width.map(|width| width.to_cm() / CM_PER_PT)
    }
}

impl Hidden {
    /// 既定の(点線の)ままか．
    #[must_use]
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Dotted)
    }
}

/// 点．座標は，数か，媒介変数と，先に置いた点の座標を使う式で書く．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    /// 識別子．座標は，式の中で`<id>_x`と`<id>_y`の名前になる．
    pub id: String,
    /// 座標(数学の座標)．
    pub at: [Bound; 2],
    /// 点の名前の`TeX`の式．なければ，名前を置かない．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 名前の箱の，点に合わせる部分．なければ，`south west`(点の右上に名前を置く)である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
    /// 点の印(塗った丸)を描くか．
    #[serde(default, skip_serializing_if = "is_false")]
    pub dot: bool,
    /// スタイル．`color`が，点の印の色になる．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

// serdeの`skip_serializing_if`は，参照を受け取る関数を要る．
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(value: &bool) -> bool {
    !*value
}

/// 向きのある線分．始点から終点へ，矢じりを付ける．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vector {
    /// 識別子．
    pub id: String,
    /// 始点の点の`id`．
    pub from: String,
    /// 終点の点の`id`．
    pub to: String,
    /// 終点の矢じり．既定は`stealth`である．
    #[serde(default)]
    pub arrow: Arrow,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// 線分．矢じりのない，2点を結ぶ線である．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    /// 識別子．
    pub id: String,
    /// 一方の端の点の`id`．
    pub from: String,
    /// もう一方の端の点の`id`．
    pub to: String,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// 領域．2つのグラフ(かグラフとx軸)の間を，定義域の中で，平行な斜線で埋める．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    /// 識別子．
    pub id: String,
    /// 領域を挟む，グラフの`id`．2つなら2つのグラフの間，1つならグラフとx軸の間である．
    pub between: Vec<String>,
    /// 領域のxの範囲．数か式で書き，グラフの定義域の中にする．
    pub domain: [Bound; 2],
    /// 斜線の角度(度)．x軸から反時計回りに測る．既定は45である．
    #[serde(default = "default_angle", skip_serializing_if = "is_default_angle")]
    pub angle: f64,
    /// 斜線の間隔．既定は3mmである．
    #[serde(default = "default_gap", skip_serializing_if = "is_default_gap")]
    pub gap: Length,
    /// スタイル．斜線の線の種類，色，太さである．太さの既定は0.4ptである．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

const DEFAULT_ANGLE: f64 = 45.0;

const fn default_angle() -> f64 {
    DEFAULT_ANGLE
}

fn default_gap() -> Length {
    Length {
        value: 3.0,
        unit: LengthUnit::Mm,
    }
}

// serdeの`skip_serializing_if`は，参照を受け取る関数を要る．
#[allow(clippy::float_cmp, clippy::trivially_copy_pass_by_ref)]
fn is_default_angle(angle: &f64) -> bool {
    *angle == DEFAULT_ANGLE
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_gap(gap: &Length) -> bool {
    *gap == default_gap()
}

/// 格子．見える範囲を，原点から数えた刻みの倍数の位置の線で区切る．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    /// 識別子．
    pub id: String,
    /// x方向の刻み．数か，媒介変数と定数を使う式．なければ，縦の線を引かない．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_step: Option<Bound>,
    /// y方向の刻み．なければ，横の線を引かない．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y_step: Option<Bound>,
    /// スタイル．線の種類の既定は点線で，線は，目盛と軸より細い．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// 球．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sphere {
    /// 識別子．
    pub id: String,
    /// 中心の座標．
    pub center: [f64; 3],
    /// 半径．
    pub radius: f64,
    /// スタイル．輪郭線の種類に使う．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
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
    #[serde(default, skip_serializing_if = "Style::is_default")]
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
    #[serde(default, skip_serializing_if = "Style::is_default")]
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
            Self::Sphere(o) => &o.id,
            Self::Grid(o) => &o.id,
            Self::Point(o) => &o.id,
            Self::Vector(o) => &o.id,
            Self::Segment(o) => &o.id,
            Self::Region(o) => &o.id,
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
            Self::Sphere(_) => "sphere",
            Self::Grid(_) => "grid",
            Self::Point(_) => "point",
            Self::Vector(_) => "vector",
            Self::Segment(_) => "segment",
            Self::Region(_) => "region",
        }
    }
}

impl View {
    /// 平面の図の見え方．空間の図なら`None`．
    #[must_use]
    pub const fn as_plane(&self) -> Option<&PlaneView> {
        match self {
            Self::Plane(view) => Some(view),
            Self::Space(_) => None,
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
