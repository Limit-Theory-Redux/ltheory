---@type UIView
local SettingsView = UICore.View {
    name = "Settings"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
local MusicPlayer = require("Shared.Audio.MusicPlayer")
local InitFiles = require("Legacy.Systems.Files.InitFiles")
local UserSettings = require("Shared.UserSettings")

-- The tabs follow the start menu mockups: Interface, Graphics, Audio, Keybinding.
-- Each tab is rebuilt every frame from GameState (so "Reset to Default" shows at
-- once); only the dropdowns are made once, since a rebuilt dropdown closes itself.

-- Supersampling slider steps (Off, 2x, 4x) as `GameState.render.superSampleRate`
-- values (the renderer reads it every frame, user.ini keeps it).
local superSampleRates = { 1, 2, 4 }

local function getSuperSampleStep()
    local rate = GameState.render.superSampleRate
    for step, stepRate in ipairs(superSampleRates) do
        if stepRate == rate then return step end
    end
    return 1
end

function SettingsView:onInput() end
function SettingsView:onUpdate(dt) end
function SettingsView:onViewOpen(isPageOpen) end

--- Leaving the Settings screen saves the settings to user.ini, so they are kept
--- however the game is closed afterwards (not only through the Exit button).
function SettingsView:onViewClose(isPageClose)
    InitFiles:writeUserInits()
end

local settingsCategory = 1
local settingsCategoryNames = {
    { "INTERFACE",  "Interface" },
    { "GRAPHICS",   "Graphics" },
    { "AUDIO",      "Audio" },
    { "KEYBINDING", "Keybinding" } }

--- Show one of the settings tabs (1 = Interface … `getCategoryCount()`), as its
--- button does. Used by the menu smoke test (`States/App/Tests/MenuSmokeTest.lua`).
---@param index integer
function SettingsView:setCategory(index)
    settingsCategory = index
end

---@return integer count the number of settings tabs
function SettingsView:getCategoryCount()
    return #settingsCategoryNames
end

---@param index integer
---@return string name the tab's button title
function SettingsView:getCategoryName(index)
    return settingsCategoryNames[index][2]
end

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 170 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local function getSettingsCategoryText()
    return settingsCategoryNames[settingsCategory][1]
end

-- CONTROLS

---An on/off switch for a setting.
---@param toolTip string
---@param get fun(): boolean
---@param set fun(value: boolean)
---@return UIComponentSwitch
local function switchControl(toolTip, get, set)
    return UIComponent.Switch {
        size = ResponsiveSize(80, 20),
        margin = { 0, 14 },
        align = { AlignHorizontal.Center, AlignVertical.Center },
        sound = Config.audio.sounds.click,
        toolTip = function() return toolTip end,
        currentValue = get,
        callback = set
    }
end

---@class SettingsSliderArgs
---@field toolTip string
---@field min number
---@field max number
---@field increment number
---@field percent boolean|nil show 0-1 as 0-100%
---@field get fun(): number
---@field set fun(value: number)

---A slider for a setting.
---@param args SettingsSliderArgs
---@return UIComponentSlider
local function sliderControl(args)
    return UIComponent.Slider {
        size = ResponsiveSize(300, 20),
        margin = { 0, 14 },
        align = { AlignHorizontal.Center, AlignVertical.Center },
        sound = Config.audio.sounds.click,
        toolTip = function() return args.toolTip end,
        showValueAsPercentage = args.percent or false,
        increment = args.increment,
        minValue = args.min,
        maxValue = args.max,
        currentValue = args.get,
        callback = args.set
    }
end

---@class SettingsDropdownArgs
---@field toolTip string
---@field selections string[]|fun(): string[] the choices' names (a function for a list that changes)
---@field getIndex fun(): integer the current choice
---@field setIndex fun(index: integer) called with the player's pick

---A dropdown for a setting. It always shows the setting's current choice (the
---Dropdown component on its own starts at "--Select--"). Make it once and keep
---it: the tab around it is rebuilt every frame, which would close a new one.
---@param args SettingsDropdownArgs
---@return UIComponentDropdown
local function dropdownControl(args)
    local dropdown = UIComponent.Dropdown {
        width = 300,
        height = 20,
        margin = { 0, 14 },
        font = { name = "Unageo-Medium", size = 20 },
        align = { AlignHorizontal.Center, AlignVertical.Center },
        sound = Config.audio.sounds.click,
        toolTip = function() return args.toolTip end,
        selections = args.selections,
        callback = args.setIndex
    }
    local listSelector = dropdown.state.listSelector()
    local render = dropdown.render
    dropdown.render = function(self)
        listSelector.state.selectedIndex = args.getIndex()
        render(self)
    end
    return dropdown
