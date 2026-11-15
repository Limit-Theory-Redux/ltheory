Config.audio.general = {
    soundEnabled         = true,
    supportedFormats     = { ".ogg", ".mp3", ".wav", ".flac" },
    mainMenuMusicEnabled = true,
    soundMin             = 0,
    soundMax             = 1,   -- SetVolume range seems to go from 0 (min) to about 2 or 3 (max)
    masterVolume         = 1.0, -- scales music and sound effects
    effectsVolume        = 1.0, -- scales every sound effect (SFXObject:play)
    musicVolume          = 0.5, -- current volume
    mainMenu             = "LTR_Surpassing_The_Limit_Redux_Ambient_Long_Fade.ogg",
}
