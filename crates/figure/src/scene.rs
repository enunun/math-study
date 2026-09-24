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
    /// 格子．
    Grid(Grid),
    /// 点．座標は，あとの式から`<id>_x`と`<id>_y`(空間の図では`<id>_z`も)で参照できる．
    Point(Point),
    /// 向きのある線分．
    Vector(Vector),
    /// 線分．
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
    /// 複体(頂点と面でできた図形)．空間の図でだけ使える．
    Complex(Complex),
    /// 正多面体(種類と中心と半径で書く複体)．空間の図でだけ使える．
    Polyhedron(Polyhedron),
    /// 多角形(辺と，塗った面)．正多角形は辺の数と中心と半径で，ほかは頂点の並びで書く．平面の図でだけ使える．
    Polygon(Polygon),
    /// 式から呼べる関数．描かない．
    Function(FunctionDef),
    /// 写像．変換(`transform`)の手順として使う．描かない．
    Map(Map),
    /// 別のオブジェクトを変換した像．
    Image(Image),
    /// グラフのテイラー展開を，途中の次数で打ち切った多項式のグラフ．平面の図でだけ使える．
    Taylor(Taylor),
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
    /// 変換．書いた順に施す．位置だけを動かし，文字の向きや大きさは変えない．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
}

/// 正多面体の種類．
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Solid {
    /// 正4面体．
    Tetrahedron,
    /// 正6面体(立方体)．
    Cube,
    /// 正8面体．
    Octahedron,
    /// 正12面体．
    Dodecahedron,
    /// 正20面体．
    Icosahedron,
}

/// 正多面体．種類と，中心と，半径(中心から頂点までの距離)だけで書く．空間の図でだけ使える．
/// 頂点と面はエンジンが決め(`polyhedron.rs`)，同じ頂点と面を持つ複体(`Complex`)と同じように描いて隠す．
/// 基準の向きは種類ごとに決まっている(立方体は，面が座標軸に垂直になる向き)．ほかの向きは，変換(`transform`)の回転で作る．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Polyhedron {
    /// 識別子．
    pub id: String,
    /// 種類．
    pub solid: Solid,
    /// 中心の座標．
    pub center: [f64; 3],
    /// 中心から頂点までの距離(外接球の半径)．
    pub radius: f64,
    /// 稜のスタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
}

/// 複体．頂点と，面(頂点の番号を周に沿って並べたもの，3個以上)でできた図形．空間の図でだけ使える．
/// 稜(辺)は，どの2つの面にも属さない稜がないよう，面から自動的に求める(手で書かない)．
/// 面は，向き(頂点の並ぶ順)がすべて外向きになるように書く：稜を隠すかどうかは，その稜に隣接する
/// 2つの面の法線の向きで決まるので，向きが逆だと隠れ方が逆になる．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Complex {
    /// 識別子．
    pub id: String,
    /// 頂点の座標．3個ずつ(x，y，z)．
    pub vertices: Vec<[f64; 3]>,
    /// 面．頂点の番号(0始まり)を，周に沿って，外から見て反時計回りに並べる．1つの面は3個以上．
    pub faces: Vec<Vec<usize>>,
    /// 稜のスタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
}

