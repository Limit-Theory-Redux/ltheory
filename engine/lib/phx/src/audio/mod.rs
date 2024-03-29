mod audio;
mod sound;
mod sound_instance;

pub use audio::*;
pub use sound::*;
pub use sound_instance::*;

use kira::CommandError;
use tracing::debug;

pub(crate) fn process_command_error(res: Result<(), CommandError>, msg: &str) {
    match res {
        Ok(_) => {}
        Err(CommandError::CommandQueueFull) => {
            debug!("{msg}. Command queue is full");
        }
        Err(err) => panic!("{msg}. Error: {err}"),
    }
}
