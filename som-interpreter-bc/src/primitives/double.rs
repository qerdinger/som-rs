use crate::interpreter::Interpreter;
use crate::pop_args_from_stack;
use crate::primitives::PrimInfo;
use crate::primitives::PrimitiveFn;
use crate::universe::Universe;
use crate::value::convert::{DoubleLike, IntoValue, Primitive};
use crate::value::Value;
use anyhow::{Context, Error};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use som_gc::gc_interface::SOMAllocator;
use som_gc::gcref::Gc;

pub static INSTANCE_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| {
    Box::new([
        ("<", self::lt.into_func(), true),
        ("<=", self::lt_or_eq.into_func(), true),
        (">", self::gt.into_func(), true),
        (">=", self::gt_or_eq.into_func(), true),
        ("=", self::eq.into_func(), true),
        ("~=", self::uneq.into_func(), true),
        ("<>", self::uneq.into_func(), true),
        ("==", self::eq_eq.into_func(), true),
        // -----------------
        ("+", self::plus.into_func(), true),
        ("-", self::minus.into_func(), true),
        ("*", self::times.into_func(), true),
        ("//", self::divide.into_func(), true),
        ("%", self::modulo.into_func(), true),
        ("sqrt", self::sqrt.into_func(), true),
        ("max:", self::max.into_func(), true),
        ("min:", self::min.into_func(), true),
        ("round", self::round.into_func(), true),
        ("cos", self::cos.into_func(), true),
        ("sin", self::sin.into_func(), true),
        ("asString", self::as_string.into_func(), true),
        ("asInteger", self::as_integer.into_func(), true),
    ])
});
pub static CLASS_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| {
    Box::new([
        ("fromString:", self::from_string.into_func(), true),
        ("PositiveInfinity", self::positive_infinity.into_func(), true),
    ])
});

macro_rules! promote {
    ($signature:expr, $value:expr) => {
        match $value {
            DoubleLike::Double(value) => value,
            // DoubleLike::AllocatedDouble(value) => *value,
            DoubleLike::Integer(value) => value as f64,
            DoubleLike::BigInteger(value) => match value.to_f64() {
                Some(value) => value,
                None => {
                    panic!("'{}': `Integer` too big to be converted to `Double`", $signature)
                }
            },
            _ => panic!("Undefined!")
        }
    };
}

fn from_string(_: Value, string: Gc<String>) -> Result<f64, Error> {
    const SIGNATURE: &str = "Double>>#fromString:";

    string.parse().with_context(|| format!("`{SIGNATURE}`: could not parse `f64` from string"))
}

fn as_string(interp: &mut Interpreter, universe: &mut Universe) -> Result<Gc<String>, Error> {
    const SIGNATURE: &str = "Double>>#asString";

    pop_args_from_stack!(interp, receiver => DoubleLike);

    let receiver = promote!(SIGNATURE, receiver);

    Ok(universe.gc_interface.alloc(receiver.to_string()))
}

fn as_integer(receiver: f64) -> Result<i32, Error> {
    const _: &str = "Double>>#asInteger";

    Ok(receiver.trunc() as i32)
}

fn sqrt(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#sqrt";

    
    pop_args_from_stack!(interp, receiver => DoubleLike);
    let receiver = promote!(SIGNATURE, receiver);
    let ops = receiver.sqrt();

    let bits = ops.to_bits();
    let exponent  = (bits >> 52) & 0x7FF;
    let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
    let heap = &mut universe.gc_interface;
    //println!("In Range : {}", in_range);
    if in_range { Ok(Value::Double(ops)) } else { Ok(Value::AllocatedDouble(heap.alloc(ops))) }
}

fn max(receiver: f64, other: DoubleLike) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#max";

    let other_val = promote!(SIGNATURE, other);
    match other_val >= receiver {
        true => Ok(other_val.into_value()),
        false => Ok(receiver.into_value()),
    }
}

fn min(receiver: f64, other: DoubleLike) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#min";

    let other_val = promote!(SIGNATURE, other);
    match other_val >= receiver {
        true => Ok(receiver.into_value()),
        false => Ok(other_val.into_value()),
    }
}

fn round(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#round";

    pop_args_from_stack!(interp, receiver => DoubleLike);
    let receiver = promote!(SIGNATURE, receiver);
    let ops = receiver.round();

    let bits = ops.to_bits();
    let exponent  = (bits >> 52) & 0x7FF;
    let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
    let heap = &mut universe.gc_interface;
    //println!("In Range : {}", in_range);
    if in_range { Ok(Value::Double(ops)) } else { Ok(Value::AllocatedDouble(heap.alloc(ops))) }
}

fn cos(receiver: DoubleLike) -> Result<f64, Error> {
    const SIGNATURE: &str = "Double>>#cos";

    let receiver = promote!(SIGNATURE, receiver);

    Ok(receiver.cos())
}

fn sin(receiver: DoubleLike) -> Result<f64, Error> {
    const SIGNATURE: &str = "Double>>#sin";

    let receiver = promote!(SIGNATURE, receiver);

    Ok(receiver.sin())
}

