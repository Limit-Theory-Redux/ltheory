local ShaderVarFuncs = {}

ShaderVarFuncs.mWorldFunc = function(renderState, entity)
    local rb = entity:findComponentByName("PhysicsRigidBody"):getRigidBody()
    return rb:getToWorldMatrix(renderState:getCameraEye())
end

ShaderVarFuncs.mWorldITFunc = function(renderState, entity)
    local rb = entity:findComponentByName("PhysicsRigidBody"):getRigidBody()
    return rb:getToLocalMatrix(renderState:getCameraEye())
end

ShaderVarFuncs.scaleFunc = function(renderState, entity)
    local rb = entity:findComponentByName("PhysicsRigidBody"):getRigidBody()
    return rb:getScale()
end

return ShaderVarFuncs