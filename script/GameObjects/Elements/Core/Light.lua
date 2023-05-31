local Entity = require('GameObjects.Entity')

function Entity:addLight (r, g, b)
if self.light then
  printf("%s has a light already!", self:getName())
end
  assert(not self.light)
  self.light = Vec3f(r, g, b)
  insert(GameState.world.currentSystem.lightList, self)
end

function Entity:deleteLight (lightRef)
  assert(GameState.world.currentSystem.lightList)
--printf("LIGHT: trying to delete light on %s", lightRef:getName())
  for i, light in ipairs(GameState.world.currentSystem.lightList) do
    if light == lightRef then
      remove(GameState.world.currentSystem.lightList, i)
      break
    end
  end
end

function Entity:getLight ()
  assert(self.light)
  return self.light
end

function Entity:hasLight ()
  return self.light ~= nil
end

function Entity:setLight (r, g, b)
  self.light.x = r
  self.light.y = g
  self.light.z = b
end
