local SFXObject = require("Types.SFXObject")

Config.audio.sounds = {
    pulseFire = SFXObject:Create {
        name = "Pulse Cannon",
        path = "pulse5.wav",
        volume = 0.0,
        isLooping = false
    },
    pulseHit = {},
    explodeShip = {},
    explodeStation = {},
    fxSensors = SFXObject:Create {
        name = "Ship Sensors",
        path = "sensors.wav",
        volume = 0.0,
        isLooping = true
    },
}
