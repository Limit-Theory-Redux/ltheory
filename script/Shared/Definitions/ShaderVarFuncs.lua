local ShaderVarFuncs = {}

local RigidBodyComponent = require("Components.Physics.RigidBodyComponent")

ShaderVarFuncs.mWorldFunc = function(eye, entity)
    local rb = entity:getComponent(RigidBodyComponent):getRigidBody()
    return rb:getToWorldMatrix(eye)
end

ShaderVarFuncs.mWorldITFunc = function(eye, entity)
    local rb = entity:getComponent(RigidBodyComponent):getRigidBody()
    return rb:getToLocalMatrix(eye)
end

ShaderVarFuncs.scaleFunc = function(eye, entity)
    local rb = entity:getComponent(RigidBodyComponent):getRigidBody()
    return rb:getScale()
end

return ShaderVarFuncs