use crate::{
    interned::Interned,
    value_ptr::{HasPointerTag, TypedPtrValue},
};
use num_bigint::BigInt;
use std::ops::Deref;

static_assertions::const_assert_eq!(size_of::<f64>(), 8);
static_assertions::assert_eq_size!(f64, u64, *const ());

/// Canonical `NaN` representation (minimum bitfield to represent `NaN`).
///
/// Since we are hijacking most bits in `NaN` values, all legitimate `NaN` will need
/// to be "canonicalized" to this representation, which will be the only actual `NaN` value.
///
/// This isn't that bad since SOM doesn't have many bit fiddling facilities anyway, so it is
/// unlikely one would need to inspect bits in `NaN` values.
///
/// An `NaN` is indicated by:
/// - sign bit is `0`
/// - exponent bits are all `1`
/// - first mantissa bit is `1`
pub const CANON_NAN_BITS: u64 = 0x7FF8000000000000;

// However as long as any bit is set in the mantissa with the exponent of all
// ones this value is a NaN, and it even ignores the sign bit.
// (NOTE: we have to use __builtin_isnan here since some isnan implementations are not constexpr)
// FIXME: `is_nan` is not a `const fn` yet
// static_assertions::const_assert!(f64::from_bits(0x7FF0000000000001).is_nan());
// static_assertions::const_assert!(f64::from_bits(0xFFF0000000040000).is_nan());

/// Base bit pattern needed to be set for all tags.
///
/// All tags will add at least one more bit, to distinguish from `CANON_NAN_BITS`.
///
/// This means (after `<< TAG_SHIFT`):
/// - sign bit is `0`
/// - exponent bits are all `1`
/// - first mantissa bit is `1`
pub const BASE_TAG: u64 = 0x7FF8;

/// Base bit pattern needed to be set for all tags that are GC-managed.
///
/// It is similar to `BASE_TAG` instead that it sets the sign bit to signify that
/// this is a pointer-type.
///
/// This means (after `<< TAG_SHIFT`):
/// - sign bit is `1`
/// - exponent bits are all `1`
/// - first mantissa bit is `1`
pub const CELL_BASE_TAG: u64 = 0x8000 | BASE_TAG;

// On all current 64-bit systems this code runs, pointers actually only use the
// lowest 6 bytes which fits neatly into our NaN payload with the top two bytes
// left over for marking it as a NaN and tagging the type.
// Note that we do need to take care when extracting the pointer value but this
// is explained in the extract_pointer method.

// Tags for non-pointer types

/// Tag bits for the `Nil` type.
pub const NIL_TAG: u64 = 0b001 | BASE_TAG;
/// Tag bits for the `System` type.
pub const SYSTEM_TAG: u64 = 0b010 | BASE_TAG;
/// Tag bits for the `Integer` type.
pub const INTEGER_TAG: u64 = 0b011 | BASE_TAG; // Same bit position as `BIG_INTEGER_TAG`
/// Tag bits for the `Boolean` type.
pub const BOOLEAN_TAG: u64 = 0b100 | BASE_TAG;
/// Tag bits for the `Symbol` type.
pub const SYMBOL_TAG: u64 = 0b101 | BASE_TAG;

pub const CHAR_TAG: u64 = 0b110 | BASE_TAG;

// /// Tag bits for the `???` type.
// const RESERVED2_TAG: u64 = 0b111 | BASE_TAG;

// Tags for pointer types

/// Tag bits for the `String` type.
pub const STRING_TAG: u64 = 0b001 | CELL_BASE_TAG;
/// Tag bits for the `BigInteger` type.
pub const BIG_INTEGER_TAG: u64 = 0b011 | CELL_BASE_TAG; // Same bit position as `INTEGER_TAG`

// Here is a nice diagram to summarize how our NaN-boxing works:
// (s = sign bit, e = exponent bit, m = mantissa bit)
//
//     tag bits                       payload bits
// SEEEEEEEEEEEMMMM MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM
// 0111111111111000 000... -> is the only real NaN
// 0111111111111xxx yyy... -> xxx = non-pointer type, yyy = value
// 1111111111111xxx yyy... -> xxx = pointer type,     yyy = pointer value

