use anyhow::{bail, Context, Error};

#[cfg(not(feature = "idiomatic"))]
use som_gc::gcslice::GcSlice;

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
use som_value::value_ptr::HasPointerTag;

use std::convert::TryFrom;

use crate::cur_frame;
use crate::gc::VecValue;
use crate::interpreter::Interpreter;
use crate::primitives::PrimitiveFn;
use crate::universe::Universe;
use crate::value::Value;
use crate::vm_objects::block::Block;
use crate::vm_objects::class::Class;
use crate::vm_objects::instance::Instance;
use crate::vm_objects::method::Method;
use num_bigint::BigInt;
use som_gc::gcref::Gc;
use som_value::interned::Interned;

#[cfg(feature = "l3bits")]
use crate::gc::BCObjMagicId;

#[cfg(feature = "l3bits")]
use som_value::value::BaseValue;

#[cfg(any(feature = "nan", feature = "l4bits"))]
pub type DoubleLike = som_value::convert::DoubleLike<Gc<f64>, Gc<BigInt>>;
#[cfg(any(feature = "nan", feature = "l4bits"))]
pub type IntegerLike = som_value::convert::IntegerLike<Gc<BigInt>>;
#[cfg(any(feature = "nan", feature = "l4bits"))]
pub type StringLike = som_value::convert::StringLike<Gc<String>>;

#[cfg(feature = "l3bits")]
use std::borrow::Cow;

#[cfg(feature = "idiomatic")]
use crate::value::value_enum::ValueEnum;

#[cfg(feature = "idiomatic")]
#[derive(Debug, Clone)]
pub enum IntegerLike {
    Integer(i32),
    BigInteger(Gc<BigInt>),
}

#[cfg(feature = "l3bits")]
#[derive(Debug, Clone)]
pub enum IntegerLike {
    Integer(i32),
    BigInteger(Gc<BigInt>),
}

#[cfg(feature = "idiomatic")]
#[derive(Debug, Clone)]
pub enum DoubleLike {
    Double(f64),
    Integer(i32),
    BigInteger(Gc<BigInt>),
}

#[cfg(feature = "l3bits")]
#[derive(Debug, Clone)]
pub enum DoubleLike {
    Double(f64),
    Integer(i32),
    BigInteger(Gc<BigInt>),
    #[doc(hidden)]
    __Phantom(std::marker::PhantomData<Gc<f64>>),
}

// #[cfg(feature = "idiomatic")]
// #[derive(Debug, Clone)]
// pub enum StringLike {
//     TinyStr(Vec<u8>),
//     String(Gc<String>),
//     Symbol(Interned),
// }

#[cfg(feature = "l3bits")]
#[derive(Debug, Clone)]
pub enum StringLike {
    TinyStr(u64),
    String(Gc<String>),
    Symbol(Interned),
}

#[cfg(feature = "l3bits")]
trait BaseValueExt {
    fn as_big_integer(self) -> Option<Gc<BigInt>>;
    fn as_string(self) -> Option<Gc<String>>;
    fn as_allocated_double(self) -> Option<Gc<f64>>;
}

#[cfg(feature = "l3bits")]
impl BaseValueExt for BaseValue {
    #[inline(always)]
    fn as_big_integer(self) -> Option<Gc<BigInt>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::BigInt => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    fn as_string(self) -> Option<Gc<String>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::String => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    fn as_allocated_double(self) -> Option<Gc<f64>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::Double => Some(ptr.into()),
                _ => None,
            }
        }
    }
}

#[cfg(feature = "idiomatic")]
impl TryFrom<ValueEnum> for IntegerLike {
    type Error = Error;

    fn try_from(value: ValueEnum) -> Result<Self, Self::Error> {
        match value {
            ValueEnum::Integer(i) => Ok(IntegerLike::Integer(i)),
            ValueEnum::BigInteger(i) => Ok(IntegerLike::BigInteger(i)),
            _ => bail!("could not resolve `Value` as `Integer` or `BigInteger`"),
        }
    }
}

