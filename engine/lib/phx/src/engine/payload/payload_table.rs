use indexmap::IndexMap;

use super::EventPayload;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventPayloadTable {
    table: IndexMap<String, EventPayload>,
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

    pub fn get_name(&self, index: usize) -> Option<&str> {
        self.table.get_index(index).map(|(name, _)| name.as_str())
    }

    pub fn get_payload(&self, index: usize) -> Option<&EventPayload> {
        self.table.get_index(index).map(|(_, payload)| payload)
    }

    pub fn add(&mut self, name: &str, value: EventPayload) {
        self.table.insert(name.into(), value);
    }
}
