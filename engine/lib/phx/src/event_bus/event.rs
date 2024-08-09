use super::EventId;

/// List of the events used in the engine.
/// In Lua scripts it should be used as an event id.
/// To extend it in Lua, call `Event.AddEvents({"MyEvent1", "MyEvent2"})` function.
// NOTE: Use same type in 'repr' as in EventId.
#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Before physics update event
    PreSim,
    /// Physics update event
    Sim,
    /// After physics update event
    PostSim,
    /// Before frame render event
    PreRender,
    /// Frame render event
    Render,
    /// After frame render event
    PostRender,
    /// Before input handling event
    PreInput,
    /// Input handling event
    Input,
    /// After input handling event
    PostInput,

    /// Specifies number of engine event types
    EngineEventsCount, // !!! SHOULD BE THE LAST ENUM VARIANT !!!
}

impl Event {
    pub fn index(&self) -> EventId {
        *self as EventId
    }
}
