//! `figure`クレートを，ブラウザから呼ぶための窓口．
//!
//! `JavaScript`とのやり取りは，`SceneOutcome`と`RenderOutcome`を`JavaScript`の値にしたものに限る．
//! シーンの読み込み，検査，描画は，`figure`クレートが行い，ここには，値の変換だけを置く．

use figure::figure::Figure;
use figure::scene::Object;
use figure::version::engine_version;
use figure::{Error, ErrorKind, Scene};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
/** シーンの中のオブジェクト1つの，識別子と種類． */
export type SceneObject = { id: string; type: string };

/**
 * シーンの誤り．種類の名前(code)と説明がある．
 * オブジェクトの誤りには，そのid(object)が，JSONの構文の誤りには，行と列(line，column)が，
 * 式の誤りには，項目(field)，項目の中の何番目の式か(index)，式の中の位置(start，end．文字の番号，終わりは含まない)が付く．
 */
export type SceneError = {
  status: "error";
  code: string;
  message: string;
  object: string | null;
  line: number | null;
  column: number | null;
  field: string | null;
  index: number | null;
  start: number | null;
  end: number | null;
};

/**
 * シーンを読んだ結果．canonicalは，読み直したシーンを，既定値を補って書き出したJSONである．
 */
export type SceneOutcome =
  | {
      status: "ok";
      version: string;
      engineVersion: string;
      description: string;
      objects: SceneObject[];
      canonical: string;
    }
  | SceneError;

/** 線の種類．`TikZ`のスタイルの名前と同じである． */
export type LineKind = "solid" | "dotted" | "dashed";

/** 線の色．nullは，文字の色である． */
export type ColorName = "gray" | "red" | "blue" | "green" | "orange" | "purple";

/** ラベルの箱の，位置に合わせる部分．`TikZ`のanchorと同じ名前である． */
export type AnchorName =
  | "center"
  | "north"
  | "south"
  | "east"
  | "west"
  | "north east"
  | "north west"
  | "south east"
  | "south west";

/** 線の端の矢じり．polygonは，塗って縁取る輪郭の4点(cm)．線は，line_endで止める． */
export type ArrowHead = {
  kind: "stealth";
  polygon: [number, number][];
  line_width: number;
  line_end: [number, number];
};

/** 折れ線．座標の単位はcm，向きは数学と同じ(yは上向き)，線幅の単位はptである． */
export type PathItem = {
  type: "path";
  points: [number, number][];
  stroke: { line: LineKind; width: number; color: ColorName | null };
  arrow: ArrowHead | null;
};

/** 点の印．塗った丸である．座標はcm，半径はptである． */
export type DotItem = {
  type: "dot";
  at: [number, number];
  radius: number;
  color: ColorName | null;
};

/** 塗った多角形．座標はcmで，始めと終わりの点は，つながっている．opacityは，0より大きく1以下である． */
export type FillItem = {
  type: "fill";
  points: [number, number][];
  color: ColorName | null;
  opacity: number;
};

/** ラベル．texは，$…$で数式を含められるTeXの文字列である． */
export type LabelItem = {
  type: "label";
  at: [number, number];
  anchor: AnchorName;
  tex: string;
};

/** 描画の中間表現．座標の単位はcmで，boundsは，ラベルの余白を含む描く範囲である． */
export type Figure = {
  description: string;
  bounds: { min: [number, number]; max: [number, number] };
  items: (PathItem | LabelItem | DotItem | FillItem)[];
};

/** 描画の結果． */
export type RenderOutcome = { status: "ok"; figure: Figure; tikz: string } | SceneError;
"#;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ObjectDto {
    id: String,
    #[serde(rename = "type")]
    kind: &'static str,
}

/// シーンの誤り．
#[derive(Debug, Serialize, PartialEq, Eq)]
struct ErrorInfo {
    code: &'static str,
    message: String,
    object: Option<String>,
    line: Option<usize>,
    column: Option<usize>,
    field: Option<&'static str>,
    index: Option<usize>,
    start: Option<usize>,
    end: Option<usize>,
}

