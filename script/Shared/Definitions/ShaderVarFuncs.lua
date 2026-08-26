local RigidBodyComponent = require("Modules.Physics.Components.RigidBodyComponent")

local ShaderVarFuncs = {}

-- Persistent per-entity scratch matrices. getToWorldMatrixInto /
-- getToLocalMatrixInto write into a reused Matrix cdata instead of
-- allocating a managed Matrix* (with finalizer) per call. Without this,
-- every entity paid ~7 managed Matrix allocations per frame (world: 3,
-- IT: 4) even when the body never moved - the main GC churn source in
-- the menu render path. The scene is static there; the scratch survives
-- across frames and is overwritten in place each frame.
local scratchCache = {}

local function getScratch(entity)
    local id = entity.id
    local s = scratchCache[id]
    if not s then
        s = { world = Matrix(), worldIT = Matrix() }
        scratchCache[id] = s
    end
    return s
end

ShaderVarFuncs.mWorldFunc = function(eye, entity)
    local rb = entity:get(RigidBodyComponent):getRigidBody()
    local s = getScratch(entity)
    rb:getToWorldMatrixInto(eye, s.world)
    return s.world
end

ShaderVarFuncs.mWorldITFunc = function(eye, entity)
    local rb = entity:get(RigidBodyComponent):getRigidBody()
    local s = getScratch(entity)
    rb:getToLocalMatrixInto(eye, s.worldIT)
    return s.worldIT
end

ShaderVarFuncs.scaleFunc = function(eye, entity)
    local rb = entity:get(RigidBodyComponent):getRigidBody()
    return rb:getScale()
end

return ShaderVarFuncs
