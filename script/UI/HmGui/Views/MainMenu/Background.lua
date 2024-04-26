---@type UIView
local BackgroundView = UICore.View {
    name = "Background"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local lastMousePos = Vec2f(0, 0)
local lastMoved = TimeStamp.Now()
local backButtonVisible = true

local function distance(x1, y1, x2, y2)
    return math.sqrt((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
end

local minDistance = 50 -- pixel

function BackgroundView:onInput()
    local mousePos = InputInstance:mouse():position()

    if distance(mousePos.x, mousePos.y, lastMousePos.x, lastMousePos.y) > minDistance then
        lastMoved = TimeStamp.Now()
        lastMousePos = mousePos
        backButtonVisible = true
        InputInstance:setCursorVisible(true)
    end

    if lastMoved:getElapsed() >= 5 then
        backButtonVisible = false
        InputInstance:setCursorVisible(false)
    end
end

function BackgroundView:onUpdate(dt) end

function BackgroundView:onViewOpen(isPageOpen)
    lastMoved = TimeStamp.Now()
end

function BackgroundView:onViewClose(isPageClose)
    backButtonVisible = true
    InputInstance:setCursorVisible(true) -- reset
end

local function getButtonWidth()
    return GameState.render.resX / 1600 * 200
end

local function getButtonHeight()
    return GameState.render.resY / 900 * 40
end

local function getBackButtonVisible()
    return backButtonVisible
end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local container = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Left, AlignVertical.Center },
    padding = { 125, 0 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.Button_MainMenu {
            visible = getBackButtonVisible,
            title = "Back to Main Menu",
            width = getButtonWidth,
            height = getButtonHeight,
            callback = switchToMainScreen
        },
    }
}

BackgroundView:addContent(container)

return BackgroundView
