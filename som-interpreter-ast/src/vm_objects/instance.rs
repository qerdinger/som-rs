use crate::value::Value;
use crate::vm_objects::class::Class;
use som_gc::gcref::Gc;
use std::fmt;

/// Represents a generic (non-primitive) class instance.
#[derive(Clone)]
pub struct Instance {
    /// The class of which this is an instance from.
    pub class: Gc<Class>,
    /// This instance's fields.
    pub fields: Vec<Value>,
}

impl Instance {
    /// Construct an instance for a given class.
    pub fn from_class(class: Gc<Class>) -> Self {
        let fields = class.fields.iter().map(|_| Value::NIL).collect();
        Self { class, fields }
    }

    /// Get the class of which this is an instance from.
    pub fn class(&self) -> Gc<Class> {
        self.class.clone()
    }

    /// Get the superclass of this instance's class.
    pub fn super_class(&self) -> Option<Gc<Class>> {
        self.class.super_class()
    }

    /// Search for a field binding.
    pub fn lookup_field(&self, idx: u8) -> &Value {
        match cfg!(debug_assertions) {
            true => self.fields.get(idx as usize).unwrap(),
            false => unsafe { self.fields.get_unchecked(idx as usize) },
        }
    }

    /// Assign a value to a field binding.
    pub fn assign_field(&mut self, idx: u8, value: Value) {
        *self.fields.get_mut(idx as usize).unwrap() = value;
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance")
            .field("name", &self.class.name())
            .field("fields", &self.fields.len())
            .field("methods", &self.class().methods.len())
            .finish()
    }
}