#[cfg(feature = "l3bits")]
impl TryFrom<BaseValue> for IntegerLike {
    type Error = Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_integer()
            .map(Self::Integer)
            .or_else(|| value.as_big_integer().map(Self::BigInteger))
            .context("could not resolve `Value` as `Integer`, or `BigInteger`")
    }
}

#[cfg(feature = "idiomatic")]
impl TryFrom<ValueEnum> for DoubleLike {
    type Error = Error;

    fn try_from(value: ValueEnum) -> Result<Self, Self::Error> {
        match value {
            ValueEnum::Double(d) => Ok(DoubleLike::Double(d)),
            ValueEnum::Integer(i) => Ok(DoubleLike::Integer(i)),
            ValueEnum::BigInteger(i) => Ok(DoubleLike::BigInteger(i)),
            _ => bail!("could not resolve `Value` as `Double`"),
        }
    }
}

#[cfg(feature = "l3bits")]
impl TryFrom<BaseValue> for DoubleLike {
    type Error = Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
            .as_double()
            .map(Self::Double)
            .or_else(|| value.as_integer().map(Self::Integer))
            // .or_else(|| value.as_allocated_double().map(|v: DOUBLEPTR | Self::Double(*v)))
            .or_else(|| value.as_big_integer().map(Self::BigInteger))
            .or_else(|| {
                value.as_allocated_double().map(|v: Gc<f64> | {
                    Self::Double(*v)
                })
            })
            .context("could not resolve `Value` as `Double`, `Integer`, or `BigInteger`")
    }
    // .or_else(|| value.as_allocated_double().map(Self::AllocatedDouble))
}

#[cfg(feature = "idiomatic")]
impl TryFrom<ValueEnum> for f64 {
    type Error = anyhow::Error;

    fn try_from(value: ValueEnum) -> Result<Self, Self::Error> {
        match value { 
            ValueEnum::Double(d) => Ok(d),
            _ => bail!("could not resolve `Value` as `f64`"),
        }
    }
}

// #[cfg(feature = "idiomatic")]
// impl TryFrom<ValueEnum> for StringLike {
//     type Error = Error;

//     fn try_from(value: ValueEnum) -> Result<Self, Self::Error> {
//         value
//         .as_string().map(Self::String)
//             .or_else(|| value.as_tiny_str().map(Self::TinyStr))
//             .or_else(|| value.as_symbol().map(Self::Symbol))
//             .context("could not resolve `Value` as `String`, `Symbol` or `TinyStr`")
//         // match value {
//         //     ValueEnum::TinyStr(s) => Ok(StringLike::TinyStr(s)),
//         //     ValueEnum::String(s) => Ok(StringLike::String(s)),
//         //     ValueEnum::Symbol(s) => Ok(StringLike::Symbol(s)),
//         //     _ => bail!("could not resolve `Value` as `String`"),
//         // }
//     }
// }

// #[cfg(feature = "idiomatic")]
// impl TryFrom<Value> for StringLike {
//     type Error = Error;

//     fn try_from(value: Value) -> Result<Self, Self::Error> {
//         value
//         .as_string().map(Self::String)
//             .or_else(|| value.as_tiny_str().map(Self::TinyStr))
//             .or_else(|| value.as_symbol().map(Self::Symbol))
//             .context("could not resolve `Value` as `String`, `Symbol` or `TinyStr`")
//         // match value.0 {
//         //     ValueEnum::TinyStr(s) => Ok(StringLike::TinyStr(s)),
//         //     ValueEnum::String(s) => Ok(StringLike::String(s)),
//         //     ValueEnum::Symbol(s) => Ok(StringLike::Symbol(s)),
//         //     _ => bail!("could not resolve `Value` as `String`"),
//         // }
//     }
// }

#[cfg(feature = "l3bits")]
impl TryFrom<BaseValue> for StringLike {
    type Error = anyhow::Error;

    fn try_from(value: BaseValue) -> Result<Self, Self::Error> {
        value
        .as_string().map(Self::String)
            .or_else(|| value.as_tiny_str().map(Self::TinyStr))
            .or_else(|| value.as_symbol().map(Self::Symbol))
            // .or_else(|| value.as_char().map(Self::Char))
            .context("could not resolve `Value` as `String`, `Symbol` or `TinyStr`")
    }
}

