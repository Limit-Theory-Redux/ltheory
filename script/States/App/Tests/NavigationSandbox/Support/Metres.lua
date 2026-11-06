-- Converts the metres a scenario is authored in to game units.
-- Design: doc/script/navigation-sandbox.md
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

---@class Metres
local Metres = {}

---@param metres number
---@return number Game units
function Metres.length(metres)
    return ScaleConfig:toGameUnits(metres, "starSystem")
end

---@param gameUnits number
---@return number Metres
function Metres.toMetres(gameUnits)
    return ScaleConfig:toRealMeters(gameUnits, "starSystem")
end

---@param v number[] { x, y, z } in metres
---@return Position
function Metres.position(v)
    return Position(Metres.length(v[1]), Metres.length(v[2]), Metres.length(v[3]))
end

return Metres
