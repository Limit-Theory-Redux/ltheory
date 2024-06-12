use std::time::Duration;

use kira::manager::{AudioManager, AudioManagerSettings, Capacities};
use kira::spatial::emitter::{EmitterDistances, EmitterSettings};
use kira::spatial::listener::{ListenerHandle, ListenerSettings};
use kira::spatial::scene::{SpatialSceneHandle, SpatialSceneSettings};
use kira::tween::{Easing, Tween};
use kira::StartTime;

use crate::math::*;

use super::{process_command_error, Sound, SoundInstance};

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

        let sound_instance = SoundInstance::new(sound_handle, init_volume, None);

        sound_instance
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

        let sound_instance = SoundInstance::new(
            sound_handle,
            init_volume,
            Some((emitter_handle, init_pos, self.audio_origin)),
        );

        sound_instance
    }

    pub fn set_listener_pos(&mut self, pos: &Position) {
        process_command_error(
            self.listener_info
                .listener
                .set_position(pos.relative_to(self.audio_origin), Tween::default()),
            "Cannot set listener position",
        );

        self.listener_info.position = *pos;

        // If the listener has strayed too far from the origin, update it.
        const UPDATE_DIST: f64 = 1_000_000.0;
        if pos.distance_squared(self.audio_origin) > (UPDATE_DIST * UPDATE_DIST) {
            self.update_origin(pos);
        }
    }

    pub fn listener_pos(&self) -> Position {
        self.listener_info.position
    }

    pub fn set_listener_rot(&mut self, rot: &Quat) {
        process_command_error(
            self.listener_info
                .listener
                .set_orientation([rot.x, rot.y, rot.z, rot.w], Tween::default()),
            "Cannot set listener orientation",
        );

        self.listener_info.orientation = *rot;
    }

    pub fn listener_rot(&self) -> Quat {
        self.listener_info.orientation
    }

    pub fn get_loaded_count(&self) -> u64 {
        self.audio_manager.num_sounds() as u64
    }

    pub fn get_total_count(&self) -> u64 {
        self.audio_manager.sound_capacity() as u64
    }
}

impl Audio {
    /// Updates the origin in Kira's coordinate system.
    ///
    /// As Kira maintains a 32-bit coordinate system, if the listener strays too far away from the origin, we will start to have difficulty with 32-bit precision.
    /// If this function is called, all currently playing sounds and the listener will have their position recalculated from the new origin in Kira's coordinate system.
    pub fn update_origin(&mut self, origin: &Position) {
        self.audio_origin = *origin;

        process_command_error(
            self.listener_info.listener.set_position(
                self.listener_info.position.relative_to(self.audio_origin),
                Tween::default(),
            ),
            "Cannot set listener position",
        );

        // TODO: Loop through currently playing sounds, and update their positions.
    }
}
