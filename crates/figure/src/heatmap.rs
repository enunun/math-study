//! 値を色で表す図(`heatmap`)と，複素関数の色塗り(`domain_coloring`)．どちらも，式を複素数の式として読み，
//! 升目の中心ごとに評価して，画像(`RasterItem`)にする．

use crate::compile::{Env, evaluate_domain};
use crate::error::ErrorKind;
use crate::expr::{Complex, Expr, is_reserved_name};
use crate::figure::RasterItem;
use crate::raster::{TRANSPARENT, cell_centers, colormap, grid_size, hsv, raster_item};
use crate::scene::{Bound, Colormap, DomainColoring, Heatmap, Shading, ValueScale, raster_limits};

/// 式を読んだ後の，値を色で表す図．
pub struct HeatmapPlot {
    /// 値の式．名前の順は，2つの変数，媒介変数と点の座標である．
    pub expr: Expr,
    /// 画像を置く範囲．なければ見える範囲．
    pub domain: Option<[[f64; 2]; 2]>,
    /// 色の両端の値．なければ，値から決める．
    pub range: Option<[f64; 2]>,
}

/// 式を読んだ後の，複素関数の色塗り．
pub struct DomainColoringPlot {
    /// 関数の式．名前の順は，複素数の変数，媒介変数と点の座標である．
    pub expr: Expr,
    /// 画像を置く範囲．なければ見える範囲．
    pub domain: Option<[[f64; 2]; 2]>,
}

/// 変数の名前を検査し，変数を先頭にした，式の名前の並びを作る．虚数単位`i`は変数にできない．
fn variable_names<'a>(vars: &'a [String], env: &Env<'a>) -> Result<Vec<&'a str>, ErrorKind> {
    for (index, var) in vars.iter().enumerate() {
        if is_reserved_name(var) || var == "i" {
            return Err(ErrorKind::ReservedName(var.clone()));
        }
        if env.names.contains(&var.as_str()) || vars.iter().take(index).any(|other| other == var) {
            return Err(ErrorKind::NameConflict(var.clone()));
        }
    }
    Ok(vars
        .iter()
        .map(String::as_str)
        .chain(env.names.iter().copied())
        .collect())
}

/// 複素数の式を読む．
fn complex_expr(source: &str, names: &[&str], env: &Env) -> Result<Expr, ErrorKind> {
    let expr = Expr::compile_complex(source, names, env.functions).map_err(|error| {
        ErrorKind::Expression {
            field: "expr",
            index: 0,
            error,
        }
    })?;
    if expr.uses_vector_functions() {
        return Err(ErrorKind::Invalid(
            "内積(`dot`)，外積(`cross`)，長さ(`norm`)は，点の式でだけ使える．".to_owned(),
        ));
    }
    Ok(expr)
}

/// 画像を置く範囲を評価する．
fn evaluate_area(
    domain: Option<&[[Bound; 2]; 2]>,
    env: &Env,
) -> Result<Option<[[f64; 2]; 2]>, ErrorKind> {
    domain
        .map(|[x, y]| Ok([evaluate_domain(x, env)?, evaluate_domain(y, env)?]))
        .transpose()
}

/// 値を色で表す図の式と範囲を読む．
///
/// # Errors
///
/// 変数の名前，式，範囲に誤りがあると，誤りを返す．
pub fn compile_heatmap(heatmap: &Heatmap, env: &Env) -> Result<HeatmapPlot, ErrorKind> {
    let names = variable_names(&heatmap.vars, env)?;
    let expr = complex_expr(&heatmap.expr, &names, env)?;
    let domain = evaluate_area(heatmap.domain.as_ref(), env)?;
    let range = match &heatmap.range {
        Some(range) => {
            let [low, high] =
                evaluate_domain(range, env).map_err(|_| ErrorKind::InvalidRange("range"))?;
            Some([low, high])
        }
        None => None,
    };
    Ok(HeatmapPlot {
        expr,
        domain,
        range,
    })
}

/// 複素関数の色塗りの式と範囲を読む．
///
/// # Errors
///
/// 変数の名前，式，範囲に誤りがあると，誤りを返す．
pub fn compile_domain_coloring(
    coloring: &DomainColoring,
    env: &Env,
) -> Result<DomainColoringPlot, ErrorKind> {
    let vars = [coloring.var.clone()];
    let names = variable_names(&vars, env)?;
    let expr = complex_expr(&coloring.expr, &names, env)?;
    let domain = evaluate_area(coloring.domain.as_ref(), env)?;
    Ok(DomainColoringPlot { expr, domain })
}

/// 画像を置く範囲と，その図の座標(cm)．
pub struct Area<'a> {
    /// 見える範囲(`[[xの下端, 上端], [yの下端, 上端]]`)．
    pub view: [[f64; 2]; 2],
    /// 単位の長さ(cm)．図の座標は，数学の座標にこれを掛けたものである．
    pub unit: [f64; 2],
    /// 媒介変数と点の座標の値．
    pub parameters: &'a [f64],
}

