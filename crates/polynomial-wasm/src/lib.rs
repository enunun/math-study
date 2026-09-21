//! `polynomial`クレートを，ブラウザから呼ぶための窓口．
//!
//! `JavaScript`とのやり取りは，`Outcome`を`JavaScript`の値にしたものに限る．
//! 式の解析と計算は，`polynomial`クレートが行い，ここには，値の変換だけを置く．

use polynomial::{Calculation, Error, Rendered};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
/** 1つの多項式の，2通りの表記． */
export type Rendered = { tex: string; text: string };

/** 1つの変数による偏微分． */
export type Derivative = { variable: string; tex: string; text: string };

/** 計算の結果．誤りには，入力の中の位置(文字の番号，終わりは含まない)がある． */
export type Outcome =
  | { status: "ok"; expanded: Rendered; variables: string[]; derivatives: Derivative[] }
  | { status: "error"; message: string; start: number; end: number };
"#;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct RenderedDto {
    tex: String,
    text: String,
}

impl From<Rendered> for RenderedDto {
    fn from(rendered: Rendered) -> Self {
        Self {
            tex: rendered.tex,
            text: rendered.text,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct DerivativeDto {
    variable: String,
    tex: String,
    text: String,
}

/// JavaScriptへ渡す，計算の結果．`status`の値で，成功と誤りを区別する．
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
enum Outcome {
    Ok {
        expanded: RenderedDto,
        variables: Vec<String>,
        derivatives: Vec<DerivativeDto>,
    },
    Error {
        message: String,
        start: usize,
        end: usize,
    },
}

impl From<Calculation> for Outcome {
    fn from(calculation: Calculation) -> Self {
        Self::Ok {
            expanded: calculation.expanded.into(),
            variables: calculation.variables.iter().map(char::to_string).collect(),
            derivatives: calculation
                .derivatives
                .into_iter()
                .map(|derivative| DerivativeDto {
                    variable: derivative.variable.to_string(),
                    tex: derivative.result.tex,
                    text: derivative.result.text,
                })
                .collect(),
        }
    }
}

impl From<Error> for Outcome {
    fn from(error: Error) -> Self {
        Self::Error {
            message: error.to_string(),
            start: error.span.start,
            end: error.span.end,
        }
    }
}

/// 式を計算する．誤りは，例外ではなく，`status`が`"error"`の値で返す．
fn outcome(source: &str) -> Outcome {
    polynomial::calculate(source).map_or_else(Outcome::from, Outcome::from)
}

/// 式を展開し，変数ごとに偏微分して，結果を返す．
///
/// # Errors
///
/// 結果をJavaScriptの値にできなかったときに，例外を投げる．式の誤りは，例外にしない．
#[wasm_bindgen(unchecked_return_type = "Outcome")]
pub fn calculate(source: &str) -> Result<JsValue, JsError> {
    Ok(serde_wasm_bindgen::to_value(&outcome(source))?)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn 成功はステータスokの値になる() {
        let json = serde_json::to_value(outcome("(x+1)^2")).expect("変換に成功する");
        assert_eq!(
            json,
            serde_json::json!({
                "status": "ok",
                "expanded": { "tex": "x^2 + 2x + 1", "text": "x^2 + 2*x + 1" },
                "variables": ["x"],
                "derivatives": [{ "variable": "x", "tex": "2x + 2", "text": "2*x + 2" }],
            })
        );
    }

    #[test]
    fn 誤りはステータスerrorで位置を持つ() {
        let json = serde_json::to_value(outcome("1 + * 2")).expect("変換に成功する");
        assert_eq!(json["status"], "error");
        assert_eq!(json["start"], 4);
        assert_eq!(json["end"], 5);
        assert!(
            json["message"]
                .as_str()
                .is_some_and(|message| message.contains('*'))
        );
    }
}
