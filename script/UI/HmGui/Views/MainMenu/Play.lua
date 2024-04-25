---@type UIView
local PlayView = UICore.View {
    name = "Play"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

function PlayView:onInput() end
function PlayView:onUpdate(dt) end
function PlayView:onCloseView() end

local function getButtonWidth()
    return GameState.render.resX / 1600 * 200
end

local function getButtonHeight()
    return GameState.render.resY / 900 * 40
end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local container = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 50, 50 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    contents = {
        UIComponent.Container {
            align = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 50, 10 },
            margin = { 0, 0 },
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Button_MainMenu {
                    title = "New Game",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                },
                UIComponent.Button_MainMenu {
                    title = "Load Game",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                },
                UIComponent.Button_MainMenu {
                    title = "Back",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = switchToMainScreen
                },
            }
        },
    }
}

PlayView:addContent(container)

return PlayView
