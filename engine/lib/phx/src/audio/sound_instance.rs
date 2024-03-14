use std::time::Duration;

use glam::Vec3;
use kira::sound::{static_sound::StaticSoundHandle, PlaybackState};
use kira::spatial::emitter::EmitterHandle;
use kira::tween::{Easing, Tween};
use kira::StartTime;

use super::process_command_error;

struct EmitterInfo {
    emitter: EmitterHandle,
    position: Vec3,
}

pub struct SoundInstance {
    handle: Option<StaticSoundHandle>,
    volume: f64, // keep track of volume because we can`t get it from the handle
    emitter_info: Option<EmitterInfo>,
}

impl SoundInstance {
    pub fn new(
        handle: StaticSoundHandle,
        init_volume: f64,
        emitter: Option<(EmitterHandle, Vec3)>,
    ) -> Self {
        Self {
            handle: Some(handle),
            volume: init_volume,
            emitter_info: emitter.map(|(emitter, position)| EmitterInfo { emitter, position }),
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl SoundInstance {
    pub fn is_playing(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.state() == PlaybackState::Playing
        } else {
            false
        }
    }

    pub fn is_paused(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.state() == PlaybackState::Paused
        } else {
            false
        }
    }

    pub fn is_stopped(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.state() == PlaybackState::Stopped
        } else {
            false
        }
    }

    pub fn get_volume(&self) -> f64 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f64, fade_millis: u64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(
                handle.set_volume(
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
                handle.pause(Tween {
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
                handle.resume(Tween {
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
                handle.stop(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(fade_millis),
                    easing: Easing::Linear,
                }),
                "Cannot stop sound",
            );
        }
    }

    pub fn free_emitter(&mut self) {
        self.emitter_info = None;
    }

    pub fn set_play_pos(&mut self, position: f64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(handle.seek_to(position), "Cannot set sound position");
        }
    }

    pub fn move_play_pos(&mut self, offset: f64) {
        if let Some(handle) = &mut self.handle {
            process_command_error(handle.seek_by(offset), "Cannot set sound position");
        }
    }

    pub fn set_emitter_pos(&mut self, position: &Vec3) {
        if let Some(emitter_info) = &mut self.emitter_info {
            process_command_error(
                emitter_info
                    .emitter
                    .set_position(*position, Tween::default()),
                "Cannot set sound emitter position",
            );
            emitter_info.position = *position;
        }
    }

    pub fn emitter_pos(&self) -> Vec3 {
        self.emitter_info
            .as_ref()
            .map(|emitter_info| emitter_info.position)
            .unwrap_or(Vec3::MAX)
    }

    pub fn emitter_distance(&self, listener_pos: &Vec3) -> f32 {
        self.emitter_info
            .as_ref()
            .map(|emitter_info| listener_pos.distance(emitter_info.position))
            .unwrap_or(f32::MAX)
    }
}
