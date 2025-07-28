#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
use som_value::value::BaseValue;

/// Value type(s!), and value-related code.
/// Used to convert types, used by primitives.
pub mod convert;

/// Our default type: NaN boxed
#[cfg(feature = "nan")]
pub mod nanboxed;

#[cfg(feature = "l3bits")]
pub mod l3bits;

#[cfg(feature = "l4bits")]
pub mod l4bits;

#[cfg(feature = "idiomatic")]
pub mod idiomatic;

/// Our enum based type
pub mod value_enum;
mod value_ptr;

/// Represents an SOM value.
#[cfg(any(feature = "nan", feature = "l4bits", feature = "l3bits"))]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Value(pub BaseValue);

#[cfg(feature = "idiomatic")]
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Value(pub value_enum::ValueEnum);

// TODO: we should be able to switch between Value (nanboxed) and ValueEnum at will. That used to be the case, but I broke those changes. TODO restore
