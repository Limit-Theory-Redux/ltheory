--Flat: don't actually need most of this as it won't be seen (because invisible)
--      keeping for cargo cult programming simplicity

-- objects
local Joint        = require('Systems.Gen.ShapeLib.Joint')
local JointField   = require('Systems.Gen.ShapeLib.JointField')
local Shape        = require('Systems.Gen.ShapeLib.Shape')
local Style        = require('Systems.Gen.ShapeLib.Style')
-- shapes
local BasicShapes  = require('Systems.Gen.ShapeLib.BasicShapes')
local Cluster      = require('Systems.Gen.ShapeLib.Cluster')
local Scaffolding  = require('Systems.Gen.ShapeLib.Scaffolding')
local Module       = require('Systems.Gen.ShapeLib.Module')
local RandomShapes = require('Systems.Gen.ShapeLib.RandomShapes')
-- local ShipWarps    = require('Gen.ShipWarps')
require('Systems.Gen.ShapeLib.Warp')
-- util
local MathUtil     = require('Systems.Gen.MathUtil')
local Parametric   = require('Systems.Gen.ShapeLib.Parametric')

local ShipInvisible = {}

-- SETTINGS [

Settings.addBool('genship.override', 'Override Generator', true)

Settings.addEnum('genship.global.surface', 'Surface Detail', 5, {
  'Random',
  'None',
  'Spikes',
  'Smooth Corners',
  'Extrude',
  'Greeble',
})

Settings.addFloat('genship.global.surfaceAmt',  'Surface Detail Amount', 0.2, 0.01, 1.0)
Settings.addBool('genship.global.randDetail',   'Add Random Detail', false)

Settings.addEnum( 'genship.standard.hullType',      'Hull Type', 1, {'Random', 'Classic', 'Round',})
Settings.addFloat('genship.standard.hullRes',       'Hull Smoothness', 6, 3, 30)
Settings.addFloat('genship.standard.hullPoint',     'Hull Pointiness', 0.5, 0.05, 2.0)
Settings.addFloat('genship.standard.hullRadius',    'Hull Radius', 1.0, 0.5, 4.0)
Settings.addFloat('genship.standard.hullLength',    'Hull Length', 1.0, 0.5, 4.0)
Settings.addEnum( 'genship.standard.wingType',      'Wing Type', 1, {'Random', 'Classic', 'Tie',})
Settings.addFloat('genship.standard.numWings',      'Num Wings', 2, 0, 8)
Settings.addFloat('genship.standard.wingLength',    'Wing Length', 2.5, 0.5, 5.0)
Settings.addFloat('genship.standard.wingWidth',     'Wing Width', 0.5, 0.05, 5.0)
Settings.addFloat('genship.standard.wingDist',      'Wing Distance', 0.5, 0.0, 5.0)
Settings.addFloat('genship.standard.wingPoint',     'Wing Pointiness', 0.5, 0.05, 2.0)
Settings.addBool( 'genship.standard.doubleTieWing', 'Tie Double Wing', true)
Settings.addEnum( 'genship.standard.tieWingShape',  'Tie Wing Shape', 1, {
  'Random',
  'Prism',
  'Round',
  'Irregular',
  'Triangle',
})

-- ] SETTINGS

-------------------------------------

-- WARPS [

function ShipInvisible.SurfaceDetail(rng, shape)
  local type = Settings.get('genship.global.surface')
  if type == 1 or Settings.get('genship.override') == false then
    if rng:chance(0.05) then
      local s = rng:getUniformRange(0.05, 0.3)
      shape:stellate(s)
      if rng:chance(0.5) then
        shape:extrude(0.2)
      end
    elseif rng:chance(0.05) then
      shape:extrude(0.2, Vec3d(
        rng:getUniformRange(0.05, 0.5),
        rng:getUniformRange(0.05, 0.5),
        rng:getUniformRange(0.05, 0.5))
      )
    elseif rng:chance(0.05) then
      shape:greeble(rng,
        1, -- tessellations
        0.01, 0.03 -- low, high size
      )
    else
      -- never bevel other details- looks uggo
      -- but always bevel if no other details applied- sharp corners are bleh
      shape = shape:bevel(rng:getUniformRange(0.1, 1.0))
    end
  else
    local amt = Settings.get('genship.global.surfaceAmt')
    -- 2 = none
    if type == 3 then
      shape:stellate(amt)
    elseif type == 4 then
      shape = shape:bevel(amt)
    elseif type == 5 then
      shape:extrude(amt)
    elseif type == 6 then
      shape:greeble(rng, 1, 0.01, 0.03, amt)
    end
  end
  return shape
end

-- ] WARPS

