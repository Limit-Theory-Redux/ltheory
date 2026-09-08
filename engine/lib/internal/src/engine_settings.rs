use std::path::PathBuf;

/// Startup parameters handed from the `ltr` launcher to the engine.
///
/// This lives in `internal` rather than `phx` on purpose. `phx` is release-built
/// with `codegen-units = 1` plus LTO (see `.cargo/config.toml`), so its rlib
/// compiles down to a single object that also contains `Engine_Entry` and the
/// rest of the engine. The moment `ltr` names *any* phx-defined item, phx
/// becomes a "used" dependency and LTO pulls that whole object statically into
/// `ltr` - duplicating the entire engine into the launcher binary even though
/// `Engine_Entry` is only meant to be reached through `libphx.so`. Defining the
/// shared type here, in a crate with no engine code, keeps `ltr` from ever
/// referencing `phx` directly.
///
/// This is the *only* launcher -> engine channel; nothing travels via
/// environment variables. The launcher leaks one of these and passes a
/// reference to `Engine_Entry`, so the engine only ever borrows it - no
/// allocation made in the executable is ever freed inside the library.
pub struct EngineSettings {
    pub entry_point: PathBuf,
    /// Empty means "unset".
    pub app_name: String,
    /// Empty means "no file logging".
    pub log_dir: String,
    pub console_log: bool,
    pub no_color: bool,
    /// `None` disables the live stats dashboard.
    #[cfg(feature = "stats-server")]
    pub stats_port: u16,
    /// `None` leaves present mode under script control.
    pub present_mode: Option<PresentMode>,
}

/// Mirrors `phx::window::PresentMode`. Kept as a separate type (rather than
/// reusing phx's enum) for the same reason [`EngineSettings`] lives here: it
/// needs to be nameable from `ltr` without making phx a used dependency there.
/// `phx` converts it to its own `PresentMode` at the one call site that reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PresentMode {
    Vsync,
    NoVsync,
}
