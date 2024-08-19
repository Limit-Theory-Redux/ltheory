local Component = require('Component')

---@class MeshComponent: Component
---@overload fun(mesh: Mesh, material: Material): MeshComponent subclass external
local MeshComponent = Subclass(Component, function(self, mesh, material)
    ---@cast self MeshComponent
    self:setComponentName("RenderingMesh")
end)

return MeshComponent
