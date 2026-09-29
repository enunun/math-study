//! フーリエ級数の部分和(`fourier_series`)，1次元のフーリエ変換(`fourier_transform`)，2次元のフーリエ変換の
//! 強さ(`fourier_intensity`)．
//!
//! どれも，関数の値を決まった点で一度だけ求めておき(式を読むとき)，描くときは，その値の重み付きの和を
//! とる．係数は，1周期の中点則(周期関数では，三角多項式に対して厳密)，変換は，区間を分けたGauss-Legendre
//! 則(4点)，2次元の変換は，中点則の2重の和を，`x`と`y`に分けて計算する．値が有限でない点は，0とみなす．

// 複素数の四則演算(`Complex`の`Add`など)は浮動小数点の演算で，あふれて止まることはない．
#![allow(clippy::arithmetic_side_effects)]

use crate::compile::{Env, evaluate_domain};
use crate::error::ErrorKind;
use crate::expr::Complex;
use crate::figure::{DotItem, Item, RasterItem, Stroke};
use crate::heatmap::{Area, ColorSettings, complex_expr, variable_names};
use crate::placement::Placement;
use crate::raster::{cell_centers, count_to_f64};
use crate::scene::{
    Bound, FourierIntensity, FourierSeries, FourierTransform, Line, Quantity, SeriesMode, Style,
    TransformPart,
};

/// 点の印の半径(pt)．
const DOT_RADIUS: f64 = 2.0;
/// 曲線の線の既定の太さ(pt)．
const CURVE_WIDTH: f64 = 0.8;
/// 係数を求める標本の数の下限．
const MIN_SERIES_SAMPLES: usize = 8192;
/// 変換の，区間の分割の数の下限と上限．
const MIN_PANELS: f64 = 64.0;
const MAX_PANELS: f64 = 16384.0;
/// 変換の，被積分関数の1周期あたりの区間の数．
const PANELS_PER_WAVE: f64 = 8.0;
/// 4点のGauss-Legendre則の，`[-1, 1]`の上の点と重み．
const GAUSS_LEGENDRE: [(f64, f64); 4] = [
    (-0.861_136_311_594_052_6, 0.347_854_845_137_453_8),
    (-0.339_981_043_584_856_3, 0.652_145_154_862_546_1),
    (0.339_981_043_584_856_3, 0.652_145_154_862_546_1),
    (0.861_136_311_594_052_6, 0.347_854_845_137_453_8),
];

/// 値が有限でなければ0にする．
fn finite_or_zero(value: Complex) -> Complex {
    if value.is_finite() {
        value
    } else {
        Complex::real(0.0)
    }
}

/// 変数の値のあとに，媒介変数と点の座標を並べた，式に渡す値．
fn arguments(variables: &[f64], parameters: &[f64]) -> Vec<Complex> {
    variables
        .iter()
        .chain(parameters)
        .map(|value| Complex::real(*value))
        .collect()
}

/// 区間を評価する．誤りは，項目の名前`field`で返す．
fn interval(bounds: &[Bound; 2], env: &Env, field: &'static str) -> Result<[f64; 2], ErrorKind> {
    evaluate_domain(bounds, env).map_err(|_| ErrorKind::InvalidRange(field))
}

/// 式を読んだ後の，フーリエ級数．
pub struct FourierSeriesPlot {
    /// 定数項`a_0 / 2`．
    pub constant: f64,
    /// `n = 1, 2, …`の係数`(a_n, b_n)`．
    pub coefficients: Vec<(f64, f64)>,
    /// 角振動数`2π / L`．
    pub omega: f64,
    /// 部分和を描く範囲．なければ見える範囲．
    pub domain: Option<[f64; 2]>,
}

impl FourierSeriesPlot {
    /// 部分和の値．
    fn sum(&self, x: f64) -> f64 {
        let mut total = self.constant;
        let mut order = 0.0;
        for (a, b) in &self.coefficients {
            order += 1.0;
            let (sin, cos) = (order * self.omega * x).sin_cos();
            total += a * cos + b * sin;
        }
        total
    }

