use indexmap::IndexMap;

use super::Payload;

/// Collection of named payloads.
/// Elements are ordered by insertion time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PayloadTable {
    table: IndexMap<String, Payload>,
}

/// Collection of named payloads.
/// Elements ordered by insertion order.
#[luajit_ffi_gen::luajit_ffi]
impl PayloadTable {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self {
            table: Default::default(),
        }
    }

    /// Returns number of elements in the table.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Checks if table is empty.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Checks if table contains an element with name 'name'.
    pub fn contains(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    /// Returns the name of the element at position 'index'.
    /// Returns `None`/`nil` if index is bigger than the number of elements in the table.
    pub fn get_name(&self, index: usize) -> Option<&str> {
        self.table.get_index(index).map(|(name, _)| name.as_str())
    }

    /// Returns payload at position 'index'.
    /// Returns `None`/`nil` if index is bigger than the number of elements in the table.
    pub fn get_payload(&self, index: usize) -> Option<&Payload> {
        self.table.get_index(index).map(|(_, payload)| payload)
    }

    /// Returns payload by name 'name'.
    /// Returns `None`/`nil` if index is bigger than the number of elements in the table.
    pub fn get_payload_by_name(&self, name: &str) -> Option<&Payload> {
        self.table.get(name)
    }

    /// Add new element to the table.
    pub fn add(&mut self, name: &str, value: &Payload) {
        // TODO: send table by value - fix FFI
        self.table.insert(name.into(), value.clone());
    }
}
