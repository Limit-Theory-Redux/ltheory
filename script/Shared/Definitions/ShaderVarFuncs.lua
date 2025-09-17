local RigidBodyComponent = require("Modules.Physics.Components.RigidBodyComponent")

local ShaderVarFuncs = {}

ShaderVarFuncs.mWorldFunc = function(eye, entity)
    local rb = entity:get(RigidBodyComponent):getRigidBody()
    return rb:getToWorldMatrix(eye)
end

ShaderVarFuncs.mWorldITFunc = function(eye, entity)
    local rb = entity:get(RigidBodyComponent):getRigidBody()
    return rb:getToLocalMatrix(eye)
end

ShaderVarFuncs.scaleFunc = function(eye, entity)
    local rb = entity:get(RigidBodyComponent):getRigidBody()
    return rb:getScale()
end

return ShaderVarFuncs
