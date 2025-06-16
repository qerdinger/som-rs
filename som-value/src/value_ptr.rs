use std::{marker::PhantomData, ops::Deref};

use num_bigint::BigInt;

#[cfg(feature = "nan")]
use crate::nan::value::{BaseValue, BIG_INTEGER_TAG, STRING_TAG};

#[cfg(feature = "lbits")]
use crate::lbits::value::{BaseValue, BIG_INTEGER_TAG, STRING_TAG, DOUBLE_BOXED_TAG};

/// Bundles a value to a pointer with the type to its pointer.
#[repr(transparent)]
pub struct TypedPtrValue<T, PTR> {
    value: BaseValue,
    _phantom: PhantomData<T>,
    _phantom2: PhantomData<PTR>,
}

pub trait HasPointerTag {
    fn get_tag() -> u64;
}

impl<T, PTR> TypedPtrValue<T, PTR>
where
    T: HasPointerTag,
    PTR: Deref<Target = T> + Into<u64> + From<u64>,
{
    pub fn new(value: PTR) -> Self {
        let ptr: u64 = value.into();
        Self {
            value: BaseValue::new(T::get_tag(), ptr),
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.value.tag() == T::get_tag()
    }

    /// Returns the underlying pointer value.
    #[inline(always)]
    pub fn get(&self) -> Option<PTR> {
        self.is_valid().then(|| {
            PTR::from(self.value.extract_pointer_bits())
        })
    }

    /// Returns the underlying pointer value, without checking if it is valid.
    /// # Safety
    /// Fine to invoke so long as we've previously checked we're working with a valid pointer.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> PTR {
        debug_assert!(self.get().is_some());
        PTR::from(self.value.extract_pointer_bits())
    }
}

impl<T, PTR> From<BaseValue> for TypedPtrValue<T, PTR> {
    fn from(value: BaseValue) -> Self {
        Self {
            value,
            _phantom: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

impl<T, PTR> From<TypedPtrValue<T, PTR>> for BaseValue {
    fn from(val: TypedPtrValue<T, PTR>) -> Self {
        val.value
    }
}

impl HasPointerTag for String {
    fn get_tag() -> u64 {
        STRING_TAG
    }
}

impl HasPointerTag for BigInt {
    fn get_tag() -> u64 {
        BIG_INTEGER_TAG
    }
}

#[cfg(feature = "lbits")]
impl HasPointerTag for f64 {
    fn get_tag() -> u64 {
        DOUBLE_BOXED_TAG
    }
}