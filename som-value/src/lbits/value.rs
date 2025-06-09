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
pub const BIG_INTEGER_TAG: u64 = 0b0111;
pub const STRING_TAG: u64 = 0b1000;
pub const DOUBLE_TAG: u64 = 0b1111;

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
        if tag == STRING_TAG || tag == BIG_INTEGER_TAG || tag == 0b1001 || tag == 0b1010 || tag == 0b1011 || tag == 0b1100 || tag == 0b1101 {
            return Self::new_ptr(tag, value);
        }
        Self {
            encoded: (value << VALUE_TAG_BITS) | (tag & TAG_BITS),
        }
    }

    #[inline(always)]
    pub const fn new_ptr(tag: u64, ptr: u64) -> Self {
        // assert_eq!(ptr & 0b111, 0, "Pointer must be 8byte aligned");
        Self {
            encoded: (ptr << 1) | (tag & TAG_BITS),
        }
    }

     #[inline(always)]
    pub fn encode_ptr(tag: u64, ptr: u64) -> u64 {
        assert_eq!(ptr & 0b111, 0, "Pointer must be 8byte aligned");
        //println!("Encoding pointer: {ptr} with tag: {tag}");
        //println!("[ in] ptr: {:#64b}", ptr);
        let shifted = ptr << 1;
        //println!("[tst] optr : {:#64b}", Self::decode_ptr(shifted | tag));
        //println!("[out] ptr: {:#64b}", shifted | tag);
        shifted | tag
    }

    #[inline(always)]
    pub fn decode_ptr(encoded: u64) -> u64 {
        // Remove the tag and shift right by 1 to get the original pointer
        (encoded & !TAG_BITS) >> 1
    }

    #[inline(always)]
    pub fn new_boolean(value: bool) -> Self {
        //println!("new_boolean : {:#b}", value as u64);
        //println!("[ in] bool: {:#64b}", value as u64);
        //println!("Creating new boolean value: {}", value);
        let tr = Self::new(BOOLEAN_TAG, value as u64);
        //println!("[out] bool: {:#64b}", tr.encoded);
        tr
    }

    #[inline(always)]
    pub fn is_ptr_type(self) -> bool {
        //println!("is_ptr_type : {}", self.encoded);
        //matches!(self.tag(), STRING_TAG | BIG_INTEGER_TAG)
        self.tag() == STRING_TAG || self.tag() == BIG_INTEGER_TAG  || self.tag() == 0b1001 || self.tag() == 0b1010 || self.tag() == 0b1011 || self.tag() == 0b1100 || self.tag() == 0b1101
    }

    pub unsafe fn as_something<PTR>(self) -> Option<PTR>
    where
        PTR: From<u64>,
    {
        //println!("as_something : {}", self.encoded);
        self.is_ptr_type().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_u64(self) -> u64 {
        //println!("as_u64 : {}", self.encoded);
        self.encoded
    }

    pub fn tag(self) -> u64 {
        //println!("tag : {}", self.encoded);
        self.encoded & TAG_BITS
    }

    #[inline(always)]
    pub fn payload(self) -> u64 {
        //println!("payload : {}", self.encoded);
        if self.is_ptr_type() {
            return Self::decode_ptr(self.encoded);
        }
        self.encoded >> VALUE_TAG_BITS
    }

    #[inline(always)]
    pub fn extract_gc_cell<Ptr>(self) -> Ptr
    where
        Ptr: From<u64>,
    {
        //println!("extract_gc_cell : {}", self.encoded);
        Ptr::from(self.payload())
    }

    #[inline(always)]
    pub fn extract_pointer_bits(self) -> u64 {
        //println!("extract_pointer_bits : {}", self.encoded);
        self.payload()
    }

    #[inline(always)]
    pub fn new_integer(value: i32) -> Self {
        //println!("new_integer : {:#b}", value);
        // //println!("[ in] int: {:#64b}", value as u64);
        let tr = Self::new(INTEGER_TAG, value as u64);
        // //println!("[out] int: {:#64b}", tr.encoded);
        tr
    }

    #[inline(always)]
    pub fn new_double(value: f64) -> Self {
        //println!("new_double : {:#64b}", value.to_bits() as u64);
        // let bits = value.to_bits();
        // Self {
        //     encoded: (bits << VALUE_TAG_BITS) | DOUBLE_TAG,
        // }
        let tr = Self::new(DOUBLE_TAG, value.to_bits());
        //println!("new_double : {:#64b}", tr.encoded);
        tr
    }

    #[inline(always)]
    pub fn new_symbol(value: Interned) -> Self {
        //println!("new_symbol : {:#b}", value.0);
        // //println!("[ in] symbol: {:#64b}", value.0 as u64);
        let tr = Self::new(SYMBOL_TAG, value.0.into());
        // //println!("[out] symbol: {:#64b}", tr.encoded);
        //println!("new_symbol : {:#64b}", tr.encoded);
        tr
    }

    #[inline(always)]
    pub fn new_char(value: char) -> Self {
        //println!("new_char : {:#b}", value as u8);
        // //println!("[ in] char: {:#64b}", value as u64);
        let tr = Self::new(CHAR_TAG, value.into());
        // //println!("[out] char: {:#64b}", tr.encoded);
        //println!("new_char : {:#64b}", tr.encoded);
        tr
    }

    #[inline(always)]
    pub fn new_big_integer<BigIntPtr>(value: BigIntPtr) -> Self
    where
        u64: From<BigIntPtr>,
        BigIntPtr: Deref<Target = BigInt> + From<u64>,
    {
        let ptr: u64 = value.into();
        //println!("new_big_integer : {:#b}", ptr);
        ////println!("Creating new big integer value with pointer: {}", ptr);
        // Self {
        //     en(BIG_INTEGER_TAG, ptr),
        // }
        // //println!("[ in] big integer: {:#64b}", ptr);
        let tr = Self::new(BIG_INTEGER_TAG, ptr);
        // //println!("[out] big integer: {:#64b}", tr.encoded);
        //println!("new_big_integer : {:#64b}", tr.encoded);
        tr
    }

    #[inline(always)]
    pub fn new_string<StringPtr>(value: StringPtr) -> Self
    where
        u64: From<StringPtr>,
        StringPtr: Deref<Target = String> + From<u64>,
    {
        let ptr: u64 = value.into();
        //println!("new_string : {:#b}", ptr);
        // Self {
        //     encoded: Self::encode_pointer(STRING_TAG, ptr),
        // }
        // //println!("[ in] string: {:#64b}", ptr);
        let tr = Self::new(STRING_TAG, ptr);
        // //println!("[out] string: {:#64b}", tr.encoded);
        //println!("new_string : {:#64b}", tr.encoded);
        tr
    }

    #[inline(always)]
    pub fn is_big_integer(self) -> bool {
        //println!("is_big_integer : {}", self.encoded);
        self.tag() == BIG_INTEGER_TAG
    }

    #[inline(always)]
    pub fn is_string(self) -> bool {
        //println!("is_string : {}", self.encoded);
        self.tag() == STRING_TAG
    }

    #[inline(always)]
    pub fn is_nil(self) -> bool {
        //println!("is_nil : {}", self.encoded);
        self.tag() == NIL_TAG
    }

    #[inline(always)]
    pub fn is_integer(self) -> bool {
        //println!("is_integer : {}", self.encoded);
        self.tag() == INTEGER_TAG
    }

    #[inline(always)]
    pub fn is_double(self) -> bool {
        //println!("is_double : {}", self.encoded);
        self.tag() == DOUBLE_TAG
    }

    #[inline(always)]
    pub fn is_boolean(self) -> bool {
        //println!("is_boolean : {}", self.encoded);
        self.tag() == BOOLEAN_TAG
    }

    #[inline(always)]
    pub fn is_boolean_true(self) -> bool {
        //println!("is_boolean_true : {}", self.encoded);
        self.payload() == 1
    }

    #[inline(always)]
    pub fn is_boolean_false(self) -> bool {
        //println!("is_boolean_false : {}", self.encoded);
        self.payload() == 0
    }

    #[inline(always)]
    pub fn is_symbol(self) -> bool {
        //println!("is_symbol : {}", self.encoded);
        self.tag() == SYMBOL_TAG
    }

    #[inline(always)]
    pub fn is_char(self) -> bool {
        //println!("is_char : {}", self.encoded);
        self.tag() == CHAR_TAG
    }

    #[inline(always)]
    pub fn as_big_integer<BigIntPtr>(self) -> Option<BigIntPtr>
    where
        u64: From<BigIntPtr>,
        BigIntPtr: From<u64>,
    {
        //println!("as_big_integer : {}", self.encoded);
        self.is_big_integer().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_string<StringPtr>(self) -> Option<StringPtr>
    where
        StringPtr: From<u64>,
        StringPtr: Deref<Target = String>,
    {
        //println!("as_string : {}", self.encoded);
        self.is_string().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_integer(self) -> Option<i32> {
        //println!("as_integer : {}", self.encoded);
        self.is_integer().then_some(self.payload() as i32)
    }

    #[inline(always)]
    pub fn as_double(self) -> Option<f64> {
        //println!("as_double : {}", self.encoded);
        self.is_double().then(|| {
            let bits = self.payload();
            f64::from_bits(bits)
        })
    }

    #[inline(always)]
    pub fn as_boolean(self) -> Option<bool> {
        //println!("as_boolean : {}", self.encoded);
        self.is_boolean().then_some(self.is_boolean_true())
    }

    #[inline(always)]
    pub fn as_char(self) -> Option<char> {
        //println!("as_char : {}", self.encoded);
        self.is_char().then_some(self.payload() as u8 as char)
    }

    #[inline(always)]
    pub fn as_boolean_unchecked(self) -> bool {
        //println!("as_boolean_unchecked : {}", self.encoded);
        self.payload() != 0
    }

    #[inline(always)]
    pub fn as_symbol(self) -> Option<Interned> {
        //println!("as_symbol : {}", self.encoded);
        self.is_symbol().then_some(Interned(self.payload() as u16))
    }

    #[inline(always)]
    pub fn is_ptr<T, PTR>(&self) -> bool
    where
        T: HasPointerTag,
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        //println!("is_ptr : {}", self.encoded);
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        value_ptr.is_valid()
    }

    #[inline(always)]
    pub fn as_ptr<T: HasPointerTag, PTR>(&self) -> Option<PTR>
    where
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        //println!("as_ptr : {}", self.encoded);
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        value_ptr.get()
    }

    #[inline(always)]
    pub unsafe fn as_ptr_unchecked<T: HasPointerTag, PTR>(&self) -> PTR
    where
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        //println!("as_ptr_unchecked : {}", self.encoded);
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        value_ptr.get_unchecked()
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Boolean(value: bool) -> Self {
        //println!("Boolean : {}", value);
        Self::new_boolean(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Integer(value: i32) -> Self {
        //println!("Integer : {}", value);
        Self::new_integer(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Double(value: f64) -> Self {
        //println!("Double : {}", value);
        Self::new_double(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Symbol(value: Interned) -> Self {
        //println!("Symbol : {}", value.0);
        Self::new_symbol(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn Char(value: char) -> Self {
        //println!("Char : {}", value);
        Self::new_char(value)
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn BigInteger<BigIntPtr>(value: BigIntPtr) -> Self
    where
        u64: From<BigIntPtr>,
        BigIntPtr: Deref<Target = BigInt> + From<u64>,
    {
        //println!("BigInteger : PTR");
        let tr = Self::new_big_integer(value);
        //println!("[out] big integer: {:#64b}", tr.encoded);
        tr
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    pub fn String<Ptr>(value: Ptr) -> Self
    where
        u64: From<Ptr>,
        Ptr: Deref<Target = String> + From<u64>,
    {
        //println!("String : PTR");
        Self::new_string(value)
    }

    pub unsafe fn as_mut_ptr(&self) -> *mut BaseValue {
        debug_assert!(
            self.is_ptr_type(),
            "calling as_mut_ptr() on a value that's not a pointer"
        );
        //println!("as_mut_ptr : {}", self.encoded);
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