impl Complex {
    /// 頂点の数の上限．
    pub const MAX_VERTICES: usize = 64;
    /// 面の数の上限．
    pub const MAX_FACES: usize = 64;
    /// 1つの面の頂点の数の上限．
    pub const MAX_FACE_VERTICES: usize = 32;
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
    /// 反復関数系の変換の並び．各要素は，1つの変換を表す，変換の手順(`TransformStep`)の並びである．
    /// 手順は，平行移動・回転・拡大縮小・対称移動・せん断に限る(写像は使えない)．1つ以上12個以下．
    pub transforms: Vec<Vec<TransformStep>>,
    /// 再帰の深さ．
    pub depth: u32,
    /// 深さ0から`depth`までの図形を，すべて重ねて描くか．既定は，深さ`depth`の図形だけを描く．
    /// ピタゴラスの木のように，途中の段も図形の一部であるときに使う．
    #[serde(default, skip_serializing_if = "is_false")]
    pub all_depths: bool,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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

/// 変換(`transform`)の手順の1つ．図形を動かす操作を，1つだけ書く．手順の並びは，書いた順に施す
/// (最初に書いた手順が，点に最初にかかる)．
///
/// 操作は，平行移動(`translate`)，回転(`rotate`，度，反時計回り．空間の図では`axis`の向きのまわりに，
/// 右ねじの向き)，拡大縮小(`scale`，1つの数か各方向の倍率)，対称移動(`reflect`，平面の図では鏡にする
/// 直線の向き，空間の図では鏡にする平面の法線)，せん断(`shear`，平面の図だけ)，写像(`map`，`map`
/// オブジェクトの`id`)である．回転・拡大縮小・対称移動・せん断は，`center`(なければ原点)を動かさない．
/// 数は，数か，媒介変数と定数を使う式で書く．
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformStep {
    /// 平行移動の量(平面の図では2個，空間の図では3個)．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translate: Option<Vec<Bound>>,
    /// 回転の角度(度)．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate: Option<Bound>,
    /// 回転の軸の向き(3個)．空間の図の回転に要る．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<Vec<Bound>>,
    /// 拡大縮小の倍率．1つの数なら全方向に同じ倍率で，並びなら方向ごとの倍率である．負の数で反転する．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Factor>,
    /// 対称移動の鏡．平面の図では，鏡にする直線の向き(2個)，空間の図では，鏡にする平面の法線(3個)である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reflect: Option<Vec<Bound>>,
    /// せん断の量(2個．xがyに，yがxに，それぞれ比例して動く量)．平面の図だけで使える．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shear: Option<Vec<Bound>>,
    /// 写像(`map`オブジェクト)の`id`．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map: Option<String>,
    /// 回転・拡大縮小・対称移動・せん断で動かない点．なければ原点である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<Vec<Bound>>,
}

/// 拡大縮小の倍率．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Factor {
    /// 全方向に同じ倍率．
    Uniform(Bound),
    /// 方向ごとの倍率(平面の図では2個，空間の図では3個)．
    PerAxis(Vec<Bound>),
}

/// 変換の手順の並びの上限．
pub const MAX_TRANSFORM_STEPS: usize = 16;

/// 多角形．辺(閉じた折れ線)と，塗った面(`fill`)でできる．平面の図でだけ使える．
///
/// 正多角形は，辺の数(`sides`)と，中心(`center`)と，半径(`radius`，中心から頂点までの距離)で書き，
/// 底辺が水平になる向きに置く．ほかの多角形は，頂点(`vertices`)を周に沿って並べて書く．どちらか一方で書く．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Polygon {
    /// 識別子．
    pub id: String,
    /// 正多角形の辺の数．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sides: Option<usize>,
    /// 正多角形の中心の座標(2個の，数か式)．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<Vec<Bound>>,
    /// 正多角形の半径(中心から頂点までの距離)．数か式．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<Bound>,
    /// 頂点．座標の並びか，点の式(`"A"`など)で，周に沿って並べる．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vertices: Vec<Position>,
    /// 面を塗る色と不透明度．なければ，塗らない．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Fill>,
    /// 辺のスタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 変換．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
}

impl Polygon {
    /// 頂点(辺)の数の下限．
    pub const MIN_VERTICES: usize = 3;
    /// 頂点(辺)の数の上限．
    pub const MAX_VERTICES: usize = 64;
}

/// 式から呼べる関数．`f(x) = x^2`なら，`id`が`f`，`vars`が`["x"]`，`expr`が`"x^2"`である．
/// 本体は，媒介変数と，先に置いた関数を使える．どのオブジェクトの式からも呼べ，合成(`f(g(x))`)もできる．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionDef {
    /// 識別子．式の中の関数の名前になる．
    pub id: String,
    /// 引数の名前(1個から3個)．
    pub vars: Vec<String>,
    /// 本体の式．
    pub expr: String,
}

impl FunctionDef {
    /// 引数の数の上限．
    pub const MAX_VARS: usize = 3;
}

