pub type WorkerId = u16;

#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Worker {
    /// Example worker that replicates input value into the output
    Echo,

    /// Specifies number of engine worker types
    EngineWorkersCount, // !!! SHOULD BE THE LAST ENUM VARIANT !!!
}

impl Worker {
    pub fn from_worker_id(id: WorkerId) -> Option<Self> {
        if id == Self::Echo as WorkerId {
            Some(Self::Echo)
        } else {
            None
        }
    }
}
