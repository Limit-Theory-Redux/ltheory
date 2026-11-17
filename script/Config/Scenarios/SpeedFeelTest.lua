-- Speed Feel Test: fly, boost and use the travel drive with the speed cues of
-- SpeedFeelSystem switched on and off, to feel what each one adds. Starts
-- next to the belt (rocks to fly past) with every cue on.
--
-- Numpad 1-7 switch one cue each (field of view, shake, radial blur,
-- aberration, vignette, travel drive overlay, space dust), 0 switches all on
-- or off, + and - scale them all (intensity 0 to 2), 8 switches what leaving
-- travel mode does (expand / collapse), / and * make the dust's streaks
-- thinner / wider. The values per cue are in Config.render.speedFeel
-- (Config/Render/SpeedFeelConfig.lua).

local SpeedFeelSystem = require("Modules.Rendering.Systems.SpeedFeelSystem")

-- The cues on keys, in SpeedFeelSystem's names
local CUES = {
    { key = Button.KeyboardNumpad1, cue = "fov", label = "field of view" },
    { key = Button.KeyboardNumpad2, cue = "shake", label = "shake" },
    { key = Button.KeyboardNumpad3, cue = "radialBlur", label = "radial blur" },
    { key = Button.KeyboardNumpad4, cue = "aberration", label = "aberration" },
    { key = Button.KeyboardNumpad5, cue = "vignette", label = "vignette" },
    { key = Button.KeyboardNumpad6, cue = "travelDrive", label = "travel overlay" },
    { key = Button.KeyboardNumpad7, cue = "dust", label = "space dust" },
}

local INTENSITY_STEP, MAX_INTENSITY = 0.25, 2.0
local WIDTH_STEP = 1.15 -- the dust's streak width, per key press

local function setAll(on)
    for _, c in ipairs(CUES) do SpeedFeelSystem:setCue(c.cue, on) end
end

local function anyOn()
    for _, c in ipairs(CUES) do
        if SpeedFeelSystem:isCueOn(c.cue) then return true end
    end
    return false
end

return {
    name = "Speed Feel Test",
    description = "Fly, boost (Shift) and travel (T) with the speed effects on numpad keys.",

    setup = function(game)
        SpeedFeelSystem:enable(game)
        SpeedFeelSystem:setIntensity(1.0)
        setAll(true)
        SpeedFeelSystem:setExitStyle(nil)
        SpeedFeelSystem:setDustWidth(1.0)
    end,

    teardown = function(game)
        setAll(false)
        SpeedFeelSystem:disable()
    end,

    onInput = function(game, dt)
        for _, c in ipairs(CUES) do
            if Input:isPressed(c.key) then
                SpeedFeelSystem:setCue(c.cue, not SpeedFeelSystem:isCueOn(c.cue))
            end
        end
        if Input:isPressed(Button.KeyboardNumpad0) then
            setAll(not anyOn())
        end
        local intensity = SpeedFeelSystem:getIntensity()
        if Input:isPressed(Button.KeyboardNumpadAdd) then
            SpeedFeelSystem:setIntensity(math.min(MAX_INTENSITY, intensity + INTENSITY_STEP))
        elseif Input:isPressed(Button.KeyboardNumpadSubtract) then
            SpeedFeelSystem:setIntensity(math.max(0, intensity - INTENSITY_STEP))
        end
        if Input:isPressed(Button.KeyboardNumpad8) then
            SpeedFeelSystem:setExitStyle(SpeedFeelSystem:getExitStyle() == "expand" and "collapse" or "expand")
        end
        local width = SpeedFeelSystem:getDustWidth()
        if Input:isPressed(Button.KeyboardNumpadMultiply) then
            SpeedFeelSystem:setDustWidth(width * WIDTH_STEP)
        elseif Input:isPressed(Button.KeyboardNumpadDivide) then
            SpeedFeelSystem:setDustWidth(width / WIDTH_STEP)
        end
    end,

    controls = {
        { keys = "Numpad 1-7", action = "Speed cue on / off (field of view, shake, radial blur, aberration, vignette, travel overlay, dust)" },
        { keys = "Numpad 0", action = "All speed cues on / off" },
        { keys = "Numpad + -", action = "Speed cues stronger / weaker" },
        { keys = "Numpad 8", action = "Leaving travel mode: expand / collapse" },
        { keys = "Numpad / *", action = "Dust streaks thinner / wider" },
    },

    overlayLines = function(game)
        local d = SpeedFeelSystem:getDrive()
        local cues = {}
        for i, c in ipairs(CUES) do
            cues[i] = string.format("%d %s %s", i, c.label, SpeedFeelSystem:isCueOn(c.cue) and "ON" or "off")
        end
        return {
            "Speed Feel Test",
            string.format("%.2fx max speed   drive %s   flight %.2f  boost %.2f  boosting %.2f  charge %.2f  travel %.2f",
                d.speed / d.maxSpeed, d.state, d.flight, d.boost, d.boosting, d.charge, d.travel),
            "Numpad: " .. table.concat(cues, "   "),
            string.format("Numpad: 0 all on / off   + - intensity (%.2f)   8 travel exit (%s)   / * streak width (%.2f)",
                SpeedFeelSystem:getIntensity(), SpeedFeelSystem:getExitStyle(), SpeedFeelSystem:getDustWidth()),
        }
    end,
}
