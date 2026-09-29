//! 結晶の回折図形(`diffraction`)．
//!
//! 逆格子の基本ベクトルは`a_i · b_j = δ_ij`で決める(`2π`を掛けない，結晶学の約束)．逆格子点
//! `G = Σ h_i b_i`の構造因子は，原子の分率座標`x_j`で`F = Σ_j f_j exp(-2πi h · x_j)`である．
//! 3次元の結晶は，晶帯軸`t = Σ w_i a_i`に垂直な平面の上の逆格子点(`h · w = 0`，ゼロ次のLaueゾーン)を，
//! その平面の正規直交な軸`e1`，`e2`の座標で描く．有限の結晶(各向きに`N_i`個の単位胞)の強さは，
//! `|F(q)|^2 Π sin^2(π N_i q·a_i) / sin^2(π q·a_i)`である．

// 複素数の四則演算(`Complex`の`Add`など)は浮動小数点の演算で，あふれて止まることはない．
#![allow(clippy::arithmetic_side_effects)]

use crate::compile::{Env, evaluate_bound};
use crate::error::ErrorKind;
use crate::expr::{Complex, Expr};
use crate::figure::{DotItem, Item, LabelItem, RasterItem};
use crate::heatmap::{Area, ColorSettings, value_raster};
use crate::placement::Placement;
use crate::raster::{cell_centers, f64_to_count};
use crate::scene::{Anchor, Bound, CM_PER_PT, Diffraction};

/// 点の半径の既定(pt)．
const SPOT_RADIUS: f64 = 3.0;
/// 描く点の強さの下限(最大に対する比)．これより弱い点(消滅則で消える点)は描かない．
const MIN_INTENSITY: f64 = 1e-10;
/// 平行かどうかを判定する，行列式の相対的な大きさ．
const PARALLEL_EPSILON: f64 = 1e-9;
/// 散乱因子の式の変数(逆格子ベクトルの長さ)．
const FACTOR_VAR: &str = "q";

type Vector3 = [f64; 3];

fn dot(a: Vector3, b: Vector3) -> f64 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: Vector3, b: Vector3) -> Vector3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn scale(a: Vector3, factor: f64) -> Vector3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn add(a: Vector3, b: Vector3) -> Vector3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn norm(a: Vector3) -> f64 {
    dot(a, a).sqrt()
}

/// 原子の散乱因子．数か，`q`の式．
enum Factor {
    Constant(f64),
    Expression(Expr),
}

/// 原子．
struct AtomPlot {
    /// 分率座標(3個にそろえる)．
    position: Vector3,
    /// 散乱因子．
    factor: Factor,
}

/// 逆格子点の点．
pub struct Spot {
    /// 平面の座標．
    pub at: [f64; 2],
    /// 構造因子の絶対値．
    pub amplitude: f64,
    /// 指数．
    pub indices: Vec<i32>,
}

/// 式を読んだ後の回折図形．
pub struct DiffractionPlot {
    /// 点．強さが下限より弱い点は除いてある．
    pub spots: Vec<Spot>,
    /// 有限の結晶の強さを求めるための値．`cells`がなければ`None`．
    finite: Option<Finite>,
}

/// 有限の結晶の強さを求めるための値．
struct Finite {
    /// 実格子の基本ベクトル(3個の成分にそろえる)．
    basis: Vec<Vector3>,
    /// 原子．
    atoms: Vec<AtomPlot>,
    /// 平面の軸．
    axes: [Vector3; 2],
    /// 各向きの単位胞の数．
    cells: Vec<f64>,
    /// 媒介変数と点の座標の値．
    parameters: Vec<f64>,
    /// 色の両端の値．
    range: Option<[f64; 2]>,
}

/// 3行3列の行列式．
fn determinant([a, b, c]: [Vector3; 3]) -> f64 {
    dot(a, cross(b, c))
}

/// 逆格子の基本ベクトル`b_j`(`a_i · b_j = δ_ij`)．
fn reciprocal(basis: &[Vector3; 3]) -> [Vector3; 3] {
    let volume = determinant(*basis);
    let [a1, a2, a3] = *basis;
    [
        scale(cross(a2, a3), 1.0 / volume),
        scale(cross(a3, a1), 1.0 / volume),
        scale(cross(a1, a2), 1.0 / volume),
    ]
}

