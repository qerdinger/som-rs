use anyhow::{Context, Error};
use num_bigint::BigInt;
use std::borrow::Cow;
use std::ops::Deref;

use crate::interned::Interned;

#[cfg(feature = "nan")]
use crate::nan::value::BaseValue;

#[cfg(feature = "lbits")]
use crate::lbits::value::BaseValue;

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
pub enum DoubleLike<DOUBLEPTR, BIGINTPTR> {
    Double(f64),
    Integer(i32),
    BigInteger(BIGINTPTR),
    #[doc(hidden)]
    __Phantom(std::marker::PhantomData<DOUBLEPTR>),
}

#[cfg(feature = "lbits")]
impl<DOUBLEPTR, BIGINTPTR> TryFrom<BaseValue> for DoubleLike<DOUBLEPTR, BIGINTPTR>
where
    DOUBLEPTR: Deref<Target = f64> + From<u64> + Into<u64>,
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
    u64: From<BIGINTPTR>,
{
    type Error = Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_double()
            .map(Self::Double)
            .or_else(|| value.as_integer().map(Self::Integer))
            // .or_else(|| value.as_allocated_double().map(|v: DOUBLEPTR | Self::Double(*v)))
            .or_else(|| value.as_big_integer().map(Self::BigInteger))
            .or_else(|| {
                value.as_allocated_double().map(|v: DOUBLEPTR| {
                    Self::Double(*v)
                })
            })
            .context("could not resolve `Value` as `Double`, `Integer`, or `BigInteger`")
    }
    // .or_else(|| value.as_allocated_double().map(Self::AllocatedDouble))
}

#[cfg(feature = "nan")]
impl<DOUBLEPTR, BIGINTPTR> TryFrom<BaseValue> for DoubleLike<DOUBLEPTR, BIGINTPTR>
where
    DOUBLEPTR: Deref<Target = f64> + From<u64> + Into<u64>,
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
    u64: From<BIGINTPTR>,
{
    type Error = Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_double()
            .map(Self::Double)
            .or_else(|| value.as_integer().map(Self::Integer))
            // .or_else(|| value.as_allocated_double().map(|v: DOUBLEPTR | Self::Double(*v)))
            .or_else(|| value.as_big_integer().map(Self::BigInteger))
            .context("could not resolve `Value` as `Double`, `Integer`, or `BigInteger`")
    }
    // .or_else(|| value.as_allocated_double().map(Self::AllocatedDouble))
}

impl<DOUBLEPTR, BIGINTPTR> DoubleLike<DOUBLEPTR, BIGINTPTR>
where
    DOUBLEPTR: Deref<Target = f64> + From<u64> + Into<u64>,
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
    u64: From<BIGINTPTR>,
{
    #[inline(always)]
    pub fn lt(&self, other: &DoubleLike<DOUBLEPTR, BIGINTPTR>) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => a < b,
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a < **b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => a < b,
            // (DoubleLike::AllocatedDouble(a), DoubleLike::AllocatedDouble(b)) => **a < **b,
            // (DoubleLike::AllocatedDouble(a), DoubleLike::Double(b)) => **a < *b,
            // (DoubleLike::Double(a), DoubleLike::AllocatedDouble(b)) => *a < **b,
            // (DoubleLike::AllocatedDouble(a), DoubleLike::Integer(b)) => **a < (*b as f64),
            // (DoubleLike::Integer(a), DoubleLike::AllocatedDouble(b)) => (*a as f64) < **b,
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
    pub fn gt(&self, other: &DoubleLike<DOUBLEPTR, BIGINTPTR>) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => a > b,
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a > **b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => a > b,
            (DoubleLike::Integer(a), DoubleLike::Double(b)) => (*a as f64) > *b,
            (DoubleLike::Double(a), DoubleLike::Integer(b)) => *a > (*b as f64),
            // (DoubleLike::AllocatedDouble(a), DoubleLike::AllocatedDouble(b)) => **a > **b,
            // (DoubleLike::AllocatedDouble(a), DoubleLike::Double(b)) => **a > *b,
            // (DoubleLike::Double(a), DoubleLike::AllocatedDouble(b)) => *a > **b,
            // (DoubleLike::AllocatedDouble(a), DoubleLike::Integer(b)) => **a > (*b as f64),
            // (DoubleLike::Integer(a), DoubleLike::AllocatedDouble(b)) => (*a as f64) > **b,
            (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) => **a > BigInt::from(*b),
            (DoubleLike::Integer(a), DoubleLike::BigInteger(b)) => BigInt::from(*a) > **b,
            _ => {
                panic!("invalid types when comparing two doublelike values");
            }
        }
    }

    #[inline(always)]
    pub fn lt_or_eq(&self, other: &DoubleLike<DOUBLEPTR, BIGINTPTR>) -> bool {
        Self::lt(self, other) || Self::eq(self, other)
    }

    #[inline(always)]
    pub fn gt_or_eq(&self, other: &DoubleLike<DOUBLEPTR, BIGINTPTR>) -> bool {
        Self::gt(self, other) || Self::eq(self, other)
    }
}

