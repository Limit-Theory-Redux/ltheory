local all = {}

local Item = class(function(self, name, mass, energyDensity, distribution)
    self.name = name -- must be unique within items
    self.mass = mass or 1
    self.energy = Math.Round(math.max(1, (energyDensity or 1) * self.mass))
    self.distribution = distribution
    insert(all, self)
end)

function Item.All()
    return all
end

function Item:getEnergy()
    return self.energy
end

function Item:getEnergyDensity()
    return self.energy / self.mass
end

function Item:getDistribution()
    return self.distribution
end

function Item:getMass()
    return self.mass
end

function Item:getName()
    return self.name
end

function Item:hasItem(itemGroup, item)
    local groupHasItem = false

    for _, gItem in ipairs(itemGroup) do
        -- Compare on item name as we're looking for an item _type_, not a particular item object
        if gItem.name == item.name then
            groupHasItem = true
            break
        end
    end

    return groupHasItem
end

function Item:setEnergy(energy)
    self.energy = energy
    return self
end

Item.Credit = Item("Credit")
Item.Credit.mass = 0

return Item
