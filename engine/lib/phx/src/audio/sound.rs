use internal::ConvertIntoString;

use kira::sound::static_sound::StaticSoundData;
use kira::sound::{EndPosition, PlaybackPosition, Region};

pub struct Sound {
    path: String,
    sound_data: StaticSoundData,
}

impl Sound {
    pub fn sound_data(&self) -> &StaticSoundData {
        &self.sound_data
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Sound {
    #[bind(name = "Load")]
    pub fn new(path: &str, is_looping: bool) -> Self {
        let mut sound_data =
            StaticSoundData::from_file(path).expect(&format!("Cannot load sound: {path}"));

        if is_looping {
            // Loop over whole audio
            sound_data = sound_data.loop_region(Region {
                start: PlaybackPosition::Seconds(0.0),
                end: EndPosition::EndOfAudio,
            });
        }

        Self {
            path: path.into(),
            sound_data,
        }
    }

    pub fn get_duration(&self) -> f32 {
        self.sound_data.duration().as_secs_f32()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}
