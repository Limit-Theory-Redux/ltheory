local Entity = require('GameObjects.Entity')
local Pulse = require('GameObjects.Entities.Effects.Pulse')

local Projectile = subclass(Entity, function (self, pR, pG, pB)
  self.pColorR = pR
  self.pColorG = pG
  self.pColorB = pB
end)

function Entity:addProjectiles ()
  assert(not self.projectiles)
  self.projectiles = {}
  self.pcount = 0

  self:register(Event.Update, Entity.updateProjectiles)
  self:register(Event.Update, Entity.updateProjectilesPost)
end

function Entity:addProjectile (source)
  assert(self.projectiles)

  -- TODO: Extend projectile types to non-pulse effects

  if Config.audio.pulseFire then Sound.Play(Config.audio.pulseFire) end

  local e = Pulse:new()
  e.source = IncRef(source)

  local newProjectile = nil

  if GameState.render.pulseLights then
    newProjectile = Projectile(source.projColorR, source.projColorG, source.projColorB)

    local mesh = Gen.Primitive.IcoSphere(5):managed()
    newProjectile:addRigidBody(true, mesh)
    newProjectile:addVisibleLodMesh(mesh, Material.Rock())
    newProjectile:setMass(1)
    newProjectile:setDrag(0, 0)

    newProjectile:addLight(newProjectile.pColorR * 3,
                           newProjectile.pColorG * 3,
                           newProjectile.pColorB * 3)

    self.pcount = self.pcount + 1
    newProjectile:setName(format("Projectile: Pulse %d", self.pcount))
  end

  insert(self.projectiles, {projectile = newProjectile, effect = e})

  return newProjectile, e
end

function Entity:renderProjectiles (state)
  Pulse.Render(self.projectiles, state)
end

function Entity:updateProjectiles (state)
  Pulse.UpdatePrePhysics(self, self.projectiles, state.dt)
end

function Entity:updateProjectilesPost (state)
  Pulse.UpdatePostPhysics(self, self.projectiles, state.dt)
end

return Projectile
