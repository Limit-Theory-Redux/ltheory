---@class Items
local Items = {}
---@private
Items.__index = Items

---@param name string
---@param itemGroup ItemGroup
function Items:new(name, itemGroup)
    name = name:gsub(" ", "")
    self[name] = {}

    for _, def in ipairs(itemGroup.items) do
        def.name = def.name:gsub(" ", "")
        self[name][def.name] = def
    end
    return self[name]
end

---@return Items
return Items
