local Entity = require('Legacy.GameObjects.Entity')
local Material = require('Legacy.GameObjects.Material')
-- The mesh generators this file uses.
local Gen = {
    ShapeLib = { BasicShapes = require("Shared.Generation.Meshes.ShapeLib.BasicShapes"), Cluster = require("Shared.Generation.Meshes.ShapeLib.Cluster"), Joint = require("Shared.Generation.Meshes.ShapeLib.Joint"), JointField = require("Shared.Generation.Meshes.ShapeLib.JointField"), Module = require("Shared.Generation.Meshes.ShapeLib.Module"), Parametric = require("Shared.Generation.Meshes.ShapeLib.Parametric"), RandomShapes = require("Shared.Generation.Meshes.ShapeLib.RandomShapes"), Scaffolding = require("Shared.Generation.Meshes.ShapeLib.Scaffolding"), Shape = require("Shared.Generation.Meshes.ShapeLib.Shape"), Style = require("Shared.Generation.Meshes.ShapeLib.Style"), Warp = require("Shared.Generation.Meshes.ShapeLib.Warp") },
}


local Sphere = Subclass("Sphere", Entity, function(self)
    local mesh = Gen.ShapeLib.BasicShapes.Ellipsoid():finalize()
    self:addRigidBody(true, mesh)
    self:addVisibleMesh(mesh, Material.Debug())
end)

return Sphere
