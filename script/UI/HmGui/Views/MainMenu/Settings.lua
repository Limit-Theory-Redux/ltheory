---@type UIView
local SettingsView = UICore.View {
    name = "Settings",
    noDarken = true -- the sidebar and the panel carry their own backgrounds
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
local MusicPlayer = require("Shared.Audio.MusicPlayer")
local InitFiles = require("Legacy.Systems.Files.InitFiles")
local UserSettings = require("Shared.UserSettings")
local ControlsHelp = require("Input.ControlsHelp")
local MenuStyle = require("UI.HmGui.Styles.MenuStyle")

-- The tabs follow the start menu mockups: Interface, Graphics, Audio, Keybinding,
-- in the menu style (UI/HmGui/Styles/MenuStyle.lua). Only settings the game acts
-- on are shown. Positions are design pixels inside the panel.

-- Supersampling choices (Off, 2x, 4x) as `GameState.render.superSampleRate`
-- values (the renderer reads it every frame, user.ini keeps it).
local superSampleRates = { 1, 2, 4 }
local superSampleNames = { "Off", "2x", "4x" }

local function getSuperSampleStep()
    local rate = GameState.render.superSampleRate
    for step, stepRate in ipairs(superSampleRates) do
        if stepRate == rate then return step end
    end
    return 1
end

-- Settings apply as they change. `savedSettings` is how they were when the
-- screen opened or Save was last pressed: Back offers to save the changes or to
-- undo them (back to `savedSettings`) when they differ.
local savedSettings
local confirmBack = false

function SettingsView:onInput() end
function SettingsView:onUpdate(dt) end

function SettingsView:onViewOpen(isPageOpen)
    savedSettings = UserSettings.snapshot()
    confirmBack = false
end

--- Leaving the Settings screen saves the settings to user.ini, so they are kept
--- however the game is closed afterwards (not only through the Exit button).
function SettingsView:onViewClose(isPageClose)
    confirmBack = false
    InitFiles:writeUserInits()
end

local settingsCategory = 1
local settingsCategoryNames = { "Interface", "Graphics", "Audio", "Keybinding" }

--- Show one of the settings tabs (1 = Interface … `getCategoryCount()`), as its
--- nav item does. Used by the menu smoke test (`States/App/Tests/MenuSmokeTest.lua`).
---@param index integer
function SettingsView:setCategory(index)
    settingsCategory = index
end

---@return integer count the number of settings tabs
function SettingsView:getCategoryCount()
    return #settingsCategoryNames
end

---@param index integer
---@return string name the tab's nav title
function SettingsView:getCategoryName(index)
    return settingsCategoryNames[index]
end

-- ROWS

---A checkbox with its label right-aligned to the left of it.
---@param labelRight number the label's right edge
---@param y number
local function checkRow(labelRight, y, label, toolTip, get, set)
    return {
        UIComponent.MenuLabel { x = labelRight, y = y, h = 40, align = "right", text = label },
        UIComponent.MenuCheckbox { x = labelRight + 30, y = y, toolTip = toolTip, get = get, set = set },
    }
end

---A dropdown with its label right-aligned to the left of it.
local function dropdownRow(labelRight, y, width, label, dropdownArgs)
    dropdownArgs.x = labelRight + 18
    dropdownArgs.y = y
    dropdownArgs.w = width
    return {
        UIComponent.MenuLabel { x = labelRight, y = y, h = 40, align = "right", text = label },
        UIComponent.MenuDropdown(dropdownArgs),
    }
end

---One list of widgets from rows (each a widget or a list of widgets).
local function flatten(rows)
    local list = {}
    for _, row in ipairs(rows) do
        if row.render then
            list[#list + 1] = row
        else
            for _, w in ipairs(row) do list[#list + 1] = w end
        end
    end
    return list
end

local function resetButton(group)
    return UIComponent.MenuButton {
        x = 56, y = 950, w = 272, h = 42, size = 25,
        title = "Reset to default",
        toolTip = "Puts the settings on this tab back to their defaults.",
        callback = function() UserSettings.reset(group) end
    }
end

local function percent(v)
    return string.format("%d%%", math.floor(v * 100 + 0.5))
end

---The names of a list of tables with a `name`.
local function names(list)
    local result = {}
    for index, entry in ipairs(list) do result[index] = entry.name end
    return result
end

---A speed effect's checkbox (GameState.render[key], read by SpeedFeelSystem).
local function speedRow(y, label, key, toolTip)
    return checkRow(972, y, label, toolTip,
        function() return GameState.render[key] end,
        function(v) GameState.render[key] = v end)
end

-- TABS

local interfaceTab = flatten {
    checkRow(378, 104, "Invert Vertical Look",
        "Inverts up and down for steering the ship and for the free camera.",
        function() return GameState.input.invertPitch end,
        function(v) GameState.input.invertPitch = v end),
    checkRow(378, 162, "Draw HUD",
        "Shows the HUD when a game starts.\nV hides and shows it during a game.",
        function() return GameState.ui.drawHud end,
        function(v) GameState.ui.drawHud = v end),
    checkRow(378, 220, "Show HUD Indicator Names",
        "Shows the names next to planets, stations and asteroids.",
        function() return GameState.ui.showIndicatorNames end,
        function(v) GameState.ui.showIndicatorNames = v end),
    UIComponent.MenuSlider {
        x = 790, y = 92, label = "Mouse Sensitivity",
        toolTip = "How far the ship and the free and map cameras turn when you move the mouse.\n50% is the default.",
        min = 0.05, max = 1, increment = 0.05, format = percent,
        get = function() return GameState.input.mouseSensitivity end,
        set = function(v) GameState.input.mouseSensitivity = v end
    },
    UIComponent.MenuSlider {
        x = 790, y = 192, label = "UI Scale",
        toolTip = "The size of the in-game HUD. At 100% it fits the window's height.\nThe menus fit themselves to the window.",
        min = 0.5, max = 1.5, increment = 0.05, format = percent,
        get = function() return GameState.ui.hudScale end,
        set = function(v) GameState.ui.hudScale = v end
    },
    dropdownRow(886, 380, 260, "HUD Style", {
        toolTip = "Switches between HUD display styles.\nNOTE: not used by the current HUD yet.",
        selections = Enums.HudStyleNames,
        getIndex = function() return GameState.ui.hudStyle end,
        setIndex = function(i) GameState.ui.hudStyle = i end
    }),
    dropdownRow(886, 442, 260, "Cursor Style", {
        toolTip = "Switch between game pointers.\nNOTE: This function is not yet fully implemented.",
        selections = Enums.CursorStyleNames,
        getIndex = function() return GameState.ui.cursorStyle end,
        setIndex = function(i) GameState.ui.cursorStyle = i end
    }),
    resetButton("interface"),
}

local graphicsTab = flatten {
    dropdownRow(272, 98, 320, "Display Mode", {
        toolTip = "Windowed, borderless fullscreen (keeps the desktop's refresh rate),\nor exclusive fullscreen.",
        selections = names(UserSettings.displayModes),
        getIndex = UserSettings.getDisplayMode,
        setIndex = UserSettings.setDisplayMode
    }),
    dropdownRow(272, 156, 320, "Resolution", {
        toolTip = "The window size in windowed mode.\nFullscreen always uses the screen's resolution.",
        selections = function()
            local list = {}
            for index, size in ipairs(UserSettings.resolutions()) do
                list[index] = UserSettings.resolutionName(size)
            end
            return list
        end,
        getIndex = function()
            for index, size in ipairs(UserSettings.resolutions()) do
                if size[1] == GameState.render.resX and size[2] == GameState.render.resY then
                    return index
                end
            end
            return nil
        end,
        setIndex = function(index)
            local size = UserSettings.resolutions()[index]
            if size then UserSettings.setResolution(size[1], size[2]) end
        end
    }),
    UIComponent.MenuSlider {
        x = 160, y = 224, label = "Brightness",
        toolTip = "Brightens or darkens the picture (the exposure).\n50% is the default.",
        min = 0, max = 1, increment = 0.05, format = percent,
        get = function() return GameState.render.brightness end,
        set = function(v)
            GameState.render.brightness = v
            UserSettings.applyBrightness()
        end
    },
    UIComponent.MenuSlider {
        x = 160, y = 330, label = "Vertical Field of View",
        toolTip = "How much of the scene you see from top to bottom, in degrees.",
        min = 50, max = 100, increment = 1,
        format = function(v) return string.format("%d°", math.floor(v + 0.5)) end,
        get = function() return GameState.render.fov end,
        set = function(v)
            GameState.render.fov = v
            UserSettings.applyFov()
        end
    },
    checkRow(272, 456, "Vertical Sync",
        "Waits for the screen's refresh before showing a frame:\nno tearing, but the frame rate is capped to the refresh rate.",
        function() return GameState.render.vsync end,
        function(v)
            GameState.render.vsync = v
            UserSettings.applyVsync()
        end),
    checkRow(272, 518, "Smoothing",
        "Smooths jagged edges (FXAA). Cheap; supersampling smooths them better.",
        function() return GameState.render.smoothing end,
        function(v)
            GameState.render.smoothing = v
            UserSettings.applyPostFx()
        end),
    checkRow(272, 580, "Bloom",
        "The glow around bright light sources.",
        function() return GameState.render.bloom end,
        function(v)
            GameState.render.bloom = v
            UserSettings.applyPostFx()
        end),
    dropdownRow(272, 642, 320, "Effect Quality", {
        toolTip = "How finely explosions are drawn.\nLower is much cheaper when an explosion fills the screen.",
        selections = names(UserSettings.effectQualities),
        getIndex = function() return GameState.render.effectQuality end,
        setIndex = function(index) GameState.render.effectQuality = index end
    }),
    dropdownRow(972, 98, 260, "Texture Quality", {
        toolTip = "Size of the planet and moon surface textures.\nShows from the next new game.",
        selections = names(UserSettings.textureQualities),
        getIndex = function() return GameState.render.textureQuality end,
        setIndex = function(index)
            GameState.render.textureQuality = index
            UserSettings.applyTextureQuality()
        end
    }),
    dropdownRow(972, 160, 260, "Supersampling", {
        toolTip =
        "Renders the scene at 2x or 4x the screen resolution, for smoother edges.\nCosts frame rate. 4x only fits screens up to 2048 pixels wide; larger screens use 2x.",
        selections = superSampleNames,
        getIndex = getSuperSampleStep,
        setIndex = function(step) GameState.render.superSampleRate = superSampleRates[step] end
    }),
    -- Speed effects (SpeedFeelSystem reads these every frame)
    UIComponent.MenuLabel { x = 972, y = 236, h = 40, align = "right", size = 25, text = "SPEED EFFECTS" },
    speedRow(290, "Field of View", "speedFov",
        "The view widens as you speed up, with a short push when boost\nstarts and when the travel drive engages."),
    speedRow(344, "Camera Shake", "speedShake",
        "A rumble while boosting, charging the travel drive and in travel mode,\nand a jolt when boost starts and when the travel drive engages or drops."),
    speedRow(398, "Motion Blur", "speedBlur",
        "Blur streaking out from your ship when boosting and in travel mode."),
    speedRow(452, "Chromatic Aberration", "speedAberration",
        "A red and blue colour split towards the edges when boosting and in travel mode."),
    speedRow(506, "Speed Vignette", "speedVignette",
        "Darker, round screen edges when boosting and in travel mode."),
    speedRow(560, "Travel Drive Effect", "speedTravelDrive",
        "The travel drive's ring in front of the ship, its streaks,\nand the blink when you leave travel mode."),
    speedRow(614, "Space Dust", "speedDust",
        "Specks of dust streaking past, so you can see how fast you fly.\nIn travel mode the drive pushes them aside."),
    UIComponent.MenuSlider {
        x = 700, y = 676, label = "Speed Effects Strength",
        toolTip = "Scales all the speed effects above together.\n100% is the default.",
        min = 0, max = 2, increment = 0.05, format = percent,
        get = function() return GameState.render.speedIntensity end,
        set = function(v) GameState.render.speedIntensity = v end
    },
    resetButton("graphics"),
}

local audioTab = flatten {
    UIComponent.MenuSlider {
        x = 110, y = 112, label = "Master Volume",
        toolTip = "Scales the music and the sound effects.",
        min = 0, max = 1, increment = 0.01, format = percent,
        get = function() return GameState.audio.masterVolume end,
        set = function(v)
            GameState.audio.masterVolume = v
            MusicPlayer:setGlobalVolume()
        end
    },
    UIComponent.MenuSlider {
        x = 110, y = 212, label = "Music Volume",
        toolTip = "Adjusts the music volume.",
        min = 0, max = 1, increment = 0.01, format = percent,
        get = function() return GameState.audio.musicVolume end,
        set = function(v) MusicPlayer:setVolume(v) end
    },
    UIComponent.MenuSlider {
        x = 110, y = 312, label = "SFX Volume",
        toolTip = "Adjusts the volume of the sound effects.",
        min = 0, max = 1, increment = 0.01, format = percent,
        get = function() return GameState.audio.effectsVolume end,
        set = function(v) GameState.audio.effectsVolume = v end
    },
    checkRow(272, 446, "Sound Enabled",
        "Turns all game audio off and on.",
        function() return GameState.audio.soundEnabled end,
        function(v)
            GameState.audio.soundEnabled = v
            MusicPlayer:setGlobalVolume()
        end),
    resetButton("audio"),
}

-- KEYBINDING: the bindings as a table, the same list as the in-game Controls
-- screen (Input/ControlsHelp.lua). Read-only for now: rebinding needs bindings
-- that can be changed and saved, which they can't yet.

-- Columns: x inside the table and width, design pixels. The table scrolls; the
-- scrollbar sits in the space right of the last column.
local KEY_COLUMNS = { { x = 0, w = 318 }, { x = 320, w = 396 }, { x = 718, w = 372 } }
local KEY_TABLE_W, KEY_TABLE_H = 1110, 760
local KEY_ROW_H, KEY_GROUP_H = 24, 32

local function keybindingTableHeight()
    local height = 0
    for _, group in ipairs(ControlsHelp.groups()) do
        height = height + KEY_GROUP_H + #group.rows * KEY_ROW_H
    end
    return height
end

---All rows, placed in a container as tall as the table.
local function keybindingRows(ctx)
    local C = MenuStyle.color
    local font = MenuStyle.fontOf("regular", 18)
    Gui:beginStackContainer()
    Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
    Gui:setFixedSize(KEY_COLUMNS[3].x + KEY_COLUMNS[3].w, keybindingTableHeight())
    local y = 0
    for _, group in ipairs(ControlsHelp.groups()) do
        MenuStyle.beginAt(ctx, 0, y, KEY_COLUMNS[3].x + KEY_COLUMNS[3].w, KEY_GROUP_H)
        Gui:text(group.title, MenuStyle.fontOf("medium", 20), C.heading)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Bottom)
        Gui:setMarginEx(2, 0, 0, 3)
        Gui:endContainer()
        y = y + KEY_GROUP_H
        for _, row in ipairs(group.rows) do
            local cells = { row.action, row.keys, row.gamepad }
            for column, cell in ipairs(cells) do
                MenuStyle.beginAt(ctx, KEY_COLUMNS[column].x, y, KEY_COLUMNS[column].w, KEY_ROW_H - 1)
                if column == 1 then
                    Gui:rect()
                    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Bottom)
                    Gui:setFixedHeight(MenuStyle.line(1, ctx.s))
                    Gui:setBackgroundColor(C.cellRule)
                else
                    Gui:setBackgroundColor(C.cell)
                end
                Gui:text(cell or "", font, column == 1 and C.label or C.text)
                Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
                Gui:setMarginEx(column == 1 and 2 or 8, 0, 0, 0)
                Gui:setMaxWidth(KEY_COLUMNS[column].w - 12) -- a longer text ends with "…"
                Gui:endContainer()
            end
            y = y + KEY_ROW_H
        end
    end
    Gui:endContainer()
end

-- Made once, so it keeps its scroll position.
local keyScale = 1 -- the menu's UI scale, for the table's rule lines
local keybindingScroll = UIComponent.ScrollArea {
    width = KEY_TABLE_W,
    height = KEY_TABLE_H,
    scrollDirection = Enums.UI.ScrollDirection.Vertical,
    scrollbarFading = false,
    scrollbarSize = 8,
    scrollbarBackgroundColor = MenuStyle.color.track,
    scrollbarKnobColor = MenuStyle.color.fill,
    contents = {
        { state = {}, render = function() keybindingRows({ s = keyScale, ox = 0, oy = 0 }) end }
    }
}

local keybindingTab = {
    UIComponent.MenuLabel { x = 101, y = 80, w = 318, size = 34, align = "center", text = "FUNCTION", color = MenuStyle.color.text },
    UIComponent.MenuLabel { x = 421, y = 80, w = 396, size = 34, align = "center", text = "KEYBOARD & MOUSE", color = MenuStyle.color.text },
    UIComponent.MenuLabel { x = 819, y = 80, w = 372, size = 34, align = "center", text = "GAMEPAD", color = MenuStyle.color.text },
    UIComponent.MenuRaw {
        x = 101, y = 150, w = KEY_TABLE_W, h = KEY_TABLE_H,
        fn = function(ctx)
            keyScale = ctx.s
            keybindingScroll:render()
        end
    },
    UIComponent.MenuLabel {
        x = 101, y = 930, size = 22, font = "regular", color = MenuStyle.color.dim,
        text = "Changing the bindings isn't possible yet."
    },
}

local tabs = { interfaceTab, graphicsTab, audioTab, keybindingTab }

-- SCREEN

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local function save()
    InitFiles:writeUserInits()
    savedSettings = UserSettings.snapshot()
end

---Back: straight to the main menu, or first ask about unsaved changes.
local function back()
    if savedSettings and UserSettings.differsFrom(savedSettings) then
        confirmBack = true
    else
        switchToMainScreen()
    end
end

-- The "unsaved changes" modal, centred.
local CONFIRM_X, CONFIRM_Y, CONFIRM_W, CONFIRM_H = 700, 400, 600, 300

local confirmModal = UIComponent.MenuOverlay {
    visible = function() return confirmBack end,
    contents = {
        UIComponent.MenuPanel {
            x = CONFIRM_X, y = CONFIRM_Y, w = CONFIRM_W, h = CONFIRM_H, band = 90,
            contents = {
                UIComponent.MenuClose { x = CONFIRM_W - 48, y = 6, callback = function() confirmBack = false end },
                UIComponent.MenuLabel { x = 0, y = 22, w = CONFIRM_W, h = 48, size = 34, align = "center", text = "UNSAVED CHANGES", color = MenuStyle.color.text },
                UIComponent.MenuLabel { x = 0, y = 118, w = CONFIRM_W, h = 34, size = 25, align = "center", text = "Some settings changed since you last saved." },
                UIComponent.MenuLabel {
                    x = 0, y = 152, w = CONFIRM_W, h = 30, size = 20, font = "regular", align = "center", color = MenuStyle.color.dim,
                    text = "Save them, or undo them and go back."
                },
                UIComponent.MenuButton {
                    x = 70, y = 214, w = 210, h = 46, size = 24, title = "Save", style = "go",
                    callback = function()
                        save()
                        switchToMainScreen()
                    end
                },
                UIComponent.MenuButton {
                    x = 320, y = 214, w = 210, h = 46, size = 24, title = "Undo changes",
                    toolTip = "Puts every setting back to how it was\nwhen you opened Settings or last saved.",
                    callback = function()
                        UserSettings.restore(savedSettings)
                        switchToMainScreen()
                    end
                },
            }
        },
    }
}

local navItems = {}
for index, name in ipairs(settingsCategoryNames) do
    navItems[index] = { title = name, callback = function() settingsCategory = index end }
end

SettingsView:addContent(UIComponent.MenuScreen {
    contents = {
        UIComponent.MenuSidebar {
            contents = {
                UIComponent.MenuLabel { x = 0, y = 140, w = MenuStyle.SIDEBAR_WIDTH, h = 70, size = 54, align = "center", text = "SETTINGS", color = MenuStyle.color.text },
                UIComponent.MenuNav { y = 288, items = navItems, align = "left", itemHeight = 55, size = 28, selected = function() return settingsCategory end },
                UIComponent.MenuButton { x = 30, y = 1006, w = 112, h = 36, title = "Back", callback = back },
                UIComponent.MenuButton {
                    x = 191, y = 1006, w = 112, h = 36, title = "Save",
                    toolTip = "Saves the settings.",
                    callback = save
                },
            }
        },
        UIComponent.MenuPanel {
            x = 585, y = 52, w = 1355, h = 1018,
            contents = function() return tabs[settingsCategory] or tabs[1] end
        },
        confirmModal,
    }
})

return SettingsView
