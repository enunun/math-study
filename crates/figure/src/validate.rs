//! 読み込んだシーンの検査．型では表せない条件を確かめる．

use std::collections::HashSet;

use crate::error::{Error, ErrorKind};
use crate::scene::{Axis, Bound, Curve, Graph, Label, Object, Scene};

/// 曲線の式の数．平面の曲線は，2個である．
const CURVE_EXPRESSIONS: usize = 2;

/// シーンを検査する．
///
/// # Errors
///
/// 図の説明が空，範囲が正しくない，`id`が識別子でないか重なっている，
/// 式や文字列が空，曲線の式の数が合わない，のいずれかで誤りを返す．
pub fn validate(scene: &Scene) -> Result<(), Error> {
    if is_blank(&scene.description) {
        return Err(Error::new(ErrorKind::EmptyText("description")));
    }
    check_view_range("view.x", scene.view.x)?;
    check_view_range("view.y", scene.view.y)?;
    let mut seen = HashSet::new();
    for object in &scene.objects {
        let id = object.id();
        if !is_identifier(id) {
            return Err(Error::in_object(id, ErrorKind::InvalidId(id.to_owned())));
        }
        if !seen.insert(id) {
            return Err(Error::in_object(id, ErrorKind::DuplicateId(id.to_owned())));
        }
        validate_object(object).map_err(|kind| Error::in_object(id, kind))?;
    }
    Ok(())
}

fn validate_object(object: &Object) -> Result<(), ErrorKind> {
    match object {
        Object::Axis(axis) => validate_axis(axis),
        Object::Label(label) => validate_label(label),
        Object::Graph(graph) => validate_graph(graph),
        Object::Curve(curve) => validate_curve(curve),
        Object::Parameter(_) => Ok(()),
    }
}

fn validate_axis(axis: &Axis) -> Result<(), ErrorKind> {
    match axis.range {
        Some(range) if !is_increasing(range) => Err(ErrorKind::InvalidRange("range")),
        _ => Ok(()),
    }
}

fn validate_label(label: &Label) -> Result<(), ErrorKind> {
    non_empty("tex", &label.tex)
}

fn validate_graph(graph: &Graph) -> Result<(), ErrorKind> {
    check_variable(&graph.var)?;
    non_empty("expr", &graph.expr)?;
    check_domain(&graph.domain)
}

fn validate_curve(curve: &Curve) -> Result<(), ErrorKind> {
    check_variable(&curve.var)?;
    if curve.expr.len() != CURVE_EXPRESSIONS {
        return Err(ErrorKind::ExpressionCount {
            expected: CURVE_EXPRESSIONS,
            found: curve.expr.len(),
        });
    }
    for expr in &curve.expr {
        non_empty("expr", expr)?;
    }
    check_domain(&curve.domain)
}

fn check_view_range(field: &'static str, range: [f64; 2]) -> Result<(), Error> {
    if is_increasing(range) {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidRange(field)))
    }
}

/// 数で書かれた定義域だけを確かめる．式で書かれた端は，式を評価する段階で確かめる．
fn check_domain(domain: &[Bound; 2]) -> Result<(), ErrorKind> {
    match domain {
        [Bound::Number(low), Bound::Number(high)] if !is_increasing([*low, *high]) => {
            Err(ErrorKind::InvalidRange("domain"))
        }
        _ => Ok(()),
    }
}

fn check_variable(name: &str) -> Result<(), ErrorKind> {
    if is_identifier(name) {
        Ok(())
    } else {
        Err(ErrorKind::InvalidVariable(name.to_owned()))
    }
}

fn non_empty(field: &'static str, text: &str) -> Result<(), ErrorKind> {
    if is_blank(text) {
        Err(ErrorKind::EmptyText(field))
    } else {
        Ok(())
    }
}

fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn is_increasing([low, high]: [f64; 2]) -> bool {
    low.is_finite() && high.is_finite() && low < high
}

/// 英字で始まり，英数字と`_`だけでできた名前．式の中で，そのまま使える．
fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
