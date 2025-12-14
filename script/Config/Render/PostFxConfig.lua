-- Alternative: Optimized for space games
Config.render.postFx = {
    aberration = {
        enable   = true,
        strength = 0.3
    },
    bloom = {
        enable = true,
        radius = 32
    },
    radialblur = {
        enable    = false,
        strength  = 1.0,
        scanlines = 1.0,
        center    = { 0.5, 0.5 }
    },
    sharpen = {
        enable   = true,
        strength = 0.5
    },
    tonemap = {
        enable     = true,
        mode       = Enums.Tonemappers.Illustris,
        exposure   = 1.2,
        autoExpose = {
            enable    = false,
            speedUp   = 0.5,
            speedDown = 0.3,
            minTarget = 0.4,
            maxTarget = 2.5
        }
    },
    colorgrade = {
        enable      = true,
        mode        = Enums.ColorGrades.Space,
        preExposure = 1.2,
        temperature = -0.1,
        tint        = 0.0,
        saturation  = 0.96,
        contrast    = 1.0,
        brightness  = 0.0,
        vibrance    = 0.1,
        lift        = { 0.0, 0.0, 0.01 },
        gamma       = { 1.0, 1.0, 1.0 },
        gain        = { 1.0, 1.0, 1.0 }
    },
    vignette = {
        enable   = true,
        strength = 0.3,
        hardness = 15.0
    },
    fxaa = {
        enable           = true,
        strength         = 0.75,
        edgeThreshold    = 0.125,
        edgeThresholdMin = 0.0312
    },
    dither = {
        enable   = true,
        strength = 0.3
    }
}
