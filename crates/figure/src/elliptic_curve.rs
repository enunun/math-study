//! 楕円曲線`y^2 = x^3 + a x + b`の実の点と，群の演算(和と2倍)の作図．
//!
//! 曲線は，`x^3 + a x + b`の実根で分かれる．実根が3つ(`r1 <= r2 <= r3`)なら，`r1 <= x <= r2`の閉じた
//! 卵形と，`x >= r3`の枝の2つ，1つなら枝だけである．根で縦に接するので，`x`で標本化すると根の近くが
//! 粗くなる．そこで，卵形は`x = (r1 + r2)/2 - (r2 - r1)/2 cos θ`，枝は`x = r3 + t^2`とおき，
//! `y`の符号を`sin θ`や`t`の符号にとる．どちらも，根の近くで滑らかな媒介変数になる．

use crate::error::ErrorKind;
use crate::figure::{DotItem, Item, LabelItem, Stroke};
use crate::placement::Placement;
use crate::scene::{Anchor, Bound, Color, EllipticCurve, Line};

/// 点の印の半径(pt)．点(`point`)の印と同じである．
const DOT_RADIUS: f64 = 2.0;

/// 作図の線の太さ(pt)．
const CONSTRUCTION_WIDTH: f64 = 0.5;

/// 曲線の線の既定の太さ(pt)．グラフや曲線と同じである．
const CURVE_WIDTH: f64 = 0.8;

/// 式を読んだ後の楕円曲線．座標は数学の座標で，変換はまだ施していない．
pub struct EllipticCurvePlot {
    /// 係数`a`．
    pub a: f64,
    /// 係数`b`．
    pub b: f64,
    /// 作図の線．
    pub lines: Vec<ConstructionLine>,
    /// 点の印と名前．
    pub marks: Vec<Mark>,
}

/// 作図の線．
pub enum ConstructionLine {
    /// 点`through`を通り，向き`direction`の直線．見える範囲の端まで描く．
    Infinite {
        /// 通る点．
        through: [f64; 2],
        /// 向き(単位ベクトル)．
        direction: [f64; 2],
    },
    /// 折り返しを示す，縦の線分．破線で描く．
    Reflection {
        /// 一方の端．
        from: [f64; 2],
        /// 他方の端．
        to: [f64; 2],
    },
}

/// 点の印と名前．
pub struct Mark {
    /// 位置．
    pub at: [f64; 2],
    /// 名前の`TeX`の式(`$…$`で囲まない)．
    pub label: String,
}

/// 曲線の上の点．`None`は無限遠点(群の単位元)である．
type CurvePoint = Option<[f64; 2]>;

/// 2つの値が，丸め誤差の程度で等しいか．
fn nearly(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * (1.0 + a.abs().max(b.abs()))
}

/// 曲線の右辺`x^3 + a x + b`．
fn cubic(a: f64, b: f64, x: f64) -> f64 {
    x.mul_add(x * x + a, b)
}

/// `x^3 + a x + b = 0`の実根を，小さい順に返す．重根は重ねて返す(3つか1つ)．
fn real_roots(a: f64, b: f64) -> Vec<f64> {
    let discriminant = -(4.0 * a * a * a + 27.0 * b * b);
    if discriminant >= 0.0 {
        if a == 0.0 {
            return vec![0.0; 3];
        }
        let radius = 2.0 * (-a / 3.0).sqrt();
        let angle = ((3.0 * b / (2.0 * a)) * (-3.0 / a).sqrt())
            .clamp(-1.0, 1.0)
            .acos()
            / 3.0;
        let third = std::f64::consts::TAU / 3.0;
        let mut roots: Vec<f64> = (0..3)
            .map(|k| radius * (angle - third * f64::from(k)).cos())
            .collect();
        roots.sort_by(f64::total_cmp);
        roots
    } else {
        let root = (b * b / 4.0 + a * a * a / 27.0).sqrt();
        vec![(-b / 2.0 + root).cbrt() + (-b / 2.0 - root).cbrt()]
    }
}

