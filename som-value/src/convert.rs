use anyhow::Context;

#[cfg(any(feature = "nan", feature = "l4bits", feature = "idiomatic"))]
use anyhow::Error;

#[cfg(any(feature = "nan", feature = "l4bits", feature = "idiomatic"))]
use num_bigint::BigInt;
#[cfg(any(feature = "nan", feature = "l4bits", feature = "idiomatic"))]
use std::borrow::Cow;
#[cfg(any(feature = "nan", feature = "l4bits", feature = "idiomatic"))]
use std::ops::Deref;
#[cfg(any(feature = "nan", feature = "l4bits", feature = "idiomatic"))]
use crate::interned::Interned;

#[cfg(feature = "nan")]
use crate::nan::value::BaseValue;

#[cfg(any(feature = "l4bits"))]
use crate::l4bits::value::BaseValue;

#[cfg(any(feature = "l3bits"))]
use crate::l3bits::value::BaseValue;

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

#[cfg(any(feature = "l4bits", feature = "nan"))]
#[derive(Debug, Clone)]
pub enum IntegerLike<BIGINTPTR> {
    Integer(i32),
    BigInteger(BIGINTPTR),
}

#[cfg(any(feature = "l4bits", feature = "nan"))]
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

#[cfg(any(feature = "l4bits", feature = "nan"))]
#[derive(Debug, Clone)]
pub enum DoubleLike<DOUBLEPTR, BIGINTPTR> {
    Double(f64),
    Integer(i32),
    BigInteger(BIGINTPTR),
    #[doc(hidden)]
    __Phantom(std::marker::PhantomData<DOUBLEPTR>),
}

#[cfg(feature = "l4bits")]
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

#[cfg(any(feature = "l4bits", feature = "nan"))]
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

#[cfg(any(feature = "nan", feature = "l4bits"))]
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

#[cfg(feature = "l4bits")]
#[derive(Debug, Clone)]
pub enum StringLike<SPTR> {
    TinyStr(u8),
    String(SPTR),
    Symbol(Interned),
}

#[cfg(feature = "nan")]
#[derive(Debug, Clone)]
pub enum StringLike<SPTR> {
    String(SPTR),
    Symbol(Interned),
    Char(char),
}

#[cfg(feature = "l4bits")]
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
            // .or_else(|| value.as_char().map(Self::Char))
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

#[cfg(any(feature = "nan", feature = "l4bits"))]
impl<SPTR: Deref<Target = String> + std::fmt::Debug> StringLike<SPTR> {

    #[cfg(any(feature = "l4bits", feature = "l3bits"))]
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
            StringLike::TinyStr(tiny_str) => Cow::from(format!("{}", *tiny_str as char)),
            StringLike::String(ref value) => Cow::from(value.as_str()),
            StringLike::Symbol(sym) => Cow::from(lookup_symbol_fn(*sym)),
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
    
    #[cfg(feature = "l4bits")]
    pub fn eq_stringlike<'a, F>(&'a self, other: &'a Self, lookup_symbol_fn: F) -> bool
    where
        F: Copy + Fn(Interned) -> &'a str,
    {
        match (&self, &other) {
            (StringLike::Symbol(sym1), StringLike::Symbol(sym2)) => {
                (*sym1 == *sym2) || (lookup_symbol_fn(*sym1) == lookup_symbol_fn(*sym2))
            },
            (StringLike::String(str1), StringLike::String(str2)) => str1.as_str().eq(str2.as_str()),
            (StringLike::TinyStr(tstr1), StringLike::TinyStr(tstr2)) => {
                tstr1 == tstr2
            },
            (StringLike::TinyStr(tstr1), StringLike::String(str2)) => {
                let str2_bytes = str2.as_str().as_bytes();
                let str1 = format!("{}", *tstr1 as char);
                let str1_bytes = str1.as_str().as_bytes();
                str1_bytes == str2_bytes
            },
            (StringLike::String(str1), StringLike::TinyStr(tstr2)) => {
                let str1_bytes = str1.as_str().as_bytes();
                let str2 = format!("{}", *tstr2 as char);
                let str2_bytes = str2.as_str().as_bytes();
                str1_bytes == str2_bytes
            },
            (StringLike::TinyStr(tstr1), StringLike::Symbol(sym2)) => {
                let s1 = format!("{}", *tstr1 as char);
                let s2 = lookup_symbol_fn(*sym2);
                s1 == s2
            },
            (StringLike::Symbol(sym1), StringLike::TinyStr(tstr2)) => {
                let s1 = lookup_symbol_fn(*sym1);
                let s2 = format!("{}", *tstr2 as char);

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