end

---A tab: its settings, and a Reset to Default button for `resetGroup`.
---@param rows UILayoutSettingRow[]
---@param resetGroup "interface"|"graphics"|"audio"|nil
---@return UIComponentContainer
local function settingsTab(rows, resetGroup)
    local contents = {
        UILayout.SettingList {
            rows = rows
        }
    }
    if resetGroup then
        contents[#contents + 1] = UIComponent.Button {
            title = "Reset to Default",
            margin = { 0, 20 },
            size = ResponsiveSize(240, 40),
            font = { name = "Unageo-Medium", size = 20 },
            sound = Config.audio.sounds.click,
            toolTip = function() return "Puts the settings on this tab back to their defaults." end,
            callback = function() UserSettings.reset(resetGroup) end
        }
    end

    return UIComponent.Container {
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
        padding = { 0, 0 },
        margin = { 80, 0 },
        widthInLayout = getRemainingWidthPercentage,
        layoutType = GuiLayoutType.Vertical,
        contents = contents
    }
end

-- DROPDOWNS (made once, see dropdownControl)

local displayModeNames = {}
for index, mode in ipairs(UserSettings.displayModes) do displayModeNames[index] = mode.name end

local displayModeDropdown = dropdownControl {
    toolTip = "Windowed, borderless fullscreen (keeps the desktop's refresh rate),\nor exclusive fullscreen.",
    selections = displayModeNames,
    getIndex = UserSettings.getDisplayMode,
    setIndex = UserSettings.setDisplayMode
}

local resolutionDropdown = dropdownControl {
    toolTip = "The window size in windowed mode.\nFullscreen always uses the screen's resolution.",
    selections = function()
        local names = {}
        for index, size in ipairs(UserSettings.resolutions()) do
            names[index] = UserSettings.resolutionName(size)
        end
        return names
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
}

local textureQualityNames = {}
for index, quality in ipairs(UserSettings.textureQualities) do textureQualityNames[index] = quality.name end

local textureQualityDropdown = dropdownControl {
    toolTip = "Size of the planet and moon surface textures.\nShows from the next new game.",
    selections = textureQualityNames,
    getIndex = function() return GameState.render.textureQuality end,
    setIndex = function(index)
        GameState.render.textureQuality = index
        UserSettings.applyTextureQuality()
    end
}

-- TABS