/// x座標と枝から，曲線の上の点を求める．曲線の上に点がなければ誤りを返す．
fn point_at(
    a: f64,
    b: f64,
    x: f64,
    lower: bool,
    field: &'static str,
) -> Result<[f64; 2], ErrorKind> {
    let value = cubic(a, b, x);
    let scale = 1.0 + x.abs().powi(3) + (a * x).abs() + b.abs();
    if !x.is_finite() || value < -1e-12 * scale || value.is_nan() {
        return Err(ErrorKind::Invalid(format!(
            "楕円曲線の点(`{field}`)のx座標{x}では，x^3 + a x + b が負なので，曲線の上に点がない．"
        )));
    }
    let y = value.max(0.0).sqrt();
    Ok([x, if lower { -y } else { y }])
}

/// 群の演算の和．
fn add(a: f64, first: CurvePoint, second: CurvePoint) -> CurvePoint {
    let ([x1, y1], [x2, y2]) = match (first, second) {
        (None, other) | (other, None) => return other,
        (Some(p), Some(q)) => (p, q),
    };
    let slope = if nearly(x1, x2) {
        if !nearly(y1, y2) || y1 == 0.0 {
            return None;
        }
        (3.0 * x1 * x1 + a) / (2.0 * y1)
    } else {
        (y2 - y1) / (x2 - x1)
    };
    let x3 = slope * slope - x1 - x2;
    Some([x3, slope * (x1 - x3) - y1])
}

/// 楕円曲線の式を評価し，作図を求める．`evaluate`は，数か式を，項目の名前つきで評価する．
///
/// # Errors
///
/// 係数や点の座標が有限の数でないか，点のx座標で曲線の上に点がなければ，誤りを返す．
pub fn compile_elliptic_curve(
    curve: &EllipticCurve,
    evaluate: &dyn Fn(&'static str, &Bound) -> Result<f64, ErrorKind>,
) -> Result<EllipticCurvePlot, ErrorKind> {
    let finite = |field: &'static str, bound: &Bound| -> Result<f64, ErrorKind> {
        let value = evaluate(field, bound)?;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(ErrorKind::Invalid(format!(
                "楕円曲線の`{field}`は，有限の数にする(今は{value})．"
            )))
        }
    };
    let a = finite("a", &curve.a)?;
    let b = finite("b", &curve.b)?;
    let mut plot = EllipticCurvePlot {
        a,
        b,
        lines: Vec::new(),
        marks: Vec::new(),
    };
    let Some(p_bound) = &curve.p else {
        return Ok(plot);
    };
    let p = point_at(a, b, finite("p", p_bound)?, curve.p_lower, "p")?;
    let q = match &curve.q {
        Some(bound) => Some(point_at(a, b, finite("q", bound)?, curve.q_lower, "q")?),
        None => None,
    };
    let mut mark = |at: [f64; 2], label: &str| {
        plot.marks.push(Mark {
            at,
            label: label.to_owned(),
        });
    };
    if curve.construction {
        mark(p, "P");
        let (other, names) = match q {
            Some(q) => {
                mark(q, "Q");
                (q, ["-(P+Q)", "P+Q"])
            }
            None => (p, ["-2P", "2P"]),
        };
        let sum = add(a, Some(p), Some(other));
        let direction = match sum {
            Some([x3, y3]) => {
                let third = [x3, -y3];
                mark(third, names[0]);
                mark([x3, y3], names[1]);
                let (dx, dy) = if nearly(p[0], other[0]) && nearly(p[1], other[1]) {
                    (1.0, (3.0 * p[0] * p[0] + a) / (2.0 * p[1]))
                } else {
                    (third[0] - p[0], third[1] - p[1])
                };
                if !nearly(y3, 0.0) {
                    plot.lines.push(ConstructionLine::Reflection {
                        from: third,
                        to: [x3, y3],
                    });
                }
                let length = dx.hypot(dy);
                [dx / length, dy / length]
            }
            // 和が無限遠点になるのは，直線が縦のときである．
            None => [0.0, 1.0],
        };
        plot.lines.insert(
            0,
            ConstructionLine::Infinite {
                through: p,
                direction,
            },
        );
    }
    if let Some(count) = curve.multiples {
        let mut current = Some(p);
        for k in 1..=count {
            if k > 1 {
                current = add(a, current, Some(p));
            }
            let Some(at) = current else { continue };
            let name = if k == 1 {
                "P".to_owned()
            } else {
                format!("{k}P")
            };
            // 作図で置いた点には，重ねて置かない．
            if !plot.marks.iter().any(|placed| placed.label == name) {
                plot.marks.push(Mark { at, label: name });
            }
        }
    }
    Ok(plot)
}

