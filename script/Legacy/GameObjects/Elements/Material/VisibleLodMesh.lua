local Entity = require('Legacy.GameObjects.Entity')

function Entity:addVisibleLodMesh(mesh, material)
    assert(not self.mesh)
    assert(mesh)
    assert(material)
    self.mesh = mesh
    self.material = material
    self:register(OldEvent.Render, Entity.renderVisibleLodMesh)
end

function Entity:renderVisibleLodMesh(state)
    if state.mode == BlendMode.Disabled then
        self.material:start()
        self.material:setState(self.body, state.eye)
        self.mesh:draw(state.eye:distanceSquared(self:getPos()) / (self:getScale() ^ 2.0))
        self.material:stop()
    end
end
