local SFXObject = require("Types.SFXObject")

Config.audio.sounds = {
    pulseFire = SFXObject:create {
        name = "Pulse Cannon",
        path = "pulse5.wav",
        volume = 0.0,
        isLooping = false
    },
    pulseHit = {},
    explodeShip = {},
    explodeStation = {},
    fxSensors = SFXObject:create {
        name = "Ship Sensors",
        path = "sensors.wav",
        volume = 0.0,
        isLooping = true
    },
    click = SFXObject:create {
        name = "UI Click",
        path = "click.wav",
        volume = 0.0,
        isLooping = false
    },
    metallicAmbience = SFXObject:create {
        name = "Metallic Ambience",
        path = "metallic_ambience.ogg",
        volume = 0.0,
        isLooping = true
    }
}
