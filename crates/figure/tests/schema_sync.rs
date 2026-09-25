//! 手書きのJSON Schema(`site/public/schema/scene.schema.json`)が，シーンの型と同じ構造を持つかを確かめる．
//!
//! 型から`schemars`で生成したスキーマと，手書きのスキーマを，どちらも構造だけの形に直して比べる．
//! 比べるのは，オブジェクトの種類(`type`)，各オブジェクトの項目名と必須の項目，列挙の値，値の型(数，文字列，
//! 配列など)である．説明文，正規表現，数の範囲，条件つきの規則(`oneOf`で書く「どちらか一方」など)は比べない．
//! これらは，型では表せないので手で書き，ajvのテスト(`site/src/figure/scene-schema*.test.ts`)で確かめる．

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};

use figure::scene::Scene;
use schemars::generate::SchemaSettings;
use serde_json::{Map, Value};

const HANDWRITTEN: &str = include_str!("../../../site/public/schema/scene.schema.json");

/// スキーマの構造だけを取り出した形．
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Shape {
    /// 決まった文字列のどれか(`enum`か`const`)．
    Strings(BTreeSet<String>),
    /// 項目を持つオブジェクト．
    Object {
        properties: BTreeMap<String, Shape>,
        required: BTreeSet<String>,
    },
    /// `type`の値で種類を選ぶオブジェクト．キーは`type`の値である．
    Tagged(BTreeMap<String, Shape>),
    /// 配列．
    Array(Box<Shape>),
    /// いくつかの形のどれか(型の選択肢)．
    Union(Vec<Shape>),
    /// 数，整数，文字列，真偽値．
    Scalar(String),
    /// 形を決めない(比べない)．
    Any,
}

/// スキーマの節を，`$ref`と`allOf`をたどって1つのオブジェクトにまとめる．
fn resolve(node: &Value, root: &Value) -> Map<String, Value> {
    let Some(object) = node.as_object() else {
        return Map::new();
    };
    let mut merged = Map::new();
    if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
        let pointer = reference.trim_start_matches('#');
        let target = root.pointer(pointer).expect("参照先がある");
        merged.extend(resolve(target, root));
    }
    for part in object
        .get("allOf")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        merged.extend(resolve(part, root));
    }
    for (key, value) in object {
        if key != "$ref" && key != "allOf" {
            merged.insert(key.clone(), value.clone());
        }
    }
    merged
}

fn is_null(node: &Value) -> bool {
    node.get("type").and_then(Value::as_str) == Some("null")
}

/// `type`の値(文字列か，文字列の配列)から，`null`を除いた型の名前．
fn types_of(node: &Map<String, Value>) -> Vec<String> {
    match node.get("type") {
        Some(Value::String(name)) => vec![name.clone()],
        Some(Value::Array(names)) => names
            .iter()
            .filter_map(Value::as_str)
            .filter(|name| *name != "null")
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

fn shape(node: &Value, root: &Value) -> Shape {
    let node = resolve(node, root);
    if let Some(Value::String(value)) = node.get("const") {
        return Shape::Strings(BTreeSet::from([value.clone()]));
    }
    if let Some(Value::Array(values)) = node.get("enum") {
        let strings: BTreeSet<String> = values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        if strings.len() == values.len() {
            return Shape::Strings(strings);
        }
    }
    if node.contains_key("properties") {
        return object_shape(&node, root);
    }
    let branches = node
        .get("oneOf")
        .or_else(|| node.get("anyOf"))
        .and_then(Value::as_array);
    if let Some(branches) = branches {
        return union_shape(
            branches
                .iter()
                .filter(|branch| !is_null(branch))
                .map(|branch| shape(branch, root))
                .collect(),
        );
    }
    let types = types_of(&node);
    match types.as_slice() {
        [single] if single == "array" => Shape::Array(Box::new(
            node.get("items")
                .map_or(Shape::Any, |items| shape(items, root)),
        )),
        [single] if single == "object" => object_shape(&node, root),
        [single] => Shape::Scalar(single.clone()),
        [] => Shape::Any,
        _ => union_shape(types.into_iter().map(Shape::Scalar).collect()),
    }
}

fn object_shape(node: &Map<String, Value>, root: &Value) -> Shape {
    let properties = node
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(key, value)| (key.clone(), shape(value, root)))
                .collect()
        })
        .unwrap_or_default();
    let required = node
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    Shape::Object {
        properties,
        required,
    }
}

