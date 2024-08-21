local EntityInfo = {}
EntityInfo.__index = EntityInfo

local sharedMeta = {
    __index = EntityInfo,
    __type = Enums.Type.EntityInfo,
    __tostring = function(self)
        return Enums.Type:getName(Enums.Type.EntityInfo)
    end
}

local classMeta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class EntityInfo
---@field id integer
---@field archetype EntityArchetype

---@class EntityInfoConstructor
---@field id integer
---@field archetype EntityArchetype

---@private
---@param args EntityInfoConstructor
---@return EntityInfo|nil
function EntityInfo:new(args)
    if not args.id or not args.archetype then
        return nil
    end

    local newEntityInfo = setmetatable({
        id = args.id,
        archetype = args.archetype,
    }, sharedMeta)

    return newEntityInfo
end

setmetatable(EntityInfo, classMeta)

return EntityInfo
