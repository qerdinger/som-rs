use std::fmt::Display;

/// An interned string.
///
/// This is fast to move, clone and compare.
///
/// NB: this was originally a u32, which I think is more sensible. It's now a u16 to have Send
/// bytecodes store an Interned reference directly without increasing the enum size. This is fine
/// on our benchmarks, but sounds like a bad choice for very large systems?
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Interned(pub u16);

// hack. we pretty much never want this, and instead to print the associated string.
impl Display for Interned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}
