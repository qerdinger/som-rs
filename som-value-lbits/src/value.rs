use crate::{
    interned::Interned,
    value_ptr::{HasPointerTag, TypedPtrValue},
};
use num_bigint::BigInt;
use std::ops::Deref;

static_assertions::const_assert_eq!(size_of::<f64>(), 8);
static_assertions::assert_eq_size!(f64, u64, *const ());

pub const TAG_BITS: u64 = 0b111;

/// Tag bits for the `Nil` type.
pub const NIL_TAG: u64 = 0b000;
/// Tag bits for the `System` type.
pub const SYSTEM_TAG: u64 = 0b001;
/// Tag bits for the `Integer` type.
pub const INTEGER_TAG: u64 = 0b010; // Same bit position as `BIG_INTEGER_TAG`
/// Tag bits for the `Boolean` type.
pub const BOOLEAN_TAG: u64 = 0b011;
/// Tag bits for the `Symbol` type.
pub const SYMBOL_TAG: u64 = 0b100;
/// Tag bits for the `Char` type.
pub const CHAR_TAG: u64 = 0b101;
/// Tag bits for the `Double` type.
pub const DOUBLE_TAG: u64 = 0b110;
/// Tag bits for the `Pointer` type.
pub const POINTER_TAG: u64 = 0b111;

pub const PTR_MASK: u64 = !TAG_BITS;

// #[repr(transparent)]
#[repr(C)]
#[allow(clippy::derived_hash_with_manual_eq)] // TODO: manually implement Hash instead...
#[derive(Copy, Clone, Hash)]
pub struct BaseValue {
    encoded: u64,
}

impl BaseValue {
    /// The boolean `true` value.
    pub const TRUE: BaseValue = Self::new(BOOLEAN_TAG, 1);
    /// The boolean `false` value.
    pub const FALSE: BaseValue = Self::new(BOOLEAN_TAG, 0);
    /// The `nil` value.
    pub const NIL: BaseValue = Self::new(NIL_TAG, 0);
    /// The `system` value.
    pub const SYSTEM: Self = Self::new(SYSTEM_TAG, 0);
    /// The integer `0` value.
    pub const INTEGER_ZERO: Self = Self::new(INTEGER_TAG, 0);
    /// The integer `1` value.
    pub const INTEGER_ONE: Self = Self::new(INTEGER_TAG, 1);

    #[inline(always)]
    pub const fn new(tag: u64, value: u64) -> Self {
        Self {
            encoded: (value << 3) | tag,
        }
    }

    /// Returns a new boolean value.
    #[inline(always)]
    pub fn new_boolean(value: bool) -> Self {
        Self {
            encoded: if value { BOOLEAN_TAG | 1 << 3 } else { BOOLEAN_TAG },
        }
    }

    /// Returns whether this value is a pointer type value.
    #[inline(always)]
    pub fn is_ptr_type(self) -> bool {
        (self.encoded & TAG_BITS) == POINTER_TAG
    }

    /// Returns it at an arbitrary pointer. Used for debugging.
    /// # Safety
    /// "Don't"
    pub unsafe fn as_something<PTR>(self) -> Option<PTR>
    where
        PTR: From<u64>,
    {
        self.is_ptr_type().then(|| self.extract_gc_cell())
    }

    /// Return the value as its internal representation: a u64 type.
    #[inline(always)]
    pub fn as_u64(self) -> u64 {
        self.encoded
    }

    /// Returns the tag bits of the value.
    //#[inline(always)]
    pub fn tag(self) -> u64 {
        self.encoded & TAG_BITS
    }
    /// Returns the payload bits of the value.
    #[inline(always)]
    pub fn payload(self) -> u64 {
        self.encoded >> 3
    }

    #[inline(always)]
    pub fn extract_gc_cell<Ptr>(self) -> Ptr
    where
        Ptr: From<u64>,
    {
        Ptr::from(self.extract_pointer_bits())
    }

    #[inline(always)]
    pub fn extract_pointer_bits(self) -> u64 {
        self.encoded & PTR_MASK
    }

