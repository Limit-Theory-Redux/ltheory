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

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 170 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local creditsGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 125, 0 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    contents = {
        UILayout.Grid {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Container {
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 10, 10 },
                    margin = { 0, 0 },
                    stackDirection = Enums.UI.StackDirection.Vertical,
                    heightInLayout = 2 / 10,
                    color = {
                        background = Color(0.1, 0.1, 0.1, 0.2)
                    },
                    contents = {
                        UIComponent.Text {
                            text = "CREDITS",
                            size = 32,
                            font = "Unageo-Medium"
                        }
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Top },
                    padding = { 0, 50 },
                    margin = { 0, 0 },
                    stackDirection = Enums.UI.StackDirection.Vertical,
                    heightInLayout = 7 / 10,
                    color = {
                        background = Color(0.1, 0.1, 0.1, 0.2)
                    },
                    contents = {
                        UIComponent.Button_MainMenu {
                            title = "Back",
                            width = getButtonWidth,
                            height = getButtonHeight,
                            callback = switchToMainScreen,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        }
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 0 },
                    margin = { 0, 0 },
                    heightInLayout = 1 / 10,
                    stackDirection = Enums.UI.StackDirection.Vertical,
                    color = {
                        background = Color(0.1, 0.1, 0.1, 0.2)
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
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.RawInput { fn = function() end }
            }
        }
    }
}


CreditsView:addContent(creditsGrid)

return CreditsView
