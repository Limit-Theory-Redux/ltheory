---@type UIView
local Paused = UICore.View {
    name = "Paused"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ApplicationBindings
local Bindings = require('States.ApplicationBindings')

function Paused:onInput()
    if InputInstance:isPressed(Bindings.Escape) then
        UIRouter:getCurrentPage():setView("In_Game")
    end
end

function Paused:onUpdate(dt) end

function Paused:onViewOpen(isPageOpen)
    GameState.paused = true
    InputInstance:setCursorVisible(true)
end

function Paused:onViewClose(isPageOpen)
    GameState.paused = false
    InputInstance:setCursorVisible(false)
end

return Paused
