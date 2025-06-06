use crate::{
    interned::Interned,
    value_ptr::{HasPointerTag, TypedPtrValue},
};
use num_bigint::BigInt;
use std::mem::size_of;
use std::ops::Deref;

static_assertions::const_assert_eq!(size_of::<f64>(), 8);
static_assertions::assert_eq_size!(f64, u64, *const ());

pub const VALUE_TAG_BITS: u64 = 4;
pub const TAG_BITS: u64 = 0b1111;

pub const NIL_TAG: u64 = 0b0001;
pub const INTEGER_TAG: u64 = 0b0010;
pub const BOOLEAN_TAG: u64 = 0b0011;
pub const SYMBOL_TAG: u64 = 0b0100;
pub const CHAR_TAG: u64 = 0b0101;
pub const DOUBLE_TAG: u64 = 0b0110;
pub const BIG_INTEGER_TAG: u64 = 0b0111;
pub const STRING_TAG: u64 = 0b1000;

pub const PTR_MASK: u64 = !TAG_BITS;

#[repr(C)]
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Copy, Clone, Hash)]
pub struct BaseValue {
    encoded: u64,
}

impl BaseValue {
    pub const TRUE: BaseValue = Self::new(BOOLEAN_TAG, 1);
    pub const FALSE: BaseValue = Self::new(BOOLEAN_TAG, 0);
    pub const NIL: BaseValue = Self::new(NIL_TAG, 0);
    pub const INTEGER_ZERO: Self = Self::new(INTEGER_TAG, 0);
    pub const INTEGER_ONE: Self = Self::new(INTEGER_TAG, 1);

    #[inline(always)]
    pub const fn new(tag: u64, value: u64) -> Self {
        Self {
            encoded: (value << VALUE_TAG_BITS) | tag,
        }
    }

    pub fn encode_pointer(tag: u64, ptr: u64) -> u64 {
        assert_eq!(ptr & 0b111, 0, "Pointer must be 8-byte (3b) aligned");
        let encoded = (ptr << VALUE_TAG_BITS) | (tag & TAG_BITS);
        //println!("Encoding pointer: tag = {tag}, ptr = {ptr} -> encoded = {encoded}");
        encoded
    }

    pub fn decode_pointer(encoded: u64) -> u64 {
        let ptr = encoded >> VALUE_TAG_BITS;
        //println!("Decoding pointer from encoded value: {encoded} → {ptr}");
        ptr
    }

    #[inline(always)]
    pub fn new_boolean(value: bool) -> Self {
        //println!("Creating new boolean value: {}", value);
        Self::new(BOOLEAN_TAG, value as u64)
    }

    #[inline(always)]
    pub fn is_ptr_type(self) -> bool {
        //println!("Checking if value is a pointer type: {}", self.encoded);
        matches!(self.tag(), STRING_TAG | BIG_INTEGER_TAG)
    }