#[cfg(feature = "idiomatic")]
impl DoubleLike {
    pub fn lt(&self, other: &DoubleLike) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => a < b,
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a < **b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => a < b,
            (DoubleLike::Integer(a), DoubleLike::Double(b)) => (*a as f64) < *b,
            (DoubleLike::Double(a), DoubleLike::Integer(b)) => *a < (*b as f64),
            (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) => **a < BigInt::from(*b),
            (DoubleLike::Integer(a), DoubleLike::BigInteger(b)) => BigInt::from(*a) < **b,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn gt(&self, other: &DoubleLike) -> bool {
        !self.lt(other) && !self.eq(other)
    }

    #[inline(always)]
    pub fn lt_or_eq(&self, other: &DoubleLike) -> bool {
        Self::lt(self, other) || Self::eq(self, other)
    }

    #[inline(always)]
    pub fn gt_or_eq(&self, other: &DoubleLike) -> bool {
        Self::gt(self, other) || Self::eq(self, other)
    }
}

#[cfg(feature = "idiomatic")]
impl PartialEq for DoubleLike {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DoubleLike::Integer(a), DoubleLike::Integer(b)) => *a == *b,
            (DoubleLike::Double(a), DoubleLike::Double(b)) => *a == *b,
            (DoubleLike::Integer(a), DoubleLike::Double(b)) => (*a as f64) == *b,
            (DoubleLike::Double(a), DoubleLike::Integer(b)) => *a == (*b as f64),
            (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => **a == **b,
            _ => false,
        }
    }
}

