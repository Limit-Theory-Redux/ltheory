---@type UIView
local CreditsView = UICore.View {
    name = "Credits"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

function CreditsView:onInput() end
function CreditsView:onUpdate(dt) end
function CreditsView:onViewOpen(isPageOpen) end
function CreditsView:onViewClose(isPageClose) end

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
                    title = "Back",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = switchToMainScreen
                },
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 5, 5 },
            margin = { 0, 0 },
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.RawInput { fn = function() end }
            }
        }
    }
}

CreditsView:addContent(container)

return CreditsView
