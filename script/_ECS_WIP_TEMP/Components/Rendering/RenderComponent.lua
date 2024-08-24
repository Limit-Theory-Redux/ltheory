local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp

---@class RenderComponent: Component
---@overload fun(self: RenderComponent, materialType: MaterialType, meshType: MeshType): RenderComponent subclass internal
---@overload fun(materialType: MaterialType, meshType: MeshType): RenderComponent subclass external
local RenderComponent = Subclass(Component, function(self, materialType, meshType)
    self:setComponentName("RenderComponent")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.RenderComponent)

    -- Set RenderComponent Data
    self:setMaterialType(materialType)
    self:setMeshType(meshType)
    self:setVisible(true) -- Assume Mesh is Visible on Creation.

    -- Set RenderComponent Registered Events
end)

---@param materialType MaterialType
function RenderComponent:setMaterialType(materialType)
    self.materialType = materialType
end

---@param meshType MeshType
function RenderComponent:setMeshType(meshType)
    self.meshType = meshType
end

---@return MaterialType
function RenderComponent:getMaterialType()
    return self.materialType
end

---@return MeshType
function RenderComponent:getMeshType()
    return self.meshType
end

---Sets Visibility of Mesh
---@param isVisible boolean
function RenderComponent:setVisible(isVisible)
    self.visibleMesh = isVisible
end

---@return boolean # Returns if Mesh is Visible
function RenderComponent:getVisible()
    return self.visibleMesh
end

return RenderComponent
