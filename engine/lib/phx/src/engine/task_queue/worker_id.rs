pub type WorkerIndex = u16;

/// Types of workers.
/// Can be extended on the Lua side.
#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerId {
    /// Example worker that replicates input value into the output
    Echo,

    /// Specifies number of engine worker types
    EngineWorkersCount, // !!! SHOULD BE THE LAST ENUM VARIANT !!!
}

impl WorkerId {
    pub fn from_worker_id(id: WorkerIndex) -> Option<Self> {
        if id == Self::Echo as WorkerIndex {
            Some(Self::Echo)
        } else {
            None
        }
    }
}
