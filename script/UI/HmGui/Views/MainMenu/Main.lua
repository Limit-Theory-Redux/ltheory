---@type UIView
local Main = UICore.View {
    name = "Main"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local someState = 0
local isVisible = true

function Main:onInput()
    if InputInstance:isPressed(Button.KeyboardF) then
        isVisible = not isVisible
    end
end

function Main:onUpdate(dt)
    someState = someState + dt
end

local function getSomeState()
    return someState
end

local function getComponentVisible()
    return isVisible
end

local function switchToTitleScreen()
    UIRouter:getCurrentPage():setView("Title_Screen")
end

---@type UIComponentContainer
local testContainerInner = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.Button { title = "Switch to title screen view", callback = switchToTitleScreen },
        UIComponent.RawInput { fn = function()
            Gui:beginVerticalContainer()
            Gui:setVerticalAlignment(AlignVertical.Stretch)
            Gui:checkbox("Checkbox1", false)
            Gui:checkbox("Checkbox2", true)
            Gui:checkbox("Checkbox3", false)
            Gui:endContainer()
        end },
        UIComponent.Text { text = getSomeState }
    }
}

---@type UIComponentContainer
local testContainerOuter = UIComponent.Container {
    visible = getComponentVisible,
    padding = { 10, 10 },
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        testContainerInner
    }
}

---@type UIComponentContainer
local textContainer = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.Text {
            text = "Press F to toggle container visibility",
            size = 42,
            color = Color(1.0, .4, .4, 1.0)
        }
    }
}

Main:addContent(testContainerOuter)
Main:addContent(textContainer)

return Main
