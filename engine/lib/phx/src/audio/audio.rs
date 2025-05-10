use std::time::Duration;

use glam::Quat;
use kira::StartTime;
use kira::manager::{AudioManager, AudioManagerSettings, Capacities};
use kira::spatial::emitter::{EmitterDistances, EmitterSettings};
use kira::spatial::listener::{ListenerHandle, ListenerSettings};
use kira::spatial::scene::{SpatialSceneHandle, SpatialSceneSettings};
use kira::tween::{Easing, Tween};

use super::{Sound, SoundInstance};
use crate::math::*;

const DEFAULT_COMMAND_CAPACITY: usize = 1024;

struct ListenerInfo {
    listener: ListenerHandle,
    position: Position,
    orientation: Quat,
}

pub struct Audio {
    audio_manager: AudioManager,
    spatial_scene: SpatialSceneHandle,
    listener_info: ListenerInfo,
    audio_origin: Position,
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
            .add_spatial_scene(
                SpatialSceneSettings::new()
                    .emitter_capacity(128)
                    .listener_capacity(2),
            )
            .expect("Cannot add spatial scene");

        let position = Position::ZERO;
        let orientation = Quat::IDENTITY;
        let listener = spatial_scene
            .add_listener(
                position.relative_to(Position::ZERO),
                orientation,
                ListenerSettings::default(),
            )
            .expect("Cannot add listener");

        Self {
            audio_manager,
            spatial_scene,
            listener_info: ListenerInfo {
                listener,
                position,
                orientation,
            },
            audio_origin: Position::ZERO,
        }
    }

    pub fn play(&mut self, sound: &mut Sound, init_volume: f64, fade_millis: u64) -> SoundInstance {
        let mut sound_data_clone = sound.sound_data().clone();
        sound_data_clone.settings.volume = init_volume.into();

        sound_data_clone.settings.fade_in_tween = Some(Tween {
            start_time: StartTime::Immediate,
            duration: Duration::from_millis(fade_millis),
            easing: Easing::Linear,
        });

        let sound_handle = self
            .audio_manager
            .play(sound_data_clone)
            .expect("Cannot play sound");

        SoundInstance::new(sound_handle, init_volume, None)
    }

    pub fn play_3d(
        &mut self,
        sound: &mut Sound,
        init_volume: f64,
        fade_millis: u64,
        init_pos: Position,
        min_distance: f32,
        max_distance: f32,
    ) -> SoundInstance {
        let emitter_handle = self
            .spatial_scene
            .add_emitter(
                init_pos.relative_to(self.audio_origin),
                EmitterSettings::new().distances(EmitterDistances {
                    min_distance,
                    max_distance,
                }),
            )
            .expect("Cannot add an emitter");

        let mut sound_data_clone = sound.sound_data().clone();
        sound_data_clone.settings.volume = init_volume.into();

        sound_data_clone.settings.fade_in_tween = Some(Tween {
            start_time: StartTime::Immediate,
            duration: Duration::from_millis(fade_millis),
            easing: Easing::Linear,
        });

        sound_data_clone.settings.output_destination = (&emitter_handle).into();

        let sound_handle = self
            .audio_manager
            .play(sound_data_clone)
            .expect("Cannot play sound");

        SoundInstance::new(
            sound_handle,
            init_volume,
            Some((emitter_handle, init_pos, self.audio_origin)),
        )
    }

    pub fn set_listener_pos(&mut self, pos: &Position) {
        self.listener_info
            .listener
            .set_position(pos.relative_to(self.audio_origin), Tween::default());

        self.listener_info.position = *pos;
    }

    pub fn listener_pos(&self) -> Position {
        self.listener_info.position
    }

    pub fn set_listener_rot(&mut self, rot: &Quat) {
        self.listener_info
            .listener
            .set_orientation([rot.x, rot.y, rot.z, rot.w], Tween::default());

        self.listener_info.orientation = *rot;
    }

    pub fn listener_rot(&self) -> Quat {
        self.listener_info.orientation
    }

    /// Updates the origin in Kira's coordinate system.
    ///
    /// As Kira maintains a 32-bit coordinate system, if the listener strays too far away from the origin, we will start to have difficulty with 32-bit precision.
    /// If this function is called, the listener and all new sounds will have their position calculated from the new origin in Kira's coordinate system.
    pub fn set_origin_pos(&mut self, origin: &Position) {
        self.audio_origin = *origin;

        self.listener_info.listener.set_position(
            self.listener_info.position.relative_to(self.audio_origin),
            Tween::default(),
        );
    }

    pub fn origin_pos(&self) -> Position {
        self.audio_origin
    }

    pub fn get_loaded_count(&self) -> u64 {
        self.audio_manager.num_sounds() as u64
    }

    pub fn get_total_count(&self) -> u64 {
        self.audio_manager.sound_capacity() as u64
    }
}