    /// Returns a new integer value.
    #[inline(always)]
    pub fn new_integer(value: i32) -> Self {
        Self::new(INTEGER_TAG, value as u64)
    }

    /// Returns a new double value.
    #[inline(always)]
    pub fn new_double(value: f64) -> Self {
        Self {
            encoded: (value.to_bits() << 3) | DOUBLE_TAG,
        }
    }

    /// Returns a new symbol value.
    #[inline(always)]
    pub fn new_symbol(value: Interned) -> Self {
        Self::new(SYMBOL_TAG, value.0.into())
    }

    #[inline(always)]
    pub fn new_char(value: char) -> Self {
        Self::new(CHAR_TAG, value.into())
    }

    /// Returns a new big integer value.
    #[inline(always)]
    pub fn new_big_integer<BigIntPtr>(value: BigIntPtr) -> Self
    where
        u64: From<BigIntPtr>,
        BigIntPtr: Deref<Target = BigInt> + From<u64>,
    {
        let ptr: u64 = value.into();
        Self {
            encoded: ptr | POINTER_TAG,
        }
    }
    /// Returns a new string value.
    #[inline(always)]
    pub fn new_string<StringPtr>(value: StringPtr) -> Self
    where
        u64: From<StringPtr>,
        StringPtr: Deref<Target = String> + From<u64>,
    {
        let ptr: u64 = value.into();
        Self {
            encoded: ptr | POINTER_TAG,
        }
    }

    // --------

    /// Returns whether this value is a big integer.
    #[inline(always)]
    pub fn is_big_integer(self) -> bool {
        self.is_ptr_type()
    }
    /// Returns whether this value is a string.
    #[inline(always)]
    pub fn is_string(self) -> bool {
        self.is_ptr_type()
    }

    /// Returns whether this value is `nil``.
    #[inline(always)]
    pub fn is_nil(self) -> bool {
        self.tag() == NIL_TAG
    }

    /// Returns whether this value is `system`.
    #[inline(always)]
    pub fn is_system(self) -> bool {
        self.tag() == SYSTEM_TAG
    }
    /// Returns whether this value is an integer.
    #[inline(always)]
    pub fn is_integer(self) -> bool {
        self.tag() == INTEGER_TAG
    }

    /// Returns whether this value is a double.
    #[inline(always)]
    pub fn is_double(self) -> bool {
        self.tag() == DOUBLE_TAG
    }

    /// Returns whether this value is a boolean.
    #[inline(always)]
    pub fn is_boolean(self) -> bool {
        self.tag() == BOOLEAN_TAG
    }

    /// Returns whether or not it's a boolean corresponding to true. NB: does NOT check if the type actually is a boolean.
    #[inline(always)]
    pub fn is_boolean_true(self) -> bool {
        self.payload() == 1
    }

    /// Returns whether or not it's a boolean corresponding to false. NB: does NOT check if the type actually is a boolean.
    #[inline(always)]
    pub fn is_boolean_false(self) -> bool {
        self.payload() == 0
    }

    /// Returns whether this value is a symbol.
    #[inline(always)]
    pub fn is_symbol(self) -> bool {
        self.tag() == SYMBOL_TAG
    }

    #[inline(always)]
    pub fn is_char(self) -> bool {
        self.tag() == CHAR_TAG
    }

    // ----------------

    /// Returns this value as a big integer, if such is its type.
    #[inline(always)]
    pub fn as_big_integer<BigIntPtr>(self) -> Option<BigIntPtr>
    where
        u64: From<BigIntPtr>,
        BigIntPtr: From<u64>,
    {
        self.is_big_integer().then(|| self.extract_gc_cell())
    }

    /// Returns this value as a string, if such is its type.
    #[inline(always)]
    pub fn as_string<StringPtr>(self) -> Option<StringPtr>
    where
        StringPtr: From<u64>,
        StringPtr: Deref<Target = String>,
    {
        self.is_string().then(|| self.extract_gc_cell())
    }

    // `as_*` for non pointer types

    /// Returns this value as an integer, if such is its type.
    #[inline(always)]
    pub fn as_integer(self) -> Option<i32> {
        self.is_integer().then_some(self.payload() as i32)
    }

