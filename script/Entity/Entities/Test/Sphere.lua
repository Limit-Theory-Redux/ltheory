local Entity = require('Entity.Entity')
local Material = require('Entity.Material')

local Sphere = subclass(Entity, function (self)
  local mesh = Gen.ShapeLib.BasicShapes.Ellipsoid():finalize()
  self:addRigidBody(true, mesh)
  self:addVisibleMesh(mesh, Material.Debug())
end)

return Sphere