    /// 次数ごとの振幅．0次は`|a_0| / 2`である．
    fn amplitudes(&self) -> impl Iterator<Item = f64> + '_ {
        std::iter::once(self.constant.abs())
            .chain(self.coefficients.iter().map(|(a, b)| a.hypot(*b)))
    }
}

/// フーリエ級数の係数を求める．
///
/// # Errors
///
/// 式，周期，次数に誤りがあると，誤りを返す．
pub fn compile_fourier_series(
    series: &FourierSeries,
    env: &Env,
) -> Result<FourierSeriesPlot, ErrorKind> {
    let names = variable_names(std::slice::from_ref(&series.var), env)?;
    let expr = complex_expr(&series.expr, &names, env)?;
    let [start, end] = interval(&series.period, env, "period")?;
    let domain = series
        .domain
        .as_ref()
        .map(|domain| interval(domain, env, "domain"))
        .transpose()?;
    let terms = crate::compile::evaluate_bound("terms", &series.terms, 0, env)?.round();
    if !(0.0..=FourierSeries::MAX_TERMS).contains(&terms) {
        return Err(ErrorKind::Invalid(format!(
            "フーリエ級数の次数`terms`は，0以上{}以下にする(今は{terms})．",
            FourierSeries::MAX_TERMS
        )));
    }
    let length = end - start;
    let omega = std::f64::consts::TAU / length;
    let count = crate::raster::f64_to_count(terms);
    let samples = MIN_SERIES_SAMPLES.max(count.saturating_add(1).saturating_mul(16));
    let step = length / count_to_f64(samples);
    let values: Vec<(f64, f64)> = (0..samples)
        .map(|index| {
            let x = (count_to_f64(index) + 0.5).mul_add(step, start);
            let value = finite_or_zero(expr.eval_complex(&arguments(&[x], env.parameters)));
            (x, value.re)
        })
        .collect();
    let scale = 2.0 / count_to_f64(samples);
    let coefficient = |order: f64| -> (f64, f64) {
        values.iter().fold((0.0, 0.0), |(a, b), (x, value)| {
            let (sin, cos) = (order * omega * x).sin_cos();
            (value.mul_add(cos, a), value.mul_add(sin, b))
        })
    };
    let (a0, _) = coefficient(0.0);
    let mut coefficients = Vec::with_capacity(count);
    let mut order = 0.0;
    for _ in 0..count {
        order += 1.0;
        let (a, b) = coefficient(order);
        coefficients.push((a * scale, b * scale));
    }
    Ok(FourierSeriesPlot {
        constant: a0 * scale / 2.0,
        coefficients,
        omega,
        domain,
    })
}

/// 線のスタイル．
fn stroke(style: &Style) -> Stroke {
    Stroke {
        line: style.line.unwrap_or(Line::Solid),
        width: style.width_pt().unwrap_or(CURVE_WIDTH),
        color: style.color,
    }
}

/// フーリエ級数を描く要素．部分和のグラフか，振幅のスペクトル(縦の線と点)．
#[must_use]
pub fn fourier_series_items(
    series: &FourierSeries,
    plot: &FourierSeriesPlot,
    placement: &Placement,
    view_x: [f64; 2],
) -> Vec<Item> {
    let line = stroke(&series.style);
    match series.mode {
        SeriesMode::Sum => {
            placement.curve(|x| [x, plot.sum(x)], plot.domain.unwrap_or(view_x), line)
        }
        SeriesMode::Amplitude => {
            let mut items = Vec::new();
            let mut order = -1.0;
            for amplitude in plot.amplitudes() {
                order += 1.0;
                if amplitude > 1e-12 {
                    items.extend(placement.curve(|t| [order, t * amplitude], [0.0, 1.0], line));
                }
                if let Some(at) = placement
                    .place([order, amplitude])
                    .filter(|at| placement.inside(*at))
                {
                    items.push(Item::Dot(DotItem {
                        at,
                        radius: DOT_RADIUS,
                        color: series.style.color,
                    }));
                }
            }
            items
        }
    }
}

/// 式を読んだ後の，1次元のフーリエ変換．積分の点，重み，関数の値．
pub struct FourierTransformPlot {
    /// 積分の点と，重みを掛けた関数の値．
    pub nodes: Vec<(f64, Complex)>,
    /// 描く`k`の範囲．
    pub domain: [f64; 2],
}

