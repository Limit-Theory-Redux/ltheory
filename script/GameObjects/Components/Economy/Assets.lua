local Entity = require('GameObjects.Entity')

local function iterateAssets (s)
  s.i = s.i + 1
  return s.list[s.i]
end

function Entity:addAsset (asset)
--printf("adding new asset = %s (%s) to %s", asset, asset:getName(), self:getName())
  assert(self.assets)
  assert(asset.owner == nil)
  insert(self.assets, asset)
  asset.owner = self
end

function Entity:addAssets ()
  assert(not self.assets)
  self.assets = {}
--printf("adding initial assets (empty) to %s", self:getName())
end

function Entity:getAssets ()
--printf("assets = %s", self.assets)
  assert(self.assets)
  return self.assets
end

function Entity:hasAssets ()
  return self.assets ~= nil
end

-- TODO : Surely there is a way to achieve 'for x in e:iterBlah' without having
--        to resort to table creation??
function Entity:iterAssets ()
  return iterateAssets, { list = self:getAssets(), i = 0 }
--  return iterateAssets, { list = self.assets, i = 0 }
end

function Entity:removeAsset (asset)
--printf("removing asset = %s (%s) from %s", asset, asset:getName(), self:getName())
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
