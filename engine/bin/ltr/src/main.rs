use std::ffi::CString;

use clap::Parser;

#[cfg(target_os = "windows")]
#[no_mangle]
pub static NvOptimusEnablement: std::os::raw::c_ulong = 0x00000001;

#[cfg(target_os = "windows")]
#[no_mangle]
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
    /// Optional application name
    app_name: Option<String>,
}

#[cfg_attr(not(windows), link(name = "phx", kind = "dylib"))]
#[cfg_attr(windows, link(name = "phx.dll", kind = "dylib"))]
extern "C" {
    fn Engine_Entry(
        entry_point: *const libc::c_char,
        app_name: *const libc::c_char,
        console_log: bool,
        log_dir: *const libc::c_char,
    );
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

    let entry_point = CString::new(cli.entry_point)
        .expect("Failed to convert entry_point argument into CString.")
        .into_raw();
    let app_name_str = cli.app_name.clone().unwrap_or("".into());
    let app_name = CString::new(app_name_str)
        .expect("Failed to convert app_name argument into CString.")
        .into_raw();
    let log_dir_str = cli.log_dir.clone().unwrap_or("".into());
    let log_dir = CString::new(log_dir_str)
        .expect("Failed to convert log_dir argument into CString.")
        .into_raw();

    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }

    #[allow(unsafe_code)] // TODO: remove
    unsafe {
        Engine_Entry(
            entry_point as *const libc::c_char,
            app_name as *const libc::c_char,
            cli.console_log,
            log_dir as *const libc::c_char,
        );
    }
}
