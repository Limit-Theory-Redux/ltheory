---@type UIView
local Main = UICore.View {
    name = "Main",
    contents = {}
}

local function someCallback()
    print("what a callback!")
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
        UIComponent.Button { title = "A button", callback = someCallback }
    }
}

Main:addContent(testContainer)

return Main
