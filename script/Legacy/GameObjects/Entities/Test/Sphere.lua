local Entity = require('Legacy.GameObjects.Entity')
local Material = require('Legacy.GameObjects.Material')

local Sphere = Subclass("Sphere", Entity, function(self)
    local mesh = Gen.ShapeLib.BasicShapes.Ellipsoid():finalize()
    self:addRigidBody(true, mesh)
    self:addVisibleMesh(mesh, Material.Debug())
end)

return Sphere
