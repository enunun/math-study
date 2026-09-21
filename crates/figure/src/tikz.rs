//! 中間表現を，`TikZ`にする．
//!
//! 出力は，手で直さない．図を直すときは，元のシーンを直して，出力し直す．
//! 元のシーンは，先頭のコメントに埋め込み，`extract_scene`で取り出せる．

use std::fmt::Write as _;

use crate::error::Error;
use crate::figure::{Figure, Item, LabelItem, Path};
use crate::parse::parse_scene;
use crate::render::render;
use crate::scene::{Anchor, Arrow, Color, Line};
use crate::version::engine_version;

/// 埋め込んだシーンの，始まりを示す行．
const SCENE_BEGIN: &str = "% --- シーン ---";
/// 埋め込んだシーンの，終わりを示す行．
const SCENE_END: &str = "% --- シーンの終わり ---";
/// 折れ線の，最初の行に並べる点の数．最初の行は，`\draw[…]`が長い．
const FIRST_LINE_POINTS: usize = 3;
/// 折れ線の，続きの行に並べる点の数．
const LINE_POINTS: usize = 4;

/// 数を，`TikZ`の座標に書く形にする．小数点以下は4桁までで，末尾の0は省く．
#[must_use]
pub fn number(value: f64) -> String {
    let text = format!("{value:.4}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn coordinate(point: [f64; 2]) -> String {
    format!("({},{})", number(point[0]), number(point[1]))
}

/// 中間表現を，`TikZ`にする．`scene_json`は，先頭のコメントに埋め込む，元のシーンである．
#[must_use]
pub fn to_tikz(figure: &Figure, scene_json: &str) -> String {
    let mut out = String::new();
    // 文字列への書き込みは，失敗しない．
    let _ = writeln!(
        out,
        "% figure {}が生成した．手で直さず，シーンを直して，出力し直す．",
        engine_version()
    );
    out.push_str("% 必要：\\usetikzlibrary{arrows.meta}\n");
    out.push_str(SCENE_BEGIN);
    out.push('\n');
    for line in scene_json.lines() {
        if line.is_empty() {
            out.push_str("%\n");
        } else {
            let _ = writeln!(out, "% {line}");
        }
    }
    out.push_str(SCENE_END);
    out.push('\n');
    out.push_str("\\begin{tikzpicture}\n");
    for item in &figure.items {
        match item {
            Item::Path(path) => write_path(&mut out, path),
            Item::Label(label) => write_label(&mut out, label),
        }
    }
    out.push_str("\\end{tikzpicture}\n");
    out
}

/// 色の名前を，`xcolor`の名前にする．どれも，`xcolor`の基本の色で，追加の指定なしに使える．
const fn color_name(color: Color) -> &'static str {
    match color {
        Color::Gray => "gray",
        Color::Red => "red",
        Color::Blue => "blue",
        Color::Green => "green!50!black",
        Color::Orange => "orange",
        Color::Purple => "violet",
    }
}

fn write_path(out: &mut String, path: &Path) {
    let mut options = vec![format!("line width={}pt", number(path.stroke.width))];
    if let Some(color) = path.stroke.color {
        options.push(color_name(color).to_owned());
    }
    match path.stroke.line {
        Line::Solid => {}
        Line::Dotted => options.push("dotted".to_owned()),
        Line::Dashed => options.push("dashed".to_owned()),
    }
    if let Some(arrow) = &path.arrow {
        match arrow.kind {
            Arrow::Stealth => options.push("-{Stealth}".to_owned()),
            Arrow::None => {}
        }
    }
    let _ = write!(out, "\\draw[{}]", options.join(", "));

    let points: Vec<String> = path.points.iter().copied().map(coordinate).collect();
    let mut rest = points.as_slice();
    let mut size = FIRST_LINE_POINTS;
    let mut first = true;
    while !rest.is_empty() {
        let (line, tail) = rest.split_at(size.min(rest.len()));
        let joined = line.join(" -- ");
        if first {
            let _ = write!(out, " {joined}");
        } else {
            let _ = write!(out, "\n  -- {joined}");
        }
        first = false;
        rest = tail;
        size = LINE_POINTS;
    }
    out.push_str(";\n");
}

fn write_label(out: &mut String, label: &LabelItem) {
    let anchor = match label.anchor {
        Anchor::Center => String::new(),
        Anchor::North => "[anchor=north]".to_owned(),
        Anchor::South => "[anchor=south]".to_owned(),
        Anchor::East => "[anchor=east]".to_owned(),
        Anchor::West => "[anchor=west]".to_owned(),
        Anchor::NorthEast => "[anchor=north east]".to_owned(),
        Anchor::NorthWest => "[anchor=north west]".to_owned(),
        Anchor::SouthEast => "[anchor=south east]".to_owned(),
        Anchor::SouthWest => "[anchor=south west]".to_owned(),
    };
    let _ = writeln!(
        out,
        "\\node{anchor} at {} {{{}}};",
        coordinate(label.at),
        label.tex
    );
}

/// `TikZ`の先頭のコメントから，埋め込まれた元のシーンを取り出す．
#[must_use]
pub fn extract_scene(tikz: &str) -> Option<String> {
    let mut lines = tikz.lines().skip_while(|line| *line != SCENE_BEGIN);
    lines.next()?;
    let mut scene = Vec::new();
    for line in lines {
        if line == SCENE_END {
            return Some(scene.join("\n"));
        }
        let text = line.strip_prefix("% ").or_else(|| line.strip_prefix('%'))?;
        scene.push(text);
    }
    None
}

/// シーンのJSONを読み，描画して，`TikZ`にする．JSONは，書かれたままの形で，先頭のコメントに埋め込む．
///
/// # Errors
///
/// JSONの誤りや，式の誤りがあると，誤りを返す．
pub fn export_tikz(scene_json: &str) -> Result<String, Error> {
    let scene = parse_scene(scene_json)?;
    let figure = render(&scene)?;
    Ok(to_tikz(&figure, scene_json))
}
