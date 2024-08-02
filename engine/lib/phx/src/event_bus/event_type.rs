// NOTE: Should be the same as in EventType 'repr' parameter and event id in the buf interface
pub type EventId = u16;

/// List of the event types used in the engine.
/// In Lua scripts should be used as an event id.
/// To extend it in Lua scripts call `EventType.Register({"MyEventType1", "MyEventType2"})` function.
// NOTE: Use for event id same type as in 'repr' parameter below.
#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    ResourceLoadingResult,
    /// Specifies number of engine event types
    EngineEventTypesCount, // !!! SHOULD BE THE LAST ENUM VARIANT !!!
}

impl EventType {
    pub fn index(&self) -> EventId {
        *self as EventId
    }
}
