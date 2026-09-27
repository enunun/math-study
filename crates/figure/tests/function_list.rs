//! 図のシーンのリファレンス(`site/src/figure-reference/expression-functions.json`)に並べた，式の関数と定数が，
//! エンジンのものと一致することを確かめる．関数を足したのにリファレンスを直し忘れると，ここで失敗する．

#![allow(clippy::expect_used, clippy::indexing_slicing)]

use std::collections::BTreeSet;

use figure::expr::{constant_names, function_names};
use serde_json::Value;

const REFERENCE: &str =
    include_str!("../../../site/src/figure-reference/expression-functions.json");

fn names_in(reference: &Value, list: &str, field: &str) -> BTreeSet<String> {
    reference[list]
        .as_array()
        .expect("並びである")
        .iter()
        .flat_map(|entry| match &entry[field] {
            Value::Array(names) => names
                .iter()
                .map(|name| name.as_str().expect("文字列").to_owned())
                .collect(),
            Value::String(name) => vec![name.clone()],
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn リファレンスの関数は_エンジンの関数と一致する() {
    let reference: Value = serde_json::from_str(REFERENCE).expect("JSONである");
    let listed = names_in(&reference, "functions", "names");
    let engine: BTreeSet<String> = function_names().map(str::to_owned).collect();
    assert_eq!(listed, engine);
}

#[test]
fn リファレンスの定数は_エンジンの定数と一致する() {
    let reference: Value = serde_json::from_str(REFERENCE).expect("JSONである");
    let listed = names_in(&reference, "constants", "name");
    let engine: BTreeSet<String> = constant_names().map(str::to_owned).collect();
    assert_eq!(listed, engine);
}
