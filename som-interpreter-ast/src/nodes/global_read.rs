use crate::evaluate::Evaluate;
use crate::invokable::Return;
use crate::universe::{GlobalValueStack, Universe};
use crate::value::Value;
use crate::vm_objects::frame::FrameAccess;
use som_value::interned::Interned;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNode {
    pub global: Interned,
    pub cached_entry: Option<Value>,
}

impl From<Interned> for GlobalNode {
    fn from(value: Interned) -> Self {
        Self {
            global: value,
            cached_entry: None,
        }
    }
}

impl Evaluate for GlobalNode {
    fn evaluate(&mut self, universe: &mut Universe, value_stack: &mut GlobalValueStack) -> Return {
        if let Some(cached_entry) = self.cached_entry {
            return Return::Local(cached_entry);
        }

        universe
            .lookup_global(self.global)
            .map(|global| {
                self.cached_entry.replace(global);
                Return::Local(global)
            })
            .or_else(|| {
                let frame = &universe.current_frame;
                let self_value = frame.get_self();
                universe.unknown_global(value_stack, self_value, self.global)
            })
            .unwrap_or_else(|| panic!("global not found and unknown_global call failed somehow?"))
    }
}
