use std::convert::TryFrom;

use anyhow::{bail, Context, Error};
use som_gc::gcslice::GcSlice;
use som_value::interned::Interned;
use som_value::value::BaseValue;
use som_value::value_ptr::HasPointerTag;

use crate::gc::VecValue;
use crate::invokable::Return;
use crate::primitives::PrimitiveFn;
use crate::universe::{GlobalValueStack, Universe};
use crate::value::Value;
use crate::vm_objects::block::Block;
use crate::vm_objects::class::Class;
use crate::vm_objects::instance::Instance;
use crate::vm_objects::method::Method;
use num_bigint::BigInt;
use som_gc::gcref::Gc;

pub type DoubleLike = som_value::convert::DoubleLike<Gc<BigInt>>;
pub type IntegerLike = som_value::convert::IntegerLike<Gc<BigInt>>;
pub type StringLike = som_value::convert::StringLike<Gc<String>>;

pub trait IntoValue {
    #[allow(clippy::wrong_self_convention)] // though i guess we could/should rename it
    fn into_value(&self) -> Value;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nil;

impl TryFrom<Value> for Nil {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if value == Value::NIL {
            Ok(Self)
        } else {
            bail!("could not resolve `Value` as `Nil`")
        }
    }
}

impl FromArgs for Nil {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Self::try_from(arg)
    }
}

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
        i32::try_from(arg.0)
    }
}

impl FromArgs for f64 {
    fn from_args(arg: Value) -> Result<Self, Error> {
        f64::try_from(arg.0)
    }
}

impl FromArgs for Interned {
    fn from_args(arg: Value) -> Result<Self, Error> {
        arg.as_symbol().context("could not resolve `Value` as `Symbol`")
    }
}

impl FromArgs for VecValue {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Ok(VecValue(GcSlice::from(arg.extract_pointer_bits())))
    }
}

impl FromArgs for Return {
    fn from_args(arg: Value) -> Result<Self, Error> {
        Ok(Return::Local(arg))
    }
}

impl<T> FromArgs for Gc<T>
where
    T: HasPointerTag,
{
    fn from_args(arg: Value) -> Result<Self, Error> {
        Ok(arg.as_value_gc_ptr::<T>().unwrap())
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

impl IntoValue for Interned {
    fn into_value(&self) -> Value {
        Value::Symbol(*self)
    }
}

impl IntoValue for Gc<String> {
    fn into_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl IntoValue for char {
    fn into_value(&self) -> Value {
        BaseValue::Char(*self).into()
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
    fn invoke(&self, universe: &mut Universe, value_stack: &mut GlobalValueStack, nbr_args: usize) -> Return;

    fn into_func(self) -> &'static PrimitiveFn {
        let boxed = Box::new(move |universe: &mut Universe, value_stack: &mut GlobalValueStack, nbr_args: usize| {
            self.invoke(universe, value_stack, nbr_args)
        });
        Box::leak(boxed)
    }
}

pub trait IntoReturn {
    fn into_return(self) -> Return;
}

impl<T: IntoValue> IntoReturn for T {
    fn into_return(self) -> Return {
        Return::Local(self.into_value())
    }
}

impl IntoReturn for Return {
    fn into_return(self) -> Return {
        self
    }
}

impl IntoValue for Value {
    fn into_value(&self) -> Value {
        *self
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

impl IntoValue for DoubleLike {
    fn into_value(&self) -> Value {
        match self {
            DoubleLike::Double(value) => value.into_value(),
            DoubleLike::Integer(value) => value.into_value(),
            DoubleLike::BigInteger(value) => value.into_value(),
        }
    }
}

pub struct NoInterp {}

/// Automatically derive primitive definitions, for functions that do not need the universe or
/// direct access to the value stack.
///
/// The reason is that a function can trigger GC if it has access to the universe (and therefore
/// the allocator), and that any function that can trigger GC MUST manage the value stack directly
/// to be wary of the possibility of a collection happening.
///
/// A few functions may need the universe but never trigger GC, thus could take the universe as a
/// parameter safely. But to be safe, we make no exceptions.
macro_rules! derive_stuff {
    ($($ty:ident),* $(,)?) => {
        impl <F, R, $($ty),*> $crate::value::convert::Primitive<(NoInterp, $($ty),*,)> for F
        where
            F: Fn($($ty),*) -> Result<R, Error> + Send + Sync + 'static,
            R: $crate::value::convert::IntoReturn,
            $(for<'a> $ty: $crate::value::convert::FromArgs),*,
        {
            fn invoke(&self, _: &mut Universe, value_stack: &mut GlobalValueStack, nbr_args: usize) -> Return {
                let mut args_iter = value_stack.drain_n_last(nbr_args);
                $(
                    #[allow(non_snake_case)]
                    let $ty = $ty::from_args(args_iter.next().unwrap()).unwrap();
                )*

                let result = (self)($($ty),*,).unwrap();
                result.into_return()
            }
        }
    };
}

derive_stuff!(_A);
derive_stuff!(_A, _B);
derive_stuff!(_A, _B, _C);
derive_stuff!(_A, _B, _C, _D);

// For functions that need the universe (likely for GC) and therefore know what they're doing.
// Have access to the universe and all values, but manage them yourself.
impl<F, R> Primitive<()> for F
where
    F: Fn(&mut Universe, &mut GlobalValueStack) -> Result<R, Error> + Send + Sync + 'static,
    R: IntoReturn,
{
    fn invoke(&self, universe: &mut Universe, value_stack: &mut GlobalValueStack, _nbr_args: usize) -> Return {
        let result = self(universe, value_stack).unwrap();
        result.into_return()
    }
}
