//! `TikZ`の出力を確かめる．最初の図は，期待する出力のファイル(`tests/golden/`)と比べる．
//!
//! 期待する出力を作り直すときは，`UPDATE_GOLDEN=1 cargo test -p figure --test tikz`を実行し，
//! 差分を目で確かめる．

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use std::fs;
use std::path::Path;

use figure::tikz::{export_tikz, extract_scene, number};
use figure::{Scene, parse_scene};

const SINE_AND_SHIFTED_SINE: &str =
    include_str!("../../../site/src/figures/sine-and-shifted-sine.json");
const GOLDEN: &str = "tests/golden/sine-and-shifted-sine.tikz";
const SPHERE_WITH_AXES: &str = include_str!("../../../site/src/figures/sphere-with-axes.json");
const SPHERE_GOLDEN: &str = "tests/golden/sphere-with-axes.tikz";

fn scene(json: &str) -> Scene {
    parse_scene(json).expect("シーンを読める")
}

fn tikz_of(json: &str) -> String {
    export_tikz(json).expect("出力できる")
}

fn scene_with(objects: &str) -> String {
    format!(
        r#"{{ "version": "0.1.0", "description": "試験の図",
             "view": {{ "x": [-2, 3], "y": [-1, 2], "unit": {{ "x": "1cm", "y": "1cm" }} }},
             "objects": [{objects}] }}"#
    )
}

fn assert_golden(json: &str, golden: &str) {
    let actual = tikz_of(json);
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(golden);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::write(&path, &actual).expect("期待する出力を書ける");
    }
    let expected = fs::read_to_string(&path).expect("期待する出力を読める");
    assert_eq!(
        actual, expected,
        "出力が変わった．意図した変更なら，UPDATE_GOLDEN=1で作り直す"
    );
}

#[test]
fn 最初の図は期待する出力と一致する() {
    assert_golden(SINE_AND_SHIFTED_SINE, GOLDEN);
}

#[test]
fn 空間の図は期待する出力と一致する() {
    assert_golden(SPHERE_WITH_AXES, SPHERE_GOLDEN);
}

#[test]
fn 空間の図の隠れた線は_点線になる() {
    let output = tikz_of(SPHERE_WITH_AXES);
    assert_eq!(output.matches("dotted").count(), 3, "{output}");
    assert!(output.contains("-{Stealth}"));
}

#[test]
fn 数は小数点以下4桁までで_末尾の0を省く() {
    assert_eq!(number(0.5), "0.5");
    assert_eq!(number(3.0), "3");
    assert_eq!(number(-7.0), "-7");
    assert_eq!(number(1.234_56), "1.2346");
    assert_eq!(number(0.000_04), "0");
    assert_eq!(number(-0.000_04), "0");
    assert_eq!(number(-0.25), "-0.25");
    assert_eq!(number(12.100_04), "12.1");
}

#[test]
fn 軸は矢じりつきの線と_名前の節点になる() {
    let tikz = tikz_of(SINE_AND_SHIFTED_SINE);
    assert!(
        tikz.contains("\\draw[line width=0.6pt, -{Stealth}] (-7,0) -- (7,0);\n"),
        "{tikz}"
    );
    assert!(tikz.contains("\\node[anchor=west] at (7,0) {$x$};\n"));
    assert!(tikz.contains("\\node[anchor=south] at (0,3.6) {$y$};\n"));
}

#[test]
fn ラベルは節点になり_中央のアンカーは省く() {
    let tikz = tikz_of(SINE_AND_SHIFTED_SINE);
    assert!(tikz.contains("\\node[anchor=north west] at (0,0) {O};\n"));
    assert!(tikz.contains("\\node[anchor=west] at (1,4) {Graph of $y=\\sin x$};\n"));
    let centered = tikz_of(&scene_with(
        r#"{ "id": "a", "type": "label", "at": [1, 1], "tex": "A" }"#,
    ));
    assert!(centered.contains("\\node at (1,1) {A};\n"), "{centered}");
}

#[test]
fn 線の種類は_tikzのスタイルの名前になる() {
    let tikz = tikz_of(SINE_AND_SHIFTED_SINE);
    assert!(tikz.contains("\\draw[line width=0.8pt] (-7,"), "実線");
    assert!(
        tikz.contains("\\draw[line width=0.8pt, dotted] (-7,"),
        "点線"
    );
    let dashed = tikz_of(&scene_with(
        r#"{ "id": "c", "type": "curve", "var": "t", "expr": ["t", "t"],
             "domain": [0, 1], "style": { "line": "dashed" } }"#,
    ));
    assert!(
        dashed.contains("\\draw[line width=0.8pt, dashed] (0,0)"),
        "{dashed}"
    );
}

#[test]
fn 矢じりのない軸は線だけになる() {
    let tikz = tikz_of(&scene_with(
        r#"{ "id": "x", "type": "axis", "direction": "x", "arrow": "none" }"#,
    ));
    assert!(
        tikz.contains("\\draw[line width=0.6pt] (-2,0) -- (3,0);\n"),
        "{tikz}"
    );
}

#[test]
fn 図は_tikzpictureで囲み_何も描かなくても正しい() {
    let tikz = tikz_of(&scene_with(""));
    assert!(
        tikz.contains("\\begin{tikzpicture}\n\\end{tikzpicture}\n"),
        "{tikz}"
    );
    assert!(tikz.ends_with('\n'));
}

#[test]
fn 行に末尾の空白がなく_長すぎる行もない() {
    let tikz = tikz_of(SINE_AND_SHIFTED_SINE);
    for line in tikz.lines() {
        assert!(!line.ends_with(' '), "末尾の空白: {line}");
        // 埋め込んだシーンの行は，元のJSONの長さに従う．
        if !line.starts_with('%') {
            assert!(line.chars().count() <= 110, "長い行: {line}");
        }
    }
}

#[test]
fn 先頭のコメントに版と必要なライブラリを書く() {
    let tikz = tikz_of(SINE_AND_SHIFTED_SINE);
    let header: Vec<&str> = tikz
        .lines()
        .take_while(|line| line.starts_with('%'))
        .collect();
    assert!(
        header
            .iter()
            .any(|line| line.contains(env!("CARGO_PKG_VERSION")))
    );
    assert!(
        header
            .iter()
            .any(|line| line.contains("\\usetikzlibrary{arrows.meta}"))
    );
}

#[test]
fn 埋め込んだシーンを取り出して_同じシーンを読める() {
    let tikz = tikz_of(SINE_AND_SHIFTED_SINE);
    let embedded = extract_scene(&tikz).expect("シーンが埋め込まれている");
    // 書かれたままの形で，取り出せる．
    assert_eq!(embedded, SINE_AND_SHIFTED_SINE.trim_end());
    assert_eq!(scene(&embedded), scene(SINE_AND_SHIFTED_SINE));
    // 取り出したシーンから作り直した出力は，元の出力と同じである．
    assert_eq!(tikz_of(&embedded), tikz);
}

#[test]
fn シーンが埋め込まれていない文字列からは何も取り出さない() {
    assert_eq!(
        extract_scene("\\begin{tikzpicture}\n\\end{tikzpicture}\n"),
        None
    );
    assert_eq!(extract_scene(""), None);
}

#[test]
fn 出力は同じ入力なら同じになる() {
    assert_eq!(
        tikz_of(SINE_AND_SHIFTED_SINE),
        tikz_of(SINE_AND_SHIFTED_SINE)
    );
}
