//! `figure`クレートを，ブラウザから呼ぶための窓口．
//!
//! `JavaScript`とのやり取りは，`Outcome`を`JavaScript`の値にしたものに限る．
//! シーンの読み込みと検査は，`figure`クレートが行い，ここには，値の変換だけを置く．

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
 * シーンを読んだ結果．誤りには，種類の名前(code)と説明がある．
 * オブジェクトの誤りには，そのid(object)が，JSONの構文の誤りには，行と列(line，column)がある．
 * canonicalは，読み直したシーンを，既定値を補って書き出したJSONである．
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
  | {
      status: "error";
      code: string;
      message: string;
      object: string | null;
      line: number | null;
      column: number | null;
    };
"#;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ObjectDto {
    id: String,
    #[serde(rename = "type")]
    kind: &'static str,
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
    Error {
        code: &'static str,
        message: String,
        object: Option<String>,
        line: Option<usize>,
        column: Option<usize>,
    },
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
            Err(error) => Self::Error {
                code: "serialize",
                message: format!("シーンを書き出せなかった：{error}"),
                object: None,
                line: None,
                column: None,
            },
        }
    }
}

impl From<Error> for Outcome {
    fn from(error: Error) -> Self {
        let (line, column) = match &error.kind {
            ErrorKind::Json { line, column, .. } => (Some(*line), Some(*column)),
            _ => (None, None),
        };
        Self::Error {
            code: error.kind.code(),
            message: error.to_string(),
            object: error.object.clone(),
            line,
            column,
        }
    }
}

/// シーンを読む．誤りは，例外ではなく，`status`が`"error"`の値で返す．
fn outcome(json: &str) -> Outcome {
    figure::parse_scene(json).map_or_else(Outcome::from, |scene| Outcome::from_scene(&scene))
}

/// シーンのJSONを読み，検査して，結果を返す．
///
/// # Errors
///
/// 結果をJavaScriptの値にできなかったときに，例外を投げる．シーンの誤りは，例外にしない．
#[wasm_bindgen(js_name = parseScene, unchecked_return_type = "SceneOutcome")]
pub fn parse_scene(json: &str) -> Result<JsValue, JsError> {
    // 型が`T | null`なので，ないものは，`undefined`ではなく`null`にする．
    let serializer = Serializer::new().serialize_missing_as_null(true);
    Ok(outcome(json).serialize(&serializer)?)
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
        // 省いた項目は，既定値で埋まる．
        assert_eq!(reread["objects"][0]["arrow"], "stealth");
        assert_eq!(reread["objects"][4]["style"]["line"], "solid");
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
