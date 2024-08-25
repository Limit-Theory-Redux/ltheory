pub enum Worker {
    /// Basic worker that replicates an input value into the output
    Echo,

    /// Specifies number of engine worker types
    EngineWorkersCount, // !!! SHOULD BE THE LAST ENUM VARIANT !!!
}
