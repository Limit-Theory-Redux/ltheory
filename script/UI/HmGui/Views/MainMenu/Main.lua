---@type UIView
local MainView = UICore.View {
    name = "Main"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
local MusicPlayer = require('Systems.SFX.MusicPlayer')

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

function MainView:onInput() end
function MainView:onUpdate(dt) end

function MainView:onViewOpen(isPageOpen)
    if isPageOpen then
        MusicPlayer:QueueTrack(GameState.audio.menuTheme, true)
    end
end

function MainView:onViewClose(isPageClose)
    if isPageClose then
        MusicPlayer:ClearQueue()
    end
end

local function getButtonWidth()
    return GameState.render.resX / 1600 * 200
end

local function getButtonHeight()
    return GameState.render.resY / 900 * 40
end

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 175 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local function switchToPlayView()
    UIRouter:getCurrentPage():setView("Play")
end

local function switchToSettingsView()
    UIRouter:getCurrentPage():setView("Settings")
end

local function switchToBackgroundView()
    UIRouter:getCurrentPage():setView("Background")
end

local function switchToCreditsView()
    UIRouter:getCurrentPage():setView("Credits")
end

local menuGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 50, 50 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    showGrid = false,
    contents = {
        UIComponent.Container {
            align = { AlignHorizontal.Default, AlignVertical.Center },
            padding = { 50, 10 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Button_MainMenu {
                    title = "Play",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    callback = switchToPlayView,
                    align = { AlignHorizontal.Center, AlignVertical.Center }
                },
                UIComponent.Button_MainMenu {
                    title = "Settings",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = switchToSettingsView
                },
                UIComponent.Button_MainMenu {
                    title = "Credits",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    callback = switchToCreditsView,
                    align = { AlignHorizontal.Center, AlignVertical.Center }
                },
                UIComponent.Button_MainMenu {
                    title = "Exit",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    callback = function() EngineInstance:exit() end,
                    align = { AlignHorizontal.Center, AlignVertical.Center }
                }
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 50, 50 },
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.RawInput { fn = function()
                    Gui:image(logo)
                    Gui:setPercentSize(70, 20)
                    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
                end }
            }
        }
    }
}

local backgroundButton = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Right, AlignVertical.Bottom },
    padding = { 10, 10 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.Button_MainMenu {
            title = "Background Mode",
            width = getButtonWidth,
            height = getButtonHeight,
            callback = switchToBackgroundView,
            align = { AlignHorizontal.Default, AlignVertical.Default }
        }
    }
}

MainView:addContent(menuGrid)
MainView:addContent(backgroundButton)

return MainView
