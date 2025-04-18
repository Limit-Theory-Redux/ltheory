---@type UIView
local SettingsView = UICore.View {
    name = "Settings"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
local MusicPlayer = require("Legacy.Systems.SFX.MusicPlayer")

function SettingsView:onInput() end
function SettingsView:onUpdate(dt) end
function SettingsView:onViewOpen(isPageOpen) end
function SettingsView:onViewClose(isPageClose) end

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

local settingsCategory = 1
local settingsCategoryNames = { { "GENERAL", "General" },
    { "AUDIO",     "Audio" },
    { "GRAPHICS",  "Graphics" },
    { "INTERFACE", "Interface" },
    { "KEYBINDS",  "Keybinds" } }

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
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

local function switchToGeneralSettings()
    settingsCategory = 1
end

local function switchToAudioSettings()
    settingsCategory = 2
end

local function switchToGraphicsSettings()
    settingsCategory = 3
end

local function switchToInterfaceSettings()
    settingsCategory = 4
end

local function switchToKeybindSettings()
    settingsCategory = 5
end

local function getSettingsCategoryText()
    return settingsCategoryNames[settingsCategory][1]
end

---@return UIComponentContainer
local function settingsGeneral()
    return UIComponent.Container {
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
        padding = { 0, 0 },
        margin = { 80, 0 },
        widthInLayout = getRemainingWidthPercentage,
        layoutType = GuiLayoutType.Vertical,
        contents = {
            UILayout.Grid {
                align = { AlignHorizontal.Stretch, AlignVertical.Center },
                stackDirection = GuiLayoutType.Horizontal,
                contents = {
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.35, -- 35% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                        }
                    },
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.65, -- 65% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                        }
                    }
                }
            }
        }
    }
end

---@return UIComponentContainer
local function settingsAudio()
    return UIComponent.Container {
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
        padding = { 0, 0 },
        margin = { 80, 0 },
        widthInLayout = getRemainingWidthPercentage,
        layoutType = GuiLayoutType.Vertical,
        contents = {
            UILayout.Grid {
                align = { AlignHorizontal.Stretch, AlignVertical.Center },
                stackDirection = GuiLayoutType.Horizontal,
                contents = {
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.35, -- 35% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                            UIComponent.Text {
                                text = "Global Audio",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            },
                            UIComponent.Text {
                                text = "Music Volume",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            }
                        }
                    },
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.65, -- 65% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                            UIComponent.Switch {
                                size = ResponsiveSize(80, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function() return "Turns all game audio off and on." end,
                                currentValue = function() return GameState.audio.soundEnabled end, -- send value back to component
                                callback = function(v)
                                    GameState.audio.soundEnabled = v;
                                    MusicPlayer:setGlobalVolume();
                                end -- get value change from component
                            },
                            UIComponent.Slider {
                                size = ResponsiveSize(300, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function() return "Adjusts the music volume." end,
                                showValueAsPercentage = true,
                                increment = 0.01,
                                minValue = 0,
                                maxValue = 1,
                                currentValue = GameState.audio.musicVolume,
                                callback = function(v) MusicPlayer:setVolume(v) end
                            }
                        }
                    }
                }
            }
        }
    }
end

---@return UIComponentContainer
local function settingsGraphics()
    return UIComponent.Container {
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
        padding = { 0, 0 },
        margin = { 80, 0 },
        widthInLayout = getRemainingWidthPercentage,
        layoutType = GuiLayoutType.Vertical,
        contents = {
            UILayout.Grid {
                align = { AlignHorizontal.Stretch, AlignVertical.Center },
                stackDirection = GuiLayoutType.Horizontal,
                contents = {
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.35, -- 35% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                            UIComponent.Text {
                                text = "Fullscreen",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            },
                            UIComponent.Text {
                                text = "Enable Exclusive Fullscreen",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            },
                            UIComponent.Text {
                                text = "Supersampling",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            }
                        }
                    },
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.65, -- 65% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                            UIComponent.Switch {
                                size = ResponsiveSize(80, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function() return "Switches between fullscreen and windowed modes." end,
                                currentValue = function() return GameState.render.fullscreen end, -- send value back to component
                                callback = function(v)
                                    GameState.render.fullscreen = v;
                                    Window:setFullscreen(GameState.render.fullscreen,
                                        GameState.render.fullscreenExclusive);
                                end -- get value change from component
                            },
                            UIComponent.Switch {
                                size = ResponsiveSize(80, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function() return "Switches between exclusive and borderless fullscreen modes." end,
                                currentValue = function() return GameState.render.fullscreenExclusive end, -- send value back to component
                                callback = function(v)
                                    GameState.render.fullscreenExclusive = v;
                                    Window:setFullscreen(GameState.render.fullscreen,
                                        GameState.render.fullscreenExclusive);
                                end -- get value change from component
                            },
                            UIComponent.Slider {
                                size = ResponsiveSize(300, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function()
                                    return
                                    "EXPERIMENTAL: Switches supersampling between Off, 2x, and 4x.\nNOTE: 2x and 4x are completely unusable at this time."
                                end,
                                increment = 1,
                                minValue = 1,
                                maxValue = 3,
                                currentValue = Settings.get("render.superSample"),
                                callback = function(v) Settings.set("render.superSample", v) end
                            }
                        }
                    }
                }
            }
        }
    }
