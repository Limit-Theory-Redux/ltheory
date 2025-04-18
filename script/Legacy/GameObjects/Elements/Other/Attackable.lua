local Entity = require('Legacy.GameObjects.Entity')

function Entity:addAttackable(attackable)
    assert(not self.attackable)
    self.attackable = attackable
end

function Entity:getAttackable()
    assert(self.attackable)
    return self.attackable
end

function Entity:hasAttackable()
    return self.attackable ~= nil
end

function Entity:isAttackable()
    return self.attackable
end

function Entity:setAttackable(attackable)
    assert(self.attackable)
    self.attackable = attackable
end
