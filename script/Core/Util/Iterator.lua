---@generic V
---@param t table<any, V>
---@return fun(): V
function Iterator(t)
    local keys = {}
    for key in pairs(t) do
        table.insert(keys, key)
    end
    local i = 0
    local n = #keys
    return function()
        i = i + 1
        if i <= n then return t[keys[i]] end
    end
end

---@generic V
---@param t table<any, V>
---@return fun(): integer, V
function IteratorIndexed(t)
    local keys = {}
    for key in pairs(t) do
        table.insert(keys, key)
    end
    local i = 0
    local n = #keys
    return function()
        i = i + 1
        if i <= n then return keys[i], t[keys[i]] end
    end
end

--[[
    example of usage:

    function Entity:iterAssets()
        return Iterator(self:getAssets())
    end
]]
