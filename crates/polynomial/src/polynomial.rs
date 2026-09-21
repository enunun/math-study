//! 多項式．複数の変数を持てる．係数は有理数で，演算があふれたときは，停止せずに誤りを返す．

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use crate::rational::Rational;

/// 結果の項の数の上限．
pub const MAX_TERMS: usize = 5_000;

/// 累乗の指数の上限．
pub const MAX_EXPONENT: u32 = 100;

/// 掛け算で，比べる項の組の数の上限．計算の時間を抑える．
const MAX_PRODUCT_SIZE: usize = 1_000_000;

/// 多項式の演算で起こる誤り．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticError {
    /// 係数や指数が大きすぎて，正確に扱えない．
    Overflow,
    /// 結果の項が多すぎる．
    TooLarge,
}

/// 単項式(係数を除いた，変数の積)．変数の名前の順に，指数が1以上の変数だけを持つ．
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Monomial {
    factors: Vec<(char, u32)>,
}

impl Monomial {
    /// 1(変数を持たない単項式)．
    #[must_use]
    pub fn one() -> Self {
        Self::default()
    }

    /// 1つの変数だけの単項式．
    #[must_use]
    pub fn variable(name: char) -> Self {
        Self {
            factors: vec![(name, 1)],
        }
    }

    /// 変数と指数の組．変数の名前の順である．
    #[must_use]
    pub fn factors(&self) -> &[(char, u32)] {
        &self.factors
    }

    /// 1かどうか．
    #[must_use]
    pub fn is_one(&self) -> bool {
        self.factors.is_empty()
    }

    /// 全次数．
    #[must_use]
    pub fn degree(&self) -> u64 {
        self.factors
            .iter()
            .map(|&(_, exponent)| u64::from(exponent))
            .sum()
    }

    /// 変数の指数．含まないときは0．
    #[must_use]
    pub fn exponent_of(&self, name: char) -> u32 {
        self.factors
            .iter()
            .find(|&&(factor, _)| factor == name)
            .map_or(0, |&(_, exponent)| exponent)
    }

    /// 積．指数があふれるときは，Noneを返す．
    #[must_use]
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        let mut merged: BTreeMap<char, u32> = self.factors.iter().copied().collect();
        for &(name, exponent) in &other.factors {
            let entry = merged.entry(name).or_insert(0);
            *entry = entry.checked_add(exponent)?;
        }
        Some(Self {
            factors: merged.into_iter().collect(),
        })
    }

    /// 変数の指数を1つ減らした単項式と，減らす前の指数を返す．変数を含まないときは，Noneを返す．
    fn lowered(&self, name: char) -> Option<(Self, u32)> {
        let exponent = self.exponent_of(name);
        if exponent == 0 {
            return None;
        }
        let factors = self
            .factors
            .iter()
            .filter_map(|&(factor, power)| {
                if factor == name {
                    power
                        .checked_sub(1)
                        .filter(|&lowered| lowered > 0)
                        .map(|lowered| (factor, lowered))
                } else {
                    Some((factor, power))
                }
            })
            .collect();
        Some((Self { factors }, exponent))
    }
}

/// 表示の順．全次数の大きい順で，同じなら，変数の名前の順に，指数の大きいものを先にする．
fn compare_descending(a: &Monomial, b: &Monomial) -> Ordering {
    b.degree().cmp(&a.degree()).then_with(|| {
        let names: BTreeSet<char> = a
            .factors
            .iter()
            .chain(&b.factors)
            .map(|&(name, _)| name)
            .collect();
        names
            .into_iter()
            .map(|name| b.exponent_of(name).cmp(&a.exponent_of(name)))
            .find(|ordering| ordering.is_ne())
            .unwrap_or(Ordering::Equal)
    })
}

/// 多項式．係数が0の項は持たない．
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Polynomial {
    terms: BTreeMap<Monomial, Rational>,
}