#[cfg(feature = "l3bits")]
impl DoubleLike {
    #[inline(always)]
    pub fn lt(&self, other: &DoubleLike) -> bool {
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
    pub fn gt(&self, other: &DoubleLike) -> bool {
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
    pub fn lt_or_eq(&self, other: &DoubleLike) -> bool {
        Self::lt(self, other) || Self::eq(self, other)
    }

    #[inline(always)]
    pub fn gt_or_eq(&self, other: &DoubleLike) -> bool {
        Self::gt(self, other) || Self::eq(self, other)
    }
}

#[cfg(feature = "l3bits")]
impl PartialEq for DoubleLike {
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

// #[cfg(feature = "idiomatic")]
// impl StringLike {
//     pub fn as_str<'a>(&'a self, lookup_symbol: impl Fn(Interned) -> &'a str) -> std::borrow::Cow<'a, str> {
//         // println!("[{:?}]", self);
//         match self {
//             StringLike::TinyStr(tiny_str) => Cow::from(std::str::from_utf8(tiny_str).unwrap()),
//             StringLike::String(ref value) => Cow::from(value.as_str()),
//             StringLike::Symbol(sym) => std::borrow::Cow::Borrowed(lookup_symbol(*sym)),
//         }
//     }

//     // pub fn eq_with_lookup(&self, other: &Self, lookup_symbol: impl Fn(Interned) -> &'static str) -> bool {
//     //     match (self, other) {
//     //         (StringLike::TinyStr(tstr1), StringLike::TinyStr(tstr2)) => c1 == c2,
//     //         (StringLike::Char(c1), StringLike::String(s2)) => s2.len() == 1 && *c1 == s2.chars().next().unwrap(),
//     //         (StringLike::String(s1), StringLike::Char(c2)) => s1.len() == 1 && s1.chars().next().unwrap() == *c2,
//     //         (StringLike::Symbol(sym1), StringLike::Symbol(sym2)) => sym1 == sym2 || lookup_symbol(*sym1) == lookup_symbol(*sym2),
//     //         (StringLike::String(s1), StringLike::String(s2)) => s1 == s2,
//     //         _ => false,
//     //     }
//     // }

//     pub fn eq_stringlike<'a, F>(&'a self, other: &'a Self, lookup_symbol_fn: F) -> bool
//     where
//         F: Copy + Fn(Interned) -> &'a str,
//     {
//         match (&self, &other) {
//             (StringLike::Symbol(sym1), StringLike::Symbol(sym2)) => {
//                 (*sym1 == *sym2) || (lookup_symbol_fn(*sym1) == lookup_symbol_fn(*sym2))
//             },
//             (StringLike::String(str1), StringLike::String(str2)) => str1.as_str().eq(str2.as_str()),
//             (StringLike::TinyStr(tstr1), StringLike::TinyStr(tstr2)) => {
//                 match (std::str::from_utf8(tstr1), std::str::from_utf8(tstr2)) {
//                     (Ok(s1), Ok(s2)) => s1 == s2,
//                     _ => false,
//                 }
//             },
//             (StringLike::TinyStr(tstr1), StringLike::String(str2)) => {
//                 let str2_bytes = str2.as_str().as_bytes();
//                 tstr1.iter()
//                     .filter(|&&b| b != 0)
//                     .eq(str2_bytes.iter().filter(|&&b| b != 0))
//             },
//             (StringLike::String(str1), StringLike::TinyStr(tstr2)) => {
//                 let str1_bytes = str1.as_str().as_bytes();
//                 tstr2.iter()
//                     .filter(|&&b| b != 0)
//                     .eq(str1_bytes.iter().filter(|&&b| b != 0))
//             },
//             (StringLike::TinyStr(tstr1), StringLike::Symbol(sym2)) => {
//                 let s1 = std::str::from_utf8(tstr1).unwrap();
//                 let s2 = lookup_symbol_fn(*sym2);
//                 s1 == s2
//             },
//             (StringLike::Symbol(sym1), StringLike::TinyStr(tstr2)) => {
//                 let s1 = lookup_symbol_fn(*sym1);
//                 let s2 = std::str::from_utf8(tstr2).unwrap();

//                 s1 == s2
//             },
//             _ => {
//                 let a = self.as_str(lookup_symbol_fn);
//                 let b = other.as_str(lookup_symbol_fn);
//                 *a == *b
//             }
//         }
//     }
// }

#[cfg(feature = "l3bits")]
impl StringLike {
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
            StringLike::TinyStr(tiny_str) => {
                let v = *tiny_str as u64;
                let mut buf = [0u8; 7];
                let mut len = 0usize;

                for i in 0..7 {
                    let b = ((v >> (i * 8)) & 0xFF) as u8;
                    if b == 0xFF { break; }
                    buf[i] = b;
                    len += 1;
                }

                match std::str::from_utf8(&buf[..len]) {
                    Ok(s)  => Cow::Owned(s.to_owned()),
                    Err(_) => Cow::Borrowed(""),
                }
            },
            StringLike::String(ref value) => Cow::from(value.as_str()),
            StringLike::Symbol(sym) => Cow::from(lookup_symbol_fn(*sym)),
        }
    }

    pub fn eq_stringlike<'a, F>(&'a self, other: &'a Self, lookup_symbol_fn: F) -> bool
    where
        F: Copy + Fn(Interned) -> &'a str,
    {
        #[inline]
        fn tinystring_as_str<'a>(v: u64, buf: &'a mut [u8; 7]) -> &'a str {
            let mut len = 0;
            for i in 0..7 {
                let b = ((v >> (i * 8)) & 0xFF) as u8;
                if b == 0xFF {
                    break;
                }
                buf[i] = b;
                len += 1;
            }
            match std::str::from_utf8(&buf[..len]) {
                Ok(s) => s,
                Err(_) => "",
            }
        }

