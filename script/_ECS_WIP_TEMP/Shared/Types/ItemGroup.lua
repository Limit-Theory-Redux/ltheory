local Items = require("_ECS_WIP_TEMP.Shared.Registries.Items")

local ItemGroup = {}
ItemGroup.__index = ItemGroup

---@class Type
---@field ItemGroup integer

local typeInt = Enums.Type:createType("ItemGroup")

local sharedMeta = {
    __index = ItemGroup,
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

---@class ItemGroup
---@field name string
---@field items ItemDefinition[]

---@class ItemGroupConstructor
---@field name string
---@field items ItemDefinition[]

---@private
---@param args ItemGroupConstructor
---@return ItemGroup|nil
function ItemGroup:new(args)
    if not args.name then
        Log.Warn("No name set for ItemGroup")
        return nil
    elseif Items[args.name] then
        Log.Warn("Attempting to Recreate ItemGroup: " .. args.name)
        return Items[args.name]
    end

    -- sets newItemGroup and returns it
    local newItemGroup = setmetatable({
        name = args.name,
        items = args.items
    }, sharedMeta)

    -- Add New ItemDef to Item Registery
    Items:new(args.name, newItemGroup)

    return newItemGroup
end

setmetatable(ItemGroup, classMeta)

return ItemGroup
