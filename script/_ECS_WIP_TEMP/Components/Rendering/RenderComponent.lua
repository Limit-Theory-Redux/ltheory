local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp

---@class RenderComponent: Component
---@overload fun(self: RenderComponent, materials: Material[], meshType: MeshType): RenderComponent subclass internal
---@overload fun(materials: Material[], meshType: MeshType): RenderComponent subclass external
local RenderComponent = Subclass(Component, function(self, materials, meshType)
    self:setComponentName("RenderComponent")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.RenderComponent)

    -- Set RenderComponent Data
    self:setMaterials(materials)
    self:setMeshType(meshType)
    self:setVisible(true) -- Assume Mesh is Visible on Creation.

    -- Set RenderComponent Registered Events
end)

---@param materials Material[]
function RenderComponent:setMaterials(materials)
    self.materials = {}
    for _, v in pairs(materials) do
        self.materials[v.blendMode] = v
    end
end

---@param meshType MeshType
function RenderComponent:setMeshType(meshType)
    self.meshType = meshType
end

---@param blendMode BlendMode
---@return Material|nil
function RenderComponent:getMaterial(blendMode)
    return self.materials[blendMode]
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