impl From<Error> for ErrorInfo {
    fn from(error: Error) -> Self {
        let (line, column) = match &error.kind {
            ErrorKind::Json { line, column, .. } => (Some(*line), Some(*column)),
            _ => (None, None),
        };
        let (field, index, start, end) = match &error.kind {
            ErrorKind::Expression {
                field,
                index,
                error,
            } => (
                Some(*field),
                Some(*index),
                Some(error.span.start),
                Some(error.span.end),
            ),
            _ => (None, None, None, None),
        };
        Self {
            code: error.kind.code(),
            message: error.to_string(),
            object: error.object.clone(),
            line,
            column,
            field,
            index,
            start,
            end,
        }
    }
}

/// JavaScriptへ渡す，シーンを読んだ結果．`status`の値で，成功と誤りを区別する．
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum Outcome {
    Ok {
        version: String,
        engine_version: String,
        description: String,
        objects: Vec<ObjectDto>,
        canonical: String,
    },
    Error(ErrorInfo),
}

/// JavaScriptへ渡す，描画の結果．
#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "camelCase")]
enum RenderOutcome {
    Ok { figure: Figure, tikz: String },
    Error(ErrorInfo),
}

impl Outcome {
    fn from_scene(scene: &Scene) -> Self {
        match serde_json::to_string_pretty(scene) {
            Ok(canonical) => Self::Ok {
                version: scene.version.to_string(),
                engine_version: engine_version().to_string(),
                description: scene.description.clone(),
                objects: scene
                    .objects
                    .iter()
                    .map(|object: &Object| ObjectDto {
                        id: object.id().to_owned(),
                        kind: object.type_name(),
                    })
                    .collect(),
                canonical,
            },
            Err(error) => Self::Error(ErrorInfo {
                code: "serialize",
                message: format!("シーンを書き出せなかった：{error}"),
                object: None,
                line: None,
                column: None,
                field: None,
                index: None,
                start: None,
                end: None,
            }),
        }
    }
}

/// シーンを読む．誤りは，例外ではなく，`status`が`"error"`の値で返す．
fn outcome(json: &str) -> Outcome {
    figure::parse_scene(json).map_or_else(
        |error| Outcome::Error(error.into()),
        |scene| Outcome::from_scene(&scene),
    )
}

/// シーンを描画する．誤りは，例外ではなく，`status`が`"error"`の値で返す．
fn render_outcome(json: &str) -> RenderOutcome {
    let rendered = figure::parse_scene(json).and_then(|scene| {
        let figure = figure::render(&scene)?;
        let tikz = figure::tikz::to_tikz(&figure);
        Ok((figure, tikz))
    });
    match rendered {
        Ok((figure, tikz)) => RenderOutcome::Ok { figure, tikz },
        Err(error) => RenderOutcome::Error(error.into()),
    }
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    // 型が`T | null`なので，ないものは，`undefined`ではなく`null`にする．
    let serializer = Serializer::new().serialize_missing_as_null(true);
    Ok(value.serialize(&serializer)?)
}

/// シーンのJSONを読み，検査して，結果を返す．
///
/// # Errors
///
/// 結果をJavaScriptの値にできなかったときに，例外を投げる．シーンの誤りは，例外にしない．
#[wasm_bindgen(js_name = parseScene, unchecked_return_type = "SceneOutcome")]
pub fn parse_scene(json: &str) -> Result<JsValue, JsError> {
    to_js(&outcome(json))
}

