use std::collections::hash_map::DefaultHasher;
use std::convert::TryFrom;
use std::hash::Hasher;

use crate::interpreter::Interpreter;
use crate::pop_args_from_stack;
use crate::primitives::PrimInfo;
use crate::primitives::PrimitiveFn;
use crate::universe::Universe;

#[cfg(not(feature = "idiomatic"))]
use crate::value::convert::{Primitive, StringLike};

#[cfg(feature = "idiomatic")]
use crate::value::convert::Primitive;

use crate::value::Value;
use anyhow::Error;
use once_cell::sync::Lazy;
use som_gc::gc_interface::SOMAllocator;
use som_gc::gcref::Gc;
use som_value::interned::Interned;

#[cfg(feature = "idiomatic")]
use crate::value::value_enum::ValueEnum;

pub static INSTANCE_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| {
    Box::new([
        ("length", self::length.into_func(), true),
        ("hashcode", self::hashcode.into_func(), true),
        ("isLetters", self::is_letters.into_func(), true),
        ("isDigits", self::is_digits.into_func(), true),
        ("isWhiteSpace", self::is_whitespace.into_func(), true),
        ("asSymbol", self::as_symbol.into_func(), true),
        ("concatenate:", self::concatenate.into_func(), true),
        ("primSubstringFrom:to:", self::prim_substring_from_to.into_func(), true),
        ("=", self::eq.into_func(), true),
        ("charAt:", self::char_at.into_func(), true),
    ])
});
pub static CLASS_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| Box::new([]));

#[cfg(feature = "l4bits")]
fn length(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);

    // tragically, we do not allow strings to have over 2 billion characters and just cast as i32
    // i apologize to everyone for that. i will strive to be better
    match receiver {
        StringLike::TinyStr(data) => {
            let v = data as u64;
            let len = if (v & 0xFF) == 0xFF { 0 }
                else if ((v >> 8) & 0xFF) == 0xFF { 1 }
                else if ((v >> 16) & 0xFF) == 0xFF { 2 }
                else if ((v >> 24) & 0xFF) == 0xFF { 3 }
                else if ((v >> 32) & 0xFF) == 0xFF { 4 }
                else if ((v >> 40) & 0xFF) == 0xFF { 5 }
                else if ((v >> 48) & 0xFF) == 0xFF { 6 }
                else { 7 };
            Ok(Value::Integer(len as i32))
        }
        StringLike::String(ref value) => Ok(Value::Integer(value.len() as i32)),
        StringLike::Symbol(sym) => Ok(Value::Integer(universe.lookup_symbol(sym).len() as i32))
    }
}

#[cfg(feature = "l3bits")]
fn length(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);

    // tragically, we do not allow strings to have over 2 billion characters and just cast as i32
    // i apologize to everyone for that. i will strive to be better
    match receiver {
        StringLike::TinyStr(data) => {
            let v = data as u64;
            let len = if (v & 0xFF) == 0xFF { 0 }
                else if ((v >> 8) & 0xFF) == 0xFF { 1 }
                else if ((v >> 16) & 0xFF) == 0xFF { 2 }
                else if ((v >> 24) & 0xFF) == 0xFF { 3 }
                else if ((v >> 32) & 0xFF) == 0xFF { 4 }
                else if ((v >> 40) & 0xFF) == 0xFF { 5 }
                else if ((v >> 48) & 0xFF) == 0xFF { 6 }
                else { 7 };
            Ok(Value::Integer(len as i32))
        }
        StringLike::String(ref value) => Ok(Value::Integer(value.len() as i32)),
        StringLike::Symbol(sym) => Ok(Value::Integer(universe.lookup_symbol(sym).len() as i32)),
    }
}

#[cfg(feature = "nan")]
fn length(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);

    // tragically, we do not allow strings to have over 2 billion characters and just cast as i32
    // i apologize to everyone for that. i will strive to be better
    match receiver {
        StringLike::String(ref value) => Ok(Value::Integer(value.len() as i32)),
        StringLike::Symbol(sym) => Ok(Value::Integer(universe.lookup_symbol(sym).len() as i32)),
        StringLike::Char(_) => Ok(Value::Integer(1)),
    }
}