impl FourierTransformPlot {
    /// `F(k)`．
    fn at(&self, k: f64) -> Complex {
        self.nodes
            .iter()
            .fold(Complex::real(0.0), |total, (x, weighted)| {
                let (sin, cos) = (k * x).sin_cos();
                total + *weighted * Complex::new(cos, -sin)
            })
    }
}

/// 1次元のフーリエ変換の積分の点を求める．区間は，描く`k`の最大の絶対値で，被積分関数の1周期に
/// 8つの区間が入るように分ける．
///
/// # Errors
///
/// 式か範囲に誤りがあると，誤りを返す．
pub fn compile_fourier_transform(
    transform: &FourierTransform,
    env: &Env,
    view_x: [f64; 2],
) -> Result<FourierTransformPlot, ErrorKind> {
    let names = variable_names(std::slice::from_ref(&transform.var), env)?;
    let expr = complex_expr(&transform.expr, &names, env)?;
    let [start, end] = interval(&transform.support, env, "support")?;
    let domain = transform
        .domain
        .as_ref()
        .map(|domain| interval(domain, env, "domain"))
        .transpose()?
        .unwrap_or(view_x);
    let reach = domain[0].abs().max(domain[1].abs());
    let length = end - start;
    let panels = (reach * length / std::f64::consts::TAU * PANELS_PER_WAVE + MIN_PANELS)
        .ceil()
        .clamp(MIN_PANELS, MAX_PANELS);
    let width = length / panels;
    let mut nodes = Vec::new();
    let mut left = start;
    for _ in 0..crate::raster::f64_to_count(panels) {
        let middle = left + width / 2.0;
        for (point, weight) in GAUSS_LEGENDRE {
            let x = (width / 2.0).mul_add(point, middle);
            let value = finite_or_zero(expr.eval_complex(&arguments(&[x], env.parameters)));
            nodes.push((x, value * Complex::real(weight * width / 2.0)));
        }
        left += width;
    }
    Ok(FourierTransformPlot { nodes, domain })
}

/// 1次元のフーリエ変換のグラフ．
#[must_use]
pub fn fourier_transform_items(
    transform: &FourierTransform,
    plot: &FourierTransformPlot,
    placement: &Placement,
) -> Vec<Item> {
    placement.curve(
        |k| {
            let value = plot.at(k);
            let y = match transform.part {
                TransformPart::Abs => value.norm(),
                TransformPart::Re => value.re,
                TransformPart::Im => value.im,
                TransformPart::Power => value.re.mul_add(value.re, value.im * value.im),
            };
            [k, y]
        },
        plot.domain,
        stroke(&transform.style),
    )
}

/// 式を読んだ後の，2次元のフーリエ変換の強さ．開口の関数を標本化した値．
pub struct FourierIntensityPlot {
    /// `x`の標本の点．
    pub xs: Vec<f64>,
    /// `y`の標本の点．
    pub ys: Vec<f64>,
    /// 重みを掛けた関数の値．行が`y`，列が`x`である．
    pub values: Vec<Vec<Complex>>,
    /// 画像を置く`(kx, ky)`の範囲．なければ見える範囲．
    pub domain: Option<[[f64; 2]; 2]>,
    /// 色の両端の値．
    pub range: Option<[f64; 2]>,
}

