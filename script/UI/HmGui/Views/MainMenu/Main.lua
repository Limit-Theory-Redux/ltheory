---@type UIView
local MainView = UICore.View {
    name = "Main"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
local InitFiles = require('Systems.Files.InitFiles')

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

local function distance(x1, y1, x2, y2)
    return math.sqrt((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
end

local lastMousePos = Vec2f(0, 0)
local lastMoved = TimeStamp.Now()
local minDistance = 50 -- pixel
local timeTillBackground = 20

function MainView:onInput()
    local mousePos = InputInstance:mouse():position()

    if distance(mousePos.x, mousePos.y, lastMousePos.x, lastMousePos.y) > minDistance then
        lastMoved = TimeStamp.Now()
        lastMousePos = mousePos
    end

    if lastMoved:getElapsed() >= timeTillBackground then
        UIRouter:getCurrentPage():setView("Background")
    end
end

function MainView:onUpdate(dt) end

function MainView:onViewOpen(isPageOpen)
    lastMoved = TimeStamp.Now()
end

function MainView:onViewClose(isPageClose) end

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 170 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local function switchToNewgameView()
    UIRouter:getCurrentPage():setView("Newgame")
end

local function switchToLoadgameView()
    UIRouter:getCurrentPage():setView("Loadgame")
end

local function switchToSettingsView()
    UIRouter:getCurrentPage():setView("Settings")
end

local function switchToCreditsView()
    UIRouter:getCurrentPage():setView("Credits")
end

local menuGrid = UILayout.Grid {
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
            color = {
                background = Color(0, 0, 0, 0.3)
            },
            contents = {
                UIComponent.RawInput {
                    heightInLayout = 2 / 10,
                    fn = function()
                        Gui:beginStackContainer()
                        Gui:setPaddingTop(10)
                        Gui:setPaddingRight(5)
                        Gui:setPaddingBottom(10)
                        Gui:setPaddingLeft(5)
                        Gui:setPercentSize(100, 100)
                        Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Center)

                        Gui:image(logo)
                        Gui:setPercentSize(100, 42)

                        Gui:endContainer()
                    end
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Top },
                    padding = { 0, 50 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 7 / 10,
                    contents = {
                        UIComponent.Button_MainMenu {
                            title = "New Game",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToNewgameView,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Load Game",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToLoadgameView,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Settings",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                            callback = switchToSettingsView
                        },
                        UIComponent.Button_MainMenu {
                            title = "Credits",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToCreditsView,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Exit",
                            size = ResponsiveSize(300, 60, true),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = function()
                                LimitTheoryRedux:exit() -- run pre-exit operations & exit game
                            end,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 0 },
                    margin = { 0, 0 },
                    heightInLayout = 1 / 10,
                    layoutType = GuiLayoutType.Vertical,
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
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
            contents = {
                UIComponent.RawInput { fn = function() end }
            }
        }
    }
}

MainView:addContent(menuGrid)

return MainView
