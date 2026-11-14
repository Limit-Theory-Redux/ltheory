local ShipActions         = require('Input.ActionBindings.ShipActions')
local CameraActions       = require('Input.ActionBindings.CameraActions')
local GeneralActions      = require('Input.ActionBindings.GeneralActions')
local MapActions          = require('Input.ActionBindings.MapActions')
local ApplicationBindings = require('States.ApplicationBindings')

--- ControlsHelp: the content of the Controls (help) screen, built from the game's
--- input bindings, so a new binding shows up there without touching the screen.
--- Bindings can set `label`, `group` and `hidden` next to their device lists
--- (see DeviceBindings in Input/ActionBinding.lua).
---@class ControlsHelp
local ControlsHelp = {}

--- Binding tables shown on the Controls screen. A new binding in one of them
--- appears automatically; a new table needs a line here.
local BINDING_TABLES = { ShipActions, CameraActions, GeneralActions, MapActions }

--- Groups in screen order; any other group follows, "Other" comes last.
local GROUP_ORDER = { "Flight", "Camera", "Map", "General", "Debug" }

--- App-level keys that still live in the legacy ApplicationBindings (plain
--- buttons, not ActionBindings). Listed by hand, only the ones that work in game.
local LEGACY_KEYS = {
    { label = "Pause menu", group = "General", buttons = { ApplicationBindings.Escape } },
    { label = "Pause",      group = "General", buttons = { ApplicationBindings.Pause } },
    { label = "Toggle HUD", group = "General", buttons = { ApplicationBindings.ToggleHUD } },
    { label = "Screenshot", group = "General", buttons = { ApplicationBindings.Screenshot } },
    { label = "Fullscreen", group = "General", buttons = { ApplicationBindings.ToggleFullscreen } },
    { label = "Quit",       group = "General", text = "Alt + Q" }, -- hard-coded in Application:handleAppInput
}

--- Names that read better than the split-up enum name.
local KEY_NAMES = {
    KeyboardControlLeft = "Left Ctrl", KeyboardControlRight = "Right Ctrl",
    KeyboardShiftLeft = "Left Shift", KeyboardShiftRight = "Right Shift",
    KeyboardAltLeft = "Left Alt", KeyboardAltRight = "Right Alt",
    MouseLeft = "Left mouse", MouseMiddle = "Middle mouse", MouseRight = "Right mouse",
    -- Gamepad, Xbox-style names. In the engine's (gilrs) naming LeftTrigger is the
    -- bumper and LeftTrigger2 the analog trigger.
    GamepadSouth = "A", GamepadEast = "B", GamepadWest = "X", GamepadNorth = "Y",
    GamepadLeftTrigger = "LB", GamepadRightTrigger = "RB",
    GamepadLeftTrigger2 = "LT", GamepadRightTrigger2 = "RT",
    GamepadLeftThumb = "L3", GamepadRightThumb = "R3",
    GamepadSelect = "Back", GamepadStart = "Start", GamepadMode = "Guide",
    GamepadDPadUp = "D-pad up", GamepadDPadDown = "D-pad down",
    GamepadDPadLeft = "D-pad left", GamepadDPadRight = "D-pad right",
}

--- Modifier names without the side, for combos ("Ctrl + Shift + F3").
local SIDELESS = {
    KeyboardControlLeft = "Ctrl", KeyboardControlRight = "Ctrl",
    KeyboardShiftLeft = "Shift", KeyboardShiftRight = "Shift",
    KeyboardAltLeft = "Alt", KeyboardAltRight = "Alt",
}

-- Button number -> enum name ("KeyboardW"), from the Button table.
local buttonNames = {}
for name, value in pairs(Button) do
    if type(value) == "number" then buttonNames[value] = name end
end

