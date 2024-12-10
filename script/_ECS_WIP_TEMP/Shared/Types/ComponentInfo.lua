local ComponentInfo = {}
ComponentInfo.__index = ComponentInfo

---@class Type
---@field ComponentInfo integer

local typeInt = Enums.Type:createType("ComponentInfo")

local sharedMeta = {
    __index = ComponentInfo,
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

---@class ComponentInfo
---@field id integer
---@field archetype ComponentArchetype
---@field entity EntityInfo

---@class ComponentInfoConstructor
---@field id integer
---@field archetype ComponentArchetype
---@field entity EntityInfo

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
        entity = args.entity
    }, sharedMeta)

    return newComponentInfo
end

setmetatable(ComponentInfo, classMeta)

return ComponentInfo
