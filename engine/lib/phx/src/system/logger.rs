use tracing::{debug, error, info, trace, warn};

pub struct Logger;

#[luajit_ffi_gen::luajit_ffi]
impl Logger {
    pub fn trace(msg: &str) {
        trace!("{msg}");
    }

    pub fn debug(msg: &str) {
        debug!("{msg}");
    }

    pub fn info(msg: &str) {
        info!("{msg}");
    }

    pub fn warn(msg: &str) {
        warn!("{msg}");
    }

    pub fn error(msg: &str) {
        error!("{msg}");
    }
}
