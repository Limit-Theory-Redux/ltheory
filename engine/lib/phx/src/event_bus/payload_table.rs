use indexmap::IndexMap;
use internal::ConvertIntoString;

use super::EventPayload;

#[derive(Debug, Clone, Default)]
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

    pub fn add(&mut self, name: &str, value: EventPayload) {
        self.table.insert(name.into(), value);
    }
}
