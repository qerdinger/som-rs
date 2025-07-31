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

pub const TINY_STRING_TAG: u64 = 0b0000;
pub const NIL_TAG: u64 = 0b0001;
pub const INTEGER_TAG: u64 = 0b0010;
pub const BOOLEAN_TAG: u64 = 0b0011;
pub const DOUBLE_TAG: u64 = 0b0100;
pub const CHAR_TAG: u64 = 0b0101;
pub const STRING_TAG: u64 = 0b0110;
pub const BIG_INTEGER_TAG: u64 = 0b0111;

pub const DOUBLE_NEG_TAG: u64 = 0b1100;
pub const DOUBLE_BOXED_TAG: u64 = 0b1111;

// 4-bit pointer tags object types
pub const ARRAY_TAG: u64 = 0b1001;
pub const BLOCK_TAG: u64 = 0b1010;
pub const CLASS_TAG: u64 = 0b1011;
pub const INSTANCE_TAG: u64 = 0b1000;
pub const INVOKABLE_TAG: u64 = 0b1101;
pub const SYMBOL_TAG: u64 = 0b1110;

pub const IMMEDIATE_OFFSET: u64 = 0x7000_0000_0000_0000;
pub const ROTATE_AMOUNT: u32 = 1;
pub const PAYLOAD_SHIFT: u32 = 3;
const IM_DOUBLE_RANGE_MIN: u64 = 0x380;
const IM_DOUBLE_RANGE_MAX: u64 = 0x47F;

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
        if matches!(
            tag,
            STRING_TAG |
            BIG_INTEGER_TAG |
            ARRAY_TAG |
            BLOCK_TAG |
            CLASS_TAG |
            INSTANCE_TAG |
            INVOKABLE_TAG
        ) { return Self::new_ptr(tag, value); }

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
        let shifted = ptr << 1;
        shifted | tag
    }

    #[inline(always)]
    pub fn decode_ptr(encoded: u64) -> u64 {
        // Remove the tag and shift right by 1 to get the original pointer
        (encoded & !TAG_BITS) >> 1
    }

    #[inline(always)]
    pub fn is_ptr_type(self) -> bool {
        matches!(
            self.tag(),
            STRING_TAG |
            BIG_INTEGER_TAG |
            DOUBLE_BOXED_TAG |
            ARRAY_TAG |
            BLOCK_TAG |
            CLASS_TAG |
            INSTANCE_TAG |
            INVOKABLE_TAG
        )
    }

    pub unsafe fn as_something<PTR>(self) -> Option<PTR>
    where
        PTR: From<u64>,
    {
        self.is_ptr_type().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_u64(self) -> u64 {
        self.encoded
    }

    pub fn tag(self) -> u64 {
        self.encoded & TAG_BITS
    }

    #[inline(always)]
    pub fn payload(self) -> u64 {
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
        Ptr::from(self.payload())
    }

    #[inline(always)]
    pub fn extract_pointer_bits(self) -> u64 {
        self.payload()
    }

    // #[inline(always)]
    // pub fn new_tiny_str(value: Vec<u8>) -> Self {
    //     //println!("new tiny str !");
    //     let mut ptr: u64 = 0;
    //     ptr |= (value[0] as u64) << 0;
    //     ptr |= (value[1] as u64) << 8;
    //     ptr |= (value[2] as u64) << 16;
    //     ptr |= (value[3] as u64) << 24;
    //     ptr |= (value[4] as u64) << 32;
    //     ptr |= (value[5] as u64) << 40;
    //     ptr |= (value[6] as u64) << 48;
    //     //ptr |= (value[7] as u64) << 56;
    //     //println!("encoding with value : {:?}", value);
    //     //println!("encoded ptr : {:#64b}", ptr);
    //     let finalptr = Self::new(TINY_STRING_TAG, ptr);
    //     //println!("encode + tag : {:#64b}", finalptr.encoded as u64);
    //     finalptr
    // }
    #[inline(always)]
    pub fn new_tiny_str(value: Vec<u8>) -> Self {
        assert!(value.len() <= 7, "tiny str must be lower or equal to 7 bytes");

        let mut ptr = 0u64;
        for (i, &b) in value.iter().take(7).enumerate() {
            ptr |= (b as u64) << (i * 8);
        }

        if value.len() < 7 {
            let shift = (value.len() * 8) as u32;
            ptr |= u64::MAX << shift;
        }

        Self::new(TINY_STRING_TAG, ptr)
    }


    #[inline(always)]
    pub fn new_integer(value: i32) -> Self {
        Self::new(INTEGER_TAG, value as u64)
    }

    #[inline(always)]
    pub fn new_boolean(value: bool) -> Self {
        Self::new(BOOLEAN_TAG, value as u64)
    }

    #[inline(always)]
    pub fn new_double(value: f64) -> Self {
        let bits = value.to_bits();
        let sign = bits >> 63;
        let tag = if sign == 0 { DOUBLE_TAG } else { DOUBLE_NEG_TAG };
        let exponent  = (bits >> 52) & 0x7FF;

        let rolled = bits.rotate_left(ROTATE_AMOUNT);

        let in_range  = (exponent >= IM_DOUBLE_RANGE_MIN && exponent <= IM_DOUBLE_RANGE_MAX)
                 || bits == 0 || bits == 1;
        
        assert!(in_range, "Error: Exponent not in the expected range for Immediate Double, use AllocatedDouble");

        // Handling +/- 0
        let payload = if rolled <= 1 { rolled } else { rolled.wrapping_sub(IMMEDIATE_OFFSET) };

        // Integrate tag
        let encoded = (payload << PAYLOAD_SHIFT) | tag;

        // println!("Final double addr : {:#64b}", encoded);
        Self { encoded }
    }

    #[inline(always)]
    pub fn new_allocated_double<DoublePtr>(value: DoublePtr) -> Self
    where
        u64: From<DoublePtr>,
        DoublePtr: Deref<Target = f64> + From<u64>,
    {
        Self::new_ptr(DOUBLE_BOXED_TAG, value.into())
    }

    #[inline(always)]
    pub fn new_symbol(value: Interned) -> Self {
        Self::new(SYMBOL_TAG, value.0.into())
    }

    #[inline(always)]
    pub fn new_char(value: char) -> Self {
        Self::new(CHAR_TAG, value.into())
    }

    #[inline(always)]
    pub fn new_big_integer<BigIntPtr>(value: BigIntPtr) -> Self
    where
        u64: From<BigIntPtr>,
        BigIntPtr: Deref<Target = BigInt> + From<u64>,
    {
        let ptr: u64 = value.into();
        Self::new(BIG_INTEGER_TAG, ptr)
    }

    #[inline(always)]
    pub fn new_string<StringPtr>(value: StringPtr) -> Self
    where
        u64: From<StringPtr>,
        StringPtr: Deref<Target = String> + From<u64>,
    {
        let ptr: u64 = value.into();
        Self::new(STRING_TAG, ptr)
    }

    #[inline(always)]
    pub fn is_big_integer(self) -> bool {
        self.tag() == BIG_INTEGER_TAG
    }

    #[inline(always)]
    pub fn is_string(self) -> bool {
        self.tag() == STRING_TAG
    }

    #[inline(always)]
    pub fn is_tiny_str(self) -> bool {
        self.tag() == TINY_STRING_TAG
    }

    #[inline(always)]
    pub fn is_nil(self) -> bool {
        self.tag() == NIL_TAG
    }

    #[inline(always)]
    pub fn is_integer(self) -> bool {
        self.tag() == INTEGER_TAG
    }

    #[inline(always)]
    pub fn is_double(self) -> bool {
        matches!(self.tag(), DOUBLE_TAG | DOUBLE_NEG_TAG)
    }

    #[inline(always)]
    pub fn is_allocated_double(self) -> bool {
        self.tag() == DOUBLE_BOXED_TAG
    }

    #[inline(always)]
    pub fn is_boolean(self) -> bool {
        self.tag() == BOOLEAN_TAG
    }

    #[inline(always)]
    pub fn is_boolean_true(self) -> bool {
        self.payload() == 1
    }

    #[inline(always)]
    pub fn is_boolean_false(self) -> bool {
        self.payload() == 0
    }

    #[inline(always)]
    pub fn is_symbol(self) -> bool {
        self.tag() == SYMBOL_TAG
    }

    #[inline(always)]
    pub fn is_char(self) -> bool {
        self.tag() == CHAR_TAG
    }

    #[inline(always)]
    pub fn as_big_integer<BigIntPtr>(self) -> Option<BigIntPtr>
    where
        u64: From<BigIntPtr>,
        BigIntPtr: From<u64>,
    {
        self.is_big_integer().then(|| self.extract_gc_cell())
    }

    #[inline(always)]
    pub fn as_string<StringPtr>(self) -> Option<StringPtr>
    where
        StringPtr: From<u64>,
        StringPtr: Deref<Target = String>,
    {
        self.is_string().then(|| self.extract_gc_cell())
    }

    // #[inline(always)]
    // pub fn as_tiny_str(self) -> Option<Vec<u8>> {
    //     if !self.is_tiny_str() {
    //         return None;
    //     }
    //     let mut bytes = [0u8];
    //     let payload: u64 = self.payload();
    //     //println!("payload : {:#64b}", payload);
    //     bytes[0] = ((payload >>  0) & 0xFF) as u8;
    //     bytes[1] = ((payload >>  8) & 0xFF) as u8;
    //     bytes[2] = ((payload >> 16) & 0xFF) as u8;
    //     bytes[3] = ((payload >> 24) & 0xFF) as u8;
    //     bytes[4] = ((payload >> 32) & 0xFF) as u8;
    //     bytes[5] = ((payload >> 40) & 0xFF) as u8;
    //     bytes[6] = ((payload >> 48) & 0xFF) as u8;
    //     //bytes[7] = ((payload >> 56) & 0xFF) as u8;
    //     //println!("tmp : {:?}", bytes);
    //     Some(bytes)
    // }

    // #[inline(always)]
    // pub fn as_tiny_str(self) -> Option<Vec<u8>> {
    //     if !self.is_tiny_str() {
    //         return None;
    //     }

    //     let mut bytes = Vec::new();
    //     let mut value = self.payload();

    //     while value != 0 {
    //         bytes.push((value & 0xFF) as u8);
    //         value >>= 8;
    //     }

    //     Some(bytes)
    // }
    #[inline(always)]
    pub fn as_tiny_str(self) -> Option<Vec<u8>> {
        if !self.is_tiny_str() {
            return None;
        }
        let mut bytes = Vec::new();
        let mut v = self.payload();

        for _ in 0..7 {
            let b = (v & 0xFF) as u8;
            if b == 0xFF {
                break;
            }
            bytes.push(b);
            v >>= 8;
        }

        Some(bytes)
    }
    
    #[inline(always)]
    pub fn as_integer(self) -> Option<i32> {
        self.is_integer().then_some(self.payload() as i32)
    }

    #[inline(always)]
    pub fn as_double(self) -> Option<f64> {
        match self.tag() {
            DOUBLE_TAG | DOUBLE_NEG_TAG => {
                // Retrieve payload
                let payload = self.encoded >> PAYLOAD_SHIFT;

                // Payload is lower or equal to 1 handle special case +/- 0
                let rebased = if payload <= 1 { payload } else { payload.wrapping_add(IMMEDIATE_OFFSET) };

                let bits = rebased.rotate_right(ROTATE_AMOUNT);
                Some(f64::from_bits(bits))
            },
            // DOUBLE_BOXED_TAG => {
            //     // println!("Decoding : {:#64b}", self.encoded);
            //     // println!("Decoding PTR : {:#64b}", Self::decode_ptr(self.encoded));
            //     // Some((*Box::new(Self::decode_ptr(self.encoded))) as f64)
            //     // let data = u64::from(self.payload());
            //     // println!("Data : {}", *data);
            //     return Some(2.5 as f64);
            // }
            _ => None,
        }
    }

    #[inline(always)]
    pub fn as_allocated_double<DoublePtr>(self) -> Option<DoublePtr>
    where
        DoublePtr: From<u64>,
        DoublePtr: Deref<Target = f64>,
    {
        self.is_allocated_double().then(|| self.extract_gc_cell())
    }


    #[inline(always)]
    pub fn as_boolean(self) -> Option<bool> {
        self.is_boolean().then_some(self.is_boolean_true())
    }

    #[inline(always)]
    pub fn as_char(self) -> Option<char> {
        self.is_char().then_some(self.payload() as u8 as char)
    }

    #[inline(always)]
    pub fn as_boolean_unchecked(self) -> bool {
        self.payload() != 0
    }

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

    #[inline(always)]
    pub unsafe fn as_ptr_unchecked<T: HasPointerTag, PTR>(&self) -> PTR
    where
        PTR: Deref<Target = T> + From<u64> + Into<u64>,
    {
        let value_ptr: TypedPtrValue<T, PTR> = (*self).into();
        value_ptr.get_unchecked()
    }

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
    pub fn AllocatedDouble<Ptr>(value: Ptr) -> Self
    where
        u64: From<Ptr>,
        Ptr: Deref<Target = f64> + From<u64>,
    {
        Self::new_allocated_double(value)
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
    pub fn TinyStr(value: Vec<u8>) -> Self {
        Self::new_tiny_str(value)
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

    pub unsafe fn as_mut_ptr(&self) -> *mut BaseValue {
        debug_assert!(self.is_ptr_type(), "calling as_mut_ptr() on a value that's not a pointer");
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
