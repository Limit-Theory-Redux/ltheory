use glam::Quat;
use kira::listener::ListenerHandle;
use kira::track::{SpatialTrackBuilder, SpatialTrackDistances};
use kira::{AudioManager, AudioManagerSettings, Tween};

use super::convert::{amplitude_to_decibels, linear_tween};
use super::{Sound, SoundInstance};
use crate::math::*;

/// How far 3D sounds pan between the ears: the ear facing away from a sound
/// keeps half of its amplitude.
const SPATIALIZATION_STRENGTH: f32 = 0.5;

struct ListenerInfo {
    listener: ListenerHandle,
    position: Position,
    orientation: Quat,
}

pub struct Audio {
    audio_manager: AudioManager,
    listener_info: ListenerInfo,
    audio_origin: Position,
}

#[luajit_ffi_gen::luajit_ffi]
impl Audio {
    /// Opens the default output device and creates the listener for 3D sounds.
    ///
    /// Panics if the output device cannot be opened.
    #[bind(name = "Create")]
    pub fn new() -> Self {
        let mut audio_manager = AudioManager::new(AudioManagerSettings::default())
            .unwrap_or_else(|err| panic!("Cannot open the audio output device: {err}"));

        let position = Position::ZERO;
        let orientation = Quat::IDENTITY;
        let listener = audio_manager
            .add_listener(
                position.relative_to(Position::ZERO).to_array(),
                orientation.to_array(),
            )
            .expect("Cannot add listener");

        Self {
            audio_manager,
            listener_info: ListenerInfo {
                listener,
                position,
                orientation,
            },
            audio_origin: Position::ZERO,
        }
    }

    pub fn play(&mut self, sound: &mut Sound, init_volume: f64, fade_millis: u64) -> SoundInstance {
        let sound_data = sound
            .sound_data()
            .volume(amplitude_to_decibels(init_volume))
            .fade_in_tween(linear_tween(fade_millis));

        let sound_handle = self
            .audio_manager
            .play(sound_data)
            .expect("Cannot play sound");

        SoundInstance::new(sound_handle, init_volume, None)
    }

    /// Plays the sound at a position on a spatial track of its own, panned and
    /// attenuated for the listener.
    ///
    /// The volume falls off linearly in decibels, from full volume at the
    /// minimum distance to silence at the maximum distance.
    pub fn play_3d(
        &mut self,
        sound: &mut Sound,
        init_volume: f64,
        fade_millis: u64,
        init_pos: Position,
        min_distance: f32,
        max_distance: f32,
    ) -> SoundInstance {
        let mut spatial_track = self
            .audio_manager
            .add_spatial_sub_track(
                &self.listener_info.listener,
                init_pos.relative_to(self.audio_origin).to_array(),
                SpatialTrackBuilder::new()
                    .distances(SpatialTrackDistances {
                        min_distance,
                        max_distance,
                    })
                    .spatialization_strength(SPATIALIZATION_STRENGTH)
                    .sound_capacity(1),
            )
            .expect("Cannot add a spatial track");

        let sound_data = sound
            .sound_data()
            .volume(amplitude_to_decibels(init_volume))
            .fade_in_tween(linear_tween(fade_millis));

        let sound_handle = spatial_track.play(sound_data).expect("Cannot play sound");

        SoundInstance::new(
            sound_handle,
            init_volume,
            Some((spatial_track, init_pos, self.audio_origin)),
        )
    }

    pub fn set_listener_pos(&mut self, pos: &Position) {
        self.listener_info.listener.set_position(
            pos.relative_to(self.audio_origin).to_array(),
            Tween::default(),
        );

        self.listener_info.position = *pos;
    }

    pub fn listener_pos(&self) -> Position {
        self.listener_info.position
    }

    pub fn set_listener_rot(&mut self, rot: &Quat) {
        self.listener_info
            .listener
            .set_orientation(rot.to_array(), Tween::default());

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
            self.listener_info
                .position
                .relative_to(self.audio_origin)
                .to_array(),
            Tween::default(),
        );
    }

    pub fn origin_pos(&self) -> Position {
        self.audio_origin
    }

    /// Number of sounds alive: sounds on the main track plus one per spatial
    /// track of a 3D sound.
    pub fn get_loaded_count(&mut self) -> u64 {
        (self.audio_manager.main_track().num_sounds() + self.audio_manager.num_sub_tracks()) as u64
    }

    /// Maximum number of sounds: the main track's sound capacity plus the
    /// maximum number of spatial tracks for 3D sounds.
    pub fn get_total_count(&mut self) -> u64 {
        (self.audio_manager.main_track().sound_capacity() + self.audio_manager.sub_track_capacity())
            as u64
    }
}
