local Entity = require('Legacy.GameObjects.Entity')
local RenderComponent = require('Modules.Rendering.Components.RenderComponent')

function Entity:addVisibleMesh(mesh, material)
    assert(mesh)
    assert(material)

    self.entity:add(RenderComponent({ material }, mesh))
end

function Entity:setRenderVisibleMesh(enabled, cullingLock)
    local renderComponent = self.entity:get(RenderComponent)
    if renderComponent then
        renderComponent:setVisible(enabled)
    end
end
