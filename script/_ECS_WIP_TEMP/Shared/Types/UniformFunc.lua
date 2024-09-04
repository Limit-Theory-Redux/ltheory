local UniformFuncs = require("_ECS_WIP_TEMP.Shared.Rendering.UniformFuncs")
local UniformFunc = {}
UniformFunc.__index = UniformFunc

---@class Type
---@field UniformFunc integer

local typeInt = Enums.Type:createType("UniformFunc")

local sharedMeta = {
    __index = UniformFunc,
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

---@class UniformFunc
---@field funcType UniformType
---@field func function 

---@class UniformFuncConstructor
---@field funcType UniformType
---@field func function

---@private
---@param args UniformFuncConstructor
---@return UniformFunc|nil
function UniformFunc:new(args)
    if not args.funcType or not args.func then
        return nil
    end
    
    UniformFuncs:new(args.funcType, args.func)

    local newUniformFunc = setmetatable({
        funcType = args.funcType,
        func = args.func,
    }, sharedMeta)
    return newUniformFunc
end

setmetatable(UniformFunc, classMeta)

return UniformFunc
