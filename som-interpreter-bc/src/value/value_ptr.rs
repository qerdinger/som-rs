use crate::gc::VecValue;

#[cfg(feature = "nan")]
use som_value::value::{ARRAY_TAG, BLOCK_TAG, CLASS_TAG, INSTANCE_TAG, INVOKABLE_TAG};

#[cfg(feature = "lbits")]
use som_value::value::{ARRAY_TAG, BLOCK_TAG, CLASS_TAG, INSTANCE_TAG, INVOKABLE_TAG};

use crate::value::Value;
use crate::vm_objects::block::Block;
use crate::vm_objects::class::Class;
use crate::vm_objects::instance::Instance;
use crate::vm_objects::method::Method;
use som_gc::gcref::Gc;
use std::ops::Deref;

#[cfg(any(feature = "nan", feature = "lbits"))]
use som_value::value_ptr::{HasPointerTag, TypedPtrValue};

use std::marker::PhantomData;
use crate::value::value_enum::ValueEnum;

#[repr(transparent)]
pub struct TypedPtrValue<T> {
    value: Value,
    _phantom: PhantomData<T>,
}

pub trait HasPointerTag {
    fn get_tag() -> ValueEnum;
}
impl<T> TypedPtrValue<T> {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        matches!(
            &self.value.0,
            ValueEnum::Array(_)
                | ValueEnum::Block(_)
                | ValueEnum::Class(_)
                | ValueEnum::Instance(_)
                | ValueEnum::Invokable(_)
        )
    }

    #[inline(always)]
    pub fn get(&self) -> Option<Gc<T>> {
        match &self.value.0 {
            ValueEnum::Array(ptr) => ptr,
            ValueEnum::Block(ptr) => ptr,
            ValueEnum::Class(ptr) => ptr,
            ValueEnum::Instance(ptr) => ptr,
            ValueEnum::Invokable(ptr) => ptr,
            _ => None,
        }
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> Gc<T> {
        debug_assert!(self.get().is_some());
        self.get().unwrap_unchecked()
    }
}

impl<T> From<Value> for TypedPtrValue<T> {
    fn from(value: Value) -> Self {
        TypedPtrValue::new(value)
    }
}

impl<T> From<TypedPtrValue<T>> for Value {
    fn from(val: TypedPtrValue<T>) -> Self {
        val.value
    }
}
/*
impl<T, PTR> From<Value> for TypedPtrValue<T, PTR> {
    fn from(value: Value) -> Self {
        Self {
            value,
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }
}
*/
/*
impl<T, PTR> From<TypedPtrValue<T, PTR>> for Value {
    fn from(val: TypedPtrValue<T, PTR>) -> Self {
        val.value
    }
}

impl<T> From<Value> for TypedPtrValue<T, Gc<T>> {
    fn from(value: Value) -> Self {
        value.0.into()
    }
}

impl<T> From<TypedPtrValue<T, Gc<T>>> for Value {
    fn from(val: TypedPtrValue<T, Gc<T>>) -> Self {
        Value(val.into())
    }
}
 */