impl Polynomial {
    /// 0．
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }

    /// 定数．
    #[must_use]
    pub fn constant(value: Rational) -> Self {
        let mut polynomial = Self::zero();
        if !value.is_zero() {
            polynomial.terms.insert(Monomial::one(), value);
        }
        polynomial
    }

    /// 1つの変数．
    #[must_use]
    pub fn variable(name: char) -> Self {
        let mut polynomial = Self::zero();
        polynomial
            .terms
            .insert(Monomial::variable(name), Rational::ONE);
        polynomial
    }

    /// 0かどうか．
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    /// 項の数．
    #[must_use]
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// 定数のとき，その値を返す．
    #[must_use]
    pub fn as_constant(&self) -> Option<Rational> {
        let mut terms = self.terms.iter();
        match (terms.next(), terms.next()) {
            (None, _) => Some(Rational::ZERO),
            (Some((monomial, coefficient)), None) if monomial.is_one() => Some(*coefficient),
            _ => None,
        }
    }

    /// 含まれる変数．名前の順である．
    #[must_use]
    pub fn variables(&self) -> Vec<char> {
        let names: BTreeSet<char> = self
            .terms
            .keys()
            .flat_map(|monomial| monomial.factors.iter().map(|&(name, _)| name))
            .collect();
        names.into_iter().collect()
    }

    /// 表示の順に並べた項．全次数の大きい順である．
    #[must_use]
    pub fn descending_terms(&self) -> Vec<(&Monomial, &Rational)> {
        let mut terms: Vec<_> = self.terms.iter().collect();
        terms.sort_by(|(a, _), (b, _)| compare_descending(a, b));
        terms
    }

    /// 項を加える．係数が0になった項は取り除く．
    fn add_term(
        &mut self,
        monomial: Monomial,
        coefficient: Rational,
    ) -> Result<(), ArithmeticError> {
        match self.terms.entry(monomial) {
            Entry::Vacant(entry) => {
                if !coefficient.is_zero() {
                    entry.insert(coefficient);
                }
            }
            Entry::Occupied(mut entry) => {
                let sum = entry
                    .get()
                    .checked_add(&coefficient)
                    .ok_or(ArithmeticError::Overflow)?;
                if sum.is_zero() {
                    entry.remove();
                } else {
                    *entry.get_mut() = sum;
                }
            }
        }
        if self.terms.len() > MAX_TERMS {
            Err(ArithmeticError::TooLarge)
        } else {
            Ok(())
        }
    }

    /// 和．
    ///
    /// # Errors
    ///
    /// 係数があふれるとき，または，項が多すぎるときに，誤りを返す．
    pub fn checked_add(&self, other: &Self) -> Result<Self, ArithmeticError> {
        let mut sum = self.clone();
        for (monomial, coefficient) in &other.terms {
            sum.add_term(monomial.clone(), *coefficient)?;
        }
        Ok(sum)
    }

    /// 符号を変えたもの．
    ///
    /// # Errors
    ///
    /// 係数があふれるときに，誤りを返す．
    pub fn checked_neg(&self) -> Result<Self, ArithmeticError> {
        self.checked_scale(&Rational::from_integer(-1))
    }

    /// 差．
    ///
    /// # Errors
    ///
    /// 係数があふれるとき，または，項が多すぎるときに，誤りを返す．
    pub fn checked_sub(&self, other: &Self) -> Result<Self, ArithmeticError> {
        self.checked_add(&other.checked_neg()?)
    }

    /// 係数を，有理数の倍にしたもの．
    ///
    /// # Errors
    ///
    /// 係数があふれるときに，誤りを返す．
    pub fn checked_scale(&self, factor: &Rational) -> Result<Self, ArithmeticError> {
        let mut scaled = Self::zero();
        for (monomial, coefficient) in &self.terms {
            let product = coefficient
                .checked_mul(factor)
                .ok_or(ArithmeticError::Overflow)?;
            scaled.add_term(monomial.clone(), product)?;
        }
        Ok(scaled)
    }

    /// 積．
    ///
    /// # Errors
    ///
    /// 係数や指数があふれるとき，または，項が多すぎるときに，誤りを返す．
    pub fn checked_mul(&self, other: &Self) -> Result<Self, ArithmeticError> {
        let pairs = self
            .term_count()
            .checked_mul(other.term_count())
            .ok_or(ArithmeticError::TooLarge)?;
        if pairs > MAX_PRODUCT_SIZE {
            return Err(ArithmeticError::TooLarge);
        }
        let mut product = Self::zero();
        for (left_monomial, left_coefficient) in &self.terms {
            for (right_monomial, right_coefficient) in &other.terms {
                let monomial = left_monomial
                    .checked_mul(right_monomial)
                    .ok_or(ArithmeticError::Overflow)?;
                let coefficient = left_coefficient
                    .checked_mul(right_coefficient)
                    .ok_or(ArithmeticError::Overflow)?;
                product.add_term(monomial, coefficient)?;
            }
        }
        Ok(product)
    }

    /// 累乗．繰り返し二乗法で計算する．
    ///
    /// # Errors
    ///
    /// 係数や指数があふれるとき，または，項が多すぎるときに，誤りを返す．
    pub fn checked_pow(&self, exponent: u32) -> Result<Self, ArithmeticError> {
        let mut result = Self::constant(Rational::ONE);
        let mut base = self.clone();
        let mut remaining = exponent;
        while remaining > 0 {
            if remaining & 1 == 1 {
                result = result.checked_mul(&base)?;
            }
            remaining >>= 1;
            if remaining > 0 {
                base = base.checked_mul(&base)?;
            }
        }
        Ok(result)
    }

    /// 変数による偏微分．
    ///
    /// # Errors
    ///
    /// 係数があふれるときに，誤りを返す．
    pub fn derivative(&self, name: char) -> Result<Self, ArithmeticError> {
        let mut derivative = Self::zero();
        for (monomial, coefficient) in &self.terms {
            if let Some((lowered, exponent)) = monomial.lowered(name) {
                let scaled = coefficient
                    .checked_mul(&Rational::from_integer(i128::from(exponent)))
                    .ok_or(ArithmeticError::Overflow)?;
                derivative.add_term(lowered, scaled)?;
            }
        }
        Ok(derivative)
    }
}
