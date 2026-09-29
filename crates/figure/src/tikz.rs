//! 中間表現を，`TikZ`にする．
//!
//! 出力は，手で直さない．図を直すときは，元のシーンを直して，出力し直す．

use std::fmt::Write as _;

use crate::error::Error;
use crate::figure::{DotItem, Figure, FillItem, Item, LabelItem, Path, RasterItem};
use crate::parse::parse_scene;
use crate::raster::count_to_f64;
use crate::render::render;
use crate::scene::{Anchor, Arrow, Color, Line};
use crate::version::engine_version;

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

/// 中間表現を，`TikZ`にする．
#[must_use]
pub fn to_tikz(figure: &Figure) -> String {
    let mut out = String::new();
    // 文字列への書き込みは，失敗しない．
    let _ = writeln!(
        out,
        "% figure {}が生成した．手で直さず，シーンを直して，出力し直す．",
        engine_version()
    );
    out.push_str("% 必要：\\usetikzlibrary{arrows.meta}\n");
    out.push_str("\\begin{tikzpicture}\n");
    for item in &figure.items {
        match item {
            Item::Path(path) => write_path(&mut out, path),
            Item::Label(label) => write_label(&mut out, label),
            Item::Dot(dot) => write_dot(&mut out, dot),
            Item::Fill(fill) => write_fill(&mut out, fill),
            Item::Raster(raster) => write_raster(&mut out, raster),
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

/// 点の印．塗った丸である．
fn write_dot(out: &mut String, dot: &DotItem) {
    let color = dot
        .color
        .map_or_else(String::new, |color| format!("[{}]", color_name(color)));
    let _ = writeln!(
        out,
        "\\fill{color} {} circle ({}pt);",
        coordinate(dot.at),
        number(dot.radius)
    );
}

/// 塗った多角形．
fn write_fill(out: &mut String, fill: &FillItem) {
    let mut options = Vec::new();
    if let Some(color) = fill.color {
        options.push(color_name(color).to_owned());
    }
    options.push(format!("opacity={}", number(fill.opacity)));
    let _ = write!(out, "\\fill[{}]", options.join(", "));
    write_points(out, &fill.points);
    out.push_str(" -- cycle;\n");
}

/// 隣の升目との継ぎ目が見えないよう，升目を右と上へ延ばす長さ(cm)．
const CELL_OVERLAP: f64 = 0.005;

/// 画像．粗い升目を，塗った長方形として書く．横に続く同じ色の升目は，1つの長方形にまとめる．
/// 透明な升目は描かない．
fn write_raster(out: &mut String, raster: &RasterItem) {
    let cells = &raster.coarse;
    if cells.columns == 0 || cells.rows == 0 {
        return;
    }
    let [x0, y0] = raster.min;
    let [x1, y1] = raster.max;
    let width = (x1 - x0) / count_to_f64(cells.columns);
    let height = (y1 - y0) / count_to_f64(cells.rows);
    for (row, colors) in cells.pixels.chunks(cells.columns).enumerate() {
        let top = height.mul_add(-count_to_f64(row), y1);
        let bottom = top - height;
        // 上の行とは，継ぎ目が見えないよう重ねる．いちばん上の行は重ねない．
        let upper = if row > 0 { top + CELL_OVERLAP } else { top };
        let mut start = 0;
        while let Some(&color) = colors.get(start) {
            let end = colors
                .iter()
                .skip(start)
                .position(|other| *other != color)
                .map_or(colors.len(), |offset| start.saturating_add(offset));
            if color[3] > 0 {
                let left = width.mul_add(count_to_f64(start), x0);
                let overlap = if end < colors.len() {
                    CELL_OVERLAP
                } else {
                    0.0
                };
                let right = width.mul_add(count_to_f64(end), x0) + overlap;
                let [red, green, blue, alpha] = color;
                let opacity = if alpha < 255 {
                    format!(", opacity={}", number(f64::from(alpha) / 255.0))
                } else {
                    String::new()
                };
                let _ = writeln!(
                    out,
                    "\\fill[color={{rgb,255:red,{red};green,{green};blue,{blue}}}{opacity}] {} rectangle {};",
                    coordinate([left, bottom]),
                    coordinate([right, upper])
                );
            }
            start = end;
        }
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
    write_points(out, &path.points);
    out.push_str(";\n");
}

/// 点を，`--`でつないで，行ごとに折り返して書く．
fn write_points(out: &mut String, points: &[[f64; 2]]) {
    let points: Vec<String> = points.iter().copied().map(coordinate).collect();
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

/// シーンのJSONを読み，描画して，`TikZ`にする．
///
/// # Errors
///
/// JSONの誤りや，式の誤りがあると，誤りを返す．
pub fn export_tikz(scene_json: &str) -> Result<String, Error> {
    let scene = parse_scene(scene_json)?;
    let figure = render(&scene)?;
    Ok(to_tikz(&figure))
}
