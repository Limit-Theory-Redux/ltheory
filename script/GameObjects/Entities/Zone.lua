local Entity = require('GameObjects.Entity')

local Zone = subclass(Entity, function(self, name)
    self.name = name
    self.children = {}
end)

function Zone:add(e)
    insert(self.children, e)
end

function Zone:getChildren()
    return self.children
end

function Zone:getName()
    return self.name
end

function Zone:getPos()
    return self.pos
end

function Zone:getExtent()
    return self.extent
end

function Zone:setExtent(extent)
    -- "extent" is a scalar radius for a spherical volume
    self.extent = extent
end

function Zone:sample(rng)
    return rng:choose(self.children)
end

function Zone:getRandomPos(rng)
    return self.pos + rng:getDir3():scale((0.1 * self.extent) * rng:getExp() ^ rng:getExp())
end

return Zone
