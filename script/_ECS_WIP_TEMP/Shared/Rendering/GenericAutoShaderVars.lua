local GenericAutoShaderVar = {}

GenericAutoShaderVar.mWorldFunc = function(renderState, shaderState, uniformInt, entity)
    local rb = entity:findComponentByName("PhysicsRigidBody"):getRigidBody()
    local val = rb:getToWorldMatrix(renderState:getCameraEye())
    shaderState:shader():iSetMatrix(uniformInt, val)
end

GenericAutoShaderVar.mWorldITFunc = function(renderState, shaderState, uniformInt, entity)
    local rb = entity:findComponentByName("PhysicsRigidBody"):getRigidBody()
    local val = rb:getToLocalMatrix(renderState:getCameraEye())
    shaderState:shader():iSetMatrixT(uniformInt, val)
end

GenericAutoShaderVar.scaleFunc = function(renderState, shaderState, uniformInt, entity)
    local rb = entity:findComponentByName("PhysicsRigidBody"):getRigidBody()
    local val = rb:getScale()
    shaderState:shader():iSetFloat(uniformInt, val)
end

return GenericAutoShaderVar