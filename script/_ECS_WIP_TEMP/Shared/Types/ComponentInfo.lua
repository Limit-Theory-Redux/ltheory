local ComponentInfo = {}
ComponentInfo.__index = ComponentInfo

-- Meta table for the ComponentInfo class
local classMeta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class ComponentInfo
---@field id integer
---@field archetype ComponentArchetype

---@class ComponentInfoConstructor
---@field id integer
---@field archetype ComponentArchetype

---@private
---@param args ComponentInfoConstructor
---@return ComponentInfo|nil
function ComponentInfo:new(args)
    if not args.id or not args.archetype then
        return
    end

    local newComponentInfo = setmetatable({}, {
        __index = self,
        __type = Enums.Type.ComponentInfo,
        __tostring = function(self)
            local mt = getmetatable(self)
            local typeName = Enums.Type:getName(mt.__type)
            return typeName
        end
    })
    newComponentInfo.id = args.id
    newComponentInfo.archetype = args.archetype

    return newComponentInfo
end

setmetatable(ComponentInfo, classMeta)

---@type ComponentInfoConstructor
ComponentInfo = ComponentInfo

return ComponentInfo
