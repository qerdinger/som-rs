use super::Value;
use crate::gc::VecValue;
use crate::universe::Universe;
use crate::value::value_enum::ValueEnum;
use crate::vm_objects::block::Block;
use crate::vm_objects::class::Class;
use crate::vm_objects::instance::Instance;
use crate::vm_objects::method::Method;
use num_bigint::BigInt;
use som_gc::debug_assert_valid_semispace_ptr_value;
use som_gc::gcref::Gc;
use som_gc::gcslice::GcSlice;
use som_value::delegate_to_base_value;
use som_value::interned::Interned;
use som_value::value::*;
use som_value::value_ptr::{HasPointerTag, TypedPtrValue};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

impl Deref for Value {
    type Target = BaseValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BaseValue> for Value {
    fn from(value: BaseValue) -> Self {
        Value(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value(BaseValue::from(value))
    }
}

#[allow(non_snake_case)]
impl Value {
    pub const TRUE: Self = Value(BaseValue::TRUE);
    pub const FALSE: Self = Value(BaseValue::FALSE);
    pub const NIL: Self = Value(BaseValue::NIL);
    pub const INTEGER_ZERO: Self = Value(BaseValue::INTEGER_ZERO);
    pub const INTEGER_ONE: Self = Value(BaseValue::INTEGER_ONE);

    delegate_to_base_value!(
        new_boolean(value: bool) -> Self,
        new_integer(value: i32) -> Self,
        new_double(value: f64) -> Self,
        new_allocated_double(value: Gc<f64>) -> Self,
        new_symbol(value: Interned) -> Self,
        new_big_integer(value: Gc<BigInt>) -> Self,
        new_tiny_str(value: u8) -> Self,
        new_string(value: Gc<String>) -> Self,
        Boolean(value: bool) -> Self,
        Integer(value: i32) -> Self,
        Double(value: f64) -> Self,
        AllocatedDouble(value: Gc<f64>) -> Self,
        Symbol(value: Interned) -> Self,
        BigInteger(value: Gc<BigInt>) -> Self,
        TinyStr(value: u8) -> Self,
        String(value: Gc<String>) -> Self,
        // new_char(value: char) -> Self,
        // Char(value: char) -> Self,
    );

    #[inline(always)]
    pub fn is_value_ptr<T: HasPointerTag>(&self) -> bool {
        self.0.is_ptr::<T, Gc<T>>()
    }

    #[inline(always)]
    pub fn as_value_ptr<T: HasPointerTag>(&self) -> Option<Gc<T>> {
        self.0.as_ptr::<T, Gc<T>>()
    }

    #[inline(always)]
    pub fn as_array(self) -> Option<VecValue> {
        (self.tag() == ARRAY_TAG).then(|| VecValue(GcSlice::from(self.extract_pointer_bits())))
    }

    #[inline(always)]
    pub fn as_block(self) -> Option<Gc<Block>> {
        self.as_value_ptr::<Block>()
    }

    #[inline(always)]
    pub fn as_class(self) -> Option<Gc<Class>> {
        self.as_value_ptr::<Class>()
    }

    #[inline(always)]
    pub fn as_instance(self) -> Option<Gc<Instance>> {
        self.as_value_ptr::<Instance>()
    }

    #[inline(always)]
    pub fn as_invokable(self) -> Option<Gc<Method>> {
        self.as_value_ptr::<Method>()
    }

    #[inline(always)]
    pub fn class(&self, universe: &Universe) -> Gc<Class> {
        debug_assert_valid_semispace_ptr_value!(self);
        match self.tag() {
            NIL_TAG => universe.core.nil_class(),
            BOOLEAN_TAG => {
                if self.as_boolean().unwrap() {
                    universe.core.true_class()
                } else {
                    universe.core.false_class()
                }
            }
            INTEGER_TAG | BIG_INTEGER_TAG => universe.core.integer_class(),
            SYMBOL_TAG => universe.core.symbol_class(),
            TINY_STRING_TAG => universe.core.string_class(),
            STRING_TAG => universe.core.string_class(),
            ARRAY_TAG => universe.core.array_class(),
            BLOCK_TAG => self.as_block().unwrap().class(universe),
            INSTANCE_TAG => self.as_instance().unwrap().class(),
            CLASS_TAG => self.as_class().unwrap().class(),
            INVOKABLE_TAG => self.as_invokable().unwrap().class(universe),
            _ => {
                if self.is_double() {
                    universe.core.double_class()
                } else if self.is_allocated_double() {
                    universe.core.double_class()
                } else {
                    panic!("unknown tag")
                }
            }
            // CHAR_TAG => universe.core.string_class(),
        }
    }

    pub fn lookup_method(&self, universe: &Universe, signature: Interned) -> Option<Gc<Method>> {
        self.class(universe).lookup_method(signature)
    }

    pub fn to_string(&self, universe: &Universe) -> String {
        match self.tag() {
            NIL_TAG => "nil".to_string(),
            BOOLEAN_TAG => self.as_boolean().unwrap().to_string(),
            INTEGER_TAG => self.as_integer().unwrap().to_string(),
            BIG_INTEGER_TAG => self.as_big_integer::<Gc<BigInt>>().unwrap().to_string(),
            _ if self.is_double() => self.as_double().unwrap().to_string(),
            SYMBOL_TAG => {
                let symbol = universe.lookup_symbol(self.as_symbol().unwrap());
                if symbol.chars().any(|ch| ch.is_whitespace() || ch == '\'') {
                    format!("#'{}'", symbol.replace("'", "\\'"))
                } else {
                    format!("#{}", symbol)
                }
            }
            TINY_STRING_TAG => format!("{}", self.as_tiny_str().unwrap() as char),
            STRING_TAG => self.as_string::<Gc<String>>().unwrap().to_string(),
            ARRAY_TAG => {
                let strings: Vec<String> = self
                    .as_array()
                    .unwrap()
                    .0
                    .iter()
                    .map(|value| value.to_string(universe))
                    .collect();
                format!("#({})", strings.join(" "))
            }
            BLOCK_TAG => {
                let block = self.as_block().unwrap();
                format!("instance of Block{}", block.nb_parameters() + 1)
            }
            INSTANCE_TAG => {
                let instance = self.as_instance().unwrap();
                format!("instance of {} class", instance.class().name())
            }
            CLASS_TAG => self.as_class().unwrap().name().to_string(),
            INVOKABLE_TAG => {
                let invokable = self.as_invokable().unwrap();
                format!("{}>>#{}", invokable.holder().name(), invokable.signature())
            }
            _ => panic!("unknown tag"),
        }
    }

    #[inline(always)]
    pub fn Array(value: VecValue) -> Self {
        Value(BaseValue::new(ARRAY_TAG, value.0.into()))
    }

    #[inline(always)]
    pub fn Block(value: Gc<Block>) -> Self {
        TypedPtrValue::new(value).into()
    }

    #[inline(always)]
    pub fn Class(value: Gc<Class>) -> Self {
        TypedPtrValue::new(value).into()
    }

    #[inline(always)]
    pub fn Instance(value: Gc<Instance>) -> Self {
        TypedPtrValue::new(value).into()
    }

    #[inline(always)]
    pub fn Invokable(value: Gc<Method>) -> Self {
        TypedPtrValue::new(value).into()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.as_u64() == other.as_u64() {
            true
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_double()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_allocated_double::<Gc<f64>>(), other.as_allocated_double::<Gc<f64>>()) {
            *a == *b
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_allocated_double::<Gc<f64>>()) {
            a == *b
        } else if let (Some(a), Some(b)) = (self.as_allocated_double::<Gc<f64>>(), other.as_double()) {
            *a == b
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_double()) {
            (a as f64) == b
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_integer()) {
            (b as f64) == a
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_allocated_double::<Gc<f64>>()) {
            (a as f64) == *b
        } else if let (Some(a), Some(b)) = (self.as_allocated_double::<Gc<f64>>(), self.as_integer()) {
            *a == (b as f64)
        } else if let (Some(a), Some(b)) = (self.as_big_integer::<Gc<BigInt>>(), other.as_big_integer()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_big_integer::<Gc<BigInt>>(), other.as_integer()) {
            (*a).eq(&BigInt::from(b))
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_big_integer::<Gc<BigInt>>()) {
            BigInt::from(a).eq(&*b)
        } else if let (Some(a), Some(b)) = (self.as_string::<Gc<String>>(), other.as_string::<Gc<String>>()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_tiny_str(), other.as_tiny_str()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_string::<Gc<String>>(), other.as_tiny_str()) {
            *a == format!("{}", b as char)
        } else if let (Some(a), Some(b)) = (self.as_tiny_str(), other.as_string::<Gc<String>>()) {
            format!("{}", a as char) == *b
        } else if let (Some(a), Some(b)) = (self.as_symbol(), other.as_symbol()) {
            a.eq(&b)
        } else {
            false
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(arr) = self.as_array() {
            f.write_fmt(format_args!("VecValue({:?})", arr.0))
        } else {
            ValueEnum::from(*self).fmt(f)
        }
    }
}