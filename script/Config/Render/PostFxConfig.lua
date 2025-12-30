-- Alternative: Optimized for space games
Config.render.postFx = {
    aberration = {
        enable   = true,
        strength = 0.3
    },
    bloom = {
        enable     = true,
        radius     = 32,       -- Original value for quality
        iterations = 3,        -- Original value for smooth bloom
        fastBlur   = false,    -- Use quality Gaussian blur
        intensity  = 0.15      -- Original value
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
        temperature = -0.05,
        tint        = 0.0,
        saturation  = 0.82,
        contrast    = 1.0,
        brightness  = 0.005,
        vibrance    = 0.125,
        lift        = { 0.0, 0.0, 0.01 },
        gamma       = { 1.0, 1.0, 1.0 },
        gain        = { 1.0, 1.0, 1.0 }
    },
    vignette = {
        enable   = true,
        strength = 0.35,
        hardness = 17.0
    },
    fxaa = {
        enable           = true,
        fast             = false,    -- Use quality 21-sample version
        strength         = 1.0,
        edgeThreshold    = 0.063,
        edgeThresholdMin = 0.0312
    },
    dither = {
        enable   = true,
        strength = 0.3
    },
    panini = {
        enable   = true,
        distance = 0.5,   -- 0 = off, 1 = full cylindrical (0.3-0.6 typical)
        scale    = 1.0    -- Vertical scale adjustment (auto-calculated if 0)
    }
}