---@return UIComponentContainer
local function settingsInterface()
    return settingsTab({
        UILayout.SettingRow {
            label = "Invert Vertical Look",
            control = switchControl(
                "Inverts up and down for steering the ship and for the free camera.",
                function() return GameState.input.invertPitch end,
                function(v) GameState.input.invertPitch = v end)
        },
        UILayout.SettingRow {
            label = "Draw HUD",
            control = switchControl(
                "Shows the HUD when a game starts.\nV hides and shows it during a game.",
                function() return GameState.ui.drawHud end,
                function(v) GameState.ui.drawHud = v end)
        },
        UILayout.SettingRow {
            label = "Show HUD Indicator Names",
            control = switchControl(
                "Shows the names next to planets, stations and asteroids.",
                function() return GameState.ui.showIndicatorNames end,
                function(v) GameState.ui.showIndicatorNames = v end)
        },
        UILayout.SettingRow {
            label = "Mouse Sensitivity",
            control = sliderControl {
                toolTip = "How far the ship and the free and map cameras turn when you move the mouse.\n50% is the default.",
                min = 0.05,
                max = 1,
                increment = 0.05,
                percent = true,
                get = function() return GameState.input.mouseSensitivity end,
                set = function(v) GameState.input.mouseSensitivity = v end
            }
        },
        UILayout.SettingRow {
            label = "UI Scale",
            control = sliderControl {
                toolTip = "Scales all responsive UI components",
                min = 0.4,
                max = 1.0,
                increment = 0.05,
                get = function() return GameState.ui.scaleFactor end,
                set = function(v) GameState.ui.scaleFactor = v end
            }
        },
        UILayout.SettingRow {
            label = "Cursor Style",
            control = sliderControl {
                toolTip = "Switch between game pointers.\nNOTE: This function is not yet fully implemented.",
                min = 1,
                max = Enums.CursorStyleCount,
                increment = 1,
                get = function() return GameState.ui.cursorStyle end,
                set = function(v)
                    GameState.ui.cursorStyle = v
                    -- TODO: find a way to call setIcon(icon, style)
                end
            }
        },
        UILayout.SettingRow {
            label = "HUD Display Style",
            control = sliderControl {
                toolTip = "Switches between HUD display styles\n(HUD Off, Cursor Only, Wide, Medium, Narrow).\nNOTE: not used by the current HUD yet.",
                min = 1,
                max = Enums.HudStyleCount,
                increment = 1,
                get = function() return GameState.ui.hudStyle end,
                set = function(v) GameState.ui.hudStyle = v end
            }
        }
    }, "interface")
end

---@return UIComponentContainer
local function settingsGraphics()
    return settingsTab({
        UILayout.SettingRow {
            label = "Display Mode",
            control = displayModeDropdown
        },
        UILayout.SettingRow {
            label = "Resolution",
            control = resolutionDropdown
        },
        UILayout.SettingRow {
            label = "Brightness",
            control = sliderControl {
                toolTip = "Brightens or darkens the picture (the exposure).\n50% is the default.",
                min = 0,
                max = 1,
                increment = 0.05,
                percent = true,
                get = function() return GameState.render.brightness end,
                set = function(v)
                    GameState.render.brightness = v
                    UserSettings.applyBrightness()
                end
            }
        },
        UILayout.SettingRow {
            label = "Vertical Field of View",
            control = sliderControl {
                toolTip = "How much of the scene you see from top to bottom, in degrees.",
                min = 50,
                max = 100,
                increment = 1,
                get = function() return GameState.render.fov end,
                set = function(v)
                    GameState.render.fov = v
                    UserSettings.applyFov()
                end
            }
        },
        UILayout.SettingRow {
            label = "Vertical Sync",
            control = switchControl(
                "Waits for the screen's refresh before showing a frame:\nno tearing, but the frame rate is capped to the refresh rate.",
                function() return GameState.render.vsync end,
                function(v)
                    GameState.render.vsync = v
                    UserSettings.applyVsync()
                end)
        },
        UILayout.SettingRow {
            label = "Smoothing",
            control = switchControl(
                "Smooths jagged edges (FXAA). Cheap; supersampling smooths them better.",
                function() return GameState.render.smoothing end,
                function(v)
                    GameState.render.smoothing = v
                    UserSettings.applyPostFx()
                end)
        },
        UILayout.SettingRow {
            label = "Supersampling",
            control = sliderControl {
                toolTip =
                "Renders the scene at 2x or 4x the screen resolution, for smoother edges.\nCosts frame rate. 4x only fits screens up to 2048 pixels wide; larger screens use 2x.",
                min = 1,
                max = 3,
                increment = 1,
                get = getSuperSampleStep,
                set = function(v) GameState.render.superSampleRate = superSampleRates[v] end
            }
        },
        UILayout.SettingRow {
            label = "Bloom",
            control = switchControl(
                "The glow around bright light sources.",
                function() return GameState.render.bloom end,
                function(v)
                    GameState.render.bloom = v
                    UserSettings.applyPostFx()
                end)
        },
        UILayout.SettingRow {
            label = "Texture Quality",
            control = textureQualityDropdown
        }
    }, "graphics")
