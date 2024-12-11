---@class Items
local Items = {}
---@private
Items.__index = Items

local itemId = 0

---@param name string
---@param itemGroup ItemGroup
function Items:new(name, itemGroup)
    name = name:gsub(" ", "")
    self[name] = {}

    for _, def in ipairs(itemGroup.items) do
        itemId = itemId + 1
        def.id = itemId
        def.name = def.name:gsub(" ", "")
        def.group = name
        self[name][def.name] = def -- for string api e.g. Items.Virtual.Credit
        self[itemId] = def         -- for comp (more memory usage vs string comparison computation)
    end
    return self[name]
end

---@return Items
return Items
