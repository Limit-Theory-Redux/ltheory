-- Render Settings Moved from 'Render/RenderPipeline' --
Config.render.general = {
    superSampleRate     = 1,
    downSampleRate      = 4,
    thrusterLights      = false,
    pulseLights         = false,
    fullscreen          = false,
    fullscreenExclusive = false,
    presentMode         = PresentMode.Vsync,
}

Config.render.window = {
    defaultResX = 1920,
    defaultResY = 1080,
}

--[[
Both showBuffers and cullFace potentially Depricated
]]                               --
Config.render.debug = {
    showBuffers = false,         -- Used Once in Rendering Pipeline, "Show Deferred Buffers"
}
Config.render.renderState = {
    cullFace = false,            -- Used Once in Rendering Pipeline, "Backface Culling"
}

-- Settings Removed --
--[[
- render.lodScale       -- Unused
- render.wireframe      -- Currently in Config.debug.physics
- render.logZNear       -- Unused
- render.logZFar        -- Unused
]]
