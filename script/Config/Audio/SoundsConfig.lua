local SFXObject = require("Types.SFXObject")

Config.audio.sounds = {
    pulseFire = SFXObject:Create {
        name = "Pulse Cannon",
        path = Config.paths.soundEffects .. "pulse5.wav",
        volume = 0.0,
        isLooping = false
    },
    pulseHit = {},
    explodeShip = {},
    explodeStation = {},
    fxSensors = SFXObject:Create {
        name = "Ship Sensors",
        path = Config.paths.soundEffects .. "sensors.wav",
        volume = 0.0,
        isLooping = false
    },
}
