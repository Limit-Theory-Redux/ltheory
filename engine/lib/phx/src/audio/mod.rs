mod audio;
mod sound;
mod sound_instance;
mod sound_group;
mod sound_group_manager;


pub use audio::*;
pub use sound::*;
pub use sound_instance::*;
pub use sound_group::*;
pub use sound_group_manager::*;

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
