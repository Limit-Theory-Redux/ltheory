--- Mesh-based UI paths of the system maps (the asteroid dots and the 3D orbit
--- trails) next to a ring and an annulus: draws them with the same pipelines and
--- `Params` blocks as `SystemMap` / `SystemMap3D`, for render validation.
local Test = require('States.Application')
local Pipelines = require('Render.Pipelines')
local ffi = require('ffi')

function Test:getTitle() return 'UI Maps' end

function Test:onInit()
    local dots = Mesh.Create()
    local n = 0
    for i = 0, 63 do
        local a = i / 64 * 2 * math.pi
        local x, z = math.cos(a) * 150, math.sin(a) * 150
        dots:addVertex(x, z, 0, 0, 0, 1, 0, 0)
        dots:addVertex(x, z, 0, 0, 0, 1, 1, 0)
        dots:addVertex(x, z, 0, 0, 0, 1, 1, 1)
        dots:addVertex(x, z, 0, 0, 0, 1, 0, 1)
        dots:addQuad(n * 4, n * 4 + 1, n * 4 + 2, n * 4 + 3)
        n = n + 1
    end
    self.dots = dots

    -- A fading trail ribbon (alpha in uv.x), in the xz plane.
    local trail = Mesh.Create()
    local len = 40
    for i = 1, len - 1 do
        local t1, t2 = (i - 1) / (len - 1), i / (len - 1)
        local function p(t) return math.cos(t * 4) * 3, math.sin(t * 4) * 3 end
        local x1, z1 = p(t1)
        local x2, z2 = p(t2)
        local w1, w2 = 0.15 * t1, 0.15 * t2
        local b = (i - 1) * 4
        trail:addVertex(x1 - w1, 0, z1, 0, 1, 0, t1, 0)
        trail:addVertex(x1 + w1, 0, z1, 0, 1, 0, t1, 0)
        trail:addVertex(x2 + w2, 0, z2, 0, 1, 0, t2, 0)
        trail:addVertex(x2 - w2, 0, z2, 0, 1, 0, t2, 0)
        trail:addQuad(b, b + 1, b + 2, b + 3)
    end
    self.trail = trail
end

function Test:onInput() end
function Test:onUpdate(dt) end

local alpha = { blend = BlendMode.Alpha }
local additive = { blend = BlendMode.Additive }

function Test:onRender()
    self:immediateUI(function()
        local pass = Renderer:currentPass()

        -- Asteroid dots (SystemMap).
        local dotShader = Cache.Shader('mappoints', 'ui/mappoints')
        pass:setPipeline(Pipelines.get(dotShader, alpha))
        local p = pass:alloc(dotShader:blockType('Params'))
        p.mapGeom.x, p.mapGeom.y, p.mapGeom.z, p.mapGeom.w = 320, 360, self.resX, self.resY
        p.mapParams.x, p.mapParams.y = 1.0, 3.0
        p.dotColor.x, p.dotColor.y, p.dotColor.z, p.dotColor.w = 0.8, 0.6, 0.3, 0.7
        pass:drawMesh(self.dots)

        -- Orbit ring and belt annulus (SystemMap).
        local c = Color(0.8, 0.8, 0.9, 0.35)
        local r, pad = 100, 8
        Imm.Shape(Shape.Ring, 320 - r - pad, 360 - r - pad, 2 * r + 2 * pad, 2 * r + 2 * pad, c, r)
        local ca = Color(0.6, 0.4, 0.2, 0.25)
        local ri, ro = 180, 200
        Imm.Shape(Shape.Annulus, 320 - ro - 4, 360 - ro - 4, 2 * ro + 8, 2 * ro + 8, ca, ri, ro)

        -- Orbit trail (SystemMap3D): the map's own view and projection.
        local trailShader = Cache.Shader('hologram3d', 'ui/trail3d')
        pass:setPipeline(Pipelines.get(trailShader, additive))
        local t = pass:alloc(trailShader:blockType('Params'))
        local view = Matrix.LookAt(Vec3f(0, 12, 0.01), Vec3f(0, 0, 0), Vec3f(0, 0, -1))
        local proj = Matrix.Perspective(60, self.resX / self.resY, 0.1, 100)
        ffi.copy(t.trailView, view.m, 64)
        ffi.copy(t.trailProj, proj.m, 64)
        t.trailColor.x, t.trailColor.y, t.trailColor.z, t.trailColor.w = 0.7, 0.7, 0.9, 0.9
        pass:drawMesh(self.trail)

        -- Debug lines and a point through the 3D debug pipeline (camera-relative).
        Renderer:setCamera(Matrix.Identity(), proj, Vec3f(0, 1, 0))
        Imm.Line3(Vec3f(-1, -1, -3), Vec3f(1, 1, -3), Color(1, 0, 0, 1), 6, false)
        Imm.Line3(Vec3f(-1, 1, -3), Vec3f(1, -1, -3), Color(0, 1, 0, 1), 3, false)
        Imm.Point3(Vec3f(0, 0, -3), Color(0, 0, 1, 1), 14, false)
    end)
end

return Test
