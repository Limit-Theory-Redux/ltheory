local ShaderVarFuncs = {}

ShaderVarFuncs.mWorldFunc = function(renderState, entity)
    local rb = entity:findComponentByArchetype(Enums.ComponentArchetype.RigidBodyComponent):getRigidBody()
    return rb:getToWorldMatrix(renderState:getCameraEye())
end

ShaderVarFuncs.mWorldITFunc = function(renderState, entity)
    local rb = entity:findComponentByArchetype(Enums.ComponentArchetype.RigidBodyComponent):getRigidBody()
    return rb:getToLocalMatrix(renderState:getCameraEye())
end

ShaderVarFuncs.scaleFunc = function(renderState, entity)
    local rb = entity:findComponentByArchetype(Enums.ComponentArchetype.RigidBodyComponent):getRigidBody()
    return rb:getScale()
end

return ShaderVarFuncs