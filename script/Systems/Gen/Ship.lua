local ShipBasic   = require('Systems.Gen.ShipBasic')
local ShipCapital = require('Systems.Gen.ShipCapital')
local ShipFighter = require('Systems.Gen.ShipFighter')

local Ship = {}

-- TODO : These function names are confusing as hell. require('Gen.ShipFighter') ~= Gen.ShipFighter

function Ship.ShipFighter(seed, hull, res)
printf("@@@ Ship.ShipFighter:(create) - hull = %s, res = %s", hull, res)
  local rng = RNG.Create(seed)

  local type = rng:choose({1, 2})
  if type == 1 then
    Profiler.Begin('Gen.ShipFighter.Standard')
    local result = ShipFighter.Standard(rng, hull)
    Profiler.End()
    return result
  elseif type == 3 then
    Profiler.Begin('Gen.ShipFighter.Surreal')
    local result = ShipFighter.Surreal(rng, hull)
    Profiler.End()
    return result
  else
    assert("Ship type non-existant. Defaulting to Standard.")
    Profiler.Begin('Gen.ShipFighter.StandardDefault')
    local result = ShipFighter.Standard(rng, hull)
    Profiler.End()
    return result
  end
end

function Ship.ShipCapital(seed, hull, res)
printf("@@@ Ship.ShipCapital:(create) - hull = %s, res = %s", hull, res)
  local rng = RNG.Create(seed)
  Profiler.Begin('Gen.ShipCapital')
  local result = ShipCapital.Sausage(rng, hull)
  Profiler.End()
  return result
end

function Ship.ShipBasic(seed, hull, res)
printf("@@@ Ship.ShipBasic:(create) - hull = %s, res = %s", hull, res)
  local rng = RNG.Create(seed)
  Profiler.Begin('Gen.ShipBasic')
  local result = ShipBasic.Tube(rng, hull)
  Profiler.End()
  return result
end

return Ship
