mod audio;
mod sound;

pub use audio::*;
pub use sound::*;

use kira::CommandError;
use tracing::debug;

pub(crate) fn process_command_error(res: Result<(), CommandError>, msg: &str) {
    match res {
        Ok(_) => {}
        Err(CommandError::CommandQueueFull) => {
            debug!("Command queue is full");
        }
        Err(err) => panic!("{msg}. Error: {err}"),
    }
}
