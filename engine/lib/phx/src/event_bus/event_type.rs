/// List of the event types used in the engine.
/// In Lua scripts should be used as an event id.
/// To extend it in Lua scripts call `EventType.Register({"MyEventType1", "MyEventType2"})` function.
// NOTE: Use for event id same type as in 'repr' below.
#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    ResourceLoadingResult,
    EngineEventTypesCount,
}

impl EventType {
    pub fn index(&self) -> u16 {
        *self as u16
    }
}