    /// Returns this value as a double, if such is its type.
    #[inline(always)]
    pub fn as_double(self) -> Option<f64> {
        self.is_double().then(|| f64::from_bits(self.payload()))
    }

    /// Returns this value as a boolean, if such is its type.
    #[inline(always)]
    pub fn as_boolean(self) -> Option<bool> {
        self.is_boolean().then_some(self.is_boolean_true())
    }

    #[inline(always)]
    pub fn as_char(self) -> Option<char> {
        self.is_char().then_some(self.payload() as u8 as char)
    }

    /// Returns this value as a boolean, but without checking whether or not it really is one.
    #[inline(always)]
    pub fn as_boolean_unchecked(self) -> bool {
        self.payload() != 0
    }

    /// Returns this value as a symbol, if such is its type.
    #[inline(always)]
    pub fn as_symbol(self) -> Option<Interned> {
        self.is_symbol().then_some(Interned(self.payload() as u16))
    }

    #[inline(always)]
    pub fn is_ptr<T, PTR>(&self) -> bool
    where
        T: HasPointerTag,
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        value_ptr.is_valid()
    }

    #[inline(always)]
    pub fn as_ptr<T: HasPointerTag, PTR>(&self) -> Option<PTR>
    where
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        value_ptr.get()
    }

    /// # Safety
    /// Only use when the type of the pointer was previously checked.
    #[inline(always)]
    pub unsafe fn as_ptr_unchecked<T: HasPointerTag, PTR>(&self) -> PTR
    where
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        value_ptr.get_unchecked()
    }

    // ----------------

    // these are all for backwards compatibility (i.e.: i don't want to do massive amounts of refactoring), but also maybe clever-ish replacement with normal Value enums

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Boolean(value: bool) -> Self {
        Self::new_boolean(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Integer(value: i32) -> Self {
        Self::new_integer(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Double(value: f64) -> Self {
        Self::new_double(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Symbol(value: Interned) -> Self {
        Self::new_symbol(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Char(value: char) -> Self {
        Self::new_char(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn BigInteger<BigIntPtr>(value: BigIntPtr) -> Self
    where
        u64: From<BigIntPtr>,
        BigIntPtr: Deref<Target = BigInt> + From<u64>,
    {
        Self::new_big_integer(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn String<Ptr>(value: Ptr) -> Self
    where
        u64: From<Ptr>,
        Ptr: Deref<Target = String> + From<u64>,
    {
        Self::new_string(value)
    }

    /// Returns a pointer to the underlying data, given a reference to a Value type.
    /// Not actually unsafe in itself, but considered as such because it's VERY dangerous unless used correctly.
    /// Why does it exist? Because GC needs to store mutable references to values to modify them when moving memory around. Most values are stored as &Value, so this function is convenient.
    /// # Safety
    /// The value used as a reference must be long-lived: if it is dropped at any point before invoking this function, we'll get undefined behavior.
    /// In practice for our cases, this means any reference passed to this function must be A POINTER TO THE GC HEAP.
    pub unsafe fn as_mut_ptr(&self) -> *mut BaseValue {
        debug_assert!(
            self.is_ptr_type(),
            "calling as_mut_ptr() on a value that's not a pointer, thus not meant to hold data to the GC heap: why?"
        );
        self as *const Self as *mut Self
    }
}

impl From<u64> for BaseValue {
    fn from(value: u64) -> Self {
        BaseValue { encoded: value }
    }
}

#[macro_export]
/// Macro used to make AST-specific and BC-specific Value type "inherit" behavior from the base value type.
/// Rust *could* avoid this by inferring that a BaseValue and a Value are the same.
/// ...but I'm not sure there's a way for me to inform it. Maybe in a future version.
macro_rules! delegate_to_base_value {
    ($($fn_name:ident($($arg:ident : $arg_ty:ty),*) -> $ret:ty),* $(,)?) => {
        $(
            pub fn $fn_name($(value: $arg_ty),*) -> $ret {
                BaseValue::$fn_name(value).into()
            }
        )*
    };
}
