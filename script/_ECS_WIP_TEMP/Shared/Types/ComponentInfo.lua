local ComponentInfo = {}
ComponentInfo.__index = ComponentInfo

local sharedMeta = {
    __index = ComponentInfo,
    __type = Enums.Type.ComponentInfo,
    __tostring = function(self)
        return Enums.Type:getName(Enums.Type.ComponentInfo)
    end
}

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
        return nil
    end

    local newComponentInfo = setmetatable({
        id = args.id,
        archetype = args.archetype,
    }, sharedMeta)

    return newComponentInfo
end

setmetatable(ComponentInfo, classMeta)

return ComponentInfo