/// 写像．点の座標(平面の図では2個，空間の図では3個)を，同じ数の座標に写す．`transform`の手順
/// `{"map": id}`で使う．式は，媒介変数と関数を使える．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Map {
    /// 識別子．
    pub id: String,
    /// 座標の変数の名前(平面の図では2個，空間の図では3個)．
    pub vars: Vec<String>,
    /// 写した先の座標の式(`vars`と同じ数)．
    pub expr: Vec<String>,
}

/// 別のオブジェクト(`of`)を，`transform`で変換した像．元のオブジェクトはそのまま残り，像が加わる．
/// 元のオブジェクトを直せば，像も変わる．スタイルは，書けば元のものの代わりに使い，書かなければ元のものを使う．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    /// 識別子．
    pub id: String,
    /// 変換する，先に置いたオブジェクトの`id`．
    pub of: String,
    /// 変換．元のオブジェクトの変換のあとに施す．
    pub transform: Vec<TransformStep>,
    /// 点の像の名前．点の像にだけ書ける．なければ，名前を置かない．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
}

/// グラフ(`of`)のテイラー展開を，`order`次で打ち切った多項式のグラフ．平面の図でだけ使える．
/// 係数は，グラフの式から，べき級数の計算で正確に求める(特殊関数を含む式は展開できない)．
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Taylor {
    /// 識別子．
    pub id: String,
    /// 展開するグラフ(`graph`)の`id`．
    pub of: String,
    /// 展開の中心．数か式．
    pub at: Bound,
    /// 打ち切る次数(0以上`MAX_ORDER`以下)．
    pub order: usize,
    /// 描く範囲．なければ，グラフの定義域である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<[Bound; 2]>,
    /// スタイル．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
}

impl Taylor {
    /// 次数の上限．
    pub const MAX_ORDER: usize = 30;
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// ワイヤーフレームの刻み(u方向，v方向)．断面は，各変数が刻みの整数倍になる所のうち，定義域の内側
    /// (両端を除く)に引く．数か式で書く．なければ，どちらも定義域の幅の4分の1である．ベジエ曲面の変数の
    /// 範囲は0から1である．`wireframe`があるときだけ使う．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wireframe_step: Option<[Bound; 2]>,
    /// ベジエ曲面の，制御点の網(行と列を結ぶ折れ線)．なければ描かない．`bezier`があるときだけ使える．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_net: Option<Style>,
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// ワイヤーフレームの刻みを書かないときの，定義域の分け方(幅の何分の1を刻みにするか)．
    pub const DEFAULT_WIREFRAME_DIVISIONS: f64 = 4.0;
    /// ワイヤーフレームの断面の本数の上限(各方向)．多すぎると，TikZの出力が大きくなりすぎる．
    pub const MAX_WIREFRAME_LINES: f64 = 50.0;
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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

/// 格子．範囲(平面の図では，なければ見える範囲)を，原点から数えた刻みの倍数の位置の線で区切る．
/// 空間の図では，xy平面(z = 0)の上に引く．ほかの平面には，変換(`transform`)で動かす．
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
    /// 線を引くxの範囲．平面の図では，なければ見える範囲である．空間の図では，必要である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_range: Option<[f64; 2]>,
    /// 線を引くyの範囲．平面の図では，なければ見える範囲である．空間の図では，必要である．
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y_range: Option<[f64; 2]>,
    /// スタイル．線の種類の既定は点線で，線は，目盛と軸より細い．
    #[serde(default, skip_serializing_if = "Style::is_default")]
    pub style: Style,
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
    /// 変換．書いた順に施す．
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<TransformStep>,
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
            Self::Complex(o) => &o.id,
            Self::Polyhedron(o) => &o.id,
            Self::Polygon(o) => &o.id,
            Self::Function(o) => &o.id,
            Self::Map(o) => &o.id,
            Self::Image(o) => &o.id,
            Self::Taylor(o) => &o.id,
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
            Self::Complex(_) => "complex",
            Self::Polyhedron(_) => "polyhedron",
            Self::Polygon(_) => "polygon",
            Self::Function(_) => "function",
            Self::Map(_) => "map",
            Self::Image(_) => "image",
            Self::Taylor(_) => "taylor",
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