--- "ThrustForward" -> "Thrust forward", "LeftStickX" -> "Left stick X".
---@param name string
---@return string
local function splitName(name)
    local words = {}
    for word in name:gsub("(%l)(%u)", "%1 %2"):gmatch("%S+") do
        words[#words + 1] = (#words > 0 and #word > 1) and word:lower() or word
    end
    return table.concat(words, " ")
end

---@param button integer a Button value
---@param inCombo boolean|nil
---@return string
local function keyName(button, inCombo)
    local name = buttonNames[button]
    if not name then return "?" end
    if inCombo and SIDELESS[name] then return SIDELESS[name] end
    if KEY_NAMES[name] then return KEY_NAMES[name] end
    local short = name:gsub("^Keyboard", ""):gsub("^Gamepad", ""):gsub("^Digit", "")
    return splitName(short)
end

---@param control integer|table a Button value or a Control table (Input/Control.lua)
---@param inCombo boolean|nil
---@return string
local function controlText(control, inCombo)
    if type(control) == "number" then return keyName(control, inCombo) end
    local t, types = control.type, Enums.ControlType
    if t == types.Single then
        return keyName(control.key, inCombo)
    elseif t == types.Pair then
        -- "D-pad up" / "D-pad down" -> "D-pad up / down"
        local positive, negative = controlText(control.positive), controlText(control.negative)
        local prefix = positive:match("^(.+ )")
        if prefix and negative:sub(1, #prefix) == prefix then
            return positive .. " / " .. negative:sub(#prefix + 1)
        end
        return positive .. " / " .. negative
    elseif t == types.Combo then
        local parts = {}
        for _, key in ipairs(control.keys) do parts[#parts + 1] = controlText(key, true) end
        return table.concat(parts, " + ")
    elseif t == types.GamepadAxis then
        return keyName(control.button)
    elseif t == types.MouseX or t == types.MouseDX then
        return inCombo and "drag left / right" or "Mouse left / right"
    elseif t == types.MouseY or t == types.MouseDY then
        return inCombo and "drag up / down" or "Mouse up / down"
    elseif t == types.MouseWheel then
        return "Mouse wheel"
    end
    return "?"
end

--- All alternatives of one or more device lists as one string: duplicates
--- dropped, and "Left X" + "Right X" merged into "X".
---@param ... table[]|nil device lists (e.g. keyboard, mouse)
---@return string
local function alternativesText(...)
    local texts, seen = {}, {}
    for i = 1, select("#", ...) do
        for _, control in ipairs(select(i, ...) or {}) do
            local text = controlText(control)
            if not seen[text] then
                seen[text] = true
                texts[#texts + 1] = text
            end
        end
    end
    for i = #texts, 1, -1 do
        local rest = texts[i]:match("^Right (.+)$")
        if rest and seen["Left " .. rest] then
            table.remove(texts, i)
            for j, text in ipairs(texts) do
                if text == "Left " .. rest then texts[j] = rest end
            end
        end
    end
    return table.concat(texts, ", ")
end

---@class ControlsHelpRow
---@field action string
---@field keys string keyboard and mouse
---@field gamepad string

---@class ControlsHelpGroup
---@field title string
---@field rows ControlsHelpRow[]

--- The Controls screen's content: groups in screen order, each with its rows
--- in declaration order (legacy app keys after the bindings). Within one binding
--- table that's the order in the file; between tables it follows the order in
--- which their modules were first loaded.
---@return ControlsHelpGroup[]
function ControlsHelp.groups()
    local entries = {}
    for _, bindings in ipairs(BINDING_TABLES) do
        for name, binding in pairs(bindings) do
            local def = binding.deviceBindings
            if def and not def.hidden then
                entries[#entries + 1] = {
                    order = binding.order,
                    group = def.group or "Other",
                    row = {
                        action = def.label or splitName(name),
                        keys = alternativesText(def.keyboard, def.mouse),
                        gamepad = alternativesText(def.gamepad),
                    },
                }
            end
        end
    end
    for i, key in ipairs(LEGACY_KEYS) do
        local buttons = {}
        for _, button in ipairs(key.buttons or {}) do buttons[#buttons + 1] = button end
        entries[#entries + 1] = {
            order = 1e9 + i, -- after every binding, in list order
            group = key.group,
            row = { action = key.label, keys = key.text or alternativesText(buttons), gamepad = "" },
        }
    end
    table.sort(entries, function(a, b) return a.order < b.order end)

    local byGroup, titles = {}, {}
    for _, entry in ipairs(entries) do
        if not byGroup[entry.group] then
            byGroup[entry.group] = {}
            titles[#titles + 1] = entry.group
        end
        table.insert(byGroup[entry.group], entry.row)
    end

    -- GROUP_ORDER first, then any other group alphabetically, "Other" last.
    local rank = {}
    for i, title in ipairs(GROUP_ORDER) do rank[title] = i end
    table.sort(titles, function(a, b)
        local ra = rank[a] or (a == "Other" and math.huge or #GROUP_ORDER + 1)
        local rb = rank[b] or (b == "Other" and math.huge or #GROUP_ORDER + 1)
        if ra ~= rb then return ra < rb end
        return a < b
    end)

    local groups = {}
    for _, title in ipairs(titles) do
        groups[#groups + 1] = { title = title, rows = byGroup[title] }
    end
    return groups
end

return ControlsHelp
