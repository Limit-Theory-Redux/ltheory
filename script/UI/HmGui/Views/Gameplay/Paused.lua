---@type UIView
local Paused = UICore.View {
    name = "Paused"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ApplicationBindings
local Bindings = require('States.ApplicationBindings')
---@type ShipSocketType
local SocketType = require('GameObjects.Entities.Ship.SocketType')

function Paused:onInput()
    if InputInstance:isPressed(Bindings.Escape) then
        UIRouter:getCurrentPage():setView("In_Game")
    end
end

function Paused:onUpdate(dt) end

function Paused:onViewOpen(isPageOpen)
    GameState:Pause()
    InputInstance:setCursorVisible(true)
end

function Paused:onViewClose(isPageOpen)
    --? UNPAUSE WHILE PROJECTILES ARE STILL ALIVE LEADS TO CRASH IN THIS INSTANCE WHY?
    --* see temporary fix in Views/InGame.lua
    --GameState:Unpause()
    --InputInstance:setCursorVisible(false)
end

local function getButtonWidth()
    return GameState.render.resX / 1600 * 200
end

local function getButtonHeight()
    return GameState.render.resY / 900 * 40
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
    stackDirection = Enums.UI.StackDirection.Vertical,
    color = {
        background = Color(0, 0, 0, 0.75)
    },
    contents = {
        UIComponent.Button_MainMenu {
            title = "Return to Game",
            width = getButtonWidth,
            height = getButtonHeight,
            callback = function()
                freezeTurrets()
                UIRouter:getCurrentPage():setView("In_Game")
            end
        },
        UIComponent.Button_MainMenu {
            title = "Exit",
            width = getButtonWidth,
            height = getButtonHeight,
            callback = function()
                EngineInstance:exit()
            end
        }
    }
}

Paused:addContent(menuContainer)

return Paused
