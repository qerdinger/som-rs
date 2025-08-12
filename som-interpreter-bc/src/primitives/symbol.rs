use anyhow::Error;
use once_cell::sync::Lazy;
use som_gc::gc_interface::SOMAllocator;
use crate::cur_frame;
use crate::interpreter::Interpreter;
use crate::primitives::PrimInfo;
use crate::primitives::PrimitiveFn;
use crate::universe::Universe;
use crate::value::convert::Primitive;

#[cfg(feature = "nan")]
use som_gc::gcref::Gc;

#[cfg(any(feature = "l4bits", feature = "l3bits", feature = "idiomatic"))]
use crate::value::Value;

pub static INSTANCE_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| Box::new([("asString", self::as_string.into_func(), true)]));
pub static CLASS_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| Box::new([]));

#[cfg(feature = "nan")]
fn as_string(interp: &mut Interpreter, universe: &mut Universe) -> Result<Gc<String>, Error> {
    let symbol = cur_frame!(interp).stack_pop().as_symbol().unwrap();
    Ok(universe.gc_interface.alloc(universe.lookup_symbol(symbol).to_owned()))
}

#[cfg(any(feature = "l4bits", feature = "l3bits", feature = "idiomatic"))]
fn as_string(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    let symbol = cur_frame!(interp).stack_pop().as_symbol().unwrap();
    let val = universe.lookup_symbol(symbol).to_owned();
    let len = val.len();

    if len < 8 {
        let b = val.as_bytes();
        let mut word: i64 = 0x00FF_FFFF_FFFF_FFFF;
        if len > 0 { word = (word & !(0xFFi64 << 0 )) | ((b[0] as i64) << 0 ); }
        if len > 1 { word = (word & !(0xFFi64 << 8 )) | ((b[1] as i64) << 8 ); }
        if len > 2 { word = (word & !(0xFFi64 << 16)) | ((b[2] as i64) << 16); }
        if len > 3 { word = (word & !(0xFFi64 << 24)) | ((b[3] as i64) << 24); }
        if len > 4 { word = (word & !(0xFFi64 << 32)) | ((b[4] as i64) << 32); }
        if len > 5 { word = (word & !(0xFFi64 << 40)) | ((b[5] as i64) << 40); }
        if len > 6 { word = (word & !(0xFFi64 << 48)) | ((b[6] as i64) << 48); }
        return Ok(Value::TinyStr(word));
    }

    Ok(Value::String(universe.gc_interface.alloc(val)))
}

/// Search for an instance primitive matching the given signature.
pub fn get_instance_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    INSTANCE_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}

/// Search for a class primitive matching the given signature.
pub fn get_class_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    CLASS_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}
