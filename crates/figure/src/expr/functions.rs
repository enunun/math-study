//! 利用者が定義した関数．呼び出しは，式を読むときに，関数の本体へ引数を埋め込んで展開する．
//!
//! 展開してしまうので，評価，点の式の検査，テイラー展開は，組み込みの関数だけの式と同じに扱える．
//! 関数の本体は，先に定義した関数を呼べる(合成)が，自分自身やあとの関数は呼べないので，再帰は起こらない．

use super::{ExprError, ExprErrorKind, Node, Span, parse_source};

/// 展開した構文木の節の数の上限．引数を何度も使う関数を重ねると，木が指数的に大きくなるのを防ぐ．
const MAX_NODES: usize = 20_000;

/// 利用者が定義した関数の集まり．定義した順に並ぶ．
#[derive(Debug, Clone, Default)]
pub struct Functions {
    entries: Vec<UserFunction>,
}

/// 利用者が定義した関数．
#[derive(Debug, Clone)]
struct UserFunction {
    name: String,
    /// 引数の数．
    arity: usize,
    /// 本体の構文木．変数の番号は，引数(0から`arity - 1`)，定義のときに使えた名前(`globals`)の順である．
    body: Node,
    /// 定義のときに使えた名前(媒介変数など)．呼ぶ側の名前の並びでの番号に，付け替える．
    globals: Vec<String>,
}

impl Functions {
    /// 関数を定義する．本体`source`は，引数`vars`と，名前`names`(媒介変数など)と，先に定義した関数を使える．
    ///
    /// # Errors
    ///
    /// 本体の式の誤りがあると，誤りを返す．
    pub fn define(
        &mut self,
        name: &str,
        vars: &[&str],
        source: &str,
        names: &[&str],
    ) -> Result<(), ExprError> {
        let scope: Vec<&str> = vars.iter().chain(names).copied().collect();
        let body = parse_source(source, &scope, self)?;
        self.entries.push(UserFunction {
            name: name.to_owned(),
            arity: vars.len(),
            body,
            globals: names.iter().map(|&name| name.to_owned()).collect(),
        });
        Ok(())
    }

    /// その名前の関数が定義されているか．
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }

    /// 関数の呼び出しを展開する．`names`は，呼ぶ側の式の名前の並びである．
    pub(super) fn expand(
        &self,
        name: &str,
        arguments: &[Node],
        names: &[&str],
        span: &Span,
    ) -> Option<Result<Node, ExprError>> {
        let entry = self.entries.iter().find(|entry| entry.name == name)?;
        let fail = |kind| ExprError {
            kind,
            span: span.clone(),
        };
        if arguments.len() != entry.arity {
            return Some(Err(fail(ExprErrorKind::ArgumentCount {
                name: name.to_owned(),
                expected: entry.arity,
                found: arguments.len(),
            })));
        }
        let expanded = substitute(&entry.body, arguments, &entry.globals, names).map_err(fail);
        Some(expanded.and_then(|node| {
            if node.size() > MAX_NODES {
                Err(fail(ExprErrorKind::TooLong))
            } else {
                Ok(node)
            }
        }))
    }
}

/// 本体の変数を，引数と，呼ぶ側の名前の番号に置き換える．
fn substitute(
    node: &Node,
    arguments: &[Node],
    globals: &[String],
    names: &[&str],
) -> Result<Node, ExprErrorKind> {
    let recurse = |child: &Node| substitute(child, arguments, globals, names).map(Box::new);
    Ok(match node {
        Node::Number(value) => Node::Number(*value),
        Node::Variable(index) => {
            if let Some(argument) = arguments.get(*index) {
                argument.clone()
            } else {
                let global = index
                    .checked_sub(arguments.len())
                    .and_then(|at| globals.get(at))
                    .ok_or_else(|| ExprErrorKind::UnknownName(index.to_string()))?;
                let position = names
                    .iter()
                    .position(|candidate| candidate == global)
                    .ok_or_else(|| ExprErrorKind::UnknownName(global.clone()))?;
                Node::Variable(position)
            }
        }
        Node::Neg(operand) => Node::Neg(recurse(operand)?),
        Node::Binary(op, left, right) => Node::Binary(*op, recurse(left)?, recurse(right)?),
        Node::Call(function, argument) => Node::Call(*function, recurse(argument)?),
    })
}

impl Node {
    /// 構文木の節の数．
    fn size(&self) -> usize {
        match self {
            Self::Number(_) | Self::Variable(_) => 1,
            Self::Neg(operand) | Self::Call(_, operand) => operand.size().saturating_add(1),
            Self::Binary(_, left, right) => {
                left.size().saturating_add(right.size()).saturating_add(1)
            }
        }
    }
}
