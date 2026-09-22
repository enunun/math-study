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
    /// グラフか曲線の接線．平面の図でだけ使える．
    TangentLine(TangentLine),
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
    /// フラクタル図形．平面の図でだけ使える．
    Fractal(Fractal),
    /// 曲面．式で書く．空間の図でだけ使える．
    Surface(Surface),
    /// 曲面の，平面による切り口．空間の図でだけ使える．
    Cut(Cut),
    /// 2つの曲面の交線．空間の図でだけ使える．
    Intersection(Intersection),
    /// 曲面の接平面．空間の図でだけ使える．
    TangentPlane(TangentPlane),
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
    /// 置く位置(数学の座標)．座標の並び(平面の図では2個，空間の図では3個の，数か式)か，
    /// 点の式(平面の図だけ)で書く．
    pub at: Position,
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

/// 位置の書き方．座標の並びか，点の式の文字列である．
///
/// 点の式は，点の`id`を，原点からの位置ベクトルとして，和と差，数(媒介変数を含む)の倍で書く．
/// 例：`"B + C - A"`(平行四辺形の頂点)，`"(A + B) / 2"`(中点)，`"A + t * (B - A)"`(線分の上の点)．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Position {
    /// 点の式．
    Vector(String),
    /// 座標の並び．各座標は，数か式である．
    Coordinates(Vec<Bound>),
}

/// 点．座標は，数か，媒介変数と，先に置いた点の座標を使う式で書く．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    /// 識別子．座標は，式の中で`<id>_x`と`<id>_y`の名前になる．
    pub id: String,
    /// 位置(数学の座標)．2個の座標の並びか，点の式で書く．
    pub at: Position,
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

/// 2つの曲面の交線．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intersection {
    /// 識別子．
    pub id: String,
    /// 交わる2つの曲面の`id`．
    pub surfaces: Vec<String>,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// グラフか曲線の接線．平面の図でだけ使える．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TangentLine {
    /// 識別子．
    pub id: String,
    /// 接する対象(`graph`か`curve`)の`id`．
    pub of: String,
    /// 接する点．`of`がグラフなら変数の値，曲線なら媒介変数の値．数か式．
    pub at: Bound,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// 曲面の接平面．空間の図でだけ使える．接する点での，2つの偏微分の向きに張る平行四辺形として描く．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TangentPlane {
    /// 識別子．
    pub id: String,
    /// 接する曲面(`surface`)の`id`．
    pub of: String,
    /// 接する点の，曲面の2つの変数の値．数か式．
    pub at: [Bound; 2],
    /// 接平面の半径(cm)．中心から各辺への距離．数か式．
    pub size: Bound,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// フラクタル図形．平面の図でだけ使える．反復関数系(IFS)：基本図形(`base`)を，`transforms`(並進・
/// 回転・拡大縮小・せん断を組み合わせた，自由な変換の並び)で，`depth`回，再帰的に写して描く．
///
/// 深さ0は，`base`をそのまま描く．深さ`n`は，深さ`n - 1`の図形全体に，`transforms`のそれぞれの
/// 変換を施したものをすべて集めたものである(標準の，IFSのアトラクターを求める反復計算)．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fractal {
    /// 識別子．
    pub id: String,
    /// 基本図形が通る点(数学の座標，2個)の列．2点以上12点以下．数か式．
    pub base: Vec<[Bound; 2]>,
    /// 基本図形を閉じるか(最後の点から最初の点へも線を引く)．既定は閉じない．
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub closed: bool,
    /// 変換の並び．各要素は，1つの変換を表す，変換の手順(`TransformOp`)の並びである．
    /// 1つ以上12個以下．
    pub transforms: Vec<Vec<TransformOp>>,
    /// 再帰の深さ．
    pub depth: u32,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

impl Fractal {
    /// 基本図形の点の数の下限．
    pub const MIN_BASE_POINTS: usize = 2;
    /// 基本図形の点の数の上限．
    pub const MAX_BASE_POINTS: usize = 12;
    /// 変換の数の上限．
    pub const MAX_TRANSFORMS: usize = 12;
    /// 再帰の深さの上限．
    pub const MAX_DEPTH: u32 = 10;
    /// 描く図形の数(`変換の数の transforms.len() 乗`)の上限．`MAX_DEPTH`だけでは足りない，
    /// 変換の数が多いときの歯止めをかける．
    pub const MAX_INSTANCES: usize = 20_000;
}

