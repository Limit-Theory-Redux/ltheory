--- The backdrop of the scene passes: the nebula skybox in the opaque pass and the
--- star field in the additive one. Both are drawn through the open pass with
--- their own pipeline; the nebula maps (`envMap`, `irMap`) come from group 0
--- (`Renderer:setEnvironment`).
local Pipelines = require("Render.Pipelines")

local Backdrop = {}

local starParamsType

--- Draw the part of the backdrop that belongs to `blendMode`. `placeholder`
--- holds `stars` (a `Starfield`) next to the generated maps.
---@param placeholder table
---@param blendMode BlendMode|nil
function Backdrop.draw(placeholder, blendMode)
    if blendMode == BlendMode.Disabled then
        local shader = Cache.Shader('farplane', 'skybox')
        local pass = Renderer:currentPass()
        pass:setPipeline(Pipelines.get(shader, Pipelines.OpaqueBackdrop))
        Draw.Box3(Box3f(-1, -1, -1, 1, 1, 1))
    elseif blendMode == BlendMode.Additive then
        local shader = Cache.Shader('farplane', 'starbg')
        local pass = Renderer:currentPass()
        pass:setPipeline(Pipelines.get(shader, Pipelines.Additive))
        starParamsType = starParamsType or shader:blockType('StarBackgroundParams')
        pass:alloc(starParamsType).brightnessScale = 3
        placeholder.stars:draw()
    end
end

return Backdrop
