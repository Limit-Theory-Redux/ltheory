local ShipFighter   = require('Systems.Gen.ShipFighter')
local ShipCapital   = require('Systems.Gen.ShipCapital')
local ShipInvisible = require('Systems.Gen.ShipInvisible')

local Ship = {}

-- TODO : These function names are confusing as hell. require('Gen.ShipFighter') ~= Gen.ShipFighter

function Ship.ShipFighter(seed, res)
  local rng = RNG.Create(seed)
  local type = rng:choose({1, 2})
  if type == 1 then
    Profiler.Begin('Gen.ShipFighter.Standard')
    local result = ShipFighter.Standard(rng)
    Profiler.End()
    return result
  elseif type == 3 then
    Profiler.Begin('Gen.ShipFighter.Surreal')
    local result = ShipFighter.Surreal(rng)
    Profiler.End()
    return result
  else
    assert("Ship type non-existant. Defaulting to Standard.")
    return ShipFighter.Standard(rng)
  end
end

function Ship.ShipCapital(seed, res)
  local rng = RNG.Create(seed)
  return ShipCapital.Sausage(rng)
end

function Ship.ShipInvisible(seed, res)
  local rng = RNG.Create(seed)
  Profiler.Begin('Gen.ShipInvisible.Standard')
  local result = ShipInvisible.Standard(rng)
  Profiler.End()
  return result
end

return Ship