/// フラクタルの1つの変換を作る，手順の1つ．並びの順に施す(最初に書いた手順が，点に最初にかかる)．
/// 並進・回転・拡大縮小・せん断を組み合わせれば，平面のどんなアフィン変換も作れる．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum TransformOp {
    /// 平行移動．数か式，2個(x，y方向)．
    Translate {
        /// 平行移動の量．
        translate: [Bound; 2],
    },
    /// 原点を中心とした回転．度，反時計回り．数か式．
    Rotate {
        /// 回転の角度(度)．
        rotate: Bound,
    },
    /// 原点を中心とした拡大縮小．数か式，2個(x，y方向)．負の数で，その向きに反転する．
    Scale {
        /// 拡大縮小の倍率．
        scale: [Bound; 2],
    },
    /// 原点を中心としたせん断．数か式，2個(x方向がyに，y方向がxに，それぞれ比例して動く量)．
    Shear {
        /// せん断の量．
        shear: [Bound; 2],
    },
}

/// 曲面の，平面による切り口．平面は，法線と定数で`normal・p = offset`と書く．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cut {
    /// 識別子．
    pub id: String,
    /// 切る曲面の`id`．
    pub surface: String,
    /// 平面の法線．3個の，数か式である．
    pub normal: Vec<Bound>,
    /// 平面の定数(`normal・p = offset`の右辺)．数か式である．
    pub offset: Bound,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// 曲面．空間の点を，2つの変数の式か，ベジエ曲面の制御点の網で表す．輪郭と，曲面に隠れる線を，三角形の網から求める．
///
/// 曲面は不透明な殻で，ほかのオブジェクトの線を隠す．輪郭は，視線が曲面に接する所である．
/// 式で書くときは`vars`，`expr`，`domain`を，ベジエ曲面で書くときは`bezier`を使う．両方は書けない．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    /// 識別子．
    pub id: String,
    /// 2つの変数の名前．式で書く曲面だけが持つ．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vars: Vec<String>,
    /// x，y，z座標の式．式で書く曲面だけが持つ．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expr: Vec<String>,
    /// 各変数の範囲．数か式で書く．式で書く曲面だけが持つ．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<[[Bound; 2]; 2]>,
    /// ベジエ曲面の制御点の網．`bezier[i][j]`は，1つ目の変数の方向にi番目，2つ目の変数の方向にj番目の点で，
    /// x，y，z座標を，数か式で書く．各方向に2点以上を並べ，行の長さを揃える．変数の範囲は，どちらも0から1である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bezier: Option<Vec<Vec<Vec<Bound>>>>,
    /// 網の細かさ(各変数の方向の分割数)．細かいほど滑らかで，重い．
    #[serde(default = "default_mesh", skip_serializing_if = "is_default_mesh")]
    pub mesh: [usize; 2],
    /// 定義域の縁(4つの辺)を描くか．球のように，縁が継ぎ目や1点になる曲面では，描かない．
    #[serde(default, skip_serializing_if = "is_false")]
    pub boundary: bool,
    /// スタイル．輪郭と縁の線に使う．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 曲面のワイヤーフレーム(u一定・v一定の断面)．なければ描かない．あれば，そのスタイルで描く．
    /// 式で書いた曲面でも，ベジエ曲面でも，同じように，曲面の上の線として描く．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wireframe: Option<Style>,
    /// ベジエ曲面の，制御点の網(行と列を結ぶ折れ線)．なければ描かない．`bezier`があるときだけ使える．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_net: Option<Style>,
}

impl Surface {
    /// 網の細かさの既定．
    pub const DEFAULT_MESH: usize = 48;
    /// 網の細かさの下限．
    pub const MIN_MESH: usize = 4;
    /// 網の細かさの上限．
    pub const MAX_MESH: usize = 200;
    /// ベジエ曲面の制御点の数の下限(各方向)．
    pub const MIN_CONTROL_POINTS: usize = 2;
    /// ベジエ曲面の制御点の数の上限(各方向)．次数が高いと，制御点の動きが，形に効きにくくなる．
    pub const MAX_CONTROL_POINTS: usize = 12;
}

