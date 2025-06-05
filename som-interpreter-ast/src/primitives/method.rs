use super::PrimInfo;
use crate::get_args_from_stack;
use crate::invokable::Return;
use crate::primitives::PrimitiveFn;
use crate::universe::{GlobalValueStack, Universe};
use crate::value::convert::FromArgs;
use crate::value::convert::Primitive;
use crate::value::Value;
use crate::vm_objects::method::Method;
use anyhow::Error;
use once_cell::sync::Lazy;
use som_gc::gcref::Gc;

pub static INSTANCE_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| {
    Box::new([
        ("holder", self::holder.into_func(), true),
        ("signature", self::signature.into_func(), true),
        ("invokeOn:with:", self::invoke_on_with.into_func(), true),
    ])
});
pub static CLASS_PRIMITIVES: Lazy<Box<[PrimInfo]>> = Lazy::new(|| Box::new([]));

fn holder(invokable: Gc<Method>) -> Result<Value, Error> {
    let holder = invokable.holder();
    Ok(Value::Class(holder.clone()))

    // match maybe_holder {
    //     Some(holder) => Ok(Value::Class(holder)),
    //     None => bail!(format!(
    //         "'{}': method holder has been collected",
    //         SIGNATURE
    //     )),
    // }
}

fn signature(universe: &mut Universe, stack: &mut GlobalValueStack) -> Result<Value, Error> {
    get_args_from_stack!(stack, invokable => Gc<Method>);
    let sym = universe.intern_symbol(invokable.signature());
    Ok(Value::Symbol(sym))
}

#[allow(unused)]
fn invoke_on_with(
    universe: &mut Universe,
    value_stack: &mut GlobalValueStack,
    //mut invokable: HeapValPtr<Method>,
    //receiver: Value,
    //arguments: VecValue,
) -> Result<Return, Error> {
    todo!()
    // let args = std::iter::once(receiver).chain(arguments.iter().cloned()).collect();

    // Ok(invokable.invoke(universe, args))
}

/// Search for an instance primitive matching the given signature.
pub fn get_instance_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    INSTANCE_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}

/// Search for a class primitive matching the given signature.
pub fn get_class_primitive(signature: &str) -> Option<&'static PrimitiveFn> {
    CLASS_PRIMITIVES.iter().find(|it| it.0 == signature).map(|it| it.1)
}