/// The amount of bits to shift tags in the correct position within a 64-bit value.
pub const TAG_SHIFT: u64 = 48;

/// Bit pattern used to quickly extract the tag bits from a 64-bit value.
pub const TAG_EXTRACTION: u64 = 0xFFFF << TAG_SHIFT;

/// Bit pattern used to quickly check if a given 64-bit value houses a pointer-type value.
pub const IS_PTR_PATTERN: u64 = CELL_BASE_TAG << TAG_SHIFT;

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
        // NOTE: Pointers in x86-64 use just 48 bits however are supposed to be
        //       sign extended up from the 47th bit.
        //       This means that all bits above the 47th should be the same as
        //       the 47th. When storing a pointer we thus drop the top 16 bits as
        //       we can recover it when extracting the pointer again.
        //       See also: Value::extract_pointer.
        Self {
            encoded: CANON_NAN_BITS | ((tag << TAG_SHIFT) & TAG_EXTRACTION) | (value & !TAG_EXTRACTION),
        }
    }

    /// Returns a new boolean value.
    #[inline(always)]
    pub fn new_boolean(value: bool) -> Self {
        if value {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }

    /// Returns whether this value is a pointer type value.
    #[inline(always)]
    pub fn is_ptr_type(self) -> bool {
        (self.encoded & IS_PTR_PATTERN) == IS_PTR_PATTERN
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
    #[inline(always)]
    pub fn tag(self) -> u64 {
        (self.encoded & TAG_EXTRACTION) >> TAG_SHIFT
    }
    /// Returns the payload bits of the value.
    #[inline(always)]
    pub fn payload(self) -> u64 {
        self.encoded & !TAG_EXTRACTION
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
        // For x86_64 the top 16 bits should be sign extending the "real" top bit (47th).
        // So first shift the top 16 bits away then using the right shift it sign extends the top 16 bits.
        (((self.encoded << 16) as i64) >> 16) as u64
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
            encoded: if value.is_nan() {
                // To represent an actual `NaN`, we canonicalize it to `CANON_NAN_BITS`.
                CANON_NAN_BITS
            } else {
                value.to_bits()
            },
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
        TypedPtrValue::new(value).into()
    }
    /// Returns a new string value.
    #[inline(always)]
    pub fn new_string<StringPtr>(value: StringPtr) -> Self
    where
        u64: From<StringPtr>,
        StringPtr: Deref<Target = String> + From<u64>,
    {
        TypedPtrValue::new(value).into()
    }

    // --------

    /// Returns whether this value is a big integer.
    #[inline(always)]
    pub fn is_big_integer(self) -> bool {
        self.tag() == BIG_INTEGER_TAG
    }
    /// Returns whether this value is a string.
    #[inline(always)]
    pub fn is_string(self) -> bool {
        self.tag() == STRING_TAG
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
        // A double is any value which does not have the full exponent and top mantissa bit set or has
        // exactly only those bits set.
        (self.encoded & CANON_NAN_BITS) != CANON_NAN_BITS || (self.encoded == CANON_NAN_BITS)
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
        self.is_integer().then_some((self.encoded & 0xFFFFFFFF) as i32)
    }

    /// Returns this value as a double, if such is its type.
    #[inline(always)]
    pub fn as_double(self) -> Option<f64> {
        self.is_double().then(|| f64::from_bits(self.encoded))
    }

    /// Returns this value as a boolean, if such is its type.
    #[inline(always)]
    pub fn as_boolean(self) -> Option<bool> {
        self.is_boolean().then_some((self.encoded & 0x1) == 0x1)
    }

    #[inline(always)]
    pub fn as_char(self) -> Option<char> {
        self.is_char().then_some((self.encoded & 0xFFFFFFFF) as u8 as char)
    }

    /// Returns this value as a boolean, but without checking whether or not it really is one.
    #[inline(always)]
    pub fn as_boolean_unchecked(self) -> bool {
        self.payload() != 0
    }

    /// Returns this value as a symbol, if such is its type.
    #[inline(always)]
    pub fn as_symbol(self) -> Option<Interned> {
        self.is_symbol().then_some(Interned((self.encoded & 0xFFFFFFFF) as u16))
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