#[cfg(feature = "idiomatic")]
fn length(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => Value);

    // tragically, we do not allow strings to have over 2 billion characters and just cast as i32
    // i apologize to everyone for that. i will strive to be better
    match receiver.0 {
        ValueEnum::String(ref value) => Ok(Value::Integer(value.len() as i32)),
        ValueEnum::Symbol(sym) => Ok(Value::Integer(universe.lookup_symbol(sym).len() as i32)),
        ValueEnum::TinyStr(data) => {
            let v = data as u64;
            let len = if (v & 0xFF) == 0xFF { 0 }
                else if ((v >> 8) & 0xFF) == 0xFF { 1 }
                else if ((v >> 16) & 0xFF) == 0xFF { 2 }
                else if ((v >> 24) & 0xFF) == 0xFF { 3 }
                else if ((v >> 32) & 0xFF) == 0xFF { 4 }
                else if ((v >> 40) & 0xFF) == 0xFF { 5 }
                else if ((v >> 48) & 0xFF) == 0xFF { 6 }
                else { 7 };
            Ok(Value::Integer(len as i32))
        },
        _ => panic!()
    }
}

#[cfg(not(feature = "idiomatic"))]
fn hashcode(interp: &mut Interpreter, universe: &mut Universe) -> Result<i32, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);
    let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    let mut hasher = DefaultHasher::new();
    hasher.write(string.as_bytes());
    let hash = (hasher.finish() as i32).abs();

    Ok(hash)
}

#[cfg(feature = "idiomatic")]
fn hashcode(interp: &mut Interpreter, universe: &mut Universe) -> Result<i32, Error> {
    pop_args_from_stack!(interp, receiver => Value);
    // let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];
    let string = match receiver.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };
    let mut hasher = DefaultHasher::new();
    hasher.write(string.as_bytes());
    let hash = (hasher.finish() as i32).abs();

    Ok(hash)
}

#[cfg(not(feature = "idiomatic"))]
fn is_letters(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);
    let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    Ok(!string.is_empty() && string.chars().all(char::is_alphabetic))
}

#[cfg(feature = "idiomatic")]
fn is_letters(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, receiver => Value);
    // let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];
    let string = match receiver.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };
    Ok(!string.is_empty() && string.chars().all(char::is_alphabetic))
}

#[cfg(not(feature = "idiomatic"))]
fn is_digits(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);
    let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    Ok(!string.is_empty() && string.chars().all(char::is_numeric))
}

#[cfg(feature = "idiomatic")]
fn is_digits(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, receiver => Value);
    // let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];
    let string = match receiver.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };
    Ok(!string.is_empty() && string.chars().all(char::is_numeric))
}

#[cfg(not(feature = "idiomatic"))]
fn is_whitespace(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);
    let string = receiver.as_str(|sym| universe.lookup_symbol(sym));

    Ok(!string.is_empty() && string.chars().all(char::is_whitespace))
}

#[cfg(feature = "idiomatic")]
fn is_whitespace(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, receiver => Value);
    // let string = receiver.as_str(|sym| universe.lookup_symbol(sym));

    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];
    let string = match receiver.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };

    Ok(!string.is_empty() && string.chars().all(char::is_whitespace))
}

#[cfg(any(feature = "l4bits", feature = "l3bits"))]
fn concatenate(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => StringLike, other => StringLike);

    let s1 = receiver.as_str(|sym| universe.lookup_symbol(sym));
    let s2 = other.as_str(|sym| universe.lookup_symbol(sym));

    let final_str = format!("{s1}{s2}");
    let final_str_len = final_str.len();

    if final_str_len < 8 {
        let b = final_str.as_bytes();
        let mut word: i64 = 0x00FF_FFFF_FFFF_FFFF;
        if final_str_len > 0 { word = (word & !(0xFFi64 << 0 )) | ((b[0] as i64) << 0 ); }
        if final_str_len > 1 { word = (word & !(0xFFi64 << 8 )) | ((b[1] as i64) << 8 ); }
        if final_str_len > 2 { word = (word & !(0xFFi64 << 16)) | ((b[2] as i64) << 16); }
        if final_str_len > 3 { word = (word & !(0xFFi64 << 24)) | ((b[3] as i64) << 24); }
        if final_str_len > 4 { word = (word & !(0xFFi64 << 32)) | ((b[4] as i64) << 32); }
        if final_str_len > 5 { word = (word & !(0xFFi64 << 40)) | ((b[5] as i64) << 40); }
        if final_str_len > 6 { word = (word & !(0xFFi64 << 48)) | ((b[6] as i64) << 48); }
        return Ok(Value::TinyStr(word));
    }

    Ok(Value::String(universe.gc_interface.alloc(final_str)))
}

