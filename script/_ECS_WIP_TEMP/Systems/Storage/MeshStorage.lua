local Log = require("Core/Util/Log")

---@class MeshStorage
---@field meshes table<MeshType, table<integer, Mesh>>
---@field initialized boolean

--!TODO: This is only for LLS purposes, as FFI doesnt show to luaffi
---@class Mesh

---@class MeshStorage
---@overload fun(self: MeshStorage): MeshStorage class internal
---@overload fun(): MeshStorage class external
local MeshStorage = Class(function(self)
    if self.initialized then
        Log.Error("You are trying to reinitialize the MeshStorage, this should not happen.")
        return
    end

    self:initStorage()

    self.initialized = true
end)

function MeshStorage:initStorage()
    self.meshes = {}

    for _, meshType in pairs(Enums.MeshType) do
        self.meshes[meshType] = {}
        SetLengthMetamethod(self.meshes[meshType])
    end

    Log.Info("Initialized MeshStorage")
end

--TODO: "seed" may need to be replaced if we implement RNG Cache
--TODO: should this return meshType or mesh? Does it matter in this context?
---@param meshType MeshType
---@param seed integer
---@param mesh Mesh
---@return MeshType|nil
function MeshStorage:storeMesh(meshType, seed, mesh)
    if not self.meshes[meshType] then
        Log.Error("Did not provide a valid MeshType for mesh: " .. 
            tostring(meshType) .. "\n".. Inspect(mesh))
    end
    self.meshes[meshType][seed] = mesh
    return meshType
end

---@param meshType MeshType
---@param seed integer
---@return boolean wasSuccessful
function MeshStorage:dropMesh(meshType, seed)
    local mesh = self.meshes[meshType][seed]

    if mesh then
        --TODO: Verify this properly cleans up data
        mesh:free()
        self.meshes[meshType][seed] = nil
        return true
    end
    return false
end

---@param meshType MeshType
---@param seed integer
---@return Mesh|nil
function MeshStorage:getMesh(meshType, seed)
    if not self.meshes[meshType] then
        Log.Error("Did not provide a valid MeshType for mesh: " .. 
            tostring(meshType) .. ", seed:".. tostring(seed))
    end

    return MeshStorage.meshes[meshType][seed]
end

--!DEBUG - To get all Meshes from Storage, only for debug
function MeshStorage:getMeshes()
    return self.meshes
end

return MeshStorage()