use super::Value;
use crate::gc::VecValue;
use crate::universe::Universe;
use crate::value::value_enum::ValueEnum;
use crate::vm_objects::block::Block;
use crate::vm_objects::class::Class;
use crate::vm_objects::instance::Instance;
use crate::vm_objects::method::Method;
use num_bigint::BigInt;
use som_gc::gcref::Gc;

#[cfg(not(feature = "idiomatic"))]
use crate::value::value_ptr::TypedPtrValue;

#[cfg(not(feature = "idiomatic"))]
use som_gc::debug_assert_valid_semispace_ptr_value;
#[cfg(not(feature = "idiomatic"))]
use som_gc::gcslice::GcSlice;

//use som_value::delegate_to_base_value;
macro_rules! delegate_to_value_enum {
    ($($fn_name:ident ( $($arg:ident : $arg_ty:ty),* ) ),* $(,)?) => {
        $(
            #[inline(always)]
            pub fn $fn_name($( $arg: $arg_ty ),*) -> Self {
                Value(ValueEnum::$fn_name($($arg),*))
            }
        )*
    };
}


use som_value::interned::Interned;
//use som_value::value::*;
//use som_value::value_ptr::{HasPointerTag, TypedPtrValue};
#[cfg(not(feature = "idiomatic"))]
use std::fmt;
#[cfg(not(feature = "idiomatic"))]
use std::fmt::{Debug, Formatter};

use std::ops::Deref;
use mmtk::util::Address;
use mmtk::vm::slot::SimpleSlot;
use som_gc::slot::SOMSlot;

impl Deref for Value {
    type Target = ValueEnum;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/*
impl From<ValueEnum> for Value {
    fn from(value: ValueEnum) -> Self {
        Value(value)
    }
}
*/

/*
impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value(ValueEnum::from(value))
    }
}
*/

impl som_gc::slot::ToSlot for Value {
    fn to_slot(&self) -> Option<SOMSlot> {
        match &self.0 {
            ValueEnum::Class(gc) => Some(SOMSlot::Simple(SimpleSlot::from_address(Address::from_ref(gc)))),
            ValueEnum::Instance(gc) => Some(SOMSlot::Simple(SimpleSlot::from_address(Address::from_ref(gc)))),
            ValueEnum::Block(gc) => Some(SOMSlot::Simple(SimpleSlot::from_address(Address::from_ref(gc)))),
            ValueEnum::Invokable(gc) => Some(SOMSlot::Simple(SimpleSlot::from_address(Address::from_ref(gc)))),
            ValueEnum::Array(gc) => Some(SOMSlot::Simple(SimpleSlot::from_address(Address::from_ref(gc)))),
            _ => None,
        }
    }
}

#[allow(non_snake_case)]
impl Value {
    pub const TRUE: Self = Value(ValueEnum::Boolean(true));
    pub const FALSE: Self = Value(ValueEnum::Boolean(false));
    pub const NIL: Self = Value(ValueEnum::Nil);
    pub const INTEGER_ZERO: Self = Value(ValueEnum::Integer(0));
    pub const INTEGER_ONE: Self = Value(ValueEnum::Integer(1));

    delegate_to_value_enum!(
        new_boolean(value: bool),
        new_integer(value: i32),
        new_double(value: f64),
        new_symbol(value: Interned),
        new_tiny_str(value: i64),
        new_big_integer(value: Gc<BigInt>),
        new_string(value: Gc<String>),
        Boolean(value: bool),
        TinyStr(value: i64),
        Integer(value: i32),
        Double(value: f64),
        Symbol(value: Interned),
        BigInteger(value: Gc<BigInt>),
        String(value: Gc<String>),
    );

    #[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
    #[inline(always)]
    pub fn is_value_ptr<T: HasPointerTag>(&self) -> bool {
        self.0.is_ptr::<T, Gc<T>>()
    }

    #[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
    #[inline(always)]
    pub fn as_value_ptr<T: HasPointerTag>(&self) -> Option<Gc<T>> {
        self.0.as_ptr::<T, Gc<T>>()
    }

    /// Returns this value as an array, if such is its type.
    #[inline(always)]
    pub fn as_array(self) -> Option<VecValue> {
        match self.0 {
            ValueEnum::Array(arr) => Some(VecValue(arr.clone().into())),
            //ValueEnum::Array(arr) => {
            //    let ptr: u64 = arr.into();
            //    Some(VecValue(GcSlice::from(ptr)))
            //},
            _ => None,
        }
    }

    /// Returns this value as a block, if such is its type.
    #[inline(always)]
    pub fn as_block(self) -> Option<Gc<Block>> {
        //self.as_value_ptr::<Block>()
        match self.0 {
            ValueEnum::Block(block) => Some(block),
            _ => None,
        }
    }

    /// Returns this value as a class, if such is its type.
    #[inline(always)]
    pub fn as_class(self) -> Option<Gc<Class>> {
        // self.as_value_ptr::<Class>()
        match self.0 {
            ValueEnum::Class(class) => Some(class),
            _ => None,
        }
    }
    /// Returns this value as an instance, if such is its type.
    #[inline(always)]
    pub fn as_instance(self) -> Option<Gc<Instance>> {
        //self.as_value_ptr::<Instance>()
        match self.0 {
            ValueEnum::Instance(instance) => Some(instance),
            _ => None,
        }
    }
    /// Returns this value as an invokable, if such is its type.
    #[inline(always)]
    pub fn as_invokable(self) -> Option<Gc<Method>> {
        //self.as_value_ptr::<Method>()
        match self.0 {
            ValueEnum::Invokable(method) => Some(method),
            _ => None,
        }
    }

