local Entity = require('GameObjects.Entity')

function Entity:getOwner()
    return self.owner
end

function Entity:getOwnerDisposition(target)
    if not self.owner then return 0 end
    return self.owner:getDisposition(target)
end

function Entity:getOwnerRoot()
    local root = self
    while root:getOwner() do
        root = root:getOwner()
    end
    return root
end

function Entity:setOwner(owner, isAsset)
    --! we can prob design a better automation process than this
    if self.owner then
        if self.owner.hasAsset and self.owner:hasAsset(self) then
            self.owner:removeAsset(self)
        end
        self.owner = nil
    end

    if owner then
        if isAsset then
            owner:addAsset(self)
        else
            self.owner = owner
        end
    end
end