fn eq(a: Value, b: Value) -> Result<bool, Error> {
    let Ok(a) = DoubleLike::try_from(a.0) else {
        return Ok(false);
    };

    let Ok(b) = DoubleLike::try_from(b.0) else {
        return Ok(false);
    };

    Ok(DoubleLike::eq(&a, &b))
}

fn eq_eq(a: Value, b: Value) -> Result<bool, Error> {
    let Ok(a) = DoubleLike::try_from(a.0) else {
        return Ok(false);
    };

    let Ok(b) = DoubleLike::try_from(b.0) else {
        return Ok(false);
    };

    match (a, b) {
        (DoubleLike::Double(a), DoubleLike::Double(b)) => Ok(a == b),
        _ => Ok(false),
    }
}

fn uneq(a: Value, b: Value) -> Result<bool, Error> {
    let Ok(a) = DoubleLike::try_from(a.0) else {
        return Ok(false);
    };

    let Ok(b) = DoubleLike::try_from(b.0) else {
        return Ok(false);
    };

    Ok(!DoubleLike::eq(&a, &b))
}

fn lt(a: f64, b: DoubleLike) -> Result<bool, Error> {
    const SIGNATURE: &str = "Double>>#<";
    Ok(a < promote!(SIGNATURE, b))
}

fn lt_or_eq(a: f64, b: DoubleLike) -> Result<bool, Error> {
    const SIGNATURE: &str = "Double>>#<=";
    Ok(a <= promote!(SIGNATURE, b))
}

fn gt(a: f64, b: DoubleLike) -> Result<bool, Error> {
    const SIGNATURE: &str = "Double>>#>";
    Ok(a > promote!(SIGNATURE, b))
}

fn gt_or_eq(a: f64, b: DoubleLike) -> Result<bool, Error> {
    const SIGNATURE: &str = "Double>>#>=";
    Ok(a >= promote!(SIGNATURE, b))
}

macro_rules! demote {
    ($heap:expr, $expr:expr) => {{
        let value = $expr;
        match value.to_i32() {
            Some(value) => Value::Integer((value)),
            None => Value::BigInteger($heap.alloc(value)),
        }
    }};
}