/// シーンのJSONを読み，描画して，中間表現と`TikZ`を返す．
///
/// # Errors
///
/// 結果をJavaScriptの値にできなかったときに，例外を投げる．シーンの誤りは，例外にしない．
#[wasm_bindgen(js_name = renderScene, unchecked_return_type = "RenderOutcome")]
pub fn render_scene(json: &str) -> Result<JsValue, JsError> {
    to_js(&render_outcome(json))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    const SINE_AND_SHIFTED_SINE: &str =
        include_str!("../../../site/src/figures/sine-and-shifted-sine.json");

    fn json_of(source: &str) -> serde_json::Value {
        serde_json::to_value(outcome(source)).expect("変換に成功する")
    }

    #[test]
    fn 成功はステータスokで版とオブジェクトの一覧を持つ() {
        let json = json_of(SINE_AND_SHIFTED_SINE);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], "0.1.0");
        assert_eq!(json["engineVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(
            json["description"],
            "y=sin x のグラフと，x軸の方向に平行移動した点線のグラフ"
        );
        assert_eq!(json["objects"].as_array().map(Vec::len), Some(7));
        assert_eq!(
            json["objects"][0],
            serde_json::json!({ "id": "x_axis", "type": "axis" })
        );
        assert_eq!(
            json["objects"][5],
            serde_json::json!({ "id": "shifted_sine", "type": "graph" })
        );
    }

    #[test]
    fn 成功の値は読み直したシーンのjsonを持つ() {
        let json = json_of(SINE_AND_SHIFTED_SINE);
        let canonical = json["canonical"].as_str().expect("文字列である");
        let reread: serde_json::Value = serde_json::from_str(canonical).expect("JSONである");
        assert_eq!(reread["objects"][4]["expr"], "sin(x)");
        // 省いた項目は，既定値で埋まる．ただし，スタイルの項目は，省いたままである．
        assert_eq!(reread["objects"][0]["arrow"], "stealth");
        assert!(reread["objects"][4].get("style").is_none());
        assert_eq!(reread["objects"][5]["style"]["line"], "dotted");
    }

    #[test]
    fn 構文の誤りは行と列を持ち_オブジェクトを持たない() {
        let json = json_of("{\n  \"version\": ,\n}");
        assert_eq!(json["status"], "error");
        assert_eq!(json["code"], "json");
        assert_eq!(json["line"], 2);
        assert!(json["column"].as_u64().is_some_and(|column| column >= 1));
        assert!(json["object"].is_null());
        assert!(
            json["message"]
                .as_str()
                .is_some_and(|message| message.contains("JSON"))
        );
    }

    #[test]
    fn オブジェクトの誤りはidを持ち_行と列を持たない() {
        let source =
            SINE_AND_SHIFTED_SINE.replace("\"arrow\": \"stealth\"", "\"arrow\": \"round\"");
        let json = json_of(&source);
        assert_eq!(json["status"], "error");
        assert_eq!(json["code"], "invalid");
        assert_eq!(json["object"], "x_axis");
        assert!(json["line"].is_null());
        assert!(json["column"].is_null());
    }

    const GOLDEN_TIKZ: &str = include_str!("../../figure/tests/golden/sine-and-shifted-sine.tikz");

    fn render_json_of(source: &str) -> serde_json::Value {
        serde_json::to_value(render_outcome(source)).expect("変換に成功する")
    }

    #[test]
    fn 描画の成功は中間表現とtikzを持つ() {
        let json = render_json_of(SINE_AND_SHIFTED_SINE);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["tikz"], GOLDEN_TIKZ);
        let figure = &json["figure"];
        assert_eq!(figure["items"].as_array().map(Vec::len), Some(8));
        assert_eq!(figure["items"][0]["type"], "path");
        assert_eq!(figure["items"][0]["arrow"]["kind"], "stealth");
        assert_eq!(figure["items"][1]["type"], "label");
        assert_eq!(figure["items"][1]["anchor"], "west");
        assert!(figure["items"][5]["arrow"].is_null());
        assert_eq!(figure["bounds"]["min"][0], -7.6);
        assert!(
            figure["description"]
                .as_str()
                .is_some_and(|d| d.contains("sin x"))
        );
    }

    #[test]
    fn 式の誤りは項目と位置を持つ() {
        let source = SINE_AND_SHIFTED_SINE.replace("sin(x - shift)", "sin(x - shift");
        for json in [json_of(&source), render_json_of(&source)] {
            assert_eq!(json["status"], "error");
            assert_eq!(json["code"], "expression");
            assert_eq!(json["object"], "shifted_sine");
            assert_eq!(json["field"], "expr");
            assert_eq!(json["index"], 0);
            assert!(json["start"].as_u64().is_some());
            assert!(json["end"].as_u64().is_some());
        }
    }

    #[test]
    fn 式の誤りでない誤りは項目と位置を持たない() {
        let json = json_of(r#"{ "version": "99.0.0" }"#);
        assert!(json["field"].is_null());
        assert!(json["index"].is_null());
        assert!(json["start"].is_null());
        assert!(json["end"].is_null());
    }

    #[test]
    fn 描画の誤りは読み込みの誤りと同じ形である() {
        let json = render_json_of("{");
        assert_eq!(json["status"], "error");
        assert_eq!(json["code"], "json");
        assert_eq!(json["line"], 1);
    }

    #[test]
    fn 新しい版は版の誤りの名前を持つ() {
        let json = json_of(r#"{ "version": "99.0.0" }"#);
        assert_eq!(json["code"], "incompatible_version");
        assert!(
            json["message"]
                .as_str()
                .is_some_and(|message| message.contains("99.0.0"))
        );
    }
}