#[cfg(feature = "nan")]
fn concatenate(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => StringLike, other => StringLike);

    let s1 = receiver.as_str(|sym| universe.lookup_symbol(sym));
    let s2 = other.as_str(|sym| universe.lookup_symbol(sym));

    let final_str = format!("{s1}{s2}");
    Ok(Value::String(universe.gc_interface.alloc(final_str)))
}

#[cfg(feature = "idiomatic")]
fn concatenate(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => Value, other => Value);

    // let s1 = receiver.as_str(|sym| universe.lookup_symbol(sym));
    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf1 = [0u8; 7];
    let s1 = match receiver.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf1),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };

    let mut buf2 = [0u8; 7];
    let s2 = match other.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf2),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };
    // let s2 = other.as_str(|sym| universe.lookup_symbol(sym));

    let final_str = format!("{s1}{s2}");
    let final_str_len = final_str.len();

    if final_str_len < 8 {
        let b = final_str.as_bytes();
        let mut word: i64 = 0x00FF_FFFF_FFFF_FFFF;
        if final_str_len > 0 { word = (word & !(0xFFi64 << 0 )) | ((b[0] as i64) << 0 ); }
        if final_str_len > 1 { word = (word & !(0xFFi64 << 8 )) | ((b[1] as i64) << 8 ); }
        if final_str_len > 2 { word = (word & !(0xFFi64 << 16)) | ((b[2] as i64) << 16); }
        if final_str_len > 3 { word = (word & !(0xFFi64 << 24)) | ((b[3] as i64) << 24); }
        if final_str_len > 4 { word = (word & !(0xFFi64 << 32)) | ((b[4] as i64) << 32); }
        if final_str_len > 5 { word = (word & !(0xFFi64 << 40)) | ((b[5] as i64) << 40); }
        if final_str_len > 6 { word = (word & !(0xFFi64 << 48)) | ((b[6] as i64) << 48); }
        return Ok(Value::TinyStr(word));
    }

    Ok(Value::String(universe.gc_interface.alloc(final_str)))
}

#[cfg(any(feature = "l4bits", feature = "l3bits"))]
fn as_symbol(interp: &mut Interpreter, universe: &mut Universe) -> Result<Interned, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);

    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];

    let symbol = match receiver {
        StringLike::TinyStr(data) => universe.intern_symbol(tinystring_as_str(data, &mut buf)),
        StringLike::String(ref value) => universe.intern_symbol(value.as_str()),
        StringLike::Symbol(symbol) => symbol,
    };

    Ok(symbol)
}

#[cfg(feature = "nan")]
fn as_symbol(interp: &mut Interpreter, universe: &mut Universe) -> Result<Interned, Error> {
    pop_args_from_stack!(interp, receiver => StringLike);

    let symbol = match receiver {
        StringLike::String(ref value) => universe.intern_symbol(value.as_str()),
        StringLike::Char(char) => universe.intern_symbol(&String::from(char)),
        StringLike::Symbol(symbol) => symbol,
    };

    Ok(symbol)
}

#[cfg(feature = "idiomatic")]
fn as_symbol(interp: &mut Interpreter, universe: &mut Universe) -> Result<Interned, Error> {
    pop_args_from_stack!(interp, receiver => Value);

    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];
    let symbol = match receiver.0 {
        ValueEnum::TinyStr(value) => universe.intern_symbol(tinystring_as_str(value, &mut buf)),
        ValueEnum::String(ref value) => universe.intern_symbol(value.as_str()),
        ValueEnum::Symbol(sym) => sym,
        _ => panic!()
    };

    Ok(symbol)
}

#[cfg(not(feature = "idiomatic"))]
fn eq(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, a => Value, b => Value);

    let Ok(a) = StringLike::try_from(a.0) else {
        return Ok(false);
    };

    let Ok(b) = StringLike::try_from(b.0) else {
        return Ok(false);
    };
    Ok(a.eq_stringlike(&b, |sym| universe.lookup_symbol(sym)))
}

