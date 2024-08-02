use std::collections::HashMap;

use super::EventPayload;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventPayloadTable {
    table: HashMap<String, EventPayload>,
}

#[luajit_ffi_gen::luajit_ffi]
impl EventPayloadTable {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self {
            table: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&EventPayload> {
        self.table.get(name)
    }

    pub fn add(&mut self, name: &str, value: EventPayload) {
        self.table.insert(name.into(), value);
    }
}