end

---@return UIComponentContainer
local function settingsInterface()
    return UIComponent.Container {
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
        padding = { 0, 0 },
        margin = { 80, 0 },
        widthInLayout = getRemainingWidthPercentage,
        layoutType = GuiLayoutType.Vertical,
        contents = {
            UILayout.Grid {
                align = { AlignHorizontal.Stretch, AlignVertical.Center },
                stackDirection = GuiLayoutType.Horizontal,
                contents = {
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.35, -- 35% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                            UIComponent.Text {
                                text = "Cursor Style",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            },
                            UIComponent.Text {
                                text = "HUD Display Style",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            },
                            UIComponent.Text {
                                text = "UI Scale",
                                size = 24,
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Left, AlignVertical.Center }
                            }
                        }
                    },
                    UILayout.Grid {
                        align = { AlignHorizontal.Center, AlignVertical.Center },
                        widthInLayout = 0.65, -- 65% of the parent grid width
                        layoutType = GuiLayoutType.Vertical,
                        contents = {
                            UIComponent.Slider {
                                size = ResponsiveSize(300, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function()
                                    return
                                    "Switch between game pointers.\nNOTE: This function is not yet fully implemented."
                                end,
                                increment = 1,
                                minValue = 1,
                                maxValue = Enums.CursorStyleCount,
                                currentValue = GameState.ui.cursorStyle,
                                callback = function(v)
                                    GameState.ui.cursorStyle = v;
                                    -- TODO: find a way to call setIcon(icon, style)
                                end
                            },
                            UIComponent.Slider {
                                size = ResponsiveSize(300, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function()
                                    return
                                    "Switches between HUD display styles\n(HUD Off, Cursor Only, Wide, Medium, Narrow)."
                                end,
                                increment = 1,
                                minValue = 1,
                                maxValue = Enums.HudStyleCount,
                                currentValue = GameState.ui.hudStyle,
                                callback = function(v) GameState.ui.hudStyle = v end
                            },
                            UIComponent.Slider {
                                size = ResponsiveSize(300, 20),
                                margin = { 0, 14 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function()
                                    return
                                    "Scales all responsive UI components"
                                end,
                                increment = 0.05,
                                minValue = 0.4,
                                maxValue = 1.0,
                                currentValue = GameState.ui.scaleFactor,
                                callback = function(v) GameState.ui.scaleFactor = v end
                            }
                        }
                    }
                }
            }
        }
    }
end

---@return UIComponentContainer
local function settingsKeybinds()
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

---@return UIComponentContainer
local function settingsSwap()
    if settingsCategory == 2 then
        return settingsAudio()
    elseif settingsCategory == 3 then
        return settingsGraphics()
    elseif settingsCategory == 4 then
        return settingsInterface()
    elseif settingsCategory == 5 then
        return settingsKeybinds()
    else
        return settingsGeneral()
    end
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
                        UIComponent.Button_MainMenu {
                            title = settingsCategoryNames[1][2],
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToGeneralSettings,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = settingsCategoryNames[2][2],
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToAudioSettings,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = settingsCategoryNames[3][2],
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToGraphicsSettings,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = settingsCategoryNames[4][2],
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToInterfaceSettings,
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                        },
                        UIComponent.Button_MainMenu {
                            title = settingsCategoryNames[5][2],
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToKeybindSettings,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Back",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
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
