use anyhow::{Context, Error};
use num_bigint::BigInt;
use std::borrow::Cow;
use std::ops::Deref;

use crate::interned::Interned;
use crate::value::BaseValue;

// Unfinished: using TryFrom to replace the convert.rs types FromArgs

impl TryFrom<BaseValue> for i32 {
    type Error = anyhow::Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value.as_integer().context("value was not an integer type")
    }
}

impl TryFrom<BaseValue> for f64 {
    type Error = anyhow::Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value.as_double().context("value was not a double type")
    }
}

#[derive(Debug, Clone)]
pub enum IntegerLike<BIGINTPTR> {
    Integer(i32),
    BigInteger(BIGINTPTR),
}

impl<BIGINTPTR> TryFrom<BaseValue> for IntegerLike<BIGINTPTR>
where
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
    u64: From<BIGINTPTR>,
{
    type Error = Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_integer()
            .map(Self::Integer)
            .or_else(|| value.as_big_integer::<BIGINTPTR>().map(Self::BigInteger))
            .context("could not resolve `Value` as `Integer`, or `BigInteger`")
    }
}

#[derive(Debug, Clone)]
pub enum DoubleLike<BIGINTPTR> {
    Double(f64),
    Integer(i32),
    BigInteger(BIGINTPTR),
}

impl<BIGINTPTR> TryFrom<BaseValue> for DoubleLike<BIGINTPTR>
where
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
    u64: From<BIGINTPTR>,
{
    type Error = Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_double()
            .map(Self::Double)
            .or_else(|| value.as_integer().map(Self::Integer))
            .or_else(|| value.as_big_integer().map(Self::BigInteger))
            .context("could not resolve `Value` as `Double`, `Integer`, or `BigInteger`")
    }
}

impl<BIGINTPTR> DoubleLike<BIGINTPTR>
where
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
    u64: From<BIGINTPTR>,
{
    #[inline(always)]
    pub fn lt(&self, other: &DoubleLike<BIGINTPTR>) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => a < b,
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a < **b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => a < b,
            (DoubleLike::Integer(a), DoubleLike::Double(b)) => (*a as f64) < *b,
            (DoubleLike::Double(a), DoubleLike::Integer(b)) => *a < (*b as f64),
            (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) => **a < BigInt::from(*b),
            (DoubleLike::Integer(a), DoubleLike::BigInteger(b)) => BigInt::from(*a) < **b,
            _ => {
                panic!("invalid types when comparing two doublelike values");
            }
        }
    }

    #[inline(always)]
    pub fn gt(&self, other: &DoubleLike<BIGINTPTR>) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => a > b,
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a > **b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => a > b,
            (DoubleLike::Integer(a), DoubleLike::Double(b)) => (*a as f64) > *b,
            (DoubleLike::Double(a), DoubleLike::Integer(b)) => *a > (*b as f64),
            (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) => **a > BigInt::from(*b),
            (DoubleLike::Integer(a), DoubleLike::BigInteger(b)) => BigInt::from(*a) > **b,
            _ => {
                panic!("invalid types when comparing two doublelike values");
            }
        }
    }

    #[inline(always)]
    pub fn lt_or_eq(&self, other: &DoubleLike<BIGINTPTR>) -> bool {
        Self::lt(self, other) || Self::eq(self, other)
    }

    #[inline(always)]
    pub fn gt_or_eq(&self, other: &DoubleLike<BIGINTPTR>) -> bool {
        Self::gt(self, other) || Self::eq(self, other)
    }
}

impl<BIGINTPTR> PartialEq for DoubleLike<BIGINTPTR>
where
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => *a == *b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => a == b,
            (DoubleLike::Integer(a), DoubleLike::Double(b)) => (*a as f64) == *b,
            (DoubleLike::Double(a), DoubleLike::Integer(b)) => *a == (*b as f64),
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a == **b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StringLike<SPTR> {
    String(SPTR),
    Symbol(Interned),
    Char(char),
}

impl<SPTR> TryFrom<BaseValue> for StringLike<SPTR>
where
    SPTR: Deref<Target = String> + From<u64> + Into<u64>,
{
    type Error = anyhow::Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_string()
            .map(Self::String)
            .or_else(|| value.as_symbol().map(Self::Symbol))
            .or_else(|| value.as_char().map(Self::Char))
            .context("could not resolve `Value` as `String`, `Symbol` or `Char`")
    }
}

impl<SPTR: Deref<Target = String> + std::fmt::Debug> StringLike<SPTR> {
    pub fn as_str<'a, F>(&'a self, lookup_symbol_fn: F) -> Cow<'a, str>
    where
        F: Fn(Interned) -> &'a str,
    {
        match self {
            StringLike::String(ref value) => Cow::from(value.as_str()),
            StringLike::Symbol(sym) => Cow::from(lookup_symbol_fn(*sym)),
            StringLike::Char(char) => Cow::from(char.to_string()),
        }
    }

    /// I wish this were in an Eq trait, but it needs to lookup symbols.
    /// Is there a way to make this more idiomatic, at least? A better name?
    pub fn eq_stringlike<'a, F>(&'a self, other: &'a Self, lookup_symbol_fn: F) -> bool
    where
        F: Copy + Fn(Interned) -> &'a str,
    {
        match (&self, &other) {
            (StringLike::Char(c1), StringLike::Char(c2)) => *c1 == *c2,
            (StringLike::Char(c1), StringLike::String(s2)) => s2.len() == 1 && *c1 == s2.chars().next().unwrap(),
            (StringLike::String(s1), StringLike::Char(c2)) => s1.len() == 1 && s1.chars().next().unwrap() == *c2,
            (StringLike::Symbol(sym1), StringLike::Symbol(sym2)) => (*sym1 == *sym2) || (lookup_symbol_fn(*sym1).eq(lookup_symbol_fn(*sym2))),
            (StringLike::String(str1), StringLike::String(str2)) => str1.as_str().eq(str2.as_str()),
            _ => {
                let a = self.as_str(lookup_symbol_fn);
                let b = other.as_str(lookup_symbol_fn);
                *a == *b
            }
        }
    }
}
