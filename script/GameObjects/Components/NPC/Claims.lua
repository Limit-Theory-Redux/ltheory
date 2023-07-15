local Entity = require('GameObjects.Entity')

function Entity:addClaimable ()
  assert(not self.claims)
  self.claims = {}
  --self:register(Event.Update, Entity.updateClaims)
end

function Entity:addClaim (claimer)
  assert(self.claims)
  assert(claimer)

  self.claims[claimer:getName()] = claimer
  --print(self:getName(), claimer:getName(), self.claims[claimer:getName()], self.claims[claimer:getName()]:getName())
end

function Entity:removeClaim (claimer)
  assert(self.claims)
  self.claims[claimer:getName()] = nil
end

function Entity:getClaims ()
  assert(self.claims)
  return self.claims
end

--! do this as LUAJIT is only 5.1 compatible, we need -DLUAJIT_ENABLE_LUA52COMPAT in the compiler
function Entity:getClaimsCount ()
  assert(self.claims)
  local count = 0
  for _, claimer in pairs(self.claims) do
    if claimer then
      count = count + 1
    end
  end
  return count
end
