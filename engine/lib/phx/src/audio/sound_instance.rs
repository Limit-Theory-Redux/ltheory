use std::{cell::RefCell, rc::Rc, time::Duration};

use kira::{
    sound::{static_sound::StaticSoundHandle, PlaybackState},
    tween::{Easing, Tween},
    StartTime,
};

use super::process_command_error;

#[derive(Clone)]
pub struct SoundInstance {
    pub handle: Option<Rc<RefCell<StaticSoundHandle>>>,
    pub volume: f64, // keep track of volume because we can`t get it from the handle
}

impl SoundInstance {
    pub fn new(handle: Rc<RefCell<StaticSoundHandle>>) -> Box<Self> {
        Box::new(Self {
            handle: Some(handle),
            volume: 0.0,
        })
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl SoundInstance {
    pub fn is_playing(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.borrow().state() == PlaybackState::Playing
        } else {
            false
        }
    }

    pub fn is_paused(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.borrow().state() == PlaybackState::Paused
        } else {
            false
        }
    }

    pub fn is_stopped(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.borrow().state() == PlaybackState::Stopped
        } else {
            false
        }
    }

    pub fn get_volume(&self) -> f64 {
        self.volume.clone()
    }

    pub fn set_volume(&mut self, volume: f64, fade_millis: u64) {
        if let Some(handle) = &self.handle {
            process_command_error(
                handle.borrow_mut().set_volume(
                    volume,
                    Tween {
                        duration: Duration::from_millis(fade_millis),
                        ..Default::default()
                    },
                ),
                "Cannot set volume on sound",
            );

            self.volume = volume;
        } else {
            self.volume = volume;
        }
    }

    pub fn pause(&mut self, fade_millis: u64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(
                handle.borrow_mut().pause(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(fade_millis),
                    easing: Easing::Linear,
                }),
                "Cannot pause sound",
            );
        }
    }

    pub fn resume(&mut self, fade_millis: u64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(
                handle.borrow_mut().resume(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(fade_millis),
                    easing: Easing::Linear,
                }),
                "Cannot resume sound",
            );
        }
    }

    pub fn stop(&mut self, fade_millis: u64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(
                handle.borrow_mut().stop(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(fade_millis),
                    easing: Easing::Linear,
                }),
                "Cannot stop sound",
            );
        }
    }

    pub fn set_play_pos(&mut self, position: f64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(
                handle.borrow_mut().seek_to(position),
                "Cannot set sound position",
            );
        }
    }

    pub fn move_play_pos(&mut self, offset: f64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(
                handle.borrow_mut().seek_by(offset),
                "Cannot set sound position",
            );
        }
    }
}
