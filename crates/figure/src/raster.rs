//! 画像(`RasterItem`)を作る．値から色への対応，偏角から色相への対応，PNGの書き出し．
//!
//! PNGは，各行にPaethの予測を施し，zlibで圧縮する．値がなめらかに変わる画像では，予測の残りが
//! 小さくなり，よく縮む．

use miniz_oxide::deflate::compress_to_vec_zlib;

use crate::figure::{Cells, RasterItem};
use crate::scene::Colormap;

/// 透明な画素．
pub const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

/// viridisの，等間隔の9点の色．間は線形に補う．
const VIRIDIS: [[u8; 3]; 9] = [
    [0x44, 0x01, 0x54],
    [0x47, 0x2d, 0x7b],
    [0x3b, 0x52, 0x8b],
    [0x2c, 0x72, 0x8e],
    [0x21, 0x91, 0x8c],
    [0x28, 0xae, 0x80],
    [0x5e, 0xc9, 0x62],
    [0xad, 0xdc, 0x30],
    [0xfd, 0xe7, 0x25],
];

/// 青，白，赤の，等間隔の5点の色．
const COOLWARM: [[u8; 3]; 5] = [
    [0x3b, 0x4c, 0xc0],
    [0x8d, 0xb0, 0xfe],
    [0xdd, 0xdd, 0xdd],
    [0xf4, 0x9a, 0x7b],
    [0xb4, 0x04, 0x26],
];

/// 個数を，浮動小数点の数にする．
#[must_use]
pub fn count_to_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// 0以上の数を，切り捨てて個数にする．負の数や有限でない数は0にする．
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#[must_use]
pub fn f64_to_count(value: f64) -> usize {
    if value.is_finite() && value > 0.0 {
        // 0以上の有限の数で，u32の範囲に収めてある．
        value.min(f64::from(u32::MAX)) as usize
    } else {
        0
    }
}

/// 0から1の数を，0から255の整数にする．
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn channel(value: f64) -> u8 {
    // 0から255に収めてから丸めるので，切り捨てても値は変わらない．非数は0になる．
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// 色の並びを，0から1の位置で補う．
fn interpolate(stops: &[[u8; 3]], t: f64) -> [u8; 4] {
    let last = stops.len().saturating_sub(1);
    let position = t.clamp(0.0, 1.0) * count_to_f64(last);
    let index = f64_to_count(position.floor()).min(last.saturating_sub(1));
    let fraction = position - count_to_f64(index);
    let (Some(low), Some(high)) = (stops.get(index), stops.get(index.saturating_add(1))) else {
        return TRANSPARENT;
    };
    let mix = |from: u8, to: u8| {
        channel((f64::from(to) - f64::from(from)).mul_add(fraction, f64::from(from)) / 255.0)
    };
    [
        mix(low[0], high[0]),
        mix(low[1], high[1]),
        mix(low[2], high[2]),
        255,
    ]
}

/// 0から1の値を，色にする．値が有限でなければ透明である．
#[must_use]
pub fn colormap(map: Colormap, t: f64) -> [u8; 4] {
    if !t.is_finite() {
        return TRANSPARENT;
    }
    match map {
        Colormap::Viridis => interpolate(&VIRIDIS, t),
        Colormap::Gray => {
            let level = channel(t);
            [level, level, level, 255]
        }
        Colormap::GrayInverse => {
            let level = channel(1.0 - t);
            [level, level, level, 255]
        }
        Colormap::Coolwarm => interpolate(&COOLWARM, t),
    }
}

/// 色相(0から1で一周)，彩度，明度から色にする．
#[must_use]
pub fn hsv(hue: f64, saturation: f64, value: f64) -> [u8; 4] {
    let scaled = hue.rem_euclid(1.0) * 6.0;
    let sector = scaled.floor();
    let rest = scaled - sector;
    let low = value * (1.0 - saturation);
    let falling = value * saturation.mul_add(-rest, 1.0);
    let rising = value * saturation.mul_add(rest - 1.0, 1.0);
    let (red, green, blue) = match f64_to_count(sector) {
        0 => (value, rising, low),
        1 => (falling, value, low),
        2 => (low, value, rising),
        3 => (low, falling, value),
        4 => (rising, low, value),
        _ => (value, low, falling),
    };
    [channel(red), channel(green), channel(blue), 255]
}

/// Paethの予測．左，上，左上の画素のうち，`左 + 上 - 左上`に最も近いもの．
fn paeth(left: u8, up: u8, up_left: u8) -> u8 {
    let to_left = up.abs_diff(up_left);
    let to_up = left.abs_diff(up_left);
    // どの値も0から255なので，差の和は，i16の範囲に収まる．
    let to_up_left = (i16::from(left).wrapping_sub(i16::from(up_left)))
        .wrapping_add(i16::from(up).wrapping_sub(i16::from(up_left)))
        .unsigned_abs();
    if u16::from(to_left) <= u16::from(to_up) && u16::from(to_left) <= to_up_left {
        left
    } else if u16::from(to_up) <= to_up_left {
        up
    } else {
        up_left
    }
}

/// PNGのチャンク．長さ，種類，データ，CRCを並べる．
fn chunk(out: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    let length = u32::try_from(data.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&length.to_be_bytes());
    let mut body = kind.to_vec();
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}

/// CRC-32(PNGとzlibの，多項式0xEDB88320)．
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = crc.wrapping_shr(1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// 画素(赤，緑，青，不透明度)を，PNGのファイルにする．各行にPaethの予測を施して，zlibで圧縮する．
#[must_use]
pub fn encode_png(columns: usize, rows: usize, pixels: &[[u8; 4]]) -> Vec<u8> {
    let lines: Vec<Vec<u8>> = pixels
        .chunks(columns.max(1))
        .take(rows)
        .map(|line| line.iter().flatten().copied().collect())
        .collect();
    let empty = Vec::new();
    let mut filtered = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        let above = row
            .checked_sub(1)
            .and_then(|previous| lines.get(previous))
            .unwrap_or(&empty);
        // 行の先頭は，予測の種類(4はPaeth)である．
        filtered.push(4);
        for (index, &current) in line.iter().enumerate() {
            let before = index.checked_sub(4);
            let left = before.and_then(|at| line.get(at)).copied().unwrap_or(0);
            let up = above.get(index).copied().unwrap_or(0);
            let up_left = before.and_then(|at| above.get(at)).copied().unwrap_or(0);
            filtered.push(current.wrapping_sub(paeth(left, up, up_left)));
        }
    }
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&u32::try_from(columns).unwrap_or(0).to_be_bytes());
    header.extend_from_slice(&u32::try_from(rows).unwrap_or(0).to_be_bytes());
    // 8ビット，RGBA，圧縮・フィルタの方式は標準，インターレースなし．
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, *b"IHDR", &header);
    chunk(&mut out, *b"IDAT", &compress_to_vec_zlib(&filtered, 9));
    chunk(&mut out, *b"IEND", &[]);
    out
}

