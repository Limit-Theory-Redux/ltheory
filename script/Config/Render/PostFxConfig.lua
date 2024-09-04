-- Render Settings Moved from 'Render/RenderPipeline' --
--[[
Given that all these Settings are by default already Disabled
render.postfx is potentially fully depricated.
--]]
Config.PostFx = {
    abberation  = {
        enable      = false,
        strength    = 1             -- Float: 0 - 1      
    },
    bloom       = {
        enable      = false,
        radius      = 48            -- Float: 4 - 64
    },
    radialblur  = {
        enable      = false,
        strength    = 1,            -- Float: 0 - 1
        scanlines   = 1             -- Float: 0 - 1
    },
    sharpen     = false,            -- 
    tonemap     = false,            -- Potentially Depricated
    --[[
    Vignette is currently unused in RenderPipeline
    TODO: Decide on removal of renderPipeline:Vignette
    vignette    = {
        enable      = false,
        strength    = 0.25,         -- Float: 0 - 1
        hardness    = 20.0          -- Float: 2 - 32
    }
    --]]
}