        let mut buf = [0u8; 7];
        match (&self, &other) {
            (StringLike::Symbol(sym1), StringLike::Symbol(sym2)) => {
                (*sym1 == *sym2) || (lookup_symbol_fn(*sym1) == lookup_symbol_fn(*sym2))
            },
            (StringLike::String(str1), StringLike::String(str2)) => str1.as_str().eq(str2.as_str()),
            (StringLike::TinyStr(tstr1), StringLike::TinyStr(tstr2)) => {
                tstr1 == tstr2
            },
            (StringLike::TinyStr(tstr1), StringLike::String(str2)) => {
                tinystring_as_str(*tstr1, &mut buf) == **str2
            },
            (StringLike::String(str1), StringLike::TinyStr(tstr2)) => {
                **str1 == tinystring_as_str(*tstr2, &mut buf)
            },
            (StringLike::TinyStr(tstr1), StringLike::Symbol(sym2)) => {
                let s1 = tinystring_as_str(*tstr1, &mut buf);
                let s2 = lookup_symbol_fn(*sym2);
                s1 == s2
            },
            (StringLike::Symbol(sym1), StringLike::TinyStr(tstr2)) => {
                let s1 = lookup_symbol_fn(*sym1);
                let s2 = tinystring_as_str(*tstr2, &mut buf);

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


pub trait IntoValue {
    #[allow(clippy::wrong_self_convention)] // though i guess we could/should rename it
    fn into_value(&self) -> Value;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nil;

impl TryFrom<Value> for Nil {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if value.is_nil() {
            Ok(Self)
        } else {
            bail!("could not resolve `Value` as `Nil`");
        }
    }
}

impl FromArgs for Nil {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Self::try_from(arg)
    }
}

#[cfg(not(feature = "idiomatic"))]
impl FromArgs for StringLike {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Self::try_from(arg.0)
    }
}

impl FromArgs for DoubleLike {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Self::try_from(arg.0)
    }
}
impl FromArgs for IntegerLike {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Self::try_from(arg.0)
    }
}

pub trait FromArgs: Sized {
    fn from_args(arg: Value) -> Result<Self, Error>;
}

impl FromArgs for Value {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Ok(arg)
    }
}

impl FromArgs for bool {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg.as_boolean().context("could not resolve `Value` as `Boolean`")
    }
}

impl FromArgs for i32 {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg.as_integer().context("could not resolve `Value` as `Integer`")
    }
}

#[cfg(any(feature = "l4bits", feature = "l3bits"))]
impl FromArgs for f64 {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg
            .as_double()
            .or_else(|| {
                arg.as_allocated_double().map(|v: Gc<f64>| {
                    *v
                })
            })
            .context("could not resolve `Value` as `Double`")
    }
}

#[cfg(feature = "nan")]
impl FromArgs for f64 {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg
            .as_double()
            .context("could not resolve `Value` as `Double`")
    }
}

#[cfg(feature = "idiomatic")]
impl FromArgs for f64 {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg
            .as_double()
            .context("could not resolve `Value` as `Double`")
    }
}

impl FromArgs for Interned {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg.as_symbol().context("could not resolve `Value` as `Symbol`")
    }
}

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
impl FromArgs for VecValue {
   fn from_args(arg: Value) -> Result<Self, Error> {
       Ok(VecValue(GcSlice::from(arg.extract_pointer_bits())))
   }
}

#[cfg(feature = "idiomatic")]
impl FromArgs for VecValue {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg.as_array().context("could not resolve `Value` as `Array`")
    }
}

#[cfg(feature = "idiomatic")]
impl FromArgs for Gc<Class> {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg.as_class().context("could not resolve `Value` as `Class`")
    }
}

#[cfg(feature = "idiomatic")]
impl FromArgs for Gc<String> {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg.as_string().context("could not resolve `Value` as `String`")
    }
}

// impl FromArgs for Gc<Method> {
//     fn from_args(arg: Value) -> Result<Self, Error> {
//         arg.as_method().context("could not resolve `Value` as `Method`")
//     }
// }

#[cfg(feature = "idiomatic")]
impl FromArgs for Gc<Method> {
    fn from_args(arg: Value) -> Result<Self, Error> {
        match arg.0 {
            ValueEnum::Invokable(method) => Ok(method),
            _ => Err(anyhow::anyhow!("could not resolve `Value` as `Method`")),
        }
    }
}

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
impl<T: HasPointerTag> FromArgs for Gc<T> {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Ok(arg.as_value_ptr().unwrap())
    }
}

