use crate::gc::VecValue;
use crate::interpreter::Interpreter;
use crate::pop_args_from_stack;
use crate::primitives::PrimInfo;
use crate::primitives::PrimitiveFn;
use crate::universe::Universe;
use crate::value::convert::Primitive;
use crate::value::Value;
use crate::vm_objects::class::Class;
use crate::vm_objects::method::{Invoke, Method};
use anyhow::Error;
use once_cell::sync::Lazy;
use som_gc::gcref::Gc;
use som_value::interned::Interned;

#[cfg(feature = "l3bits")]
use som_gc::gc_interface::SOMAllocator;

pub static INSTANCE_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| {
    Box::new([
        ("holder", self::holder.into_func(), true),
        ("signature", self::signature.into_func(), true),
        ("invokeOn:with:", self::invoke_on_with.into_func(), true),
    ])
});
pub static CLASS_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| Box::new([]));

fn holder(invokable: Gc<Method>) -> Result<Gc<Class>, Error> {
    Ok(invokable.holder().clone())
}

#[cfg(feature = "l3bits")]
fn signature(interp: &mut Interpreter, universe: &mut Universe) -> Result<Gc<Interned>, Error> {
    pop_args_from_stack!(interp, invokable => Gc<Method>);
    let sym: Interned = universe.intern_symbol(invokable.signature());
    Ok(universe.gc_interface.alloc(sym))
}

#[cfg(any(feature = "nan", feature = "idiomatic", feature = "l4bits"))]
fn signature(interp: &mut Interpreter, universe: &mut Universe) -> Result<Interned, Error> {
    pop_args_from_stack!(interp, invokable => Gc<Method>);
    Ok(universe.intern_symbol(invokable.signature()))
}

#[cfg(feature = "idiomatic")]
fn invoke_on_with(interpreter: &mut Interpreter, universe: &mut Universe) -> Result<(), Error> {
    pop_args_from_stack!(interpreter, invokable => Gc<Method>, receiver => Value, arguments => VecValue);

    // TODO: this should NOT pop. a frame allocation causes a GC bug here, as far as I know.

    invokable.invoke(
        interpreter,
        universe,
        receiver,
        arguments.iter().cloned().collect(), // todo lame to clone tbh
    );
    Ok(())
}

#[cfg(not(feature = "idiomatic"))]
fn invoke_on_with(interpreter: &mut Interpreter, universe: &mut Universe) -> Result<(), Error> {
    pop_args_from_stack!(interpreter, invokable => Gc<Method>, receiver => Value, arguments => VecValue);

    // TODO: this should NOT pop. a frame allocation causes a GC bug here, as far as I know.

    invokable.invoke(
        interpreter,
        universe,
        receiver,
        arguments.iter().copied().collect(), // todo lame to clone tbh
    );
    Ok(())
}

/// Search for an instance primitive matching the given signature.
pub fn get_instance_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    INSTANCE_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}

/// Search for a class primitive matching the given signature.
pub fn get_class_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    CLASS_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}
