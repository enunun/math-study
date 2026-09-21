//! シーンを書き出して読み直すと，同じになる．GUIが保存したファイルを，そのまま読めるための条件である．

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use figure::parse_scene;

const SINE_AND_SHIFTED_SINE: &str =
    include_str!("../../../site/src/figures/sine-and-shifted-sine.json");

#[test]
fn 書き出して読み直すと同じになる() {
    let scene = parse_scene(SINE_AND_SHIFTED_SINE).expect("最初の図を読める");
    let written = serde_json::to_string_pretty(&scene).expect("書き出せる");
    let reread = parse_scene(&written).expect("読み直せる");
    assert_eq!(reread, scene);
}

#[test]
fn 書き出した長さと定義域の端は元の書き方に戻る() {
    let scene = parse_scene(
        r#"{ "version": "0.1.0", "description": "a",
             "view": { "x": [0, 1], "y": [0, 1], "unit": { "x": "1cm", "y": "2.5mm" } },
             "objects": [
               { "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"], "domain": [0, "2*pi"] }
             ] }"#,
    )
    .expect("読める");
    let written = serde_json::to_value(&scene).expect("書き出せる");
    assert_eq!(written["view"]["unit"]["y"], "2.5mm");
    assert_eq!(written["objects"][0]["domain"][1], "2*pi");
    assert_eq!(written["objects"][0]["domain"][0], 0.0);
}