impl IntoValue for bool {
    fn into_value(&self) -> Value {
        Value::Boolean(*self)
    }
}

impl IntoValue for i32 {
    fn into_value(&self) -> Value {
        Value::Integer(*self)
    }
}

impl IntoValue for f64 {
    fn into_value(&self) -> Value {
        Value::Double(*self)
    }
}

#[cfg(feature = "nan")]
impl IntoValue for char {
    fn into_value(&self) -> Value {
        Value::Char(*self)
    }
}

impl IntoValue for Interned {
    fn into_value(&self) -> Value {
        Value::Symbol(*self)
    }
}

#[cfg(any(feature = "l4bits", feature = "l3bits"))]
impl IntoValue for Gc<f64> {
    fn into_value(&self) -> Value {
        Value::AllocatedDouble(self.clone())
    }
}

#[cfg(any(feature = "l4bits", feature = "l3bits", feature = "idiomatic"))]
impl IntoValue for u64 {
    fn into_value(&self) -> Value {
        Value::TinyStr(*self)
    }
}

impl IntoValue for Gc<String> {
    fn into_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl IntoValue for Gc<BigInt> {
    fn into_value(&self) -> Value {
        Value::BigInteger(self.clone())
    }
}

impl IntoValue for VecValue {
    fn into_value(&self) -> Value {
        Value::Array(self.clone())
    }
}

impl IntoValue for Gc<Class> {
    fn into_value(&self) -> Value {
        Value::Class(self.clone())
    }
}

impl IntoValue for Gc<Instance> {
    fn into_value(&self) -> Value {
        Value::Instance(self.clone())
    }
}

impl IntoValue for Gc<Block> {
    fn into_value(&self) -> Value {
        Value::Block(self.clone())
    }
}

impl IntoValue for Gc<Method> {
    fn into_value(&self) -> Value {
        Value::Invokable(self.clone())
    }
}

pub trait Primitive<T>: Sized + Send + Sync + 'static {
    fn invoke(&self, interpreter: &mut Interpreter, universe: &mut Universe, nbr_args: usize) -> Result<(), Error>;

    fn into_func(self) -> &'static PrimitiveFn {
        let boxed =
            Box::new(move |interpreter: &mut Interpreter, universe: &mut Universe, nbr_args: usize| self.invoke(interpreter, universe, nbr_args));
        Box::leak(boxed)
    }
}

pub trait IntoReturn {
    fn into_return(self, interpreter: &mut Interpreter, nbr_args: usize) -> Result<(), Error>;
}

impl<T: IntoValue> IntoReturn for T {
    fn into_return(self, interpreter: &mut Interpreter, nbr_args: usize) -> Result<(), Error> {
        interpreter.get_current_frame().remove_n_last_elements(nbr_args);
        interpreter.get_current_frame().stack_push(self.into_value());
        Ok(())
    }
}

impl IntoValue for Value {
    fn into_value(&self) -> Value {
        self.clone()
    }
}

impl IntoValue for Nil {
    fn into_value(&self) -> Value {
        Value::NIL
    }
}

impl<T: IntoValue> IntoValue for Option<T> {
    fn into_value(&self) -> Value {
        self.as_ref().map_or(Value::NIL, |it| it.into_value())
    }
}

impl IntoReturn for () {
    fn into_return(self, _: &mut Interpreter, _: usize) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(feature = "l4bits")]
impl IntoValue for StringLike {
    fn into_value(&self) -> Value {
        match self {
            StringLike::TinyStr(value) => value.into_value(),
            StringLike::String(value) => value.into_value(),
            StringLike::Symbol(value) => value.into_value(),
        }
    }
}

// #[cfg(feature = "idiomatic")]
// impl IntoValue for StringLike {
//     fn into_value(&self) -> Value {
//         match self {
//             StringLike::TinyStr(value) => value.into_value(),
//             StringLike::String(value) => value.into_value(),
//             StringLike::Symbol(value) => value.into_value(),
//         }
//     }
// }

