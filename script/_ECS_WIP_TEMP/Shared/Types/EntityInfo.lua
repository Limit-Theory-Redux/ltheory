local EntityInfo = {}
EntityInfo.__index = EntityInfo

---@class Type
---@field EntityInfo integer

local typeInt = Enums.Type:createType("EntityInfo")

local sharedMeta = {
    __index = EntityInfo,
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
