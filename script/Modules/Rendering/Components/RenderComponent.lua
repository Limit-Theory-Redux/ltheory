local Component = require("Core.ECS.Component")

---@class MeshWithMaterial
---@field mesh Mesh
---@field material Material

---@class RenderComponent: Component
---@overload fun(self: RenderComponent, meshesOrRenderFn: MeshWithMaterial[]|function): RenderComponent subclass internal
---@overload fun(meshesOrRenderFn: MeshWithMaterial[]|function): RenderComponent subclass external
local RenderComponent = Subclass("RenderComponent", Component, function(self, meshesOrRenderFn)
    self:setComponentName("RenderComponent")

    if rawtype(meshesOrRenderFn) == 'function' then
        ---@cast meshesOrRenderFn function
        self:setRenderFn(meshesOrRenderFn)
    else
        ---@cast meshesOrRenderFn MeshWithMaterial[]
        self:setMeshes(meshesOrRenderFn)
    end
    self:setVisible(true)
end)

---@param meshes MeshWithMaterial[]
function RenderComponent:setMeshes(meshes)
    self.meshes = meshes
end

---@return MeshWithMaterial[]
function RenderComponent:getMeshes()
    return self.meshes
end

---@param renderFn function
function RenderComponent:setRenderFn(renderFn)
    self.renderFn = renderFn
end

---@return function|nil
function RenderComponent:getRenderFn()
    return self.renderFn
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
