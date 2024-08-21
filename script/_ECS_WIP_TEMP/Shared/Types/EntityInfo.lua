local EntityInfo = {}
EntityInfo.__index = EntityInfo

-- Meta table for the EntityInfo class
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
        return
    end

    local newEntityInfo = setmetatable({}, {
        __index = self,
        __type = Enums.Type.EntityInfo,
        __tostring = function(self)
            local mt = getmetatable(self)
            local typeName = Enums.Type:getName(mt.__type)
            return typeName
        end
    })
    newEntityInfo.id = args.id
    newEntityInfo.archetype = args.archetype

    return newEntityInfo
end

setmetatable(EntityInfo, classMeta)

---@type EntityInfoConstructor
EntityInfo = EntityInfo

return EntityInfo
