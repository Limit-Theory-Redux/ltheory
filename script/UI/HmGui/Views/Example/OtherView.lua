---@type UIView
local OtherView = UICore.View {
    name = "Other_View"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

function OtherView:onInput() end
function OtherView:onUpdate(dt) end
function OtherView:onCloseView() end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

---@type UIComponentContainer
local testContainer = UIComponent.Container {
    padding = { 10, 10 },
    align = { AlignHorizontal.Left, AlignVertical.Center },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.Text { font = "Exo2Bold", size = 14, color = Color(1, 1, 1, 1),
            text = "Hello World!" },
        UIComponent.Spacer { size = 16 },
        UIComponent.Text { font = "Exo2Bold", size = 12, color = Color(1, .4, .4, 1), text = "Some red text" },
        UIComponent.Button { title = "Switch to main view", callback = switchToMainScreen }
    }
}

OtherView:addContent(testContainer)

return OtherView
