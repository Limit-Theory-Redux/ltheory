-- objects
local Shape        = require('Systems.Gen.ShapeLib.Shape')
-- shapes
local BasicShapes  = require('Systems.Gen.ShapeLib.BasicShapes')
local RandomShapes = require('Systems.Gen.ShapeLib.RandomShapes')
-- ships
local ShipWarps    = require('Systems.Gen.ShipLib.ShipWarps')
local ShipHull     = require('Systems.Gen.ShipLib.ShipCapitalHull')
local ShipDetail   = require('Systems.Gen.ShipLib.ShipDetail')
require('Systems.Gen.ShapeLib.Warp')
-- util
local MathUtil     = require('Systems.Gen.MathUtil')
local Parametric   = require('Systems.Gen.ShapeLib.Parametric')

local ShipCapitalCockpit = {}

function ShipCapitalCockpit.CockpitLarge (rng)
  local cockpitSizeMin = 0.2
  local cockpitSizeMax = 0.5

  -- cockpit
  local shape = ShipHull.Hull(rng)

  -- tower thing
  local aabb = shape:getAABB()
  local box = BasicShapes.Box()
  box:scale(abs(aabb.upper.x - aabb.lower.x)*0.25, 2, 0.25*abs(aabb.upper.z - aabb.lower.z))
  box:center(0, -1.8, 0)
  shape:addAtIntersection(Vec3d(0,-10,0), Vec3d(0,10,0), box)

  shape:scale(rng:getUniformRange(cockpitSizeMin, cockpitSizeMax))
  aabb = shape:getAABB()
  shape:center(0, 0.4*abs(aabb.upper.y-aabb.lower.y), 0)
  return shape
end

return ShipCapitalCockpit
