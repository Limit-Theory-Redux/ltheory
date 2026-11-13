---@type UIView
local InGame = UICore.View {
    name = "In_Game",
    noDarken = true -- in-game overlay: never dim the game view
}

-- NOTE: in-game input (pause, map, camera, autopilot) is owned by
-- LimitTheoryRedux:onInput and the PlayerController; the In_Game view only
-- draws.

function InGame:onInput() end

function InGame:onUpdate(dt) end

function InGame:onViewOpen(isPageOpen)
    GameState:SetState(Enums.GameStates.InGame)
end

return InGame
