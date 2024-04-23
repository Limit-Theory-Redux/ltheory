---@type UIView
local TitleScreen = UICore.View {
    name = "Title_Screen"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

function TitleScreen:onInput() end
function TitleScreen:onUpdate(dt) end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

---@type UIComponentContainer
local testContainer = UIComponent.Container {
    padding = { 10, 10 },
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.Text { font = "Exo2Bold", size = 14, color = Color(1, 1, 1, 1),
            text = "Hello World!" },
        UIComponent.Spacer { size = 16 },
        UIComponent.Text { font = "Exo2Bold", size = 12, color = Color(1, .4, .4, 1), text = "Some red text" },
        UIComponent.Button { title = "Switch to main view", callback = switchToMainScreen }
    }
}

TitleScreen:addContent(testContainer)

return TitleScreen
