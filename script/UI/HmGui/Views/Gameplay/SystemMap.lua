---@type UIView
local SystemMap = UICore.View {
    name = "System_Map"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ApplicationBindings
local Bindings = require('States.ApplicationBindings')

local systemMap = nil

function SystemMap:onInput()
    if Input:isPressed(Bindings.SystemMap) then
        UIRouter:getCurrentPage():setView("In_Game")
    elseif Input:isPressed(Bindings.AutoNav) then
        if GameState.player.currentShip then
            if not GameState.player.autonavActive then
                local target = GameState.player.currentShip:getTarget()
                if not GameState.player.currentShip:isDestroyed() and GameState.player.currentShip:isShipDocked() == nil and target and target ~= GameState.player.currentShip then
                    if GameState.player.currentShip:getCurrentAction() == nil or not string.find(GameState.player.currentShip:getCurrentAction():getName(), "MoveTo") then
                        -- Move undestroyed, undocked player ship to area of selected target
                        local autoDistance = Config.game.autonavRanges[target:getType()]
                        GameState.player.currentShip:pushAction(Actions.MoveTo(target, autoDistance, true))
                        GameState.player.autonavActive = true
                    end
                end
            else
                GameState.player.currentShip.travelDriveActive = false --* temporary
                GameState.player.currentShip.travelDriveTimer = 0      --* temporary
                GameState.player.currentShip:clearActions()
                GameState.player.autonavActive = false
            end
        end
    end
end

function SystemMap:onUpdate(dt) end

function SystemMap:onViewOpen(isPageOpen)
    systemMap = Systems.CommandView.SystemMap(GameState.world.currentSystem)
    GameState.render.uiCanvas:remove(GameState.render.gameView)
    GameState.render.uiCanvas:add(systemMap)
    Input:setCursorVisible(true)
    Log.Debug("Draw System View")
end

function SystemMap:onViewClose(isPageClose)
    GameState.render.uiCanvas:remove(systemMap)
    GameState.render.uiCanvas:add(GameState.render.gameView)
    systemMap = nil -- reset
    Input:setCursorVisible(false)
    Log.Debug("Draw Game View")
end

return SystemMap
