
#[cfg(all(feature = "use-self", feature = "use-lbits"))]
compile_error!("Only one tagging scheme should be enabled at a time.");


#[cfg(feature = "use-lbits")]
pub use som_value_lbits::*;


#[cfg(feature = "use-self")]
/// To convert values to types, and vice versa.
pub mod convert;

#[cfg(feature = "use-self")]
/// Shared value representation logic (NaN boxing really)
pub mod value;

#[cfg(feature = "use-self")]
/// Class for storing a value itself as a typed pointer.
pub mod value_ptr;

#[cfg(feature = "use-self")]
/// The representation for interned strings. Made to work with som-core/interner.
/// Values need to be able to use it, and som-core depends on som-value, so we'd have a circular
/// dependency if this wasn't here.
/// It is in the scope of the crate ("how we represent values in SOM") but is annoying if this
/// crate is ever to be standalone. It's a bit annoying for it not to be there, but it can be
/// kicked out.
pub mod interned;

#[cfg(feature = "use-self")]
pub use convert::*;
#[cfg(feature = "use-self")]
pub use value::*;
#[cfg(feature = "use-self")]
pub use value_ptr::*;
#[cfg(feature = "use-self")]
pub use interned::*;