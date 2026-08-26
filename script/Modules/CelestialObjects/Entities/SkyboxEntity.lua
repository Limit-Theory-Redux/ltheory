local PhysicalEntity = require("Modules.PhysicalEntity")
local Rendering = require("Modules.Rendering.Components")

---@param seed integer
---@param meshesOrRenderFn MeshWithMaterial[]|function
---@return Entity
return function(seed, meshesOrRenderFn)
    return PhysicalEntity("SkyboxEntity", seed,
        Rendering.Render(meshesOrRenderFn))
end