#[cfg(feature = "l3bits")]
impl IntoValue for StringLike {
    fn into_value(&self) -> Value {
        match self {
            StringLike::TinyStr(value) => value.into_value(),
            StringLike::String(value) => value.into_value(),
            StringLike::Symbol(value) => value.into_value(),
        }
    }
}

#[cfg(feature = "nan")]
impl IntoValue for StringLike {
    fn into_value(&self) -> Value {
        match self {
            StringLike::String(value) => value.into_value(),
            StringLike::Char(value) => value.into_value(),
            StringLike::Symbol(value) => value.into_value(),
        }
    }
}

impl IntoValue for IntegerLike {
    fn into_value(&self) -> Value {
        match self {
            IntegerLike::Integer(value) => value.into_value(),
            IntegerLike::BigInteger(value) => value.into_value(),
        }
    }
}

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
impl IntoValue for DoubleLike {
    fn into_value(&self) -> Value {
        match self {
            DoubleLike::Double(value) => value.into_value(),
            // DoubleLike::AllocatedDouble(value) => value.into_value(),
            DoubleLike::Integer(value) => value.into_value(),
            DoubleLike::BigInteger(value) => value.into_value(),
            _ => panic!("Undefined DoubleLike : {:?}", self)
        }
    }
}

#[cfg(feature = "idiomatic")]
impl IntoValue for DoubleLike {
    fn into_value(&self) -> Value {
        match self {
            DoubleLike::Double(value) => value.into_value(),
            // DoubleLike::AllocatedDouble(value) => value.into_value(),
            DoubleLike::Integer(value) => value.into_value(),
            DoubleLike::BigInteger(value) => value.into_value(),
            //_ => panic!("Undefined DoubleLike : {:?}", self)
        }
    }
}

macro_rules! derive_prims {
    ($($ty:ident),* $(,)?) => {

        impl <F, R, $($ty),*> $crate::value::convert::Primitive<($($ty),*,)> for F
        where
            F: Fn($($ty),*) -> Result<R, Error> + Send + Sync + 'static,
            R: $crate::value::convert::IntoValue,
            $($ty: $crate::value::convert::FromArgs),*,
        {
            fn invoke(&self, interpreter: &mut $crate::interpreter::Interpreter, _: &mut $crate::universe::Universe, nbr_args: usize) -> Result<(), Error> {
                let mut cur_frame = interpreter.get_current_frame();

                let result = {
                    let args: &[Value] = cur_frame.stack_n_last_elements(nbr_args);
                    let mut args_iter = args.iter();
                    $(
                        #[allow(non_snake_case)]
                        let $ty = $ty::from_args(args_iter.next().unwrap().clone()).unwrap();
                    )*

                   (self)($($ty),*,)?.into_value()
                };

                cur_frame.remove_n_last_elements(nbr_args);
                cur_frame.stack_push(result);
                Ok(())
            }
        }
    };
}

derive_prims!(_A);
derive_prims!(_A, _B);
derive_prims!(_A, _B, _C);
derive_prims!(_A, _B, _C, _D);

/// Primitives that need access to the universe may trigger GC, which can move variables.
/// Therefore, they take arguments from the stack (previous frame) themselves, and are responsible
/// for ensuring possible GC triggers can't invalidate their arguments, or the primitive's behavior.
impl<F, R> Primitive<R> for F
where
    F: Fn(&mut Interpreter, &mut Universe) -> Result<R, Error> + Send + Sync + 'static,
    R: crate::value::convert::IntoValue,
{
    fn invoke(&self, interpreter: &mut Interpreter, universe: &mut Universe, _: usize) -> Result<(), Error> {
        let result = self(interpreter, universe)?.into_value();
        cur_frame!(interpreter).stack_push(result);
        Ok(())
    }
}

/// For primitives who have no return values... or want complete control over their arguments and return value.
impl<F> Primitive<()> for F
where
    F: Fn(&mut Interpreter, &mut Universe) -> Result<(), Error> + Send + Sync + 'static,
{
    fn invoke(&self, interpreter: &mut Interpreter, universe: &mut Universe, _: usize) -> Result<(), Error> {
        self(interpreter, universe)
    }
}
