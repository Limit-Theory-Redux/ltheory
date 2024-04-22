---@type UIView
local Main = UICore.View {
    name = "Main"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local function switchToTitleScreen()
    UIRouter:getCurrentPage():setView("Title_Screen")
end

---@type UIComponentContainer
local testContainerInner = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.Button { title = "Switch to title screen view", callback = switchToTitleScreen }
    }
}

---@type UIComponentContainer
local testContainerOuter = UIComponent.Container {
    padding = { 10, 10 },
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        testContainerInner
    }
}

Main:addContent(testContainerOuter)

return Main