/// 楕円曲線を描く要素．曲線，作図の線，点の印と名前の順に並べる．
#[must_use]
pub fn elliptic_curve_items(
    curve: &EllipticCurve,
    plot: &EllipticCurvePlot,
    placement: &Placement,
) -> Vec<Item> {
    let mut items = curve_items(curve, plot, placement);
    items.extend(construction_items(curve, plot, placement));
    items.extend(mark_items(curve, plot, placement));
    items
}

/// 曲線そのもの．卵形(実根が3つのとき)と枝．
fn curve_items(
    curve: &EllipticCurve,
    plot: &EllipticCurvePlot,
    placement: &Placement,
) -> Vec<Item> {
    let stroke = Stroke {
        line: curve.style.line.unwrap_or(Line::Solid),
        width: curve.style.width_pt().unwrap_or(CURVE_WIDTH),
        color: curve.style.color,
    };
    let (a, b) = (plot.a, plot.b);
    let y_at = |x: f64| cubic(a, b, x).max(0.0).sqrt();
    let roots = real_roots(a, b);
    let mut items = Vec::new();
    if let [r1, r2, _] = roots.as_slice()
        && r2 - r1 > 1e-12 * (1.0 + r1.abs())
    {
        let (middle, half) = (f64::midpoint(*r1, *r2), (r2 - r1) / 2.0);
        items.extend(placement.curve(
            |theta| {
                let x = middle - half * theta.cos();
                let y = y_at(x);
                [x, if theta > std::f64::consts::PI { -y } else { y }]
            },
            [0.0, std::f64::consts::TAU],
            stroke,
        ));
    }
    if let Some(&start) = roots.last() {
        let far = start.abs() + 4.0 * placement.extent + 1.0;
        let reach = (far - start).max(0.0).sqrt();
        items.extend(placement.curve(
            |t| {
                let x = t.mul_add(t, start);
                [x, y_at(x).copysign(t)]
            },
            [-reach, reach],
            stroke,
        ));
    }
    items
}

/// 作図の線．直線は細い実線，折り返しは細い破線で，色は既定では灰色である．
fn construction_items(
    curve: &EllipticCurve,
    plot: &EllipticCurvePlot,
    placement: &Placement,
) -> Vec<Item> {
    let stroke = |line| Stroke {
        line,
        width: CONSTRUCTION_WIDTH,
        color: Some(curve.style.color.unwrap_or(Color::Gray)),
    };
    plot.lines
        .iter()
        .flat_map(|line| match line {
            ConstructionLine::Infinite { through, direction } => {
                let reach = 4.0 * placement.extent + through[0].hypot(through[1]);
                placement.curve(
                    |t| {
                        [
                            t.mul_add(direction[0], through[0]),
                            t.mul_add(direction[1], through[1]),
                        ]
                    },
                    [-reach, reach],
                    stroke(Line::Solid),
                )
            }
            ConstructionLine::Reflection { from, to } => placement.curve(
                |t| [from[0], t.mul_add(to[1] - from[1], from[1])],
                [0.0, 1.0],
                stroke(Line::Dashed),
            ),
        })
        .collect()
}

/// 点の印と名前．見える範囲の外の点は描かない．
fn mark_items(curve: &EllipticCurve, plot: &EllipticCurvePlot, placement: &Placement) -> Vec<Item> {
    let mut items = Vec::new();
    for mark in &plot.marks {
        let Some(at) = placement.place(mark.at).filter(|at| placement.inside(*at)) else {
            continue;
        };
        items.push(Item::Dot(DotItem {
            at,
            radius: DOT_RADIUS,
            color: curve.style.color,
        }));
        if curve.labels {
            items.push(Item::Label(LabelItem {
                at,
                anchor: Anchor::SouthWest,
                tex: format!("${}$", mark.label),
            }));
        }
    }
    items
}
