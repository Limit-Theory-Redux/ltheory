use std::time::Duration;

use crate::internal::*;
use crate::math::*;

use kira::sound::PlaybackState;
use kira::{
    modulator::lfo::LfoHandle,
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
        EndPosition, PlaybackPosition, Region,
    },
    spatial::emitter::EmitterHandle,
    tween::{Easing, Tween},
    StartTime,
};

pub struct Sound {
    path: String,
    sound_data: StaticSoundData,
    emitter: Option<EmitterHandle>,
    sound_handle: Option<StaticSoundHandle>,
}

impl Sound {
    pub fn has_sound_handle(&self) -> bool {
        self.sound_handle.is_some()
    }

    pub fn sound_data(&self) -> &StaticSoundData {
        &self.sound_data
    }

    pub fn set_emitter(&mut self, emitter: EmitterHandle) {
        // FIXME: for some reason adding emitter kills the sound
        // self.sound_data.settings.output_destination = emitter.id().into();
        self.emitter = Some(emitter);
    }

    pub fn set_sound_handle(&mut self, sound_handle: StaticSoundHandle) {
        self.sound_handle = Some(sound_handle);
    }
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Sound {
    #[bind(name = "Load")]
    pub fn new(path: &str, is_looping: bool) -> Self {
        let mut settings = StaticSoundSettings::new();

        if is_looping {
            // Loop over whole audio
            settings = settings.loop_region(Region {
                start: PlaybackPosition::Seconds(0.0),
                end: EndPosition::EndOfAudio,
            });
        }

        let sound_data = StaticSoundData::from_file(path, settings).expect("Cannot load sound");

        Self {
            path: path.into(),
            sound_data,
            emitter: None,
            sound_handle: None,
        }
    }

    pub fn get_duration(&self) -> f32 {
        self.sound_data.duration().as_secs_f32()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sound_handle) = &self.sound_handle {
            sound_handle.state() == PlaybackState::Playing
        } else {
            false
        }
    }

    pub fn is_paused(&self) -> bool {
        if let Some(sound_handle) = &self.sound_handle {
            sound_handle.state() == PlaybackState::Paused
        } else {
            false
        }
    }

    pub fn is_stopped(&self) -> bool {
        if let Some(sound_handle) = &self.sound_handle {
            sound_handle.state() == PlaybackState::Stopped
        } else {
            false
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.sound_data.settings.volume = volume.into();
    }

    pub fn pause(&mut self, fade_millis: u64) {
        if let Some(sound_handle) = &mut self.sound_handle {
            sound_handle
                .pause(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(fade_millis),
                    easing: Easing::Linear,
                })
                .expect("Cannot pause sound");
        }
    }

    pub fn resume(&mut self, fade_millis: u64) {
        if let Some(sound_handle) = &mut self.sound_handle {
            sound_handle
                .resume(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(fade_millis),
                    easing: Easing::Linear,
                })
                .expect("Cannot resume sound");
        }
    }

    pub fn stop(&mut self, fade_millis: u64) {
        if let Some(sound_handle) = &mut self.sound_handle {
            sound_handle
                .stop(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(fade_millis),
                    easing: Easing::Linear,
                })
                .expect("Cannot stop sound");
        }
    }

    pub fn set_play_pos(&mut self, position: f64) {
        if let Some(sound_handle) = &mut self.sound_handle {
            sound_handle
                .seek_to(position)
                .expect("Cannot set sound position");
        }
    }

    pub fn move_play_pos(&mut self, offset: f64) {
        if let Some(sound_handle) = &mut self.sound_handle {
            sound_handle
                .seek_by(offset)
                .expect("Cannot set sound position");
        }
    }

    pub fn set_emitter_pos(&mut self, position: &Vec3) {
        if let Some(emitter) = &mut self.emitter {
            emitter
                .set_position(*position, Tween::default())
                .expect("Cannot set sound emitter position");
        }
    }
}
