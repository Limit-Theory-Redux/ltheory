local Entity = require('Legacy.GameObjects.Entity')
local RenderComponent = require('Modules.Rendering.Components.RenderComponent')

function Entity:addVisibleMesh(mesh, material)
    assert(mesh)
    assert(material)

    self.entity:add(RenderComponent({
        { mesh = mesh, material = material },
    }))
end

---@return Mesh|nil
function Entity:getVisibleMesh()
    local renderComponent = self.entity:get(RenderComponent)
    if not renderComponent then return nil end

    local meshes = renderComponent:getMeshes()
    return meshes and meshes[1] and meshes[1].mesh
end

function Entity:setRenderVisibleMesh(enabled, cullingLock)
    local renderComponent = self.entity:get(RenderComponent)
    if renderComponent then
        renderComponent:setVisible(enabled)
    end
end
