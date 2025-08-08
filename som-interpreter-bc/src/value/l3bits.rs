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
use som_value::delegate_to_base_value;
use som_value::interned::Interned;
use som_value::value::*;
use som_value::value_ptr::HasPointerTag;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use crate::gc::BCObjMagicId;

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
        new_tiny_str(value: Vec<u8>) -> Self,
        new_string(value: Gc<String>) -> Self,
        Boolean(value: bool) -> Self,
        Integer(value: i32) -> Self,
        Double(value: f64) -> Self,
        AllocatedDouble(value: Gc<f64>) -> Self,
        Symbol(value: Interned) -> Self,
        BigInteger(value: Gc<BigInt>) -> Self,
        TinyStr(value: Vec<u8>) -> Self,
        String(value: Gc<String>) -> Self,
        // new_char(value: char) -> Self,
        // Char(value: char) -> Self,
    );

    #[inline(always)]
    pub fn is_value_ptr<T: HasPointerTag>(&self) -> bool {
        // self.0.is_ptr::<T, Gc<T>>()
        self.is_ptr_type()
    }

    #[inline(always)]
    pub fn as_value_ptr<T: HasPointerTag>(&self) -> Option<Gc<T>> {
        // self.0.as_ptr::<T, Gc<T>>()
        Some(self.extract_pointer_bits().into())
    }

    #[inline(always)]
    pub fn as_big_integer(self) -> Option<Gc<BigInt>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::BigInt => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    pub fn as_string(self) -> Option<Gc<String>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::String => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    pub fn as_allocated_double(self) -> Option<Gc<f64>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::Double => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    pub fn as_array(self) -> Option<VecValue> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::ArrayVal => Some(VecValue(ptr.into())),
                _ => None,
            }
        }
    }

    #[inline(always)]
    pub fn as_block(self) -> Option<Gc<Block>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::Block => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    pub fn as_class(self) -> Option<Gc<Class>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::Class => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    pub fn as_instance(self) -> Option<Gc<Instance>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::Instance => Some(ptr.into()),
                _ => None,
            }
        }
    }

    #[inline(always)]
    pub fn as_invokable(self) -> Option<Gc<Method>> {
        if !self.is_ptr_type() {
            return None;
        }
        let ptr = self.extract_pointer_bits();
        unsafe {
            let header: &BCObjMagicId = &*((ptr - 8) as *const BCObjMagicId);
            match header {
                BCObjMagicId::Method => Some(ptr.into()),
                _ => None,
            }
        }
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
            SYMBOL_TAG => universe.core.symbol_class(),
            INTEGER_TAG => universe.core.integer_class(),
            TINY_STRING_TAG => universe.core.string_class(),
            _ => {
                if self.is_double() {
                    return universe.core.double_class();
                } else if self.is_ptr_type() {
                    if self.as_array().is_some() {
                        return universe.core.array_class(); 
                    } else if let Some(blk) = self.as_block() {
                        return blk.class(universe);
                    } else if let Some(instance) = self.as_instance() {
                        return instance.class();
                    } else if let Some(cls) = self.as_class() {
                        return cls.class();
                    } else if let Some(invokable) = self.as_invokable() {
                        return invokable.class(universe);
                    } else if let Some(_) = self.as_big_integer() {
                        return universe.core.integer_class();
                    } else if let Some(_) = self.as_string() {
                        return universe.core.string_class();
                    } else if let Some(_) = self.as_allocated_double() {
                        return universe.core.double_class();
                    } else {
                        panic!("Error: Pointer not recognized!")
                    }
                } else {
                    panic!("unknown tag")
                }
            }
            // CHAR_TAG => universe.core.string_class(),
            //ARRAY_TAG => universe.core.array_class(),
            //BLOCK_TAG => self.as_block().unwrap().class(universe),
            //INSTANCE_TAG => self.as_instance().unwrap().class(),
            //CLASS_TAG => self.as_class().unwrap().class(),
            //INVOKABLE_TAG => self.as_invokable().unwrap().class(universe),
            /*_ => {
                if self.is_double() {
                    universe.core.double_class()
                } else if self.is_allocated_double() {
                    universe.core.double_class()
                } else {
                    panic!("unknown tag")
                }
            }*/
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
            _ if self.is_double() => self.as_double().unwrap().to_string(),
            TINY_STRING_TAG => String::from_utf8(self.as_tiny_str().unwrap().to_vec()).unwrap(),
            _ => {
                if let Some(block) = self.as_block() {
                    format!("instance of Block{}", block.nb_parameters() + 1)
                } else if let Some(class) = self.as_class() {
                    class.name().to_string()
                } else if let Some(instance) = self.as_instance() {
                    format!("instance of {} class", instance.class().name(),)
                } else if let Some(invokable) = self.as_invokable() {
                    format!("{}>>#{}", invokable.holder().name(), invokable.signature(),)
                } else if let Some(arr) = self.as_array() {
                    // TODO: I think we can do better here (less allocations).
                    let strings: Vec<String> = arr.iter().map(|value| value.to_string(universe)).collect();
                    format!("#({})", strings.join(" "))
                } else if let Some(big_int) = self.as_big_integer() {
                    big_int.to_string()
                } else if let Some(string) = self.as_string() {
                    string.to_string()
                } else if let Some(symbol) = self.as_symbol() {
                    let symbol = universe.lookup_symbol(symbol);
                    if symbol.chars().any(|ch| ch.is_whitespace() || ch == '\'') {
                        format!("#'{}'", symbol.replace("'", "\\'"))
                    } else {
                        format!("#{}", symbol)
                    }
                } else {
                    panic!("unknown tag")
                }
            }
            /*
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
             */
        }
    }

    #[inline(always)]
    pub fn Array(value: VecValue) -> Self {
        Value(BaseValue::new(PTR_TAG, value.0.into()))
    }

    #[inline(always)]
    pub fn Block(value: Gc<Block>) -> Self {
        Value(BaseValue::new(PTR_TAG, value.into()))
    }

    #[inline(always)]
    pub fn Class(value: Gc<Class>) -> Self {
        Value(BaseValue::new(PTR_TAG, value.into()))
    }

    #[inline(always)]
    pub fn Instance(value: Gc<Instance>) -> Self {
        Value(BaseValue::new(PTR_TAG, value.into()))
    }

    #[inline(always)]
    pub fn Invokable(value: Gc<Method>) -> Self {
        Value(BaseValue::new(PTR_TAG, value.into()))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.as_u64() == other.as_u64() {
            true
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_double()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_allocated_double(), other.as_allocated_double()) {
            *a == *b
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_allocated_double()) {
            a == *b
        } else if let (Some(a), Some(b)) = (self.as_allocated_double(), other.as_double()) {
            *a == b
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_double()) {
            (a as f64) == b
        } else if let (Some(a), Some(b)) = (self.as_double(), other.as_integer()) {
            (b as f64) == a
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_allocated_double()) {
            (a as f64) == *b
        } else if let (Some(a), Some(b)) = (self.as_allocated_double(), self.as_integer()) {
            *a == (b as f64)
        } else if let (Some(a), Some(b)) = (self.as_big_integer(), other.as_big_integer()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_big_integer(), other.as_integer()) {
            (*a).eq(&BigInt::from(b))
        } else if let (Some(a), Some(b)) = (self.as_integer(), other.as_big_integer()) {
            BigInt::from(a).eq(&*b)
        } else if let (Some(a), Some(b)) = (self.as_string(), other.as_string()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_tiny_str(), other.as_tiny_str()) {
            a == b
        } else if let (Some(a), Some(b)) = (self.as_string(), other.as_tiny_str()) {
            *a == String::from_utf8(b.to_vec()).expect("Cannot be converted into String")
        } else if let (Some(a), Some(b)) = (self.as_tiny_str(), other.as_string()) {
            String::from_utf8(a.to_vec()).expect("Cannot be converted into String") == *b
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