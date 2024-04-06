use internal::ConvertIntoString;

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
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
        }
    }

    pub fn get_duration(&self) -> f32 {
        self.sound_data.duration().as_secs_f32()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}
