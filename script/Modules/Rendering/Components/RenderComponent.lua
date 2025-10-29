local Component = require("Core.ECS.Component")

---@class MeshWithMaterial
---@field mesh Mesh
---@field material Material

---@class RenderComponent: Component
---@overload fun(self: RenderComponent, meshes: MeshWithMaterial[]): RenderComponent subclass internal
---@overload fun(meshes: MeshWithMaterial[]): RenderComponent subclass external
local RenderComponent = Subclass("RenderComponent", Component, function(self, meshes)
    self:setComponentName("RenderComponent")

    self:setMeshes(meshes)
    self:setVisible(true)
end)

---@param meshes MeshWithMaterial[]
function RenderComponent:setMeshes(meshes)
    self.meshes = meshes
end

function RenderComponent:getMeshes()
    return self.meshes
end

---Sets Visibility of Mesh
---@param isVisible boolean
function RenderComponent:setVisible(isVisible)
    self.visibleMesh = isVisible
end

---@return boolean # Returns if Mesh is Visible
function RenderComponent:isVisible()
    return self.visibleMesh
end

return RenderComponent