end

---@return UIComponentContainer
local function settingsAudio()
    return settingsTab({
        UILayout.SettingRow {
            label = "Global Audio",
            control = switchControl(
                "Turns all game audio off and on.",
                function() return GameState.audio.soundEnabled end,
                function(v)
                    GameState.audio.soundEnabled = v
                    MusicPlayer:setGlobalVolume()
                end)
        },
        UILayout.SettingRow {
            label = "Master Volume",
            control = sliderControl {
                toolTip = "Scales the music and the sound effects.",
                min = 0,
                max = 1,
                increment = 0.01,
                percent = true,
                get = function() return GameState.audio.masterVolume end,
                set = function(v)
                    GameState.audio.masterVolume = v
                    MusicPlayer:setGlobalVolume()
                end
            }
        },
        UILayout.SettingRow {
            label = "Music Volume",
            control = sliderControl {
                toolTip = "Adjusts the music volume.",
                min = 0,
                max = 1,
                increment = 0.01,
                percent = true,
                get = function() return GameState.audio.musicVolume end,
                set = function(v) MusicPlayer:setVolume(v) end
            }
        },
        UILayout.SettingRow {
            label = "SFX Volume",
            control = sliderControl {
                toolTip = "Adjusts the volume of the sound effects.",
                min = 0,
                max = 1,
                increment = 0.01,
                percent = true,
                get = function() return GameState.audio.effectsVolume end,
                set = function(v) GameState.audio.effectsVolume = v end
            }
        }
    }, "audio")
end

---@return UIComponentContainer
local function settingsKeybinding()
    return UIComponent.Container {
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
        padding = { 0, 0 },
        margin = { 0, 0 },
        widthInLayout = getRemainingWidthPercentage,
        layoutType = GuiLayoutType.Vertical,
        contents = {}
    }
end

local settingsTabs = { settingsInterface, settingsGraphics, settingsAudio, settingsKeybinding }

---@return UIComponentContainer
local function settingsSwap()
    return (settingsTabs[settingsCategory] or settingsInterface)()
end

---A tab button in the left column.
---@param index integer
---@return UIComponentButton_MainMenu
local function tabButton(index)
    return UIComponent.Button_MainMenu {
        title = settingsCategoryNames[index][2],
        size = ResponsiveSize(300, 60, true),
        font = { name = "Unageo-Medium", size = 24 },
        callback = function() settingsCategory = index end,
        align = { AlignHorizontal.Center, AlignVertical.Center }
    }
end

local settingsGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 125, 0 },
    margin = { 0, 0 },
    stackDirection = GuiLayoutType.Horizontal,
    contents = {
        UILayout.Grid {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
            color = {
                background = Color(0, 0, 0, 0.3)
            },
            contents = {
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 10 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 2 / 10,
                    contents = {
                        UIComponent.Text {
                            text = getSettingsCategoryText,
                            size = 40,
                            font = "Unageo-Medium"
                        }
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Top },
                    padding = { 0, 50 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 7 / 10,
                    contents = {
                        tabButton(1),
                        tabButton(2),
                        tabButton(3),
                        tabButton(4),
                        UIComponent.Button_MainMenu {
                            title = "Back",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToMainScreen,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            -- Leaving the screen saves too (onViewClose); this saves now.
                            title = "Save",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = function() InitFiles:writeUserInits() end,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        }
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 0 },
                    margin = { 0, 0 },
                    heightInLayout = 1 / 10,
                    layoutType = GuiLayoutType.Vertical,
                    contents = {
                        UIComponent.Text {
                            text = Config.gameVersion,
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                            font = "Exo2",
                            size = 12
                        }
                    }
                }
            }
        },
        settingsSwap
    }
}

SettingsView:addContent(settingsGrid)

return SettingsView