/// 基本ベクトルを評価し，3個の成分にそろえる．平面の格子の3つ目は`(0, 0, 1)`にする．
fn evaluate_basis(basis: &[Vec<Bound>], env: &Env) -> Result<(usize, [Vector3; 3]), ErrorKind> {
    let dimension = basis.len();
    let mut vectors = [[0.0, 0.0, 1.0]; 3];
    for (slot, vector) in vectors.iter_mut().zip(basis) {
        let mut value = [0.0; 3];
        for (index, (component, bound)) in value.iter_mut().zip(vector).enumerate() {
            *component = evaluate_bound("basis", bound, index, env)?;
        }
        if value.iter().any(|component| !component.is_finite()) {
            return Err(ErrorKind::Invalid(
                "回折図形の基本ベクトル(`basis`)は，有限の数にする．".to_owned(),
            ));
        }
        *slot = value;
    }
    let size: f64 = vectors
        .iter()
        .take(dimension)
        .map(|vector| norm(*vector))
        .product();
    if determinant(vectors).abs() <= PARALLEL_EPSILON * size {
        return Err(ErrorKind::Invalid(
            "回折図形の基本ベクトル(`basis`)は，平行でない(同じ平面にない)ベクトルにする．"
                .to_owned(),
        ));
    }
    Ok((dimension, vectors))
}

/// 原子を評価する．なければ，原点に散乱因子1の原子を1つ置く．
fn evaluate_atoms(diffraction: &Diffraction, env: &Env) -> Result<Vec<AtomPlot>, ErrorKind> {
    if diffraction.atoms.is_empty() {
        return Ok(vec![AtomPlot {
            position: [0.0; 3],
            factor: Factor::Constant(1.0),
        }]);
    }
    if env.names.contains(&FACTOR_VAR) {
        return Err(ErrorKind::NameConflict(FACTOR_VAR.to_owned()));
    }
    let names: Vec<&str> = std::iter::once(FACTOR_VAR)
        .chain(env.names.iter().copied())
        .collect();
    diffraction
        .atoms
        .iter()
        .map(|atom| {
            let mut position = [0.0; 3];
            for (index, (coordinate, bound)) in position.iter_mut().zip(&atom.position).enumerate()
            {
                *coordinate = evaluate_bound("position", bound, index, env)?;
            }
            let factor = match &atom.factor {
                None => Factor::Constant(1.0),
                Some(Bound::Number(value)) => Factor::Constant(*value),
                Some(Bound::Expression(source)) => {
                    Factor::Expression(Expr::compile_with(source, &names, env.functions).map_err(
                        |error| ErrorKind::Expression {
                            field: "factor",
                            index: 0,
                            error,
                        },
                    )?)
                }
            };
            Ok(AtomPlot { position, factor })
        })
        .collect()
}

impl AtomPlot {
    /// 逆格子ベクトルの長さ`q`での散乱因子．
    fn factor_at(&self, q: f64, parameters: &[f64]) -> f64 {
        match &self.factor {
            Factor::Constant(value) => *value,
            Factor::Expression(expr) => {
                let values: Vec<f64> = std::iter::once(q)
                    .chain(parameters.iter().copied())
                    .collect();
                expr.eval(&values)
            }
        }
    }
}

/// 構造因子．`phase`は，原子ごとの`h · x_j`(分率座標での内積)を返す．
fn structure_factor(
    atoms: &[AtomPlot],
    q: f64,
    parameters: &[f64],
    phase: &dyn Fn(Vector3) -> f64,
) -> Complex {
    atoms.iter().fold(Complex::real(0.0), |total, atom| {
        let (sin, cos) = (std::f64::consts::TAU * phase(atom.position)).sin_cos();
        total + Complex::new(cos, -sin) * Complex::real(atom.factor_at(q, parameters))
    })
}

