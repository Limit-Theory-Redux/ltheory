local Entity = require('Legacy.GameObjects.Entity')

function Entity:addAsset(asset)
    --Log.Debug("adding new asset = %s (%s) to %s", asset, asset:getName(), self:getName())
    assert(self.assets)
    assert(asset.owner == nil)
    insert(self.assets, asset)
    asset.owner = self
end

function Entity:addAssets()
    assert(not self.assets)
    self.assets = {}
end

function Entity:getAssets()
    assert(self.assets)
    return self.assets
end

function Entity:hasAssets()
    return self.assets ~= nil
end

function Entity:hasAsset(asset)
    for itAsset in Iterator(self:getAssets()) do
        if itAsset == asset then
            return true
        end
    end
    return false
end

function Entity:iterAssets()
    return Iterator(self:getAssets())
end

function Entity:removeAsset(asset)
    --Log.Debug("removing asset = %s (%s) from %s", asset, asset:getName(), self:getName())
    assert(self.assets)
    assert(asset.owner == self)
    asset.owner = nil
    for i, v in ipairs(self.assets) do
        if v == asset then
            self.assets[i] = self.assets[#self.assets]
            self.assets[#self.assets] = nil
            break
        end
    end
end
