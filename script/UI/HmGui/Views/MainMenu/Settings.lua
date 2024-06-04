---@type UIView
local SettingsView = UICore.View {
    name = "Settings"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
local MusicPlayer = require("Systems.SFX.MusicPlayer")

function SettingsView:onInput() end
function SettingsView:onUpdate(dt) end
function SettingsView:onViewOpen(isPageOpen) end
function SettingsView:onViewClose(isPageClose) end

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

local someSliderValue = 50

local function getButtonWidth()
    return GameState.render.resX / 1600 * 200
end

local function getButtonHeight()
    return GameState.render.resY / 900 * 40
end

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
                            width = getButtonWidth,
                            height = getButtonHeight,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Interface",
                            width = getButtonWidth,
                            height = getButtonHeight,
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                        },
                        UIComponent.Button_MainMenu {
                            title = "Graphics",
                            width = getButtonWidth,
                            height = getButtonHeight,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Keybinding",
                            width = getButtonWidth,
                            height = getButtonHeight,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Back",
                            width = getButtonWidth,
                            height = getButtonHeight,
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
                    width = 200,
                    height = 30,
                    currentValue = someSliderValue,
                    sound = Config.audio.sounds.click,
                    callback = setSliderValue
                },
                UIComponent.Slider {
                    title = "Incremented Slider",
                    width = 200,
                    height = 30,
                    increment = 1, -- step of 1 (0.01 also works == 1%)
                    currentValue = someSliderValue,
                    sound = Config.audio.sounds.click,
                    callback = function(value) print(value) end
                }
            }
        }
    }
}

SettingsView:addContent(settingsGrid)

return SettingsView
