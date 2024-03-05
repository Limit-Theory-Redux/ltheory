use std::cell::RefCell;
use std::rc::Rc;

use kira::manager::{AudioManager, AudioManagerSettings, Capacities};
use kira::sound::PlaybackState;
use kira::spatial::emitter::EmitterSettings;
use kira::spatial::listener::{ListenerHandle, ListenerSettings};
use kira::spatial::scene::{SpatialSceneHandle, SpatialSceneSettings};
use kira::tween::Tween;

use crate::math::*;

use super::{process_command_error, Sound, SoundInstance};

const DEFAULT_COMMAND_CAPACITY: usize = 1024;

pub struct Audio {
    audio_manager: AudioManager,
    spatial_scene: SpatialSceneHandle,
    listener: ListenerHandle,
}

#[luajit_ffi_gen::luajit_ffi]
impl Audio {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        let settings = AudioManagerSettings {
            capacities: Capacities {
                command_capacity: DEFAULT_COMMAND_CAPACITY,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut audio_manager = AudioManager::new(settings).expect("Cannot create audio manager");

        let mut spatial_scene = audio_manager
            .add_spatial_scene(SpatialSceneSettings::default())
            .expect("Cannot add spatial scene");

        let listener = spatial_scene
            .add_listener(
                [0.0, 0.0, 0.0],      // TODO: fix this
                [0.0, 0.0, 0.0, 0.0], // TODO: fix this
                ListenerSettings::default(),
            )
            .expect("Cannot add listener");

        Self {
            audio_manager,
            spatial_scene,
            listener,
        }
    }

    pub fn play(&mut self, sound: &mut Sound, init_volume: f64) -> Box<SoundInstance> {
        let emitter = self
            .spatial_scene
            .add_emitter([0.0, 0.0, 0.0], EmitterSettings::default())
            .expect("Cannot add an emitter");

        sound.set_emitter(emitter);

        let mut sound_data_clone = sound.sound_data().clone();
        sound_data_clone.settings.volume = init_volume.into();

        let sound_handle = self
            .audio_manager
            .play(sound_data_clone)
            .expect("Cannot play sound");

        let sound_handle = Rc::new(RefCell::new(sound_handle));
        let sound_instance = SoundInstance::new(sound_handle, init_volume);

        sound_instance
    }

    pub fn set_listener_pos(&mut self, pos: &Vec3, rot: &Quat) {
        process_command_error(
            self.listener.set_position(*pos, Tween::default()),
            "Cannot set listener position",
        );
        process_command_error(
            self.listener
                .set_orientation([rot.x, rot.y, rot.z, rot.w], Tween::default()),
            "Cannot set listener position",
        );
    }

    pub fn get_loaded_count(&self) -> u64 {
        self.audio_manager.num_sounds() as u64
    }

    pub fn get_total_count(&self) -> u64 {
        self.audio_manager.sound_capacity() as u64
    }
}