/// 平面の軸．平面の格子では`x`，`y`の向き．3次元の結晶では，晶帯軸に垂直な平面の中で，晶帯軸といちばん
/// 平行でない逆格子の基本ベクトルの向きを`e1`，晶帯軸と`e1`の外積を`e2`とする．
fn plane_axes(
    dimension: usize,
    basis: &[Vector3; 3],
    reciprocal: &[Vector3; 3],
    zone: [i32; 3],
) -> [Vector3; 2] {
    if dimension == 2 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    }
    let axis = basis
        .iter()
        .zip(zone)
        .fold([0.0; 3], |total, (vector, weight)| {
            add(total, scale(*vector, f64::from(weight)))
        });
    let axis = scale(axis, 1.0 / norm(axis));
    let (_, chosen) = reciprocal
        .iter()
        .map(|vector| (norm(cross(*vector, axis)) / norm(*vector), *vector))
        .fold((f64::NEG_INFINITY, [1.0, 0.0, 0.0]), |best, candidate| {
            if candidate.0 > best.0 {
                candidate
            } else {
                best
            }
        });
    let in_plane = add(chosen, scale(axis, -dot(chosen, axis)));
    let first = scale(in_plane, 1.0 / norm(in_plane));
    [first, cross(axis, first)]
}

/// 回折図形の逆格子点と構造因子を求める．
///
/// # Errors
///
/// 基本ベクトル，原子，晶帯軸，半径に誤りがあると，誤りを返す．
pub fn compile_diffraction(
    diffraction: &Diffraction,
    env: &Env,
    view: [[f64; 2]; 2],
) -> Result<DiffractionPlot, ErrorKind> {
    let (dimension, basis) = evaluate_basis(&diffraction.basis, env)?;
    let atoms = evaluate_atoms(diffraction, env)?;
    let reciprocal = reciprocal(&basis);
    let zone = diffraction.zone.unwrap_or([0, 0, 1]);
    let axes = plane_axes(dimension, &basis, &reciprocal, zone);
    let [[x0, x1], [y0, y1]] = view;
    let reach = match &diffraction.radius {
        Some(bound) => evaluate_bound("radius", bound, 0, env)?,
        None => x0.abs().max(x1.abs()).hypot(y0.abs().max(y1.abs())),
    };
    if !(reach.is_finite() && reach >= 0.0) {
        return Err(ErrorKind::Invalid(
            "回折図形の半径(`radius`)は，0以上の有限の数にする．".to_owned(),
        ));
    }
    let limits: Vec<i32> = basis
        .iter()
        .take(dimension)
        .map(|vector| {
            i32::try_from(f64_to_count((reach * norm(*vector)).floor().min(1000.0))).unwrap_or(0)
        })
        .collect();
    let mut spots = Vec::new();
    for indices in index_tuples(&limits) {
        if dimension == 3 && indices.iter().zip(zone).map(|(h, w)| h * w).sum::<i32>() != 0 {
            continue;
        }
        let g = indices
            .iter()
            .zip(&reciprocal)
            .fold([0.0; 3], |total, (h, b)| {
                add(total, scale(*b, f64::from(*h)))
            });
        let length = norm(g);
        if length > reach * (1.0 + 1e-12) {
            continue;
        }
        let factor = structure_factor(&atoms, length, env.parameters, &|position| {
            indices
                .iter()
                .zip(position)
                .map(|(h, x)| f64::from(*h) * x)
                .sum()
        });
        spots.push(Spot {
            at: [dot(g, axes[0]), dot(g, axes[1])],
            amplitude: factor.norm(),
            indices,
        });
    }
    let strongest = spots.iter().map(|spot| spot.amplitude).fold(0.0, f64::max);
    spots.retain(|spot| spot.amplitude * spot.amplitude > MIN_INTENSITY * strongest * strongest);
    let finite = match &diffraction.cells {
        None => None,
        Some(cells) => Some(Finite {
            basis: basis.iter().take(dimension).copied().collect(),
            atoms,
            axes,
            cells: evaluate_cells(cells, env)?,
            parameters: env.parameters.to_vec(),
            range: diffraction
                .range
                .as_ref()
                .map(|range| crate::compile::evaluate_domain(range, env))
                .transpose()
                .map_err(|_| ErrorKind::InvalidRange("range"))?,
        }),
    };
    Ok(DiffractionPlot { spots, finite })
}

/// 単位胞の数を評価し，整数に丸める．1以上，上限以下である．
fn evaluate_cells(cells: &[Bound], env: &Env) -> Result<Vec<f64>, ErrorKind> {
    cells
        .iter()
        .enumerate()
        .map(|(index, bound)| {
            let count = evaluate_bound("cells", bound, index, env)?.round();
            if (1.0..=Diffraction::MAX_CELLS).contains(&count) {
                Ok(count)
            } else {
                Err(ErrorKind::Invalid(format!(
                    "単位胞の数(`cells`)は，1以上{}以下にする(今は{count})．",
                    Diffraction::MAX_CELLS
                )))
            }
        })
        .collect()
}

