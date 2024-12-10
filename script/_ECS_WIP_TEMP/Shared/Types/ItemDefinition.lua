local ItemDefinition = {}
ItemDefinition.__index = ItemDefinition

---@class Type
---@field ItemDefinition integer

local typeInt = Enums.Type:createType("ItemDefinition")

local sharedMeta = {
    __index = ItemDefinition,
    __type = typeInt,
    __tostring = function(self)
        return Enums.Type:getName(typeInt)
    end
}

local classMeta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class ItemDefinition
---@field name string
---@field mass number
---@field energy number

---@class ItemDefinitionConstructor
---@field name string
---@field mass number
---@field energyDensity number

---@private
---@param args ItemDefinitionConstructor
---@return ItemDefinition|nil
function ItemDefinition:new(args)
    if not args.name then
        Log.Warn("No name Set for ItemDefinition")
        return nil
    end

    -- sets newItemDefinition and returns it
    local newItemDefinition = setmetatable({
        name = args.name,
        mass = args.mass,
        energy = Math.Round(math.max(0, (args.energyDensity or 1) * args.mass))
    }, sharedMeta)

    return newItemDefinition
end

setmetatable(ItemDefinition, classMeta)

return ItemDefinition
