use indexmap::IndexMap;

use super::Payload;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PayloadTable {
    table: IndexMap<String, Payload>,
}

#[luajit_ffi_gen::luajit_ffi]
impl PayloadTable {
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

    pub fn get_payload(&self, index: usize) -> Option<&Payload> {
        self.table.get_index(index).map(|(_, payload)| payload)
    }

    pub fn add(&mut self, name: &str, value: Payload) {
        self.table.insert(name.into(), value);
    }
}
