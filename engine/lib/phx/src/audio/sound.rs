use std::time::Duration;

use crate::internal::*;
use crate::math::*;

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

pub struct PlayData {
    handle: StaticSoundHandle,
    emitter: EmitterHandle,
}

impl PlayData {
    pub fn new(handle: StaticSoundHandle, emitter: EmitterHandle) -> Self {
        Self { handle, emitter }
    }
}

pub struct Sound {
    path: String,
    sound_data: StaticSoundData,
    play_data: Option<PlayData>,
}

impl Sound {
    pub fn sound_data(&self) -> &StaticSoundData {
        &self.sound_data
    }

    pub fn set_play_data(&mut self, play_data: PlayData) {
        // TODO: check if play data is already set?
        self.play_data = Some(play_data);
    }
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Sound {
    #[bind(name = "Load")]
    pub fn new(path: &str, is_looping: bool) -> Self {
        let mut settings = StaticSoundSettings::new();

        if is_looping {
            settings = settings.loop_region(Region {
                start: PlaybackPosition::Seconds(0.0),
                end: EndPosition::EndOfAudio,
            });
        }

        let sound_data = StaticSoundData::from_file(path, settings).expect("Cannot load sound");

        Self {
            path: path.into(),
            sound_data,
            play_data: None,
        }
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn is_playing(&self) -> bool {
        // TODO: properly process pause and stop states
        self.play_data.is_some()
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.sound_data.settings.volume = volume.into();
    }

    pub fn pause(&mut self, duration: u64) {
        if let Some(play_data) = &mut self.play_data {
            play_data
                .handle
                .pause(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(duration),
                    easing: Easing::Linear,
                })
                .expect("Cannot pause sound");
        }
    }

    pub fn resume(&mut self, duration: u64) {
        if let Some(play_data) = &mut self.play_data {
            play_data
                .handle
                .resume(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(duration),
                    easing: Easing::Linear,
                })
                .expect("Cannot resume sound");
        }
    }

    pub fn stop(&mut self, duration: u64) {
        // TODO: should we set play_data = None here?
        if let Some(play_data) = &mut self.play_data {
            play_data
                .handle
                .stop(Tween {
                    start_time: StartTime::Immediate,
                    duration: Duration::from_millis(duration),
                    easing: Easing::Linear,
                })
                .expect("Cannot stop sound");
        }
    }

    pub fn set_play_pos(&mut self, position: f64) {
        if let Some(play_data) = &mut self.play_data {
            play_data
                .handle
                .seek_to(position)
                .expect("Cannot set sound position");
        }
    }

    pub fn move_play_pos(&mut self, offset: f64) {
        if let Some(play_data) = &mut self.play_data {
            play_data
                .handle
                .seek_by(offset)
                .expect("Cannot set sound position");
        }
    }

    pub fn set_emitter_pos(&mut self, pos: &Vec3) {
        if let Some(play_data) = &mut self.play_data {
            play_data
                .emitter
                .set_position(*pos, Tween::default())
                .expect("Cannot set sound emitter position");
        }
    }
}
