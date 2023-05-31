local Entity = require('GameObjects.Entity')
local SocketType = require('GameObjects.Entities.Ship.SocketType')
local Material = require('GameObjects.Material')

local mesh
local material
local meshJet
local rng = RNG.FromTime()

local Thruster
Thruster = subclass(Entity, function (self, parentShip)
  if not mesh then
    local parentHullSize = parentShip:getHull()
    if parentHullSize == Enums.ShipHulls.Solo then
      mesh = Gen.ShipFighter.EngineSingle(rng)
    elseif parentHullSize == Enums.ShipHulls.VeryLarge then
      mesh = Gen.ShipBasic.EngineSingle(rng, parentHullSize)
    else
      mesh = Gen.ShipCapital.EngineSingle(rng)
    end
    mesh:computeNormals()
    mesh:computeAO(0.1)
    meshJet = Gen.Primitive.Billboard(-1, 0, 1, 1)
  end

  if not material then
    material = Material.Create(
      'material/metal',
      Cache.Texture('metal/02_d_gray'),
      Cache.Texture('metal/02_n'),
      Cache.Texture('metal/02_s'))
  end

  self:addRigidBody(true, mesh)
  self:addVisibleMesh(mesh, material)

  self.parentShip = parentShip
  self.activation = 0
  self.activationT = 0
  self.boost = 0
  self.boostT = 0
  self.time = rng:getUniformRange(0, 1000)

--printf("Register: Thruster type = %s, handler = %s", Event.Render, Thruster.render)
  self:register(Event.Render, Thruster.render)
--printf("Register: Thruster type = %s, handler = %s", Event.Update, Thruster.update)
  self:register(Event.Update, Thruster.update)
end)

function Thruster:getSocketType ()
  return SocketType.Thruster
end

function Thruster:render (state)
  if state.mode == BlendMode.Additive then
    local a = math.abs(self.activation)
    if a < 1e-3 then return end
    local shader = Cache.Shader('billboard/axis', 'effect/thruster')
    shader:start()
    Shader.SetFloat('alpha', a)
    Shader.SetFloat('time', self.time)
    Shader.SetFloat2('size', 2, 32 * self.activation)
    Shader.SetFloat3('color', 0.1 + 1.2 * self.boost, 0.3 + 0.2 * self.boost, 1.0 - 0.7 * self.boost)
    Shader.SetMatrix('mWorld', self:getToWorldMatrix())
    meshJet:draw()
    shader:stop()
  end
end

function Thruster:update (state)
  local t = 1.0 - exp(-4.0 * state.dt)
  self.activation = Math.Lerp(self.activation, self.activationT, t)
  self.boost = Math.Lerp(self.boost, self.boostT, t)
  self.time = self.time + state.dt

  -- Add local lighting based on ship's thruster activation
  if GameState.render.thrusterLights and self.parentShip:hasLight() then
    self.parentShip:setLight((0.2 + (1.9 * self.boost)) * abs(self.activation) * 2,
                             (0.8 + (0.3 * self.boost)) * abs(self.activation) * 2,
                             (3.0 - (2.4 * self.boost)) * abs(self.activation) * 2)
  end
end

return Thruster
