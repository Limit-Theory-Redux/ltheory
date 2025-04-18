local UniformFuncs = require("Shared.Rendering.UniformFuncs")

---@class UniformFunc
---@field funcType UniformType
---@field func function 
---@overload fun(args: {funcType: UniformType, func: function}): UniformFunc
local UniformFunc = Class("UniformFunc")

function UniformFunc.new(args)
    if not args.funcType or not args.func then
        return nil
    end
    
    UniformFuncs:new(args.funcType, args.func)

    return setmetatable({
        funcType = args.funcType,
        func = args.func,
    }, UniformFunc)
end

return UniformFunc