    pub unsafe fn as_something<PTR>(self) -> Option<PTR>
    where
        PTR: From<u64>,
    {
        //println!("Attempting to extract GC cell from value: {}", self.encoded);
        self.is_ptr_type().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_u64(self) -> u64 {
        //println!("Returning encoded value: {}", self.encoded);
        self.encoded
    }

    pub fn tag(self) -> u64 {
        //println!("Extracting tag from encoded value: {}", self.encoded);
        self.encoded & TAG_BITS
    }

    #[inline(always)]
    pub fn payload(self) -> u64 {
        //println!("Extracting payload from encoded value: {}", self.encoded);
        self.encoded >> VALUE_TAG_BITS
    }

    #[inline(always)]
    pub fn extract_gc_cell<Ptr>(self) -> Ptr
    where
        Ptr: From<u64>,
    {
        //println!("Extracting GC cell from encoded value: {}", self.encoded);
        Ptr::from(Self::decode_pointer(self.encoded))
    }

    #[inline(always)]
    pub fn extract_pointer_bits(self) -> u64 {
        //println!("Extracting pointer bits from encoded value: {}", self.encoded);
        Self::decode_pointer(self.encoded)
    }

    #[inline(always)]
    pub fn new_integer(value: i32) -> Self {
        //println!("Creating new integer value: {}", value);
        Self::new(INTEGER_TAG, value as u64)
    }

    #[inline(always)]
    pub fn new_double(value: f64) -> Self {
        //println!("Creating new double value: {}", value);
        Self {
            encoded: (value.to_bits() & !TAG_BITS) | DOUBLE_TAG
        }
    }

    #[inline(always)]
    pub fn new_symbol(value: Interned) -> Self {
        //println!("Creating new symbol value: {}", value.0);
        Self::new(SYMBOL_TAG, value.0.into())
    }

    #[inline(always)]
    pub fn new_char(value: char) -> Self {
        //println!("Creating new char value: {}", value);
        Self::new(CHAR_TAG, value.into())
    }

    #[inline(always)]
    pub fn new_big_integer<BigIntPtr>(value: BigIntPtr) -> Self
    where
        u64: From<BigIntPtr>,
        BigIntPtr: Deref<Target = BigInt> + From<u64>,
    {
        let ptr: u64 = value.into();
        //println!("Creating new big integer value with pointer: {}", ptr);
        Self {
            encoded: Self::encode_pointer(BIG_INTEGER_TAG, ptr),
        }
    }

    #[inline(always)]
    pub fn new_string<StringPtr>(value: StringPtr) -> Self
    where
        u64: From<StringPtr>,
        StringPtr: Deref<Target = String> + From<u64>,
    {
        let ptr: u64 = value.into();
        //println!("Creating new string value with pointer: {}", ptr);
        Self {
            encoded: Self::encode_pointer(STRING_TAG, ptr),
        }
    }

    #[inline(always)]
    pub fn is_big_integer(self) -> bool {
        //println!("Checking if value is a big integer: {}", self.encoded);
        self.tag() == BIG_INTEGER_TAG
    }

    #[inline(always)]
    pub fn is_string(self) -> bool {
        //println!("Checking if value is a string: {}", self.encoded);
        self.tag() == STRING_TAG
    }

    #[inline(always)]
    pub fn is_nil(self) -> bool {
        //println!("Checking if value is nil: {}", self.encoded);
        self.tag() == NIL_TAG
    }

    #[inline(always)]
    pub fn is_integer(self) -> bool {
        //println!("Checking if value is an integer: {}", self.encoded);
        self.tag() == INTEGER_TAG
    }

    #[inline(always)]
    pub fn is_double(self) -> bool {
        //println!("Checking if value is a double: {}", self.encoded);
        self.tag() == DOUBLE_TAG
    }

    #[inline(always)]
    pub fn is_boolean(self) -> bool {
        //println!("Checking if value is a boolean: {}", self.encoded);
        self.tag() == BOOLEAN_TAG
    }

    #[inline(always)]
    pub fn is_boolean_true(self) -> bool {
        //println!("Checking if boolean value is true: {}", self.encoded);
        self.payload() == 1
    }

    #[inline(always)]
    pub fn is_boolean_false(self) -> bool {
        //println!("Checking if boolean value is false: {}", self.encoded);
        self.payload() == 0
    }

    #[inline(always)]
    pub fn is_symbol(self) -> bool {
        //println!("Checking if value is a symbol: {}", self.encoded);
        self.tag() == SYMBOL_TAG
    }

    #[inline(always)]
    pub fn is_char(self) -> bool {
        //println!("Checking if value is a char: {}", self.encoded);
        self.tag() == CHAR_TAG
    }

    #[inline(always)]
    pub fn as_big_integer<BigIntPtr>(self) -> Option<BigIntPtr>
    where
        u64: From<BigIntPtr>,
        BigIntPtr: From<u64>,
    {
        //println!("Attempting to extract big integer from value: {}", self.encoded);
        self.is_big_integer().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_string<StringPtr>(self) -> Option<StringPtr>
    where
        StringPtr: From<u64>,
        StringPtr: Deref<Target = String>,
    {
        //println!("Attempting to extract string from value: {}", self.encoded);
        self.is_string().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_integer(self) -> Option<i32> {
        //println!("Attempting to extract integer from value: {}", self.encoded);
        self.is_integer().then_some(self.payload() as i32)
    }

    #[inline(always)]
    pub fn as_double(self) -> Option<f64> {
        //println!("Attempting to extract double from value: {}", self.encoded);
        self.is_double().then(|| f64::from_bits(self.encoded & !TAG_BITS))
    }

    #[inline(always)]
    pub fn as_boolean(self) -> Option<bool> {
        //println!("Attempting to extract boolean from value: {}", self.encoded);
        self.is_boolean().then_some(self.is_boolean_true())
    }

    #[inline(always)]
    pub fn as_char(self) -> Option<char> {
        //println!("Attempting to extract char from value: {}", self.encoded);
        self.is_char().then_some(self.payload() as u8 as char)
    }

    #[inline(always)]
    pub fn as_boolean_unchecked(self) -> bool {
        //println!("Unchecked extraction of boolean from value: {}", self.encoded);
        self.payload() != 0
    }

    #[inline(always)]
    pub fn as_symbol(self) -> Option<Interned> {
        //println!("Attempting to extract symbol from value: {}", self.encoded);
        self.is_symbol().then_some(Interned(self.payload() as u16))
    }

    #[inline(always)]
    pub fn is_ptr<T, PTR>(&self) -> bool
    where
        T: HasPointerTag,
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        //println!("Checking if value is a pointer");
        value_ptr.is_valid()
    }

    #[inline(always)]
    pub fn as_ptr<T: HasPointerTag, PTR>(&self) -> Option<PTR>
    where
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        //println!("Attempting to extract pointer");
        value_ptr.get()
    }

    #[inline(always)]
    pub unsafe fn as_ptr_unchecked<T: HasPointerTag, PTR>(&self) -> PTR
    where
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        //println!("Unchecked extraction of pointer");
        value_ptr.get_unchecked()
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Boolean(value: bool) -> Self {
        //println!("Creating new boolean value: {}", value);
        Self::new_boolean(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Integer(value: i32) -> Self {
        //println!("Creating new integer value: {}", value);
        Self::new_integer(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Double(value: f64) -> Self {
        //println!("Creating new double value: {}", value);
        Self::new_double(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Symbol(value: Interned) -> Self {
        //println!("Creating new symbol value: {}", value.0);
        Self::new_symbol(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Char(value: char) -> Self {
        //println!("Creating new char value: {}", value);
        Self::new_char(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn BigInteger<BigIntPtr>(value: BigIntPtr) -> Self
    where
        u64: From<BigIntPtr>,
        BigIntPtr: Deref<Target = BigInt> + From<u64>,
    {
        //println!("Creating new big integer value with pointer");
        Self::new_big_integer(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn String<Ptr>(value: Ptr) -> Self
    where
        u64: From<Ptr>,
        Ptr: Deref<Target = String> + From<u64>,
    {
        //println!("Creating new string value with pointer");
        Self::new_string(value)
    }

    pub unsafe fn as_mut_ptr(&self) -> *mut BaseValue {
        debug_assert!(
            self.is_ptr_type(),
            "calling as_mut_ptr() on a value that's not a pointer"
        );
        //println!("Converting BaseValue to mutable pointer",);
        self as *const Self as *mut Self
    }
}

impl From<u64> for BaseValue {
    fn from(value: u64) -> Self {
        BaseValue { encoded: value }
    }
}

#[macro_export]
macro_rules! delegate_to_base_value {
    ($($fn_name:ident($($arg:ident : $arg_ty:ty),*) -> $ret:ty),* $(,)?) => {
        $(
            pub fn $fn_name($(value: $arg_ty),*) -> $ret {
                BaseValue::$fn_name(value).into()
            }
        )*
    };
}
