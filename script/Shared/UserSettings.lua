--[[
User settings: applying them, their choices and their defaults.

The values live in GameState (user.ini keeps every plain value there, see
InitFiles). Most are read where they are used (volumes, mouse sensitivity,
invert, HUD switches); the ones here have to be pushed into the engine or into
Config when they change: by the Settings screen, and once at startup after
user.ini is read (`UserSettings.applyAll`).

`UserSettings.reset(group)` puts one Settings tab's values back to their
defaults (the Settings screen's "Reset to Default").
]]

local UserSettings = {}

-- The tonemap exposure that Brightness 50% stands for, taken before
-- `applyBrightness` changes it.
local baseExposure = Config.render.postFx.tonemap.exposure

---Planet and moon surface texture sizes for the Texture Quality setting
---(`GameState.render.textureQuality` is an index into this list).
UserSettings.textureQualities = {
    { name = "Low",    resolution = 256 },
    { name = "Medium", resolution = 512 },
    { name = "High",   resolution = 1024 },
}

---The Display Mode setting's choices, as GameState.render.fullscreen and
---fullscreenExclusive.
UserSettings.displayModes = {
    { name = "Windowed",            fullscreen = false, exclusive = false },
    { name = "Fullscreen Windowed", fullscreen = true,  exclusive = false },
    { name = "Fullscreen",          fullscreen = true,  exclusive = true },
}

---Window sizes offered by the Resolution setting (the current size is added
---when it isn't one of them, see `UserSettings.resolutions`).
local standardResolutions = {
    { 1280, 720 },
    { 1366, 768 },
    { 1600, 900 },
    { 1920, 1080 },
    { 2560, 1440 },
    { 3840, 2160 },
}

-- Defaults for "Reset to Default", taken before anything changes the Config
-- values they come from. The display defaults are the game's presentation
-- (borderless fullscreen, VSync off), not Config's windowed test-app defaults.
local defaults = {
    render = {
        fullscreen          = true,
        fullscreenExclusive = false,
        vsync               = Config.render.general.vsync,
        superSampleRate     = Config.render.general.superSampleRate,
        fov                 = Config.render.camera.fov,
        brightness          = Config.render.general.brightness,
        bloom               = Config.render.postFx.bloom.enable,
        smoothing           = Config.render.postFx.fxaa.enable,
        textureQuality      = Config.render.general.textureQuality,
    },
    audio = {
        soundEnabled  = Config.audio.general.soundEnabled,
        masterVolume  = Config.audio.general.masterVolume,
        musicVolume   = Config.audio.general.musicVolume,
        effectsVolume = Config.audio.general.effectsVolume,
    },
    input = {
        invertPitch      = false,
        mouseSensitivity = 0.5,
    },
    ui = {
        drawHud            = Config.ui.general.drawHud,
        showIndicatorNames = Config.ui.general.showIndicatorNames,
        scaleFactor        = Config.ui.general.scaleFactor,
        cursorStyle        = Config.ui.general.cursorStyle,
        hudStyle           = Config.ui.general.hudStyle,
    },
}

-- DISPLAY

---Switch the window to GameState.render's fullscreen / fullscreenExclusive.
function UserSettings.applyDisplayMode()
    Window:setFullscreen(GameState.render.fullscreen, GameState.render.fullscreenExclusive)
end

---@return integer index into `UserSettings.displayModes` for the current mode
function UserSettings.getDisplayMode()
    for index, mode in ipairs(UserSettings.displayModes) do
        if mode.fullscreen == GameState.render.fullscreen
            and (not mode.fullscreen or mode.exclusive == GameState.render.fullscreenExclusive) then
            return index
        end
    end
    return 1
end

---@param index integer index into `UserSettings.displayModes`
function UserSettings.setDisplayMode(index)
    local mode = UserSettings.displayModes[index]
    if not mode then return end
    GameState.render.fullscreen = mode.fullscreen
    GameState.render.fullscreenExclusive = mode.exclusive
    UserSettings.applyDisplayMode()
end

---The Resolution setting's choices: the standard sizes plus the current window
---size, smallest first.
---@return { [1]: integer, [2]: integer }[] sizes
function UserSettings.resolutions()
    local resX, resY = GameState.render.resX, GameState.render.resY
    local sizes, hasCurrent = {}, false
    for _, size in ipairs(standardResolutions) do
        sizes[#sizes + 1] = { size[1], size[2] }
        if size[1] == resX and size[2] == resY then hasCurrent = true end
    end
    if not hasCurrent and resX and resY and resX > 0 and resY > 0 then
        sizes[#sizes + 1] = { resX, resY }
        table.sort(sizes, function(a, b) return a[1] * a[2] < b[1] * b[2] end)
    end
    return sizes
end

---@param size { [1]: integer, [2]: integer }
---@return string
function UserSettings.resolutionName(size)
    return string.format("%dx%d", size[1], size[2])
end

---Resize the window to GameState.render.resX x resY, in windowed mode only: a
---fullscreen window has the screen's size, and resX/resY must keep saying so
---(UI code reads them as the window size; Application updates them on resize).
function UserSettings.applyResolution()
    if GameState.render.fullscreen then
        return
    end
    Window:setSize(GameState.render.resX, GameState.render.resY)
    Window:setCenteredPosition()
end

---Pick a window size (the Resolution setting). Does nothing in fullscreen.
---@param width integer
---@param height integer
function UserSettings.setResolution(width, height)
    if GameState.render.fullscreen then
        return
    end
    GameState.render.resX = width
    GameState.render.resY = height
    UserSettings.applyResolution()
end

---VSync on or off as GameState.render.vsync (presentMode follows it, so the two
---never disagree in user.ini).
function UserSettings.applyVsync()
    GameState.render.presentMode = GameState.render.vsync and PresentMode.Vsync or PresentMode.NoVsync
    Window:setPresentMode(GameState.render.presentMode)
end

-- PICTURE

---The camera's vertical field of view, in degrees: the projection and the
---systems that place things on screen read Config.render.camera.fov every frame.
function UserSettings.applyFov()
    Config.render.camera.fov = GameState.render.fov
end

---Brightness 0-1 as the tonemap's exposure: 0.5 is the exposure PostFxConfig
---sets, 0 is half of it, 1 is twice it. The renderer reads it every frame.
function UserSettings.applyBrightness()
    local brightness = GameState.render.brightness or 0.5
    Config.render.postFx.tonemap.exposure = baseExposure * 2 ^ ((brightness - 0.5) * 2)
end

---Bloom and Smoothing (FXAA) on or off: the renderer checks both every frame.
function UserSettings.applyPostFx()
    Config.render.postFx.bloom.enable = GameState.render.bloom
    Config.render.postFx.fxaa.enable = GameState.render.smoothing
end

---Planet and moon texture size from the Texture Quality setting. Used when
---planets are generated, so it shows from the next new game (or menu background).
function UserSettings.applyTextureQuality()
    local quality = UserSettings.textureQualities[GameState.render.textureQuality]
        or UserSettings.textureQualities[defaults.render.textureQuality]
    Config.game.planetTexRes = quality.resolution
    Config.game.moonTexRes = quality.resolution
end

-- INPUT

---The multiplier the Mouse Sensitivity setting puts on mouse look and mouse
---flight: 50% (the default) is 1x, 100% is 2x, 5% is 0.1x.
---@return number
function UserSettings.mouseSensitivityFactor()
    return 2 * (GameState.input.mouseSensitivity or 0.5)
end

-- ALL

---Push every setting into the engine and Config: at startup, after user.ini is read.
function UserSettings.applyAll()
    UserSettings.applyDisplayMode()
    UserSettings.applyVsync()
    UserSettings.applyFov()
    UserSettings.applyBrightness()
    UserSettings.applyPostFx()
    UserSettings.applyTextureQuality()
end

---The game's own display defaults: borderless fullscreen with VSync off
---(exclusive fullscreen switches the display mode, which can lower the frame
---rate). Called by the game before it reads user.ini, so a saved choice wins.
---Test apps keep Config's windowed defaults.
function UserSettings.useGameDisplayDefaults()
    GameState.render.fullscreen = defaults.render.fullscreen
    GameState.render.fullscreenExclusive = defaults.render.fullscreenExclusive
    GameState.render.vsync = defaults.render.vsync
end

-- The values each Settings tab resets, as { GameState category, key }.
local groups = {
    interface = {
        { "input", "invertPitch" }, { "input", "mouseSensitivity" },
        { "ui", "drawHud" }, { "ui", "showIndicatorNames" },
        { "ui", "scaleFactor" }, { "ui", "cursorStyle" }, { "ui", "hudStyle" },
    },
    graphics = {
        -- not resX/resY: they are the window's current size (see applyResolution)
        { "render", "fullscreen" }, { "render", "fullscreenExclusive" }, { "render", "vsync" },
        { "render", "superSampleRate" }, { "render", "fov" }, { "render", "brightness" },
        { "render", "bloom" }, { "render", "smoothing" }, { "render", "textureQuality" },
    },
    audio = {
        { "audio", "soundEnabled" }, { "audio", "masterVolume" },
        { "audio", "musicVolume" }, { "audio", "effectsVolume" },
    },
}

---A copy of every value the Settings screen changes (all tabs, plus the window
---size), to compare with later (`differsFrom`) or to go back to (`restore`).
---@return table<string, any> snapshot
function UserSettings.snapshot()
    local snapshot = {}
    for _, entries in pairs(groups) do
        for _, entry in ipairs(entries) do
            snapshot[entry[1] .. "." .. entry[2]] = GameState[entry[1]][entry[2]]
        end
    end
    snapshot["render.resX"] = GameState.render.resX
    snapshot["render.resY"] = GameState.render.resY
    return snapshot
end

---Whether a windowed window's size differs from a snapshot's. Only then is the
---window size a setting: in fullscreen it follows the screen.
local function windowSizeDiffers(snapshot)
    return not snapshot["render.fullscreen"] and not GameState.render.fullscreen
        and (snapshot["render.resX"] ~= GameState.render.resX or snapshot["render.resY"] ~= GameState.render.resY)
end

---Whether any value the Settings screen changes differs from a snapshot.
---@param snapshot table<string, any> from `UserSettings.snapshot`
---@return boolean
function UserSettings.differsFrom(snapshot)
    for _, entries in pairs(groups) do
        for _, entry in ipairs(entries) do
            if snapshot[entry[1] .. "." .. entry[2]] ~= GameState[entry[1]][entry[2]] then
                return true
            end
        end
    end
    return windowSizeDiffers(snapshot)
end

---Put every value the Settings screen changes back as it was in a snapshot,
---and apply them (the Settings screen's "Undo changes").
---@param snapshot table<string, any> from `UserSettings.snapshot`
function UserSettings.restore(snapshot)
    for _, entries in pairs(groups) do
        for _, entry in ipairs(entries) do
            GameState[entry[1]][entry[2]] = snapshot[entry[1] .. "." .. entry[2]]
        end
    end
    UserSettings.applyAll()
    if windowSizeDiffers(snapshot) then
        UserSettings.setResolution(snapshot["render.resX"], snapshot["render.resY"])
    end
    require("Shared.Audio.MusicPlayer"):setGlobalVolume()
end

---Put one Settings tab's values back to their defaults and apply them.
---@param group "interface"|"graphics"|"audio"
function UserSettings.reset(group)
    for _, entry in ipairs(groups[group] or {}) do
        local category, key = entry[1], entry[2]
        GameState[category][key] = defaults[category][key]
    end

    if group == "graphics" then
        UserSettings.applyAll()
    elseif group == "audio" then
        -- required here, so flight and camera code can use this module without the audio code
        require("Shared.Audio.MusicPlayer"):setGlobalVolume()
    end
end

return UserSettings
