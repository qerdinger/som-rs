/// To convert values to types, and vice versa.
pub mod convert;
/// Shared value representation logic (NaN boxing really)
pub mod value;
/// Class for storing a value itself as a typed pointer.
pub mod value_ptr;

/// The representation for interned strings. Made to work with som-core/interner.
/// Values need to be able to use it, and som-core depends on som-value, so we'd have a circular
/// dependency if this wasn't here.
/// It is in the scope of the crate ("how we represent values in SOM") but is annoying if this
/// crate is ever to be standalone. It's a bit annoying for it not to be there, but it can be
/// kicked out.
pub mod interned;