/// 選択肢をまとめる．文字列だけなら1つの`Strings`に，`type`で選ぶオブジェクトなら`Tagged`にする．
fn union_shape(mut branches: Vec<Shape>) -> Shape {
    if branches.len() == 1 {
        return branches.pop().unwrap_or(Shape::Any);
    }
    if branches
        .iter()
        .all(|branch| matches!(branch, Shape::Strings(_)))
    {
        return Shape::Strings(
            branches
                .into_iter()
                .flat_map(|branch| match branch {
                    Shape::Strings(values) => values,
                    _ => BTreeSet::new(),
                })
                .collect(),
        );
    }
    let tags: Option<BTreeMap<String, Shape>> = branches
        .iter()
        .map(|branch| match branch {
            Shape::Object { properties, .. } => match properties.get("type") {
                Some(Shape::Strings(values)) if values.len() == 1 => {
                    Some((values.first()?.clone(), branch.clone()))
                }
                _ => None,
            },
            _ => None,
        })
        .collect();
    if let Some(tags) = tags {
        return Shape::Tagged(tags);
    }
    branches.sort();
    Shape::Union(branches)
}

fn keys<T>(map: &BTreeMap<String, T>) -> BTreeSet<String> {
    map.keys().cloned().collect()
}

/// 2つの集合の違いを，「型だけにある」「スキーマだけにある」の形で書く．
fn set_difference(
    path: &str,
    what: &str,
    typed: &BTreeSet<String>,
    written: &BTreeSet<String>,
    differences: &mut Vec<String>,
) {
    if typed != written {
        let only_typed: Vec<&String> = typed.difference(written).collect();
        let only_written: Vec<&String> = written.difference(typed).collect();
        differences.push(format!(
            "{path}の{what}：型だけにある{only_typed:?}，スキーマだけにある{only_written:?}"
        ));
    }
}

fn compare(path: &str, typed: &Shape, written: &Shape, differences: &mut Vec<String>) {
    match (typed, written) {
        (Shape::Any, _) | (_, Shape::Any) => {}
        (Shape::Strings(a), Shape::Strings(b)) => set_difference(path, "値", a, b, differences),
        (
            Shape::Object {
                properties: a,
                required: a_required,
            },
            Shape::Object {
                properties: b,
                required: b_required,
            },
        ) => {
            set_difference(path, "項目", &keys(a), &keys(b), differences);
            set_difference(path, "必須の項目", a_required, b_required, differences);
            for (key, a_shape) in a {
                if let Some(b_shape) = b.get(key) {
                    compare(&format!("{path}.{key}"), a_shape, b_shape, differences);
                }
            }
        }
        (Shape::Tagged(a), Shape::Tagged(b)) => {
            set_difference(path, "種類", &keys(a), &keys(b), differences);
            for (tag, a_shape) in a {
                if let Some(b_shape) = b.get(tag) {
                    compare(&format!("{path}[{tag}]"), a_shape, b_shape, differences);
                }
            }
        }
        (Shape::Array(a), Shape::Array(b)) => {
            compare(&format!("{path}[]"), a, b, differences);
        }
        (Shape::Union(a), Shape::Union(b)) if a.len() == b.len() => {
            for (index, (a_shape, b_shape)) in a.iter().zip(b).enumerate() {
                compare(&format!("{path}|{index}"), a_shape, b_shape, differences);
            }
        }
        _ if typed == written => {}
        _ => differences.push(format!("{path}の形：型は{typed:?}，スキーマは{written:?}")),
    }
}

fn differences() -> Vec<String> {
    let generated = serde_json::to_value(
        SchemaSettings::draft2020_12()
            .into_generator()
            .into_root_schema_for::<Scene>(),
    )
    .expect("生成したスキーマを値にできる");
    let written: Value = serde_json::from_str(HANDWRITTEN).expect("手書きのスキーマを読める");
    let mut differences = Vec::new();
    compare(
        "scene",
        &shape(&generated, &generated),
        &shape(&written, &written),
        &mut differences,
    );
    differences
}

#[test]
fn 手書きのスキーマは_シーンの型と同じ構造を持つ() {
    let differences = differences();
    assert!(
        differences.is_empty(),
        "スキーマと型が食い違う．型を変えたら，site/public/schema/scene.schema.jsonも直す．\n{}",
        differences.join("\n")
    );
}
