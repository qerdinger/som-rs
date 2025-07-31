use crate::gcref::Gc;
use crate::gcslice::GcSlice;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::slot::{SimpleSlot, Slot};

#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
use som_value::value::BaseValue;

// pub type SOMSlot = mmtk::vm::slot::SimpleSlot;

// because of NaN boxing, we make a new slot specifically for accessing values, which contain internally a GCRef
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SOMSlot {
    Simple(SimpleSlot),
    RefValueSlot(RefValueSlot),
}

/// Most common case: turn any pointer to any heap address into a slot.
/// This pointer must be on the heap or in a static variable! Otherwise, it becomes invalid when Rust discards it.
impl<T> From<&Gc<T>> for SOMSlot {
    fn from(value: &Gc<T>) -> Self {
        SOMSlot::Simple(SimpleSlot::from_address(Address::from_ref(value)))
    }
}

impl<T> From<&GcSlice<T>> for SOMSlot {
    fn from(value: &GcSlice<T>) -> Self {
        SOMSlot::Simple(SimpleSlot::from_address(Address::from_ref(value)))
    }
}

#[cfg(feature = "idiomatic")]
pub trait ToSlot {
    fn to_slot(&self) -> Option<SOMSlot>;
}

#[cfg(not(feature = "idiomatic"))]
impl From<*mut BaseValue> for SOMSlot {
    // we allow unsafe derefs since it's just for debugging
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(value: *mut BaseValue) -> Self {
        SOMSlot::RefValueSlot(RefValueSlot {
            value,
            #[cfg(debug_assertions)]
            expected_tag: (unsafe { *value }).tag(),
        })
    }
}


impl Slot for SOMSlot {
    fn load(&self) -> Option<ObjectReference> {
        match self {
            SOMSlot::Simple(e) => e.load(),
            SOMSlot::RefValueSlot(e) => e.load(),
        }
    }

    fn store(&self, object: ObjectReference) {
        match self {
            SOMSlot::Simple(e) => e.store(object),
            SOMSlot::RefValueSlot(e) => e.store(object),
        }
    }
}

#[cfg(not(feature = "idiomatic"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RefValueSlot {
    value: *mut BaseValue,
    #[cfg(debug_assertions)]
    /// for debugging. Sometimes, a bug makes it that the value's type changes in between the time
    /// it's stored and the time it's loaded.
    /// So this can be used to check the original type, by manually comparing it to the NaN boxing tag list.
    expected_tag: u64,
}

#[cfg(feature = "idiomatic")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RefValueSlot {
    value: *mut Box<String>,
    #[cfg(debug_assertions)]
    /// for debugging. Sometimes, a bug makes it that the value's type changes in between the time
    /// it's stored and the time it's loaded.
    /// So this can be used to check the original type, by manually comparing it to the NaN boxing tag list.
    expected_tag: u64,
}

unsafe impl Send for RefValueSlot {}

impl Slot for RefValueSlot {
    #[cfg(not(feature = "idiomatic"))]
    fn load(&self) -> Option<ObjectReference> {
        unsafe {
            #[cfg(debug_assertions)] // a bit silly, but otherwise rust complains release versions don't have expected_tag
            debug_assert!(
                (*self.value).is_ptr_type(),
                "load failed, pointer 0x{:x} does not point to a value pointer type (value: {}, tag: {}, expected_tag: {})",
                self.value as usize,
                (*self.value).as_u64(),
                (*self.value).tag(),
                self.expected_tag
            );
            ObjectReference::from_raw_address(Address::from_usize((*self.value).extract_pointer_bits() as usize))
        }
    }

    #[cfg(feature = "idiomatic")]
    fn load(&self) -> Option<ObjectReference> {
        //unsafe { TODO
            //#[cfg(debug_assertions)] // a bit silly, but otherwise rust complains release versions don't have expected_tag
            /*debug_assert!(
                (*self.value).is_ptr_type(),
                "load failed, pointer 0x{:x} does not point to a value pointer type (value: {}, tag: {}, expected_tag: {})",
                self.value as usize,
                (*self.value).as_u64(),
                (*self.value).tag(),
                self.expected_tag
            );*/
            //ObjectReference::from_raw_address(Address::from_usize((*self.value).extract_pointer_bits() as usize))
            //}
        todo!("To be implemented");
    }

    #[cfg(not(feature = "idiomatic"))]
    fn store(&self, object: ObjectReference) {
        unsafe {
            debug_assert!((*self.value).is_ptr_type());
            *self.value = BaseValue::new((*self.value).tag(), object.to_raw_address().as_usize() as u64);
            debug_assert!((*self.value).is_ptr_type());
        }
    }

    #[cfg(feature = "idiomatic")]
    fn store(&self, object: ObjectReference) {
        //unsafe { TODO
            //debug_assert!((*self.value).is_ptr_type());
            //*self.value = BaseValue::new((*self.value).tag(), object.to_raw_address().as_usize() as u64);
            //debug_assert!((*self.value).is_ptr_type());
        //}
        todo!("To be implemented");
    }
}
