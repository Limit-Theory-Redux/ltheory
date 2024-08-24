local Log = require("Core/Util/Log")

---@class MaterialStorage
---@field materials table<MaterialType, Material>
---@field initialized boolean

---@class MaterialStorage
---@overload fun(self: MaterialStorage): MaterialStorage class internal
---@overload fun(): MaterialStorage class external
local MaterialStorage = Class(function(self)
    if self.initialized then
        Log.Error("You are trying to reinitialize the MeshStorage, this should not happen.")
        return
    end

    self:initStorage()

    self.initialized = true
end)

function MaterialStorage:initStorage()
    self.materials = {}

    -- k: MaterialType Key
    -- v: MaterialType Value
    -- Defined like this for initializing materials here.
    for k, v in pairs(Enums.MaterialType) do
        -- Initialize as nil?
        self.materials[v] = nil
        -- Or Initialize all Materials?
        -- self.materials[v] = CreateMaterial:Initialize(k)
    end

    Log.Info("Initialized MaterialStorage")
end

--TODO: should this return meshType or mesh? Does it matter in this context?
--TODO: Is this necessary if Materials are initialized in initStorage?
---@param materialType MaterialType
---@param material Material
---@return MaterialType|nil
function MaterialStorage:storeMaterial(materialType, material)
    if not self.materials[materialType] then
        Log.Error("Did not provide a valid MaterialType for material: " .. 
        tostring(materialType) .. "\n".. Inspect(material))
    end
    self.materials[materialType] = material
    return materialType
end

--TODO: should this be necessary if we're initializing materials in initStorage?
---@param materialType MeshType
---@return boolean wasSuccessful
function MaterialStorage:dropMaterial(materialType)
    local mesh = self.materials[materialType]

    if mesh then
        --TODO: Verify this properly cleans up data
        self.materials[materialType] = nil
        return true
    end
    return false
end

---@param materialType MaterialType
---@return Material|nil
function MaterialStorage:getMaterial(materialType)
    if not self.materials[materialType] then
        Log.Error("Did not provide a valid MaterialType for material: " .. 
            tostring(materialType))
    end

    return MaterialStorage.materials[materialType]
end

--!DEBUG - To get all Meshes from Storage, only for debug
function MaterialStorage:getMaterials()
    return self.materials
end

return MaterialStorage()