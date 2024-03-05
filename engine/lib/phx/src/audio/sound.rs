use std::cell::RefCell;
use std::rc::Rc;

use internal::ConvertIntoString;

use crate::math::*;

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::{EndPosition, PlaybackPosition, Region};
use kira::spatial::emitter::EmitterHandle;
use kira::tween::Tween;

use super::{process_command_error, SoundInstance};

pub struct Sound {
    path: String,
    sound_data: StaticSoundData,
    emitter: Option<EmitterHandle>,
    pub instances: Vec<Rc<RefCell<SoundInstance>>>,
}

impl Sound {
    pub fn sound_data(&self) -> &StaticSoundData {
        &self.sound_data
    }

    pub fn set_emitter(&mut self, emitter: EmitterHandle) {
        // FIXME: for some reason adding emitter kills the sound
        // self.sound_data.settings.output_destination = emitter.id().into();
        self.emitter = Some(emitter);
    }

    pub fn add_instance(&mut self, instance: Rc<RefCell<SoundInstance>>) {
        self.instances.push(instance);
    }
}

#[luajit_ffi_gen::luajit_ffi]
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

        let sound_data = StaticSoundData::from_file(path, settings)
            .expect(&format!("Cannot load sound: {path}"));

        Self {
            path: path.into(),
            sound_data,
            emitter: None,
            instances: vec![],
        }
    }

    pub fn get_duration(&self) -> f32 {
        self.sound_data.duration().as_secs_f32()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn set_emitter_pos(&mut self, position: &Vec3) {
        if let Some(emitter) = &mut self.emitter {
            process_command_error(
                emitter.set_position(*position, Tween::default()),
                "Cannot set sound emitter position",
            );
        }
    }
}