-------------------------------------

-- PARTS [

function ShipInvisible.EngineSingle(rng)
  local res = rng:choose({3, 4, 6, 8, 10, 20})
  local engine = BasicShapes.Prism(2, res)

  local r = rng:getUniformRange(0.1, 0.3)

  engine:scale(r, r, r)
  engine:rotate(0, math.pi/2, 0)

  local aabb = engine:getAABB()
  local z = math.abs(aabb.upper.z - aabb.lower.z)
  engine:center(0, 0, -z/2.0)

  -- extrude forward-facing face so that it looks more 'attached' to the ship
  local pi = engine:getPolyWithNormal(Vec3d(0, 0, 1))
  local t = math.pi*1.05
  local l = 0.1
  r = 0.25
  engine:extrudePoly(pi, l,
            Vec3d(r, r, r),
            Vec3d(0, math.sin(t), -math.cos(t)))

  return engine:finalize()
end

function ShipInvisible.TurretSingle(rng)
  local res = rng:choose({3, 4, 6, 8, 10, 20})
  local r = rng:getUniformRange(0.1, 0.3)
  local turret = BasicShapes.Prism(2, res)
  turret:scale(r, r, r)
  turret:rotate(0, math.pi/2, 0)

  -- extrude to create gun shape
  local pi = turret:getPolyWithNormal(Vec3d(0, 0, 1))
  local t = math.pi*1.05
  local l = rng:getUniformRange(0.05, 0.5)
  r = rng:getUniformRange(0.05, 0.5)
  turret:extrudePoly(pi, l,
            Vec3d(r, r, r),
            Vec3d(0, math.sin(t), -math.cos(t)))

  local aabb = turret:getAABB()
  local z = math.abs(aabb.upper.z - aabb.lower.z)
  turret:center(0, 0, -z/2.0)

  -- extrude backward-facing face so that it looks more 'attached' to the ship
  local pi = turret:getPolyWithNormal(Vec3d(0, 0, -1))
  local t = math.pi*1.05
  local l = 0.1
  r = 0.25
  turret:extrudePoly(pi, l,
            Vec3d(r, r, r),
            Vec3d(0, math.sin(t), math.cos(t)))

  turret:center()
  return turret:finalize()
end

function ShipInvisible.WingMounts(rng, bodyAABB, res)
  local mount = BasicShapes.Prism(2, res)
  mount:rotate(0, math.pi/2, 0)
  local r = Math.Clamp(rng:getExp()*0.2 + 0.5, 0.2, 1)
  local l = rng:getUniformRange(0.2, math.abs(bodyAABB.upper.z - bodyAABB.lower.z))
  local gunScale = Vec3d(r, r, l)
  mount:scale(gunScale.x, gunScale.y, gunScale.z)

  local xPos
  if Settings.get('genship.override') then
    xPos = Settings.get('genship.standard.wingDist')
  else
    xPos = bodyAABB.lower.x
  end
  mount:translate(xPos, 0, 0)

  mount = ShipInvisible.SurfaceDetail(rng, mount)

  local mount2 = mount:clone()
  mount2:mirror(true, false, false)
  mount:add(mount2)

  return mount
end

function ShipInvisible.HullStandard(rng)
  -- settings
  local length, cxy, r, res
  if Settings.get('genship.override') then
    r = Settings.get('genship.standard.hullRadius')
    length = Settings.get('genship.standard.hullLength')
    local c = Settings.get('genship.standard.hullPoint')
    cxy = {c, c}
    res = math.floor(Settings.get('genship.standard.hullRes'))
  else
    r = 1
    length = rng:getUniformRange(0.5, 3)
    cxy = {rng:getUniformRange(0.1, 0.5), rng:getUniformRange(0.1, 0.5)}
    res = rng:choose({3, 4, 5, 6, 8, 10, 20, 24, 28, 30})
  end

  -- basic shape
  local shape, type

  local type = Settings.get('genship.standard.hullType')
  if type == 1 or Settings.get('genship.override') == false then
    local dist = Distribution()
    dist:add(2, 0.85) -- standard prism
    dist:add(3, 0.15) -- sphere
    type = dist:sample(rng)
  end

  if type == 2 then -- standard prism
    shape = BasicShapes.Prism(2, res)
    shape:rotate(0, math.pi * 0.5, 0)
    if res % 2 ~= 0 then
      shape:rotate(0, 0, math.pi * 0.5) -- for bilateral symmetry
    end

    local pi = shape:getPolyWithNormal(Vec3d(0, 0, 1))
    local t = math.pi
    shape:extrudePoly(pi, length,
              Vec3d(cxy[1], cxy[2], 1.0),
              Vec3d(0, math.sin(t), -math.cos(t)))

    local back = shape:getPolyWithNormal(Vec3d(0, 0, -1))
    shape:extrudePoly(back, 0.3, Vec3d(0.5, 0.5, 0.5), Vec3d(0, math.sin(t), math.cos(t)))
  elseif type == 3 then -- sphere
    shape = BasicShapes.Ellipsoid(res)
    if res % 2 ~= 0 then
      shape:rotate(math.pi*0.5, 0, 0)
    end
    shape:scale(1, 1, length)
  end

  -- scaling & detail
  shape:scale(r, r, 1)

  if Settings['genship.override'] then
    shape = ShipInvisible.SurfaceDetail(rng, shape)
  end

  return shape
end

function ShipInvisible.WingsStandard(rng, bodyAABB)
  local shape = Shape()

  local n
  if Settings.get('genship.override') then
    n = math.floor(Settings.get('genship.standard.numWings'))
  else
    n = rng:getUniformRange(1, 3)
  end

  for i = 1, n do
    local wing1 = BasicShapes.Box(0)

    local l, w, point
    if Settings.get('genship.override') then
      l = Settings.get('genship.standard.wingLength')
      w = Settings.get('genship.standard.wingWidth')
      point = Settings.get('genship.standard.wingPoint')
    else
      l = rng:getUniformRange(0.5, 3.0)
      w = rng:getUniformRange(0.5, 3.0)
      point = rng:getUniformRange(0.05, 1.0)
    end

    -- scale & extrude to create shape
    wing1:scale(0.1, 0.2, w)
    local pi1 = wing1:getPolyWithNormal(Vec3d(1, 0, 0))
    wing1:extrudePoly(pi1, l, Vec3d(1.0,
      rng:getUniformRange(0.05, 0.5), -- thin-ness
      point)
    )

    -- make tips pointy
    pi1 = wing1:getPolyWithNormal(Vec3d(1, 0, 0))
    wing1:extrudePoly(pi1, 0.2, Vec3d(1, 0.1, 1))

    -- winglets
    if rng:chance(0.5) then
      local winglet = wing1:clone():scale(0.5, 0.5, 0.5)
      local wingAABB = wing1:getAABB()
      winglet:rotate(0, 0, rng:getUniformRange(0, math.pi))
      winglet:center(wingAABB.upper.x, 0, 0)
      wing1:add(winglet)
    end

    if Settings.get('genship.override') then
      wing1 = ShipInvisible.SurfaceDetail(rng, wing1)
    end

    -- rotate & position
    local xPos
    if Settings.get('genship.override') then
      xPos = Settings.get('genship.standard.wingDist')
    else
      xPos = bodyAABB.upper.x
    end

    local roll = rng:getUniformRange(math.pi* -0.5, math.pi * 0.5)
    local yaw = rng:getUniformRange(0.0, math.pi*0.5)
    wing1:rotate(yaw, 0, roll)
    wing1:translate(xPos, 0, 0)

    -- decoration
    wing1:tessellate(rng:getInt(0,2))
    if Settings.get('genship.override') == false or
       (Settings.get('genship.override') and Settings.get('genship.global.randDetail')) then
      wing1:extrudePoly(rng:getInt(1, #wing1.polys), rng:getUniformRange(0.2, 1.0))
    end

    local wing2 = wing1:clone()
    wing2:mirror(true, false, false)

    shape:add(wing1)
    shape:add(wing2)
  end

  return shape
end

function ShipInvisible.WingsTie (rng)
  local shape = Shape()

  -- base shape
  local type = Settings.get('genship.standard.tieWingShape')
  if type == 1 or Settings.get('genship.override') == false then
    type = rng:choose({2, 3, 4, 5})
  end

  local wing
  if type == 2 then
    wing = BasicShapes.Prism(2, rng:choose({4,5,6,8}))
  elseif type == 3 then
    wing = BasicShapes.Prism(2, 30)
  elseif type == 4 then
    wing = BasicShapes.IrregularPrism(rng)
    wing:scale(1.5, 1, 1.5)
  elseif type == 5 then
    wing = BasicShapes.IrregularPrism(rng, 2, 3)
  end

  -- make wide, flat shape
  local r, split, dist
  if Settings.get('genship.override') then
    r = Settings.get('genship.standard.wingLength')
    dist = Settings.get('genship.standard.wingDist')
    split = Settings.get('genship.standard.doubleTieWing')
  else
    r = rng:getUniformRange(0.5, 3.0)
    dist = rng:getExp()*0.25 + 1.5
    split = type == 5 -- by default, only split triangle shape
  end
  wing:scale(r, 0.1, r)

  -- decoration
  if Settings.get('genship.override') == false or
       (Settings.get('genship.override') and Settings.get('genship.global.randDetail')) then
    local ndist = Distribution()
    ndist:add(0, 0.30)
    ndist:add(1, 0.20)
    ndist:add(2, 0.20)
    ndist:add(3, 0.20)
    ndist:add(10, 0.10)
    local n = ndist:sample(rng)
    for i = 0, n do
      local ind = rng:getInt(1, #wing.polys)
      local length = 0.5
      local norm = wing:getFaceNormal(wing.polys[ind])
      if norm.y ~= 1 and norm.y ~= -1 then
        -- don't increase width of wing
        length = rng:getUniformRange(0.1, 1.0)
      end
      wing:extrudePoly(ind, length)
    end
  end

  -- double wing
  if split then
    local wingHalf = wing:clone()
    wingHalf:mirror(false, false, true)
    local gap = r*0.5 + rng:getUniformRange(0.1, 0.5)
    wingHalf:translate(0, 0, -gap)
    wing:add(wingHalf)
    -- add connector between the two wings
    wing:center()
    local bar = BasicShapes.Box()
    bar:scale(rng:getUniformRange(0.05, 0.5), 0.05, gap*0.5)
    wing:add(bar)
  end

  -- rotate
  wing:rotate(math.pi*0.5, 0, math.pi*0.5)
  -- place
  wing:translate(dist, 0, 0)

  -- add connection
  local connector = BasicShapes.Prism(2, 6)
  connector:rotate(0, 0, math.pi*0.5)
  local cr = 1.0
  connector:scale(0.1, cr, cr)
  local pi = connector:getPolyWithNormal(Vec3d(1, 0, 0))
  connector:extrudePoly(pi, dist, Vec3d(1, 0.5, 0.5))
  wing:add(connector)

  -- wing decoration
  wing = ShipInvisible.SurfaceDetail(rng, wing)

  -- wing warping
  local wingAABB = wing:getAABB()
  local yMin = wingAABB.lower.y
  local yMax = wingAABB.upper.y
  if rng:chance(0.5) then
    local dir = rng:choose({1, -1})
    local amt = rng:getUniformRange(0.1, 0.5)
    wing:warp(
      function(v)
        local y = (v.y - yMin)/(yMax - yMin)
        y = Math.Lerp(0, math.pi, y)
        v.x = v.x + math.sin(y)*dir*amt
      end
    )
  end

  -- add second wing
  local wing2 = wing:clone()
  wing2:mirror(true, false, false)
  shape:add(wing):add(wing2)
  return shape
end


-- ] PARTS

-------------------------------------

-- SHIPS [

function ShipInvisible.Standard (rng)
  -- hull
  local res = rng:choose({3, 4, 6, 8, 10, 20})
  local shape = ShipInvisible.HullStandard(rng)

  local bodyAABB = shape:getAABB()

  -- wings
  local wingType = Settings.get('genship.standard.wingType')
  if wingType == 1 or Settings.get('genship.override') == false then
    local dist = Distribution()
    dist:add(2, 0.75) -- standard
    dist:add(3, 0.25) -- tie
    wingType = dist:sample(rng)
  end
  if wingType == 2 then
    shape:add(ShipInvisible.WingsStandard(rng, bodyAABB))
  elseif wingType == 3 then
    shape:add(ShipInvisible.WingsTie(rng, bodyAABB))
  end

  -- other parts
  shape:add(ShipInvisible.WingMounts(rng, bodyAABB, res))

  -- final warps
  -- shape = ShipWarps.CurveWarps(rng, shape)
  shape = shape:bevel(rng:getUniformRange(0.1, 0.8))
  local rcpRadius = 3.0 / shape:getRadius()
  shape:scale(rcpRadius, rcpRadius, rcpRadius)

  return shape:finalize()
end

-- ] SHIPS

return ShipInvisible
