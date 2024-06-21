---@type UIView
local InGame = UICore.View {
    name = "In_Game"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ApplicationBindings
local Bindings = require('States.ApplicationBindings')
---@type AIActions
local Actions = requireAll('GameObjects.Actions')

function InGame:onInput()
    if InputInstance:isPressed(Bindings.Escape) and GameState.player.currentShip and not GameState.player.currentShip:isDestroyed() then
        UIRouter:getCurrentPage():setView("Paused")
    elseif InputInstance:isPressed(Bindings.SystemMap) then
        UIRouter:getCurrentPage():setView("System_Map")
    elseif InputInstance:isPressed(Bindings.CameraFirstPerson) then
        if GameState.player.currentCamera ~= Enums.CameraMode.FirstPerson then
            GameState.render.gameView:setCameraMode(Enums.CameraMode.FirstPerson)
        end
    elseif InputInstance:isPressed(Bindings.CameraChase) then
        if GameState.player.currentCamera ~= Enums.CameraMode.Chase then
            GameState.render.gameView:setCameraMode(Enums.CameraMode.Chase)
        end
    elseif InputInstance:isPressed(Bindings.CameraOrbit) then
        -- if GameState.player.currentCamera ~= Enums.CameraMode.Orbit then
        --     self.gameView:setCameraMode(Enums.CameraMode.Orbit)
        -- end
    elseif InputInstance:isPressed(Bindings.AutoNav) then
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

function InGame:onUpdate(dt) end

function InGame:onViewOpen(isPageOpen)
    GameState:SetState(Enums.GameStates.InGame)
end

return InGame
