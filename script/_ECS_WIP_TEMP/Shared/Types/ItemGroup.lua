local Items = require("_ECS_WIP_TEMP.Shared.Registries.Items")

---@class ItemGroup
---@field name string
---@field items ItemDefinition[]
---@overload fun(args: {name: string, items: ItemDefinition[]}): ItemGroup
local ItemGroup = Class("ItemGroup")

function ItemGroup.new(args)
    if not args.name then
        Log.Warn("No name set for ItemGroup")
        return nil
    elseif Items[args.name] then
        Log.Warn("Attempting to Recreate ItemGroup: " .. args.name)
        return Items[args.name]
    end

    local self = setmetatable({
        name = args.name,
        items = args.items
    }, ItemGroup)

    -- Add New ItemDef to Item Registery
    Items:new(args.name, self)

    return self
end

return ItemGroup
