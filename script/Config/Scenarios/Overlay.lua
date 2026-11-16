-- Text a scenario shows over the game (what it shows, its keys), at the top
-- left under the HUD's zone line. Call it from the scenario's drawOverlay
-- (which runs inside the HUD's immediate-mode pass and hides with the HUD).

local DrawEx = require("UI.DrawEx")

local Overlay = {}

---@param lines string[]
function Overlay.draw(lines)
    -- Scales with the window height like the HUD (1080 p = 1)
    local k = math.max(0.7, math.min(1.3, Window:height() / 1080))
    local size = math.floor(15 * k + 0.5)
    local step = math.floor(24 * k + 0.5)
    local x, y = math.floor(32 * k), math.floor(84 * k)
    for _, line in ipairs(lines) do
        DrawEx.TextAdditive("Unageo-Medium", line, size, x, y, 40, 20, 0.85, 0.9, 1.0, 0.9, 0.0, 0.5)
        y = y + step
    end
end

return Overlay
