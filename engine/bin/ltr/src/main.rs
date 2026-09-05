use clap::{Parser, ValueEnum};
use internal::{EngineSettings, PresentMode};

/// clap-facing mirror of `internal::PresentMode`.
///
/// Not strictly needed for the type itself (`internal::PresentMode` could
/// derive `ValueEnum` directly), but `ltr` must never reference anything
/// defined in `phx`: `phx` is release-built with `codegen-units = 1` plus
/// LTO, so its rlib is a single object containing the whole engine
/// (`Engine_Entry` included), and naming even a bare enum from it would make
/// phx a "used" dependency, pulling that entire object statically into
/// `ltr`. Keeping `internal` (a tiny, engine-free crate) as the only type
/// shared between the two sidesteps that. This CLI-only enum just keeps
/// clap's `ValueEnum` derive (and the `internal` crate) decoupled.
#[derive(Clone, Copy, ValueEnum)]
enum PresentModeArg {
    Vsync,
    NoVsync,
}

impl From<PresentModeArg> for PresentMode {
    fn from(value: PresentModeArg) -> Self {
        match value {
            PresentModeArg::Vsync => PresentMode::Vsync,
            PresentModeArg::NoVsync => PresentMode::NoVsync,
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub static NvOptimusEnablement: std::os::raw::c_ulong = 0x00000001;

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub static AmdPowerXpressRequestHighPerformance: std::os::raw::c_int = 1;

const BUILD_TIME: &str = build_time::build_time_utc!("%Y-%m-%d / %H:%M:%S UTC");
const GIT_VERSION: &str = git_version::git_version!(args = ["--tags", "--always", "--dirty=M"]);

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Application starting Lua script
    #[arg(short, long, default_value = "./script/Main.lua")]
    entry_point: String,
    /// Specify if console log should be shown
    #[arg(short, long, default_value_t = true)]
    console_log: bool,
    /// Specify if console should disable colors
    #[arg(short, long, default_value_t = false)]
    no_color: bool,
    /// Log will be written into the log file if log_dir is specified
    #[arg(short, long)]
    log_dir: Option<String>,
    /// Serve the live render-stats dashboard on this port (requires the
    /// `stats-server` cargo feature; open http://127.0.0.1:<port> in a browser)
    #[arg(long, default_value_t = 8777)]
    #[cfg(feature = "stats-server")]
    stats_server: u16,
    /// Force the swap interval, overriding whatever the Lua state requests
    /// via `Window:setPresentMode`. Useful for A/B'ing benchmark runs
    /// without editing scripts.
    #[arg(long, value_enum)]
    present_mode: Option<PresentModeArg>,
    /// Optional application name
    app_name: Option<String>,
}

#[cfg_attr(not(windows), link(name = "phx", kind = "dylib"))]
#[cfg_attr(windows, link(name = "phx.dll", kind = "dylib"))]
#[allow(unsafe_code, improper_ctypes)]
unsafe extern "C-unwind" {
    fn Engine_Entry(settings: &'static EngineSettings);
}

pub fn main() {
    println!(
        "App: {}, ver: {}, git: {}, build time: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        GIT_VERSION,
        BUILD_TIME
    );

    let cli = Cli::parse();

    // Leaked on purpose: the engine borrows this for the whole run, and nothing
    // allocated here may be freed on the library side of the boundary.
    let settings: &'static EngineSettings = Box::leak(Box::new(EngineSettings {
        entry_point: cli.entry_point.into(),
        app_name: cli.app_name.unwrap_or_default(),
        log_dir: cli.log_dir.unwrap_or_default(),
        console_log: cli.console_log,
        no_color: cli.no_color,
        #[cfg(feature = "stats-server")]
        stats_port: cli.stats_server,
        present_mode: cli.present_mode.map(PresentMode::from),
    }));

    #[allow(unsafe_code)]
    unsafe {
        Engine_Entry(settings)
    };
}
