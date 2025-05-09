use std::time::Duration;

use kira::StartTime;
use kira::sound::PlaybackState;
use kira::sound::static_sound::StaticSoundHandle;
use kira::spatial::emitter::EmitterHandle;
use kira::tween::{Easing, Tween};

use crate::math::Position;

struct EmitterInfo {
    emitter: EmitterHandle,
    position: Position,
    audio_origin: Position,
}

pub struct SoundInstance {
    handle: Option<StaticSoundHandle>,
    volume: f64, // keep track of volume because we can't get it from the handle
    emitter_info: Option<EmitterInfo>,
}

impl SoundInstance {
    pub fn new(
        handle: StaticSoundHandle,
        init_volume: f64,
        emitter: Option<(EmitterHandle, Position, Position)>,
    ) -> Self {
        Self {
            handle: Some(handle),
            volume: init_volume,
            emitter_info: emitter.map(|(emitter, position, audio_origin)| EmitterInfo {
                emitter,
                position,
                audio_origin,
            }),
        }
    }

    // This recomputes the emitters position relative to the current listeners position. This should be called anytime that the listener's origin is updated.
    fn update_kira_emitter_position(&mut self) {
        if let Some(emitter_info) = &mut self.emitter_info {
            emitter_info.emitter.set_position(
                emitter_info.position.relative_to(emitter_info.audio_origin),
                Tween::default(),
            );
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
            handle.set_volume(
                volume,
                Tween {
                    duration: Duration::from_millis(fade_millis),
                    ..Default::default()
                },
            );
        }
        self.volume = volume;
    }

    pub fn pause(&mut self, fade_millis: u64) {
        if let Some(handle) = &mut self.handle {
            handle.pause(Tween {
                start_time: StartTime::Immediate,
                duration: Duration::from_millis(fade_millis),
                easing: Easing::Linear,
            });
        }
    }

    pub fn resume(&mut self, fade_millis: u64) {
        if let Some(handle) = &mut self.handle {
            handle.resume(Tween {
                start_time: StartTime::Immediate,
                duration: Duration::from_millis(fade_millis),
                easing: Easing::Linear,
            });
        }
    }

    pub fn stop(&mut self, fade_millis: u64) {
        if let Some(handle) = &mut self.handle {
            handle.stop(Tween {
                start_time: StartTime::Immediate,
                duration: Duration::from_millis(fade_millis),
                easing: Easing::Linear,
            });
        }
    }

    pub fn free_emitter(&mut self) {
        self.emitter_info = None;
    }

    pub fn set_play_pos(&mut self, position: f64) {
        if let Some(handle) = &mut self.handle {
            handle.seek_to(position);
        }
    }

    pub fn move_play_pos(&mut self, offset: f64) {
        if let Some(handle) = &mut self.handle {
            handle.seek_by(offset);
        }
    }

    pub fn set_emitter_pos(&mut self, position: &Position) {
        if let Some(emitter_info) = &mut self.emitter_info {
            emitter_info.position = *position;
            self.update_kira_emitter_position()
        }
    }

    pub fn set_emitter_origin_pos(&mut self, origin: &Position) {
        if let Some(emitter_info) = &mut self.emitter_info {
            emitter_info.audio_origin = *origin;
            self.update_kira_emitter_position();
        }
    }

    pub fn emitter_pos(&self) -> Position {
        self.emitter_info
            .as_ref()
            .map(|emitter_info| emitter_info.position)
            .unwrap_or_default()
    }

    pub fn emitter_distance(&self, listener_pos: &Position) -> f32 {
        self.emitter_info
            .as_ref()
            .map(|emitter_info| listener_pos.distance(emitter_info.position) as f32)
            .unwrap_or(f32::MAX)
    }
}