    /// Get the class of the current value.
    #[inline(always)]
    pub fn class(&self, universe: &Universe) -> Gc<Class> {
        //debug_assert_valid_semispace_ptr_value!(self);
        match self.0 {
            ValueEnum::Nil => universe.core.nil_class(),
            ValueEnum::Boolean(_) => {
                if self.as_boolean().unwrap() {
                    universe.core.true_class()
                } else {
                    universe.core.false_class()
                }
            }
            ValueEnum::Integer(_) | ValueEnum::BigInteger(_) => universe.core.integer_class(),
            ValueEnum::Symbol(_) => universe.core.symbol_class(),
            ValueEnum::String(_) => universe.core.string_class(),
            ValueEnum::TinyStr(_) => universe.core.string_class(),
            ValueEnum::Array(_) => universe.core.array_class(),
            ValueEnum::Block(_) => self.clone().as_block().unwrap().class(universe),
            ValueEnum::Instance(_) => self.clone().as_instance().unwrap().class(),
            ValueEnum::Class(_) => self.clone().as_class().unwrap().class(),
            ValueEnum::Invokable(_) => self.clone().as_invokable().unwrap().class(universe),
            _ => {
                if self.is_double() {
                    universe.core.double_class()
                } else {
                    panic!("unknown tag");
                }
            }
        }
    }

    /// Search for a given method for this value.
    pub fn lookup_method(&self, universe: &Universe, signature: Interned) -> Option<Gc<Method>> {
        self.class(universe).lookup_method(signature)
    }

    /// Get the string representation of this value.
    pub fn to_string(&self, universe: &Universe) -> String {
        match &self.0 {
            ValueEnum::Nil => "nil".to_string(),
            ValueEnum::Boolean(boolean) => boolean.to_string(),
            ValueEnum::Integer(integer) => integer.to_string(),
            ValueEnum::BigInteger(big_int) => big_int.to_string(),
            ValueEnum::Double(double) => double.to_string(),
            ValueEnum::Symbol(sym) => {
                let symbol = universe.lookup_symbol(*sym);
                if symbol.chars().any(|ch| ch.is_whitespace() || ch == '\'') {
                    format!("#'{}'", symbol.replace("'", "\\'"))
                } else {
                    format!("#{}", symbol)
                }
            }
            ValueEnum::String(s) => s.to_string(),
            ValueEnum::Array(_) => {
                // TODO: I think we can do better here (less allocations).
                let strings: Vec<String> = self.clone().as_array().unwrap().0.iter().map(|value| value.to_string(universe)).collect();
                format!("#({})", strings.join(" "))
            }
            ValueEnum::Block(block) => {
                // let block = self.as_block().unwrap();
                format!("instance of Block{}", block.nb_parameters() + 1)
            }
            ValueEnum::Instance(instance) => {
                // let instance = self.as_instance().unwrap();
                format!("instance of {} class", instance.class().name(),)
            }
            ValueEnum::Class(class) => class.name().to_string(),
            ValueEnum::Invokable(invokable) => {
                // let invokable = self.as_invokable().unwrap();
                format!("{}>>#{}", invokable.holder().name(), invokable.signature(),)
            }
            _ => {
                panic!("unknown tag")
            }
        }
    }
}

// for backwards compatibility with current code... and maybe easy replacement with ValueEnum?
#[allow(non_snake_case)]
impl Value {
    #[inline(always)]
    pub fn Array(value: VecValue) -> Self {
        // TODO use TypedPtrValue somehow instead
        Value(ValueEnum::Array(value.0.into()))
    }

    #[inline(always)]
    pub fn Block(value: Gc<Block>) -> Self {
        //TypedPtrValue::new(ValueEnum::Block(value)).into()
        Value(ValueEnum::Block(value))
    }

    #[inline(always)]
    pub fn Class(value: Gc<Class>) -> Self {
        //TypedPtrValue::new(value).into()
        Value(ValueEnum::Class(value))
    }

    #[inline(always)]
    pub fn Instance(value: Gc<Instance>) -> Self {
        //TypedPtrValue::new(value).into()
        Value(ValueEnum::Instance(value))
    }

    #[inline(always)]
    pub fn Invokable(value: Gc<Method>) -> Self {
        //TypedPtrValue::new(value).into()
        Value(ValueEnum::Invokable(value))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // println!("PE2 [{:?}]==[{:?}]", self, other);
        if self.is_nil() && other.is_nil() {
            true
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_integer()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_double()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_double()) {
            (a as f64) == b
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_integer()) {
            (b as f64) == a
        } else if let (Some(a), Some(b)) = (self.as_big_integer(), other.as_big_integer()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_big_integer(), other.as_integer()) {
            (*a).eq(&BigInt::from(b))
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_big_integer()) {
            BigInt::from(a).eq(&*b)
        } else if let (Some(a), Some(b)) = (self.as_tiny_str(), other.as_tiny_str()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_string(), other.as_string()) {
            *a == *b
        } else if let (Some(a), Some(b)) = (self.as_symbol(), other.as_symbol()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.clone().as_block(), other.clone().as_block()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.clone().as_class(), other.clone().as_class()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.clone().as_instance(), other.clone().as_instance()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.clone().as_invokable(), other.clone().as_invokable()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.clone().as_array(), other.clone().as_array()) {
            a.eq(&b)
        } else {
            false
        }
    }
}

/*
impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(arr) = self.as_array() {
            f.write_fmt(format_args!("VecValue({:?})", arr.0))
        } else {
            ValueEnum::from(*self).fmt(f)
        }
    }
}
TODO
*/