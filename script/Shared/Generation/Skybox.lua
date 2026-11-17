-- The game's skybox: a nebula and a starfield generated from a seed, drawn
-- behind everything (Backdrop). It also sets the direction of the main light
-- (`CameraManager:setStarDir`) and the environment and irradiance maps the
-- materials reflect (`Renderer:setEnvironment`).

local Backdrop = require("Render.Backdrop")
local CoreComponents = require("Modules.Core.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local SkyboxEntity = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local Generator = require("Shared.Generation.Generator")
local Starfield = require("Shared.Generation.Starfield")

local Skybox = {}

---Generates the nebula, the stars and the light direction the first time it
---runs, then draws them (with `blendMode` nil it only generates).
---@param entity Entity
---@param blendMode BlendMode|nil
local function render(entity, blendMode)
    local placeholder = entity:get(CoreComponents.Empty)
    if not placeholder then
        placeholder = entity:add(CoreComponents.Empty)
    end

    if not placeholder.envMap then
        require("Shared.Generation.Nebula1")
        local nebulaRNG     = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
        local starAngle     = nebulaRNG:getDir2()
        placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
        placeholder.envMap  = Generator.Get('Nebula', nebulaRNG)(nebulaRNG, Config.gen.nebulaRes, placeholder.starDir)
        placeholder.irMap   = placeholder.envMap:genIRMap(256)
        placeholder.stars   = Starfield(nebulaRNG, Config.gen.nStars(nebulaRNG))
        CameraManager:setStarDir(placeholder.starDir)
        Renderer:setEnvironment(placeholder.envMap, placeholder.irMap)
    end

    Backdrop.draw(placeholder, blendMode)
end

---Creates the skybox entity for a seed and generates it right away: the
---generation renders to textures, and no render pass may be open while it
---does, so it can't wait for the first frame.
---@param seed integer
---@return Entity
function Skybox.Create(seed)
    local entity = SkyboxEntity(seed, render)
    render(entity, nil)
    return entity
end

return Skybox
