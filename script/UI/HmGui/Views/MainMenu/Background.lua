---@type UIView
local BackgroundView = UICore.View {
    name = "Background"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")

local lastMousePos = Vec2f(0, 0)
local lastMoved = TimeStamp.Now()
local menuVisible = true

local function distance(x1, y1, x2, y2)
    return math.sqrt((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
end

local minDistance = 50 -- pixel

function BackgroundView:onInput()
    local mousePos = InputInstance:mouse():position()

    if distance(mousePos.x, mousePos.y, lastMousePos.x, lastMousePos.y) > minDistance then
        lastMoved = TimeStamp.Now()
        lastMousePos = mousePos
        menuVisible = true
        InputInstance:setCursorVisible(true)
    end

    if lastMoved:getElapsed() >= 5 then
        menuVisible = false
        InputInstance:setCursorVisible(false)
    end
end

function BackgroundView:onUpdate(dt) end

function BackgroundView:onViewOpen(isPageOpen)
    lastMoved = TimeStamp.Now()
end

function BackgroundView:onViewClose(isPageClose)
    menuVisible = true
    InputInstance:setCursorVisible(true) -- reset
end

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 170 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local function getMenuVisible()
    return menuVisible
end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local function toggleClock()
    GameState.ui.backgroundClockEnabled = not GameState.ui.backgroundClockEnabled
end

local backgroundGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 125, 0 },
    margin = { 0, 0 },
    stackDirection = GuiLayoutType.Horizontal,
    contents = {
        UILayout.Grid {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
            contents = {
                UIComponent.Container {
                    visible = getMenuVisible,
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 10 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 2 / 10,
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
                    contents = {
                        UIComponent.Text {
                            text = "SCREENSAVER",
                            size = 32,
                            font = "Unageo-Medium"
                        }
                    }
                },
                UIComponent.Container {
                    visible = getMenuVisible,
                    align = { AlignHorizontal.Stretch, AlignVertical.Top },
                    padding = { 0, 50 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 7 / 10,
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
                    contents = {
                        UIComponent.Button_MainMenu {
                            title = "Toggle Clock",
                            size = ResponsiveSize(300, 60),
                            callback = toggleClock,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Back to Main Menu",
                            size = ResponsiveSize(300, 60),
                            callback = switchToMainScreen,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                    }
                },
                UIComponent.Container {
                    visible = getMenuVisible,
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 0 },
                    margin = { 0, 0 },
                    heightInLayout = 1 / 10,
                    layoutType = GuiLayoutType.Vertical,
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
                    contents = {
                        UIComponent.Text {
                            text = Config.gameVersion,
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                            font = "Exo2",
                            size = 12
                        }
                    }
                }
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 0, 10 },
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
            contents = {
                UIComponent.Text {
                    visible = function() return GameState.ui.backgroundClockEnabled end,
                    text = function() return os.date("%H:%M") end,
                    size = 64,
                    font = "Unageo-Semibold",
                    align = { AlignHorizontal.Right, AlignVertical.Center },
                },
                UIComponent.Text {
                    visible = function() return GameState.ui.backgroundClockEnabled end,
                    text = function()
                        local currentDate = os.date("%B %d %Y")
                        ---@cast currentDate string
                        currentDate = string.upper(currentDate)
                        return currentDate
                    end,
                    size = 32,
                    font = "Unageo-Semibold",
                    align = { AlignHorizontal.Right, AlignVertical.Center },
                }
            }
        }
    }
}

BackgroundView:addContent(backgroundGrid)

return BackgroundView
