#[luajit_ffi_gen::luajit_ffi]
#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum SoundGroup {
    Ambient,
    Effects,
    Music,
}

impl SoundGroup {
    pub fn max_playing_sounds(&self) -> usize {
        match self {
            SoundGroup::Ambient => 32,
            SoundGroup::Effects => 64,
            SoundGroup::Music => 2,
        }
    }
}