fn plus(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#+";

    pop_args_from_stack!(interp, a => DoubleLike, b => DoubleLike);
    let heap = &mut universe.gc_interface;

    let value = match (a, b) {
        (DoubleLike::Integer(a), DoubleLike::Integer(b)) => match a.checked_add(b) {
            Some(value) => Value::Integer(value),
            None => demote!(heap, BigInt::from(a) + BigInt::from(b)),
        },
        (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => {
            demote!(heap, &*a + &*b)
        }
        (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) | (DoubleLike::Integer(b), DoubleLike::BigInteger(a)) => {
            demote!(heap, &*a + BigInt::from(b))
        }
        (DoubleLike::Double(a), DoubleLike::Double(b)) => {
            let tot = a + b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::Integer(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::Integer(a)) => {
            //println!("HERE total(2) : {}", a as f64 + b);
            let tot = a as f64 + b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::BigInteger(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::BigInteger(a)) => match a.to_f64() {
            Some(a) => {
                let tot = a + b;
                let bits = tot.to_bits();
                let exponent  = (bits >> 52) & 0x7FF;
                let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
                //println!("In Range : {}", in_range);
                if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
            },
            None => panic!("'{}': `Integer` too big to be converted to `Double`", SIGNATURE),
        },
        (_, _) => panic!("Undefined!")
    };

    Ok(value)
}

fn minus(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#-";

    pop_args_from_stack!(interp, a => DoubleLike, b => DoubleLike);
    let heap = &mut universe.gc_interface;

    let value = match (a, b) {
        (DoubleLike::Integer(a), DoubleLike::Integer(b)) => match a.checked_add(b) {
            Some(value) => Value::Integer(value),
            None => demote!(heap, BigInt::from(a) - BigInt::from(b)),
        },
        (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => {
            demote!(heap, &*a - &*b)
        }
        (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) | (DoubleLike::Integer(b), DoubleLike::BigInteger(a)) => {
            demote!(heap, &*a - BigInt::from(b))
        }
        (DoubleLike::Double(a), DoubleLike::Double(b)) => {
            let tot = a - b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::Integer(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::Integer(a)) => {
            //println!("HERE total(2) : {}", a as f64 + b);
            let tot = a as f64 - b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::BigInteger(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::BigInteger(a)) => match a.to_f64() {
            Some(a) => {
                let tot = a - b;
                let bits = tot.to_bits();
                let exponent  = (bits >> 52) & 0x7FF;
                let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
                //println!("In Range : {}", in_range);
                if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
            },
            None => panic!("'{}': `Integer` too big to be converted to `Double`", SIGNATURE),
        },
        (_, _) => panic!("Undefined!")
    };

    Ok(value)
}

fn times(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#*";

    pop_args_from_stack!(interp, a => DoubleLike, b => DoubleLike);
    let heap = &mut universe.gc_interface;

    let value = match (a, b) {
        (DoubleLike::Integer(a), DoubleLike::Integer(b)) => match a.checked_add(b) {
            Some(value) => Value::Integer(value),
            None => demote!(heap, BigInt::from(a) * BigInt::from(b)),
        },
        (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => {
            demote!(heap, &*a * &*b)
        }
        (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) | (DoubleLike::Integer(b), DoubleLike::BigInteger(a)) => {
            demote!(heap, &*a * BigInt::from(b))
        }
        (DoubleLike::Double(a), DoubleLike::Double(b)) => {
            let tot = a * b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::Integer(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::Integer(a)) => {
            //println!("HERE total(2) : {}", a as f64 + b);
            let tot = a as f64 * b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::BigInteger(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::BigInteger(a)) => match a.to_f64() {
            Some(a) => {
                let tot = a * b;
                let bits = tot.to_bits();
                let exponent  = (bits >> 52) & 0x7FF;
                let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
                //println!("In Range : {}", in_range);
                if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
            },
            None => panic!("'{}': `Integer` too big to be converted to `Double`", SIGNATURE),
        },
        (_, _) => panic!("Undefined!")
    };

    Ok(value)
}

fn divide(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#//";

    pop_args_from_stack!(interp, a => DoubleLike, b => DoubleLike);
    let heap = &mut universe.gc_interface;

    let value = match (a, b) {
        (DoubleLike::Integer(a), DoubleLike::Integer(b)) => match a.checked_add(b) {
            Some(value) => Value::Integer(value),
            None => demote!(heap, BigInt::from(a) / BigInt::from(b)),
        },
        (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => {
            demote!(heap, &*a / &*b)
        }
        (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) | (DoubleLike::Integer(b), DoubleLike::BigInteger(a)) => {
            demote!(heap, &*a / BigInt::from(b))
        }
        (DoubleLike::Double(a), DoubleLike::Double(b)) => {
            let tot = a / b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::Integer(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::Integer(a)) => {
            //println!("HERE total(2) : {}", a as f64 + b);
            let tot = a as f64 / b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::BigInteger(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::BigInteger(a)) => match a.to_f64() {
            Some(a) => {
                let tot = a / b;
                let bits = tot.to_bits();
                let exponent  = (bits >> 52) & 0x7FF;
                let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
                //println!("In Range : {}", in_range);
                if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
            },
            None => panic!("'{}': `Integer` too big to be converted to `Double`", SIGNATURE),
        },
        (_, _) => panic!("Undefined!")
    };

    Ok(value)
}

fn modulo(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const SIGNATURE: &str = "Double>>#%";

    pop_args_from_stack!(interp, a => DoubleLike, b => DoubleLike);
    let heap = &mut universe.gc_interface;

    let value = match (a, b) {
        (DoubleLike::Integer(a), DoubleLike::Integer(b)) => match a.checked_add(b) {
            Some(value) => Value::Integer(value),
            None => demote!(heap, BigInt::from(a) / BigInt::from(b)),
        },
        (DoubleLike::BigInteger(a), DoubleLike::BigInteger(b)) => {
            demote!(heap, &*a % &*b)
        }
        (DoubleLike::BigInteger(a), DoubleLike::Integer(b)) | (DoubleLike::Integer(b), DoubleLike::BigInteger(a)) => {
            demote!(heap, &*a % BigInt::from(b))
        }
        (DoubleLike::Double(a), DoubleLike::Double(b)) => {
            let tot = a % b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::Integer(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::Integer(a)) => {
            //println!("HERE total(2) : {}", a as f64 + b);
            let tot = a as f64 % b;
            let bits = tot.to_bits();
            let exponent  = (bits >> 52) & 0x7FF;
            let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
            //println!("In Range : {}", in_range);
            if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
        },
        (DoubleLike::BigInteger(a), DoubleLike::Double(b)) | (DoubleLike::Double(b), DoubleLike::BigInteger(a)) => match a.to_f64() {
            Some(a) => {
                let tot = a % b;
                let bits = tot.to_bits();
                let exponent  = (bits >> 52) & 0x7FF;
                let in_range = (exponent >= 0x380 && exponent <= 0x47F) || bits == 0 || bits == 1;
                //println!("In Range : {}", in_range);
                if in_range { Value::Double(tot) } else { Value::AllocatedDouble(heap.alloc(tot)) }
            },
            None => panic!("'{}': `Integer` too big to be converted to `Double`", SIGNATURE),
        },
        (_, _) => panic!("Undefined!")
    };

    Ok(value)
}

fn positive_infinity(_: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    const _: &str = "Double>>#positiveInfinity";

    let heap = &mut universe.gc_interface;
    Ok(Value::AllocatedDouble(heap.alloc(f64::INFINITY)))
}

/// Search for an instance primitive matching the given signature.
pub fn get_instance_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    INSTANCE_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}

/// Search for a class primitive matching the given signature.
pub fn get_class_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    CLASS_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}
