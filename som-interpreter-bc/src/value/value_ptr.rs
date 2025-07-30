#[cfg(not(feature = "idiomatic"))]
use crate::gc::VecValue;

#[cfg(feature = "nan")]
use som_value::value::{ARRAY_TAG, BLOCK_TAG, CLASS_TAG, INSTANCE_TAG, INVOKABLE_TAG};

#[cfg(feature = "l4bits")]
use som_value::value::{ARRAY_TAG, BLOCK_TAG, CLASS_TAG, INSTANCE_TAG, INVOKABLE_TAG};

use crate::value::Value;
use crate::vm_objects::block::Block;
use crate::vm_objects::class::Class;
use crate::vm_objects::instance::Instance;
use crate::vm_objects::method::Method;
use som_gc::gcref::Gc;

#[cfg(feature = "idiomatic")]
use std::marker::PhantomData;

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
use som_value::value_ptr::{HasPointerTag, TypedPtrValue};

#[cfg(feature = "idiomatic")]
use crate::value::value_enum::ValueEnum;

#[cfg(feature = "idiomatic")]
#[repr(transparent)]
pub struct TypedPtrValue<T> {
    value: Value,
    _phantom: PhantomData<T>,
}

#[cfg(feature = "idiomatic")]
pub trait GetPtr<T> {
    fn get(&self) -> Option<Gc<T>>;
}

#[cfg(feature = "idiomatic")]
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
}

#[cfg(feature = "idiomatic")]
impl<T> TypedPtrValue<T>
where
    TypedPtrValue<T>: GetPtr<T>,
{
    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> Gc<T> {
        debug_assert!(self.get().is_some());
        self.get().unwrap_unchecked()
    }
}

#[cfg(feature = "idiomatic")]
impl<T> From<Value> for TypedPtrValue<T> {
    fn from(value: Value) -> Self {
        TypedPtrValue::new(value)
    }
}

#[cfg(feature = "idiomatic")]
impl<T> From<TypedPtrValue<T>> for Value {
    fn from(val: TypedPtrValue<T>) -> Self {
        val.value
    }
}

#[cfg(feature = "idiomatic")]
impl GetPtr<Class> for TypedPtrValue<Class> {
    fn get(&self) -> Option<Gc<Class>> {
        match &self.value.0 {
            ValueEnum::Class(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}

#[cfg(feature = "idiomatic")]
impl GetPtr<Block> for TypedPtrValue<Block> {
    fn get(&self) -> Option<Gc<Block>> {
        match &self.value.0 {
            ValueEnum::Block(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}

#[cfg(feature = "idiomatic")]
impl GetPtr<Instance> for TypedPtrValue<Instance> {
    fn get(&self) -> Option<Gc<Instance>> {
        match &self.value.0 {
            ValueEnum::Instance(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}

#[cfg(feature = "idiomatic")]
impl GetPtr<Method> for TypedPtrValue<Method> {
    fn get(&self) -> Option<Gc<Method>> {
        match &self.value.0 {
            ValueEnum::Invokable(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
impl<T> From<Value> for TypedPtrValue<T, Gc<T>> {
    fn from(value: Value) -> Self {
        value.0.into()
    }
}

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
impl<T> From<TypedPtrValue<T, Gc<T>>> for Value {
    fn from(val: TypedPtrValue<T, Gc<T>>) -> Self {
        Value(val.into())
    }
}

#[cfg(any(feature = "nan", feature = "l4bits"))]
impl HasPointerTag for VecValue {
    fn get_tag() -> u64 {
        ARRAY_TAG
    }
}

#[cfg(feature = "l3bits")]
impl HasPointerTag for VecValue {
    fn get_tag() -> u64 {
        unreachable!("Not used in the Lower 3-bit implementation")
    }
}

#[cfg(any(feature = "nan", feature = "l4bits"))]
impl HasPointerTag for Block {
    fn get_tag() -> u64 {
        BLOCK_TAG
    }
}

#[cfg(feature = "l3bits")]
impl HasPointerTag for Block {
    fn get_tag() -> u64 {
        unreachable!("Not used in the Lower 3-bit implementation")
    }
}

#[cfg(any(feature = "nan", feature = "l4bits"))]
impl HasPointerTag for Class {
    fn get_tag() -> u64 {
        CLASS_TAG
    }
}

#[cfg(feature = "l3bits")]
impl HasPointerTag for Class {
    fn get_tag() -> u64 {
        unreachable!("Not used in the Lower 3-bit implementation")
    }
}

#[cfg(any(feature = "nan", feature = "l4bits"))]
impl HasPointerTag for Method {
    fn get_tag() -> u64 {
        INVOKABLE_TAG
    }
}

#[cfg(feature = "l3bits")]
impl HasPointerTag for Method {
    fn get_tag() -> u64 {
        unreachable!("Not used in the Lower 3-bit implementation")
    }
}

#[cfg(any(feature = "nan", feature = "l4bits"))]
impl HasPointerTag for Instance {
    fn get_tag() -> u64 {
        INSTANCE_TAG
    }
}

#[cfg(feature = "l3bits")]
impl HasPointerTag for Instance {
    fn get_tag() -> u64 {
        unreachable!("Not used in the Lower 3-bit implementation")
    }
}