/// Base64で書く．
#[must_use]
pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for group in bytes.chunks(3) {
        let [first, second, third] = [0, 1, 2].map(|at| group.get(at).copied().unwrap_or(0));
        let triple = u32::from_be_bytes([0, first, second, third]);
        for (position, shift) in [18_u32, 12, 6, 0].into_iter().enumerate() {
            if position <= group.len() {
                let index = usize::try_from(triple.wrapping_shr(shift) & 0x3F).unwrap_or(0);
                out.push(char::from(ALPHABET.get(index).copied().unwrap_or(b'=')));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// 長方形の範囲を，長い辺が`resolution`個になる升目に分けたときの，横と縦の数．どちらも1以上である．
#[must_use]
pub fn grid_size(width: f64, height: f64, resolution: u32) -> (usize, usize) {
    let resolution = f64::from(resolution.max(1));
    let (columns, rows) = if width >= height {
        (resolution, (resolution * height / width).round())
    } else {
        ((resolution * width / height).round(), resolution)
    };
    (f64_to_count(columns).max(1), f64_to_count(rows).max(1))
}

/// 升目の中心(数学の座標)．行は上から，列は左から並べる．
#[must_use]
pub fn cell_centers(domain: [[f64; 2]; 2], (columns, rows): (usize, usize)) -> Vec<[f64; 2]> {
    let [[x0, x1], [y0, y1]] = domain;
    let (dx, dy) = (
        (x1 - x0) / count_to_f64(columns),
        (y1 - y0) / count_to_f64(rows),
    );
    let mut centers = Vec::with_capacity(columns.saturating_mul(rows));
    for row in 0..rows {
        let y = (count_to_f64(row) + 0.5).mul_add(-dy, y1);
        for column in 0..columns {
            let x = (count_to_f64(column) + 0.5).mul_add(dx, x0);
            centers.push([x, y]);
        }
    }
    centers
}

/// 点ごとの色を，升目の中心で求める．`color`は，数学の座標から色を返す．
#[must_use]
pub fn sample_cells(
    domain: [[f64; 2]; 2],
    size: (usize, usize),
    color: &dyn Fn(f64, f64) -> [u8; 4],
) -> Vec<[u8; 4]> {
    cell_centers(domain, size)
        .into_iter()
        .map(|[x, y]| color(x, y))
        .collect()
}

/// 升目の大きさごとに色の並びを返す関数．範囲と，升目の(横，縦)の数を受け取る．
pub type GridColors<'a> = &'a dyn Fn([[f64; 2]; 2], (usize, usize)) -> Vec<[u8; 4]>;

/// 画像の要素を作る．`domain`は数学の座標，`min`，`max`は図の座標(cm)で，画素と粗い升目は，
/// それぞれの解像度で`colors`から求める．
#[must_use]
pub fn raster_item(
    domain: [[f64; 2]; 2],
    [min, max]: [[f64; 2]; 2],
    [resolution, coarse_resolution]: [u32; 2],
    colors: GridColors,
) -> RasterItem {
    let [[x0, x1], [y0, y1]] = domain;
    let (columns, rows) = grid_size(x1 - x0, y1 - y0, resolution);
    let pixels = colors(domain, (columns, rows));
    let coarse_size = grid_size(x1 - x0, y1 - y0, coarse_resolution);
    let coarse = Cells {
        columns: coarse_size.0,
        rows: coarse_size.1,
        pixels: colors(domain, coarse_size),
    };
    let png = encode_png(columns, rows, &pixels);
    RasterItem {
        min,
        max,
        columns,
        rows,
        href: format!("data:image/png;base64,{}", base64(&png)),
        pixels,
        coarse,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64は標準の符号である() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
        assert_eq!(base64(b""), "");
    }

    #[test]
    fn crc32は既知の値をとる() {
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn pngは署名と3つのチャンクからなる() {
        let png = encode_png(2, 1, &[[255, 0, 0, 255], [0, 0, 255, 128]]);
        assert_eq!(
            png.get(..8),
            Some(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A][..])
        );
        assert_eq!(png.get(12..16), Some(&b"IHDR"[..]));
        assert_eq!(png.get(png.len() - 8..png.len() - 4), Some(&b"IEND"[..]));
    }
}
