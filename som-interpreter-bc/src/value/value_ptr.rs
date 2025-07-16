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
use std::marker::PhantomData;
use som_gc::gcslice::GcSlice;
#[cfg(any(feature = "nan", feature = "lbits"))]
use som_value::value_ptr::{HasPointerTag, TypedPtrValue};

use crate::value::value_enum::ValueEnum;

#[repr(transparent)]
pub struct TypedPtrValue<T> {
    value: Value,
    _phantom: PhantomData<T>,
}

pub trait HasPointerTag {
    fn get_tag() -> ValueEnum;
}

pub trait GetPtr<T> {
    fn get(&self) -> Option<Gc<T>>;
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
}

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
impl GetPtr<VecValue> for TypedPtrValue<VecValue> {
    fn get(&self) -> Option<Gc<VecValue>> {
        match &self.value.0 {
            ValueEnum::Array(ptr) => {
                //Gc::fro
                //Some(ptr.clone())
                let arrptr: u64 = *ptr;
                let ptr: u64 = arrptr.into();
                Some(VecValue(GcSlice::from(ptr)))
            },
            _ => None,
        }
    }
}
TODO
 */

impl GetPtr<Class> for TypedPtrValue<Class> {
    fn get(&self) -> Option<Gc<Class>> {
        match &self.value.0 {
            ValueEnum::Class(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}

impl GetPtr<Block> for TypedPtrValue<Block> {
    fn get(&self) -> Option<Gc<Block>> {
        match &self.value.0 {
            ValueEnum::Block(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}

impl GetPtr<Instance> for TypedPtrValue<Instance> {
    fn get(&self) -> Option<Gc<Instance>> {
        match &self.value.0 {
            ValueEnum::Instance(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}

impl GetPtr<Method> for TypedPtrValue<Method> {
    fn get(&self) -> Option<Gc<Method>> {
        match &self.value.0 {
            ValueEnum::Invokable(ptr) => Some(ptr.clone()),
            _ => None,
        }
    }
}