#[cfg(feature = "idiomatic")]
fn eq(interp: &mut Interpreter, universe: &mut Universe) -> Result<bool, Error> {
    pop_args_from_stack!(interp, a => Value, b => Value);

    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf_a = [0u8; 7];
    let mut buf_b = [0u8; 7];

    let res = match (&a.0, &b.0) {
        (ValueEnum::String(sa), ValueEnum::String(sb)) => **sa == **sb,

        (ValueEnum::Symbol(x), ValueEnum::Symbol(y)) => {
            *x == *y || universe.lookup_symbol(*x) == universe.lookup_symbol(*y)
        }

        (ValueEnum::TinyStr(ta), ValueEnum::TinyStr(tb)) => {
            ta == tb
        }

        (ValueEnum::TinyStr(ta), ValueEnum::String(sb)) => {
            
            tinystring_as_str(*ta, &mut buf_a) == **sb
        }
        (ValueEnum::String(sa), ValueEnum::TinyStr(tb)) => {
            tinystring_as_str(*tb, &mut buf_b) == **sa
        }

        (ValueEnum::TinyStr(ta), ValueEnum::Symbol(y)) => {
            tinystring_as_str(*ta, &mut buf_a) == universe.lookup_symbol(*y)
        }
        (ValueEnum::Symbol(x), ValueEnum::TinyStr(tb)) => {
            universe.lookup_symbol(*x) == tinystring_as_str(*tb, &mut buf_b)
        }

        (ValueEnum::String(sa), ValueEnum::Symbol(y)) => **sa == universe.lookup_symbol(*y),
        (ValueEnum::Symbol(x), ValueEnum::String(sb)) => universe.lookup_symbol(*x) == **sb,

        _ => false,
    };

    Ok(res)
}

#[cfg(not(feature = "idiomatic"))]
fn prim_substring_from_to(interp: &mut Interpreter, universe: &mut Universe) -> Result<Gc<String>, Error> {
    pop_args_from_stack!(interp, receiver => StringLike, from => i32, to => i32);

    let from = usize::try_from(from - 1)?;
    let to = usize::try_from(to)?;

    let string = receiver.as_str(|sym| universe.lookup_symbol(sym));

    Ok(universe.gc_interface.alloc(string.chars().skip(from).take(to - from).collect()))
}

#[cfg(feature = "idiomatic")]
fn prim_substring_from_to(interp: &mut Interpreter, universe: &mut Universe) -> Result<Gc<String>, Error> {
    pop_args_from_stack!(interp, receiver => Value, from => i32, to => i32);

    let from = usize::try_from(from - 1)?;
    let to = usize::try_from(to)?;

    // let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];
    let string = match receiver.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };

    Ok(universe.gc_interface.alloc(string.chars().skip(from).take(to - from).collect()))
}

#[cfg(any(feature = "l3bits", feature = "l4bits"))]
fn char_at(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => StringLike, idx => i32);
    let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    let chr = *string.as_bytes().get((idx - 1) as usize).unwrap();
    let mut word: i64 = 0x00FF_FFFF_FFFF_FFFF;
    word = (word & !(0xFFi64 << 0 )) | ((chr as i64) << 0 );
    Ok(Value::TinyStr(word))
}

#[cfg(feature = "idiomatic")]
fn char_at(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => Value, idx => i32);

    #[inline]
    fn tinystring_as_str<'a>(value: i64, buf: &'a mut [u8; 7]) -> &'a str {
        let v = value as u64;
        for i in 0..7 {
            let b = ((v >> (i * 8)) & 0xFF) as u8;
            if b == 0xFF {
                return unsafe { std::str::from_utf8_unchecked(&buf[..i]) };
            }
            buf[i] = b;
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..7]) }
    }

    let mut buf = [0u8; 7];
    let string = match receiver.0 {
        ValueEnum::TinyStr(value) => tinystring_as_str(value, &mut buf),
        ValueEnum::String(ref value) => value.as_str(),
        ValueEnum::Symbol(sym) => universe.lookup_symbol(sym),
        _ => panic!()
    };
    let chr = *string.as_bytes().get((idx - 1) as usize).unwrap();
    let mut word: i64 = 0x00FF_FFFF_FFFF_FFFF;
    word = (word & !(0xFFi64 << 0 )) | ((chr as i64) << 0 );
    Ok(Value::TinyStr(word))
}

#[cfg(feature = "nan")]
fn char_at(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    pop_args_from_stack!(interp, receiver => StringLike, idx => i32);
    let string = receiver.as_str(|sym| universe.lookup_symbol(sym));
    let char = *string.as_bytes().get((idx - 1) as usize).unwrap();
    Ok(Value::Char(char.into()))
}

/// Search for an instance primitive matching the given signature.
pub fn get_instance_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    INSTANCE_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}

/// Search for a class primitive matching the given signature.
pub fn get_class_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    CLASS_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}