impl<DOUBLEPTR, BIGINTPTR> PartialEq for DoubleLike<DOUBLEPTR, BIGINTPTR>
where
    DOUBLEPTR: Deref<Target = f64> + From<u64> + Into<u64>,
    BIGINTPTR: Deref<Target = BigInt> + From<u64> + Into<u64>,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => *a == *b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => a == b,
            (DoubleLike::Integer(a), DoubleLike::Double(b)) => (*a as f64) == *b,
            (DoubleLike::Double(a), DoubleLike::Integer(b)) => *a == (*b as f64),
            // (DoubleLike::AllocatedDouble(a), DoubleLike::AllocatedDouble(b)) => **a == **b,
            // (DoubleLike::AllocatedDouble(a), DoubleLike::Double(b)) => **a == *b,
            // (DoubleLike::Double(a), DoubleLike::AllocatedDouble(b)) => *a == **b,
            // (DoubleLike::AllocatedDouble(a), DoubleLike::Integer(b)) => **a == (*b as f64),
            // (DoubleLike::Integer(a), DoubleLike::AllocatedDouble(b)) => (*a as f64) == **b,
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a == **b,
            _ => false,
        }
    }
}

#[cfg(feature = "lbits")]
#[derive(Debug, Clone)]
pub enum StringLike<SPTR> {
    TinyStr(Vec<u8>),
    String(SPTR),
    Symbol(Interned),
    Char(char),
}

#[cfg(feature = "nan")]
#[derive(Debug, Clone)]
pub enum StringLike<SPTR> {
    String(SPTR),
    Symbol(Interned),
    Char(char),
}

#[cfg(feature = "idiomatic")]
#[derive(Debug, Clone)]
pub enum StringLike<SPTR> {
    String(SPTR),
    Symbol(Interned),
    Char(char),
}

#[cfg(feature = "lbits")]
impl<SPTR> TryFrom<BaseValue> for StringLike<SPTR>
where
    SPTR: Deref<Target = String> + From<u64> + Into<u64>,
{
    type Error = anyhow::Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
        .as_string().map(Self::String)
            .or_else(|| value.as_tiny_str().map(Self::TinyStr))
            .or_else(|| value.as_symbol().map(Self::Symbol))
            .or_else(|| value.as_char().map(Self::Char))
            .context("could not resolve `Value` as `String`, `Symbol` or `Char`")
    }
}

#[cfg(feature = "nan")]
impl<SPTR> TryFrom<BaseValue> for StringLike<SPTR>
where
    SPTR: Deref<Target = String> + From<u64> + Into<u64>,
{
    type Error = anyhow::Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_string().map(Self::String)
            .or_else(|| value.as_symbol().map(Self::Symbol))
            .or_else(|| value.as_char().map(Self::Char))
            .context("could not resolve `Value` as `String`, `Symbol` or `Char`")
    }
}

