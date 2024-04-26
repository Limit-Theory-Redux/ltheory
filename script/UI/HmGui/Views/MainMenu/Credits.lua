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
    return GameState.render.resX / 1600 * 175 * 2 / GameState.render.resX
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
    showGrid = false,
    contents = {
        UIComponent.Container {
            align = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            color = {
                background = Color(0.1, 0.1, 0.1, 0.2)
            },
            contents = {
                UIComponent.Button_MainMenu {
                    title = "Back",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = switchToMainScreen
                },
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 0, 125 },
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.RawInput { fn = function()
                    --! mockup
                    Gui:beginStackContainer()
                    Gui:setPropertyFont(GuiProperties.TextFont, Cache.Font("Exo2", 28))
                    Gui:text("Credits")
                    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Top)
                    Gui:setMargin(0, 20)
                    Gui:rect(Color(.1, .1, .115, .25))
                    Gui:setPercentSize(100, 100)
                    Gui:endContainer()
                    Gui:setPercentSize(100, 100)
                end }
            }
        }
    }
}


CreditsView:addContent(creditsGrid)

return CreditsView
