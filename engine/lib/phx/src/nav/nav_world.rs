/// Lua handle on the navigation state of one simulated space.
///
/// The only wrapper over the `nav` crate; the algorithms live there.
pub struct NavWorld {
    // Read by the methods that arrive with the first navigation step.
    #[expect(dead_code, reason = "the wrapper has no methods yet")]
    world: ::nav::World,
}

#[luajit_ffi_gen::luajit_ffi]
impl NavWorld {
    /// Creates an empty world.
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self {
            world: ::nav::World::new(),
        }
    }
}