impl Area<'_> {
    /// 範囲を，見える範囲で切り取る．重なりがなければ`None`．
    fn clip(&self, domain: Option<[[f64; 2]; 2]>) -> Option<[[f64; 2]; 2]> {
        let [[x0, x1], [y0, y1]] = domain.unwrap_or(self.view);
        let [[vx0, vx1], [vy0, vy1]] = self.view;
        let clipped = [[x0.max(vx0), x1.min(vx1)], [y0.max(vy0), y1.min(vy1)]];
        let [[a, b], [c, d]] = clipped;
        (a < b && c < d).then_some(clipped)
    }

    /// 範囲の画像を作る．
    fn raster(
        &self,
        domain: [[f64; 2]; 2],
        resolution: Option<u32>,
        tikz_resolution: Option<u32>,
        color: &dyn Fn(f64, f64) -> [u8; 4],
    ) -> RasterItem {
        let [[x0, x1], [y0, y1]] = domain;
        raster_item(
            domain,
            [
                [x0 * self.unit[0], y0 * self.unit[1]],
                [x1 * self.unit[0], y1 * self.unit[1]],
            ],
            [
                resolution.unwrap_or(raster_limits::RESOLUTION),
                tikz_resolution.unwrap_or(raster_limits::TIKZ_RESOLUTION),
            ],
            color,
        )
    }

    /// 式に渡す値(変数のあとに，媒介変数と点の座標)．
    fn values(&self, variables: &[Complex]) -> Vec<Complex> {
        variables
            .iter()
            .copied()
            .chain(self.parameters.iter().map(|value| Complex::real(*value)))
            .collect()
    }
}

/// 実数の値．虚部が，実部に比べて丸め誤差の程度でなければ，非数とする．
fn real_value(value: Complex) -> f64 {
    if value.im.abs() <= 1e-12 * value.re.abs().max(1.0) {
        value.re
    } else {
        f64::NAN
    }
}

/// 値を色で表す図の画像．見える範囲と重ならなければ`None`．
#[must_use]
pub fn heatmap_item(heatmap: &Heatmap, plot: &HeatmapPlot, area: &Area) -> Option<RasterItem> {
    let domain = area.clip(plot.domain)?;
    let value = |x: f64, y: f64| {
        let raw = real_value(
            plot.expr
                .eval_complex(&area.values(&[Complex::real(x), Complex::real(y)])),
        );
        match heatmap.scale {
            ValueScale::Linear => raw,
            ValueScale::Log if raw > 0.0 => raw.log10(),
            ValueScale::Log => f64::NAN,
        }
    };
    let [low, high] = match plot.range {
        Some([low, high]) => match heatmap.scale {
            ValueScale::Linear => [low, high],
            ValueScale::Log => [low.log10(), high.log10()],
        },
        None => auto_range(heatmap, domain, &value),
    };
    let color = |x: f64, y: f64| {
        let t = (value(x, y) - low) / (high - low);
        let t = if high > low { t } else { 0.5 };
        colormap(heatmap.colormap, t)
    };
    Some(area.raster(domain, heatmap.resolution, heatmap.tikz_resolution, &color))
}

/// 値の最小と最大．`coolwarm`では，0を中心にした範囲にする．値がなければ`[0, 1]`．
fn auto_range(
    heatmap: &Heatmap,
    domain: [[f64; 2]; 2],
    value: &dyn Fn(f64, f64) -> f64,
) -> [f64; 2] {
    let resolution = heatmap.resolution.unwrap_or(raster_limits::RESOLUTION);
    let [[x0, x1], [y0, y1]] = domain;
    let size = grid_size(x1 - x0, y1 - y0, resolution);
    let (low, high) = cell_centers(domain, size)
        .into_iter()
        .map(|[x, y]| value(x, y))
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), v| {
            (low.min(v), high.max(v))
        });
    if !(low.is_finite() && high.is_finite()) {
        return [0.0, 1.0];
    }
    if heatmap.colormap == Colormap::Coolwarm {
        let bound = low.abs().max(high.abs());
        return [-bound, bound];
    }
    [low, high]
}

/// 複素関数の色塗りの画像．偏角を色相(正の実数が赤，負の実数が青緑)で，絶対値を明るさの縞で表す．
#[must_use]
pub fn domain_coloring_item(
    coloring: &DomainColoring,
    plot: &DomainColoringPlot,
    area: &Area,
) -> Option<RasterItem> {
    let domain = area.clip(plot.domain)?;
    let color = |x: f64, y: f64| {
        let value = plot.expr.eval_complex(&area.values(&[Complex::new(x, y)]));
        if !value.is_finite() {
            return TRANSPARENT;
        }
        let hue = value.arg() / std::f64::consts::TAU;
        let brightness = match coloring.shading {
            Shading::None => 1.0,
            Shading::Modulus => {
                let level = value.norm().log2();
                if level.is_finite() {
                    0.3f64.mul_add(level - level.floor(), 0.7)
                } else {
                    0.7
                }
            }
        };
        hsv(hue, 1.0, brightness)
    };
    Some(area.raster(
        domain,
        coloring.resolution,
        coloring.tikz_resolution,
        &color,
    ))
}
