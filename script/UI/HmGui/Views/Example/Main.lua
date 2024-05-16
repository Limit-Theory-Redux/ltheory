---@type UIView
local Main = UICore.View {
    name = "Main"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local someState = 0
local isVisible = true
local isVisibleColor = Color(1, .4, .4, 1)
local isInvisibleColor = Color(.4, 1, .4, 1)
local checkBoxActive = false

function Main:onInput()
    if InputInstance:isPressed(Button.KeyboardF) then
        isVisible = not isVisible
    end
end

function Main:onUpdate(dt)
    someState = someState + dt
end

function Main:onViewOpen(isPageOpen) end
function Main:onViewClose(isPageClose) end

local function getSomeState()
    return math.floor(someState)
end

local function getComponentVisible()
    return isVisible
end

local function getTextColor()
    if isVisible then
        return isVisibleColor
    else
        return isInvisibleColor
    end
end

local function getText()
    if isVisible then
        return "Press F to hide container"
    else
        return "Press F to show container"
    end
end

local function switchToTitleScreen()
    UIRouter:getCurrentPage():setView("Other_View")
end

---@type UIComponentContainer
local testContainerInner = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    layoutType = GuiLayoutType.Vertical,
    contents = {
        UIComponent.Button { title = "Switch to other view", width = 160, callback = switchToTitleScreen },
        UIComponent.RawInput { fn = function()
            Gui:beginVerticalContainer()
            Gui:setVerticalAlignment(AlignVertical.Stretch)

            Gui:setFixedWidth(160)
            local checkBox = Gui:checkbox("Checkbox1", checkBoxActive)
            checkBoxActive = checkBox

            Gui:setFixedWidth(160)
            if Gui:button("Hide Container") then
                isVisible = false
            end

            Gui:setFixedWidth(160)
            if Gui:button("Reset Timer") then
                someState = 0
            end
            Gui:setFixedWidth(160)

            Gui:endContainer()
        end },
        UIComponent.Text { text = getSomeState }
    }
}

---@type UIComponentContainer
local testContainerOuter = UIComponent.Container {
    visible = getComponentVisible,
    padding = { 10, 10 },
    align = { AlignHorizontal.Left, AlignVertical.Center },
    layoutType = GuiLayoutType.Vertical,
    contents = {
        testContainerInner
    }
}

---@type UIComponentContainer
local textContainer = UIComponent.Container {
    padding = { 10, 10 },
    align = { AlignHorizontal.Center, AlignVertical.Center },
    layoutType = GuiLayoutType.Vertical,
    contents = {
        UIComponent.Text {
            text = getText,
            size = 42,
            color = getTextColor
        }
    }
}

Main:addContent(testContainerOuter)
Main:addContent(textContainer)

return Main