/// 開口の関数を標本化する．
///
/// # Errors
///
/// 変数の名前，式，範囲に誤りがあると，誤りを返す．
pub fn compile_fourier_intensity(
    intensity: &FourierIntensity,
    env: &Env,
) -> Result<FourierIntensityPlot, ErrorKind> {
    let names = variable_names(&intensity.vars, env)?;
    let expr = complex_expr(&intensity.expr, &names, env)?;
    let [x_range, y_range] = &intensity.support;
    let [x0, x1] = interval(x_range, env, "support")?;
    let [y0, y1] = interval(y_range, env, "support")?;
    let count =
        usize::try_from(intensity.samples.unwrap_or(FourierIntensity::SAMPLES)).unwrap_or(1);
    let points = |low: f64, high: f64| -> Vec<f64> {
        let step = (high - low) / count_to_f64(count);
        (0..count)
            .map(|index| (count_to_f64(index) + 0.5).mul_add(step, low))
            .collect()
    };
    let (xs, ys) = (points(x0, x1), points(y0, y1));
    let weight = Complex::real((x1 - x0) * (y1 - y0) / count_to_f64(count.saturating_mul(count)));
    let values = ys
        .iter()
        .map(|y| {
            xs.iter()
                .map(|x| {
                    finite_or_zero(expr.eval_complex(&arguments(&[*x, *y], env.parameters)))
                        * weight
                })
                .collect()
        })
        .collect();
    let domain = intensity
        .domain
        .as_ref()
        .map(|[x, y]| {
            Ok::<_, ErrorKind>([interval(x, env, "domain")?, interval(y, env, "domain")?])
        })
        .transpose()?;
    let range = intensity
        .range
        .as_ref()
        .map(|range| interval(range, env, "range"))
        .transpose()?;
    Ok(FourierIntensityPlot {
        xs,
        ys,
        values,
        domain,
        range,
    })
}

/// 点ごとの`e^{-ikx}`の表．行が`k`，列が`x`である．
fn phases(ks: &[f64], xs: &[f64]) -> Vec<Vec<Complex>> {
    ks.iter()
        .map(|k| {
            xs.iter()
                .map(|x| {
                    let (sin, cos) = (k * x).sin_cos();
                    Complex::new(cos, -sin)
                })
                .collect()
        })
        .collect()
}

/// 升目の中心の`(kx, ky)`での`|F|^2`か`|F|`．`x`の和を先にとり，次に`y`の和をとる．
fn intensity_grid(
    intensity: &FourierIntensity,
    plot: &FourierIntensityPlot,
    domain: [[f64; 2]; 2],
    size: (usize, usize),
) -> Vec<f64> {
    let centers = cell_centers(domain, size);
    let (columns, rows) = size;
    let kxs: Vec<f64> = centers.iter().take(columns).map(|[kx, _]| *kx).collect();
    let kys: Vec<f64> = centers
        .iter()
        .step_by(columns.max(1))
        .take(rows)
        .map(|[_, ky]| *ky)
        .collect();
    let x_phases = phases(&kxs, &plot.xs);
    let y_phases = phases(&kys, &plot.ys);
    // partial[c][j] = Σ_i f(x_i, y_j) e^{-i kx_c x_i}．
    let partial: Vec<Vec<Complex>> = x_phases
        .iter()
        .map(|row_phase| {
            plot.values
                .iter()
                .map(|line| {
                    line.iter()
                        .zip(row_phase)
                        .fold(Complex::real(0.0), |total, (value, phase)| {
                            total + *value * *phase
                        })
                })
                .collect()
        })
        .collect();
    let mut grid = Vec::with_capacity(columns.saturating_mul(rows));
    for column_phase in &y_phases {
        for sums in &partial {
            let value = sums
                .iter()
                .zip(column_phase)
                .fold(Complex::real(0.0), |total, (sum, phase)| {
                    total + *sum * *phase
                });
            let amplitude = value.norm();
            grid.push(match intensity.quantity {
                Quantity::Intensity => amplitude * amplitude,
                Quantity::Amplitude => amplitude,
            });
        }
    }
    grid
}

/// 2次元のフーリエ変換の強さの画像．見える範囲と重ならなければ`None`．
#[must_use]
pub fn fourier_intensity_item(
    intensity: &FourierIntensity,
    plot: &FourierIntensityPlot,
    area: &Area,
) -> Option<RasterItem> {
    let domain = area.clip(plot.domain)?;
    let settings = ColorSettings {
        colormap: intensity.colormap,
        scale: intensity.scale,
        range: plot.range,
        normalize: true,
        resolution: intensity.resolution,
        tikz_resolution: intensity.tikz_resolution,
    };
    let values =
        |domain: [[f64; 2]; 2], size: (usize, usize)| intensity_grid(intensity, plot, domain, size);
    Some(crate::heatmap::value_raster(
        area, domain, &settings, &values,
    ))
}
