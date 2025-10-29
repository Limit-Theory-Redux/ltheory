---@class GenerationContext
---@field data table<string, any>
local GenerationContext = Class("GenerationContext", function(self)
    self.data = {}
end)

---@param key string
---@param value any
function GenerationContext:set(key, value)
    self.data[key] = value
end

---@param key string
---@return any
function GenerationContext:get(key)
    return self.data[key]
end

---@return table<string, any> data
function GenerationContext:getAll()
    return self.data
end

return GenerationContext
