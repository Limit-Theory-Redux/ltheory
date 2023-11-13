-- objects
local Shape        = require('Systems.Gen.ShapeLib.Shape')
-- shapes
local BasicShapes  = require('Systems.Gen.ShapeLib.BasicShapes')
local RandomShapes = require('Systems.Gen.ShapeLib.RandomShapes')
-- ships
local ShipWarps    = require('Systems.Gen.ShipLib.ShipWarps')
local ShipDetail   = require('Systems.Gen.ShipLib.ShipDetail')
local ShipHull     = require('Systems.Gen.ShipLib.ShipBasicHull')
require('Systems.Gen.ShapeLib.Warp')
-- util
local MathUtil   = require('Systems.Gen.MathUtil')
local Parametric = require('Systems.Gen.ShapeLib.Parametric')

local ShipBasic  = {}

function ShipBasic.EngineSingle(rng, hull)
    local res = rng:choose({ 5, 6, 8, 10, 20 })
    local engine = BasicShapes.Prism(2, res)

    local r = 0.5 * hull
    --  local r = rng:getUniformRange(0.1, 0.3)
    Log.Debug("@@@ ShipBasic.EngineSingle - hull = %s, r = %s", hull, r)

    engine:scale(r, r, r)
    engine:rotate(0, math.pi / 2, 0)

    -- extrude rear-facing face to create engine shape
    local pi = engine:getPolyWithNormal(Vec3d(0, 0, -1))
    local t = math.pi * 1.05
    local l = 0.1
    --  local l = rng:getUniformRange(0.1, 0.3)
    r = 0.5
    --  r = rng:getUniformRange(0.05, 0.5)
    engine:extrudePoly(pi, l,
        Vec3d(r, r, r),
        Vec3d(0, math.sin(t), -math.cos(t)))

    local aabb = engine:getAABB()
    local z = math.abs(aabb.upper.z - aabb.lower.z)
    engine:center(0, 0, -z / 2.0)

    -- extrude forward-facing face so that it looks more 'attached' to the ship
    local pi = engine:getPolyWithNormal(Vec3d(0, 0, 1))
    local t = math.pi * 1.05
    local l = 0.2
    r = 0.4
    engine:extrudePoly(pi, l,
        Vec3d(r, r, r),
        Vec3d(0, math.sin(t), math.cos(t)))
    engine:center()

    return engine:finalize()
end

function ShipBasic.TurretSingle(rng)
    local res = rng:choose({ 3, 4, 6, 8, 10, 20 })
    local r = rng:getUniformRange(0.1, 0.3)
    local turret = BasicShapes.Prism(2, res)
    turret:scale(r, r, r)
    turret:rotate(0, math.pi / 2, 0)

    -- extrude to create gun shape
    local pi = turret:getPolyWithNormal(Vec3d(0, 0, 1))
    local t = math.pi * 1.05
    local l = 0.03
    --  local l = rng:getUniformRange(0.05, 0.5)
    r = 5
    --  r = rng:getUniformRange(0.05, 0.5)
    turret:extrudePoly(pi, l,
        Vec3d(r, r, r),
        Vec3d(0, math.sin(t), -math.cos(t)))

    local aabb = turret:getAABB()
    local z = math.abs(aabb.upper.z - aabb.lower.z)
    turret:center(0, 0, -z / 2.0)

    -- extrude backward-facing face so that it looks more 'attached' to the ship
    local pi = turret:getPolyWithNormal(Vec3d(0, 0, -1))
    local t = math.pi * 1.05
    local l = 0.1
    r = 0.25
    turret:extrudePoly(pi, l,
        Vec3d(r, r, r),
        Vec3d(0, math.sin(t), math.cos(t)))

    turret:center()
    return turret:finalize()
end

function ShipBasic.Tube(rng, hull)
    local shape = Shape()

    -- settings
    -- overall size settings
    local minS = 0.5
    local maxS = 1.5

    -- base segments
    Log.Debug("@@@ ShipBasic.Tube - hull = %s", hull)
    shape:add(ShipHull.Hull(rng, hull))
    --  for i = 1, hull do
    --    local seg = ShipHull.Hull(rng, hull)

    --    local overlap = rng:chance(0.5)
    --    if overlap then
    --      shape:add(seg)
    --    else
    --      local aabb = shape:getAABB()
    --      seg:translate(0, 0, aabb.upper.z)
    --      shape:add(seg)
    --    end
    --  end

    -- plates
    --  if rng:chance(0.5) then
    --    local plate = ShipDetail.Plate(rng)
    --    local aabb = shape:getAABB()
    --    plate:scale(0.5*math.abs(aabb.upper.x-aabb.lower.x), 1, 0.3*math.abs(aabb.upper.z-aabb.lower.z))
    --    plate:center()
    --    shape:center()
    --    shape:add(plate)
    --  end

    -- Scale the ship's length to fit the hull type
    -- NOTE: This is needed because applyForce() and applyTorque() apparently use radius rather than mass
    local r = shape:getRadius()
    local lrc = Config.gen.shipHullRadius[hull] / r
    Log.Debug("@@@ ShipBasic.Tube - hull = %d, radius = %s, lrc = %s", hull, r, lrc)
    shape:scale(lrc, lrc, lrc)
    --  shape:scale(1, 1, lrc)

    -- random scale
    --  shape:scale(rng:getUniformRange(minS, maxS), rng:getUniformRange(minS, maxS), 1)

    local newshape = shape:finalize()

    Log.Debug("@@@ ShipBasic.Tube - new radius = %s", newshape:getRadius())

    return newshape
end

return ShipBasic
