//! JSONの誤りと，シーンの版の扱いを確かめる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use figure::version::{engine_version, is_readable};
use figure::{ErrorKind, parse_scene};
use semver::Version;

fn version(text: &str) -> Version {
    text.parse().expect("版として正しい")
}

fn kind(json: &str) -> ErrorKind {
    parse_scene(json).expect_err("誤りになる").kind
}

/// 版だけを変えた，正しいシーン．
fn scene_of_version(version: &str) -> String {
    format!(
        r#"{{ "version": "{version}", "description": "a",
             "view": {{ "x": [0, 1], "y": [0, 1], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [] }}"#
    )
}

#[test]
fn エンジンの版はワークスペースの版と同じである() {
    assert_eq!(engine_version().to_string(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn 主版が0の間は副版が同じで新しくない版だけを読める() {
    let engine = version("0.1.3");
    assert!(is_readable(&version("0.1.0"), &engine));
    assert!(is_readable(&version("0.1.3"), &engine));
    assert!(!is_readable(&version("0.1.4"), &engine), "新しい");
    assert!(!is_readable(&version("0.0.9"), &engine), "副版が違う");
    assert!(!is_readable(&version("0.2.0"), &engine), "副版が違う");
    assert!(!is_readable(&version("1.0.0"), &engine), "主版が違う");
}

#[test]
fn 主版が1以上なら主版が同じで新しくない版を読める() {
    let engine = version("1.2.3");
    assert!(is_readable(&version("1.0.0"), &engine));
    assert!(is_readable(&version("1.2.0"), &engine));
    assert!(is_readable(&version("1.2.3"), &engine));
    assert!(!is_readable(&version("1.2.4"), &engine), "新しい");
    assert!(!is_readable(&version("1.3.0"), &engine), "新しい");
    assert!(!is_readable(&version("2.0.0"), &engine), "主版が違う");
    assert!(!is_readable(&version("0.9.0"), &engine), "主版が違う");
}

#[test]
fn エンジンと同じ版のシーンを読める() {
    let scene = parse_scene(&scene_of_version(&engine_version().to_string())).expect("読める");
    assert_eq!(scene.version, engine_version());
}

#[test]
fn 新しい版のシーンは未知の項目より先に版の誤りにする() {
    let json = r#"{ "version": "99.0.0", "futureField": 1 }"#;
    assert_eq!(
        kind(json),
        ErrorKind::IncompatibleVersion {
            scene: version("99.0.0"),
            engine: engine_version(),
        }
    );
}

#[test]
fn 版の誤りの説明に両方の版が出る() {
    let message = parse_scene(&scene_of_version("99.0.0"))
        .expect_err("誤りになる")
        .to_string();
    assert!(message.contains("99.0.0"), "{message}");
    assert!(message.contains(&engine_version().to_string()), "{message}");
}

#[test]
fn versionがないと誤りになる() {
    assert_eq!(kind(r#"{ "description": "a" }"#), ErrorKind::MissingVersion);
}

#[test]
fn 版として読めないversionは誤りになる() {
    for text in ["1.0", "abc", "", "v1.0.0"] {
        let json = format!(r#"{{ "version": "{text}" }}"#);
        assert_eq!(kind(&json), ErrorKind::InvalidVersion(text.to_owned()));
    }
    assert_eq!(
        kind(r#"{ "version": 1 }"#),
        ErrorKind::InvalidVersion("1".to_owned())
    );
}

#[test]
fn 壊れたjsonは行と列を返す() {
    let ErrorKind::Json { line, column, .. } = kind("{\n  \"version\": ,\n}") else {
        panic!("JSONの誤りである");
    };
    assert_eq!(line, 2);
    assert!(column >= 1);
}

#[test]
fn 途中で終わったjsonは誤りになる() {
    assert!(matches!(kind("{"), ErrorKind::Json { line: 1, .. }));
}

#[test]
fn オブジェクトでないjsonは誤りになる() {
    assert!(matches!(kind("[1, 2]"), ErrorKind::Invalid(_)));
}
