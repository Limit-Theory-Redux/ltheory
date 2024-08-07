use super::EventId;

/// List of the event types used in the engine.
/// In Lua scripts should be used as an event id.
/// To extend it in Lua scripts call `EventType.AddEventTypes({"MyEventType1", "MyEventType2"})` function.
// NOTE: Use same type in 'repr' as in EventId.
#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    // Before physics update event
    PreSim,
    // Physics update event
    Sim,
    // After physics update event
    PostSim,
    // Before frame render event
    PreRender,
    // Frame render event
    Render,
    // After frame render event
    PostRender,
    // Before input handling event
    PreInput,
    // Input handling event
    Input,
    // After input handling event
    PostInput,

    /// Specifies number of engine event types
    EngineEventTypesCount, // !!! SHOULD BE THE LAST ENUM VARIANT !!!
}

impl EventType {
    pub fn index(&self) -> EventId {
        *self as EventId
    }
}
