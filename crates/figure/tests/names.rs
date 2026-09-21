//! 誤りの種類とオブジェクトの種類の，機械が読む名前を確かめる．GUIやページが，名前で分岐できる．

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use figure::expr::{ExprError, ExprErrorKind};
use figure::scene::Object;
use figure::{ErrorKind, parse_scene};
use semver::Version;

const SINE_AND_SHIFTED_SINE: &str =
    include_str!("../../../site/src/figures/sine-and-shifted-sine.json");

fn code_of(json: &str) -> &'static str {
    parse_scene(json).expect_err("誤りになる").kind.code()
}

fn scene_with(objects: &str, description: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "{description}",
             "view": {{ "x": [0, 1], "y": [0, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

#[test]
fn 最初の図のオブジェクトの種類の名前を返す() {
    let scene = parse_scene(SINE_AND_SHIFTED_SINE).expect("読める");
    let names: Vec<&str> = scene.objects.iter().map(Object::type_name).collect();
    assert_eq!(
        names,
        [
            "axis",
            "axis",
            "label",
            "parameter",
            "graph",
            "graph",
            "label"
        ]
    );
}

#[test]
fn 曲線の種類の名前はcurveである() {
    let json = scene_with(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": [0, 1] }"#,
        "a",
    );
    let scene = parse_scene(&json).expect("読める");
    assert_eq!(scene.objects[0].type_name(), "curve");
}

#[test]
fn 誤りの種類ごとに名前を返す() {
    assert_eq!(code_of("{"), "json");
    assert_eq!(code_of("{}"), "missing_version");
    assert_eq!(code_of(r#"{ "version": "x" }"#), "invalid_version");
    assert_eq!(
        code_of(r#"{ "version": "99.0.0" }"#),
        "incompatible_version"
    );
    assert_eq!(code_of(r#"{ "version": "0.1.0" }"#), "invalid");
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "1a", "type": "parameter", "value": 1 }"#,
            "a"
        )),
        "invalid_id"
    );
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "a", "type": "parameter", "value": 1 }, { "id": "a", "type": "parameter", "value": 1 }"#,
            "a"
        )),
        "duplicate_id"
    );
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "f", "type": "graph", "var": "1", "expr": "x", "domain": [0, 1] }"#,
            "a"
        )),
        "invalid_variable"
    );
    assert_eq!(code_of(&scene_with("", " ")), "empty_text");
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "f", "type": "graph", "var": "x", "expr": "1 +", "domain": [0, 1] }"#,
            "a"
        )),
        "expression"
    );
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "pi", "type": "parameter", "value": 1 }"#,
            "a"
        )),
        "reserved_name"
    );
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "a", "type": "parameter", "value": 1 },
               { "id": "f", "type": "graph", "var": "a", "expr": "1", "domain": [0, 1] }"#,
            "a"
        )),
        "name_conflict"
    );
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "x", "type": "axis", "direction": "x", "range": [1, 0] }"#,
            "a"
        )),
        "invalid_range"
    );
    assert_eq!(
        code_of(&scene_with(
            r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t"], "domain": [0, 1] }"#,
            "a"
        )),
        "expression_count"
    );
}

#[test]
fn 誤りの名前は重ならない() {
    let kinds = [
        ErrorKind::Json {
            line: 1,
            column: 1,
            message: String::new(),
        },
        ErrorKind::MissingVersion,
        ErrorKind::InvalidVersion(String::new()),
        ErrorKind::IncompatibleVersion {
            scene: Version::new(1, 0, 0),
            engine: Version::new(0, 1, 0),
        },
        ErrorKind::Invalid(String::new()),
        ErrorKind::InvalidId(String::new()),
        ErrorKind::DuplicateId(String::new()),
        ErrorKind::InvalidVariable(String::new()),
        ErrorKind::EmptyText("x"),
        ErrorKind::InvalidRange("x"),
        ErrorKind::ReservedName(String::new()),
        ErrorKind::NameConflict(String::new()),
        ErrorKind::ExpressionCount {
            expected: 2,
            found: 1,
        },
        ErrorKind::Expression {
            field: "expr",
            index: 0,
            error: ExprError {
                kind: ExprErrorKind::UnexpectedEnd,
                span: 0..0,
            },
        },
    ];
    let mut codes: Vec<&str> = kinds.iter().map(ErrorKind::code).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), kinds.len());
}