impl<SPTR: Deref<Target = String> + std::fmt::Debug> StringLike<SPTR> {

    #[cfg(feature = "lbits")]
    pub fn as_str<'a, F>(&'a self, lookup_symbol_fn: F) -> Cow<'a, str>
    where
        F: Fn(Interned) -> &'a str,
    {
        match self {
            // StringLike::TinyStr(tiny_str) => {
            //     let full = std::str::from_utf8(&tiny_str[..])
            //         .expect("TinyStr buffer was not valid UTF-8");

            //     let trimmed = full.trim_end_matches('\0');
            //     Cow::from(trimmed)
            // },
            StringLike::TinyStr(tiny_str) => Cow::from(std::str::from_utf8(tiny_str).unwrap()),
            StringLike::String(ref value) => Cow::from(value.as_str()),
            StringLike::Symbol(sym) => Cow::from(lookup_symbol_fn(*sym)),
            StringLike::Char(char) => Cow::from(char.to_string()),
        }
    }

    #[cfg(feature = "nan")]
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
    #[cfg(feature = "nan")]
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
    
    #[cfg(feature = "lbits")]
    pub fn eq_stringlike<'a, F>(&'a self, other: &'a Self, lookup_symbol_fn: F) -> bool
    where
        F: Copy + Fn(Interned) -> &'a str,
    {
        match (&self, &other) {
            (StringLike::Char(c1), StringLike::Char(c2)) => *c1 == *c2,
            (StringLike::Char(c1), StringLike::String(s2)) => s2.len() == 1 && *c1 == s2.chars().next().unwrap(),
            (StringLike::String(s1), StringLike::Char(c2)) => s1.len() == 1 && s1.chars().next().unwrap() == *c2,
            (StringLike::Symbol(sym1), StringLike::Symbol(sym2)) => {
                (*sym1 == *sym2) || (lookup_symbol_fn(*sym1) == lookup_symbol_fn(*sym2))
            },
            (StringLike::String(str1), StringLike::String(str2)) => str1.as_str().eq(str2.as_str()),
            (StringLike::TinyStr(tstr1), StringLike::TinyStr(tstr2)) => std::str::from_utf8(tstr1).unwrap() == std::str::from_utf8(tstr2).unwrap(),
            (StringLike::TinyStr(tstr1), StringLike::Char(c2)) => {
                let s1 = std::str::from_utf8(tstr1).unwrap();
                s1.len() == 1 &&  s1.chars().next().unwrap() == *c2
            },
            (StringLike::Char(c1), StringLike::TinyStr(tstr2)) => {
                let s2 = std::str::from_utf8(tstr2).unwrap();
                s2.len() == 1 &&  s2.chars().next().unwrap() == *c1
            },
            (StringLike::TinyStr(tstr1), StringLike::String(str2)) => {
                let str2_bytes = str2.as_str().as_bytes();
                tstr1.iter()
                    .filter(|&&b| b != 0)
                    .eq(str2_bytes.iter().filter(|&&b| b != 0))
            },
            (StringLike::String(str1), StringLike::TinyStr(tstr2)) => {
                let str1_bytes = str1.as_str().as_bytes();
                tstr2.iter()
                    .filter(|&&b| b != 0)
                    .eq(str1_bytes.iter().filter(|&&b| b != 0))
            },
            (StringLike::TinyStr(tstr1), StringLike::Symbol(sym2)) => {
                let s1 = std::str::from_utf8(tstr1).unwrap();
                let s2 = lookup_symbol_fn(*sym2);
                s1 == s2
            },
            (StringLike::Symbol(sym1), StringLike::TinyStr(tstr2)) => {
                let s1 = lookup_symbol_fn(*sym1);
                let s2 = std::str::from_utf8(tstr2).unwrap();

                s1 == s2
            },
            _ => {
                let a = self.as_str(lookup_symbol_fn);
                let b = other.as_str(lookup_symbol_fn);
                *a == *b
            }
        }
    }
}
