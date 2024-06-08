---@type UIView
local SettingsView = UICore.View {
    name = "Settings"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
local MusicPlayer = require("Systems.SFX.MusicPlayer")

function SettingsView:onInput() end
function SettingsView:onUpdate(dt) end
function SettingsView:onViewOpen(isPageOpen) end
function SettingsView:onViewClose(isPageClose) end

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

local someSliderValue = 0.5

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 170 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local function setSliderValue(value)
    someSliderValue = value
end

local function setMusicVolume(value)
    MusicPlayer:SetVolume(value)
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
            contents = {
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 10 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 2 / 10,
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
                    contents = {
                        UIComponent.Text {
                            text = "AUDIO",
                            size = 32,
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
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
                    contents = {
                        UIComponent.Button_MainMenu {
                            title = "Audio",
                            size = ResponsiveSize(200, 40),
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Interface",
                            size = ResponsiveSize(200, 40),
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                        },
                        UIComponent.Button_MainMenu {
                            title = "Graphics",
                            size = ResponsiveSize(200, 40),
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Keybinding",
                            size = ResponsiveSize(200, 40),
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Back",
                            size = ResponsiveSize(200, 40),
                            callback = switchToMainScreen,
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
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
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
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
            contents = {
                UIComponent.Slider {
                    title = "Smooth Slider",
                    size = ResponsiveSize(200, 30),
                    currentValue = someSliderValue,
                    sound = Config.audio.sounds.click,
                    callback = setSliderValue
                },
                UIComponent.Slider {
                    title = "Incremented Slider",
                    size = ResponsiveSize(200, 30),
                    increment = 0.01,
                    minValue = 0,
                    maxValue = 1,
                    currentValue = GameState.audio.musicVolume,
                    showValueAsPercentage = true,
                    sound = Config.audio.sounds.click,
                    callback = setMusicVolume
                },
                UIComponent.Switch {
                    title = "switchWithTitle",
                    size = ResponsiveSize(40, 10),
                    margin = { 0, 10 },
                    callback = function(v) print(v) end
                },
                UIComponent.Switch {
                    size = ResponsiveSize(40, 10),
                    margin = { 0, 10 },
                    callback = function(v) print(v) end
                }
            }
        }
    }
}

SettingsView:addContent(settingsGrid)

return SettingsView