const fn default_mesh() -> [usize; 2] {
    [Surface::DEFAULT_MESH; 2]
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_mesh(mesh: &[usize; 2]) -> bool {
    *mesh == default_mesh()
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
    /// 斜線を引くか．既定は引く．
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub hatch: bool,
    /// 領域を塗る色．なければ，塗らない．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Fill>,
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

/// 領域の塗り．
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fill {
    /// 塗る色．なければ，文字の色である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// 不透明度(0より大きく1以下)．既定は0.25で，下の線が透ける．
    #[serde(
        default = "default_opacity",
        skip_serializing_if = "is_default_opacity"
    )]
    pub opacity: f64,
}

const DEFAULT_OPACITY: f64 = 0.25;

const fn default_opacity() -> f64 {
    DEFAULT_OPACITY
}

#[allow(clippy::float_cmp, clippy::trivially_copy_pass_by_ref)]
fn is_default_opacity(opacity: &f64) -> bool {
    *opacity == DEFAULT_OPACITY
}

const fn default_true() -> bool {
    true
}

// serdeの`skip_serializing_if`は，参照を受け取る関数を要る．
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_true(value: &bool) -> bool {
    *value
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
    /// 球のワイヤーフレーム(経線と緯線)．なければ描かない．あれば，そのスタイルで描く．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wireframe: Option<Style>,
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

/// 媒介変数表示の曲線．式(`var`，`expr`，`domain`)か，ベジエ曲線の制御点(`bezier`)か，
/// スプライン曲線が通る点(`spline`)の，どれか1つで書く．2つ以上は書けない．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Curve {
    /// 識別子．
    pub id: String,
    /// 媒介変数の名前．式で書く曲線だけが持つ．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub var: Option<String>,
    /// 各座標の式．式で書く曲線だけが持つ．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expr: Vec<String>,
    /// 媒介変数の範囲．式で書く曲線だけが持つ．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<[Bound; 2]>,
    /// ベジエ曲線の制御点．各点は，平面なら2個，空間なら3個の，数か式で書く座標である．
    /// 2点以上12点以下を並べる．媒介変数の範囲は，0から1である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bezier: Option<Vec<Vec<Bound>>>,
    /// スプライン曲線が，順に通る点．各点は，平面なら2個，空間なら3個の，数か式で書く座標である．
    /// 2点以上12点以下を並べる．媒介変数の範囲は，0から1である．Catmull-Romの方法で，
    /// 与えた点をすべて通る滑らかな曲線になる．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spline: Option<Vec<Vec<Bound>>>,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

impl Curve {
    /// ベジエ曲線・スプライン曲線の制御点(通る点)の数の下限．
    pub const MIN_CONTROL_POINTS: usize = Surface::MIN_CONTROL_POINTS;
    /// ベジエ曲線・スプライン曲線の制御点(通る点)の数の上限．
    pub const MAX_CONTROL_POINTS: usize = Surface::MAX_CONTROL_POINTS;
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
            Self::TangentLine(o) => &o.id,
            Self::Sphere(o) => &o.id,
            Self::Grid(o) => &o.id,
            Self::Point(o) => &o.id,
            Self::Vector(o) => &o.id,
            Self::Segment(o) => &o.id,
            Self::Region(o) => &o.id,
            Self::Fractal(o) => &o.id,
            Self::Surface(o) => &o.id,
            Self::Cut(o) => &o.id,
            Self::Intersection(o) => &o.id,
            Self::TangentPlane(o) => &o.id,
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
            Self::TangentLine(_) => "tangent_line",
            Self::Sphere(_) => "sphere",
            Self::Grid(_) => "grid",
            Self::Point(_) => "point",
            Self::Vector(_) => "vector",
            Self::Segment(_) => "segment",
            Self::Region(_) => "region",
            Self::Fractal(_) => "fractal",
            Self::Surface(_) => "surface",
            Self::Cut(_) => "cut",
            Self::Intersection(_) => "intersection",
            Self::TangentPlane(_) => "tangent_plane",
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
