---@type UIView
local Paused = UICore.View {
    name = "Paused"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
---@type ApplicationBindings
local Bindings = require('States.ApplicationBindings')
---@type ShipSocketType
local SocketType = require('GameObjects.Entities.Ship.SocketType')

function Paused:onInput()
    if Input:isPressed(Bindings.Escape) then
        UIRouter:getCurrentPage():setView("In_Game")
    end
end

function Paused:onUpdate(dt) end

function Paused:onViewOpen(isPageOpen)
    GameState:Pause()
    Input:setCursorVisible(true)
end

function Paused:onViewClose(isPageClose)
    GameState:Unpause()

    if not isPageClose then
        Input:setCursorVisible(false)
    end
end

local function freezeTurrets()
    -- When taking down a dialog, Turret:updateTurret sees the button click input and thinks it means "Fire"
    -- So this routine adds a very brief cooldown to the player ship's turrets
    if GameState.player.currentShip then
        for turret in GameState.player.currentShip:iterSocketsByType(SocketType.Turret) do
            turret:addCooldown(0.2)
        end
        for bay in GameState.player.currentShip:iterSocketsByType(SocketType.Bay) do
            bay:addCooldown(0.2)
        end
    end
end

local menuContainer = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
    padding = { 0, 0 },
    layoutType = GuiLayoutType.Vertical,
    color = {
        background = Color(0, 0, 0, 0.75)
    },
    contents = {
        UIComponent.Button_MainMenu {
            title = "Return to Game",
            size = ResponsiveSize(200, 40, true),
            font = { name = "Unageo-Medium", size = 20 },
            callback = function()
                freezeTurrets()
                UIRouter:getCurrentPage():setView("In_Game")
            end
        },
        UIComponent.Button_MainMenu {
            title = "Back to Main Menu",
            size = ResponsiveSize(200, 40, true),
            font = { name = "Unageo-Medium", size = 20 },
            align = { AlignHorizontal.Center, AlignVertical.Center },
            callback = function() LimitTheoryRedux:initMainMenu() end
        },
        UIComponent.Button_MainMenu {
            title = "Exit",
            size = ResponsiveSize(200, 40, true),
            font = { name = "Unageo-Medium", size = 20 },
            callback = function()
                LimitTheoryRedux:exit() -- run pre-exit operations & exit game
            end
        }
    }
}

Paused:addContent(menuContainer)

return Paused
