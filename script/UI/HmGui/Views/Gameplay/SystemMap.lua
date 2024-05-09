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
    if InputInstance:isPressed(Bindings.SystemMap) then
        UIRouter:getCurrentPage():setView("In_Game")
    end
end

function SystemMap:onUpdate(dt) end

function SystemMap:onViewOpen(isPageOpen)
    systemMap = Systems.CommandView.SystemMap(GameState.world.currentSystem)
    GameState.render.uiCanvas:remove(GameState.render.gameView)
    GameState.render.uiCanvas:add(systemMap)
    InputInstance:setCursorVisible(true)
    Log.Debug("Draw System View")
end

function SystemMap:onViewClose(isPageClose)
    GameState.render.uiCanvas:remove(systemMap)
    GameState.render.uiCanvas:add(GameState.render.gameView)
    systemMap = nil -- reset
    InputInstance:setCursorVisible(false)
    Log.Debug("Draw Game View")
end

return SystemMap
