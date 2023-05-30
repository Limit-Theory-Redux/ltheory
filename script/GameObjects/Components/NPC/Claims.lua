local Entity = require('GameObjects.Entity')

function Entity:addClaimable ()
  assert(not self.claims)
  self.claims = {}

  local claimsLength = {}
  --self:register(Event.Update, Entity.updateClaims)
  function claimsLength.__len()
    local count = 0
    for _, hasClaim in pairs(self.claims) do
      if hasClaim then
        count = count + 1
      end
    end
    return count
  end

  setmetatable(self.claims, claimsLength)
end

function Entity:addClaim (claimer)
  assert(self.claims)
  assert(claimer)

  self.claims[claimer] = true
end

function Entity:removeClaim (claimer)
  assert(self.claims)
  self.claims[claimer] = nil
end

function Entity:getClaims ()
  assert(self.claims)
  return self.claims
end
