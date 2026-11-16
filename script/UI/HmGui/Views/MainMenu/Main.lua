---@type UIView
local MainView = UICore.View {
    name = "Main",
    noDarken = true -- the sidebar carries its own background
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

local function distance(x1, y1, x2, y2)
    return math.sqrt((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
end

local lastMousePos = Vec2f(0, 0)
local lastMoved = TimeStamp.Now()
local minDistance = 50 -- pixel
local timeTillBackground = 20

function MainView:onInput()
    local mousePos = Input:mouse():position()

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

-- The idle screensaver view, on demand (it also opens after `timeTillBackground`
-- seconds without mouse movement).
local function switchToBackgroundView()
    UIRouter:getCurrentPage():setView("Background")
end

MainView:addContent(UIComponent.MenuScreen {
    contents = {
        UIComponent.MenuSidebar {
            contents = {
                UIComponent.MenuImage { x = 32, y = 92, w = 268, image = logo },
                UIComponent.MenuMainNav {},
                UIComponent.MenuNav {
                    y = 980, size = 26, itemHeight = 60,
                    items = { { title = "Background", callback = switchToBackgroundView } }
                },
                UIComponent.MenuVersion(),
            }
        },
    }
})

return MainView
