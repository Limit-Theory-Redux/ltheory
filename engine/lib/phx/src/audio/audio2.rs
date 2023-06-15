use kira::manager::{AudioManager, AudioManagerSettings};
use kira::spatial::emitter::{EmitterHandle, EmitterSettings};
use kira::spatial::listener::{ListenerHandle, ListenerSettings};
use kira::spatial::scene::{SpatialSceneHandle, SpatialSceneSettings};
use kira::tween::Tween;

use crate::math::*;

use super::{PlayData, Sound2};

pub struct Audio2 {
    audio_manager: AudioManager,
    spatial_scene: SpatialSceneHandle,
    listener: ListenerHandle,
}

#[luajit_ffi_gen::luajit_ffi(managed = true)]
impl Audio2 {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        let settings = AudioManagerSettings::default();
        let mut audio_manager = AudioManager::new(settings).expect("Cannot create audio manager");

        let mut spatial_scene = audio_manager
            .add_spatial_scene(SpatialSceneSettings::default())
            .expect("Cannot add spatial scene");
        let mut listener = spatial_scene
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

    pub fn play(&mut self, sound: &mut Sound2) {
        let handle = self
            .audio_manager
            .play(sound.sound_data().clone())
            .expect("Cannot play sound");
        let emitter = self
            .spatial_scene
            .add_emitter([0.0, 0.0, 0.0], EmitterSettings::default())
            .unwrap();
        let play_data = PlayData::new(handle, emitter);

        sound.set_play_data(play_data);
    }

    pub fn set_listener_pos(&mut self, pos: &Vec3, rot: &Quat) {
        self.listener
            .set_position(*pos, Tween::default()) // TODO: set correct tween
            .expect("Cannot set listener position");
        self.listener
            .set_orientation([rot.x, rot.y, rot.z, rot.w], Tween::default()) // TODO: set correct tween
            .expect("Cannot set listener position");
    }
}
