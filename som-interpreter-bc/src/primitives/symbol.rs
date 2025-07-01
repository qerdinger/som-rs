use anyhow::Error;
use once_cell::sync::Lazy;
use som_gc::gc_interface::SOMAllocator;
use som_gc::gcref::Gc;

use crate::cur_frame;
use crate::interpreter::Interpreter;
use crate::primitives::PrimInfo;
use crate::primitives::PrimitiveFn;
use crate::universe::Universe;
use crate::value::convert::Primitive;
use crate::value::Value;

pub static INSTANCE_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| Box::new([("asString", self::as_string.into_func(), true)]));
pub static CLASS_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| Box::new([]));

#[cfg(feature = "nan")]
fn as_string(interp: &mut Interpreter, universe: &mut Universe) -> Result<Gc<String>, Error> {
    let symbol = cur_frame!(interp).stack_pop().as_symbol().unwrap();
    Ok(universe.gc_interface.alloc(universe.lookup_symbol(symbol).to_owned()))
}

#[cfg(feature = "lbits")]
fn as_string(interp: &mut Interpreter, universe: &mut Universe) -> Result<Value, Error> {
    let symbol = cur_frame!(interp).stack_pop().as_symbol().unwrap();
    let val = universe.lookup_symbol(symbol).to_owned();
    let val_len = val.len();

    // println!("SYMBOL_AS_STRING: {}", val_len);
    // println!("SYMBOL : {}", val);

    // if val_len < 8 {
    //     let mut data_buf = [0u8; 8];
    //     data_buf[..val_len].copy_from_slice((*val).as_bytes());
    //     // println!("buf : {:?}", data_buf);
    //     // println!("readable : {}", std::str::from_utf8(&data_buf).unwrap());
    //     return Ok(Value::TinyStr(data_buf));
    // }

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
