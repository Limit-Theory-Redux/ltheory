/// Statistics from batch rendering
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    /// Total entities submitted this frame
    pub entities_submitted: u32,
    /// Entities visible after culling
    pub entities_visible: u32,
    /// Entities culled
    pub entities_culled: u32,
    /// Total entities culled
    pub total_entities: u32,
    /// Commands generated
    pub commands_generated: u32,
    /// Batches processed
    pub batches_processed: u32,
}

#[luajit_ffi_gen::luajit_ffi]
impl BatchStats {
    pub fn get_entities_submitted(&self) -> u32 {
        self.entities_submitted
    }
    pub fn get_entities_visible(&self) -> u32 {
        self.entities_visible
    }
    pub fn get_entities_culled(&self) -> u32 {
        self.entities_culled
    }
    pub fn get_total_entities(&self) -> u32 {
        self.total_entities
    }
    pub fn get_commands_generated(&self) -> u32 {
        self.commands_generated
    }
    pub fn get_batches_processed(&self) -> u32 {
        self.batches_processed
    }
}
