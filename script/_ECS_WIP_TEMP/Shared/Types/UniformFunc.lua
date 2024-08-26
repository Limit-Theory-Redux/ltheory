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
---@field type UniformType
---@field func function 

---@class UniformFuncConstructor
---@field type UniformType
---@field paramLen integer
---@field paramTypes ffi.ct*[]
---@field func function

---@private
---@param args UniformFuncConstructor
---@return UniformFunc|nil
function UniformFunc:new(args)
    if not args.type or not args.func then
        return nil
    end

    -- Type checks parameters going into Uniform Function
    local func = {
        __call = function(shader, uniformInt, ...) 
            if #{...} ~= args.paramLen then
                Log.Error("Incorrect number of Parameters for Function: " .. args.type)
            end
            for i,v in ipairs(...) do
                if ~ffi.istype(args.paramTypes[i],v) then
                    Log.Error("Incorrect parameter type for Function: " .. args.type .. 
                        "\n Expected: " .. args.paramTypes[i] .. ", Given: " .. v)
                end
            end
            args.func(shader, uniformInt, ...)
        end
    }

    local newUniformFunc = setmetatable({
        type = args.type,
        func = func,
    }, sharedMeta)
    


    return newUniformFunc
end

setmetatable(UniformFunc, classMeta)

return UniformFunc