/// 各指数が`-limit`から`limit`の，指数の組の全部．
fn index_tuples(limits: &[i32]) -> Vec<Vec<i32>> {
    limits.iter().fold(vec![Vec::new()], |tuples, limit| {
        tuples
            .iter()
            .flat_map(|prefix| {
                (-limit..=*limit).map(move |index| {
                    let mut tuple = prefix.clone();
                    tuple.push(index);
                    tuple
                })
            })
            .collect()
    })
}

/// 指数の名前．負の指数には上線を付ける．2桁以上の指数があれば，指数の間に細い空白を入れる．
fn index_label(indices: &[i32]) -> String {
    let wide = indices.iter().any(|index| index.unsigned_abs() >= 10);
    let parts: Vec<String> = indices
        .iter()
        .map(|index| {
            if *index < 0 {
                format!("\\bar{{{}}}", index.unsigned_abs())
            } else {
                index.to_string()
            }
        })
        .collect();
    format!("$({})$", parts.join(if wide { "\\," } else { "" }))
}

/// 回折図形の点と名前．
#[must_use]
pub fn diffraction_items(
    diffraction: &Diffraction,
    plot: &DiffractionPlot,
    placement: &Placement,
) -> Vec<Item> {
    let strongest = plot
        .spots
        .iter()
        .map(|spot| spot.amplitude)
        .fold(0.0, f64::max);
    let full = diffraction
        .spot
        .map_or(SPOT_RADIUS, |length| length.to_cm() / CM_PER_PT);
    let mut items = Vec::new();
    for spot in &plot.spots {
        let Some(at) = placement.place(spot.at).filter(|at| placement.inside(*at)) else {
            continue;
        };
        items.push(Item::Dot(DotItem {
            at,
            radius: full * spot.amplitude / strongest,
            color: diffraction.style.color,
        }));
        if diffraction.labels {
            items.push(Item::Label(LabelItem {
                at,
                anchor: Anchor::North,
                tex: index_label(&spot.indices),
            }));
        }
    }
    items
}

/// `sin^2(π N φ) / sin^2(π φ)`．`φ`が整数に近いときは，極限の`N^2`．
fn laue(count: f64, phase: f64) -> f64 {
    let denominator = (std::f64::consts::PI * phase).sin();
    if denominator.abs() < 1e-12 {
        return count * count;
    }
    let numerator = (std::f64::consts::PI * count * phase).sin();
    (numerator * numerator) / (denominator * denominator)
}

impl Finite {
    /// 平面の座標`(x, y)`での強さ．
    fn intensity(&self, x: f64, y: f64) -> f64 {
        let q = add(scale(self.axes[0], x), scale(self.axes[1], y));
        let phases: Vec<f64> = self.basis.iter().map(|vector| dot(q, *vector)).collect();
        let factor = structure_factor(&self.atoms, norm(q), &self.parameters, &|position| {
            phases
                .iter()
                .zip(position)
                .map(|(phase, coordinate)| phase * coordinate)
                .sum()
        });
        let lattice: f64 = self
            .cells
            .iter()
            .zip(&phases)
            .map(|(count, phase)| laue(*count, *phase))
            .product();
        factor.norm() * factor.norm() * lattice
    }
}

/// 有限の結晶の強さの画像．`cells`がないか，見える範囲と重ならなければ`None`．
#[must_use]
pub fn diffraction_raster(
    diffraction: &Diffraction,
    plot: &DiffractionPlot,
    area: &Area,
) -> Option<RasterItem> {
    let finite = plot.finite.as_ref()?;
    let domain = area.clip(None)?;
    let settings = ColorSettings {
        colormap: diffraction.colormap,
        scale: diffraction.scale,
        range: finite.range,
        normalize: true,
        resolution: diffraction.resolution,
        tikz_resolution: diffraction.tikz_resolution,
    };
    let values = |domain: [[f64; 2]; 2], size: (usize, usize)| -> Vec<f64> {
        cell_centers(domain, size)
            .into_iter()
            .map(|[x, y]| finite.intensity(x, y))
            .collect()
    };
    Some(value_raster(area, domain, &settings, &values))
}
