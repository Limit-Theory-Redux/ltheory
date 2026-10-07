local Pipelines = require('Render.Pipelines')
local ffi = require('ffi')
local Entity = require('Legacy.GameObjects.Entity')

local Pulse = CType.Struct('Pulse')
Pulse:add(CType.Int32, 'source')
Pulse:add(CType.Int32, 'type')
Pulse:add(CType.Position, 'pos')
Pulse:add(CType.Vec3f, 'vel')
Pulse:add(CType.Vec3f, 'dir')
Pulse:add(CType.Float32, 'lifeMax')
Pulse:add(CType.Float32, 'life')
Pulse:add(CType.Float32, 'dist')
Pulse:add(CType.Matrix, 'matrix')

local meshHead
local meshTail
local shaderHead
local shaderTail
local shaderMissileHead
local shaderMissileTail
local shaderLaserBoltHead
local shaderLaserBoltTail
local DrawBlock

Preload.Add(function()
    meshHead = Gen.Primitive.Billboard(-1, -1, 1, 1)
    meshTail = Gen.Primitive.Billboard(-1, -1, 1, 0)
    shaderHead = Cache.Shader('billboard/quad_draw', 'effect/pulsehead_draw')
    shaderTail = Cache.Shader('billboard/axis_draw', 'effect/pulsetail_draw')
    shaderMissileHead = Cache.Shader('billboard/quad_draw', 'effect/missilehead_draw')
    shaderMissileTail = Cache.Shader('billboard/axis_draw', 'effect/missiletail_draw')
    shaderLaserBoltHead = Cache.Shader('billboard/quad_draw', 'effect/laserbolthead_draw')
    shaderLaserBoltTail = Cache.Shader('billboard/axis_draw', 'effect/laserbolttail_draw')
    DrawBlock = shaderHead:blockType('DrawBlock')
end)

Pulse:setInitializer(function(self)
    self.matrix = Matrix.Identity()
end)

Pulse:addOnDestruct(function(self)
    DecRef(self.source)
end)

Pulse:define()

function Pulse:refreshMatrix(eye)
    self.matrix = Matrix.LookUp(self.pos:relativeTo(eye), -self.dir, Math.OrthoVector(self.dir))
end

local function isPulse(proj)
    return proj.shaderKey ~= 'missile' and proj.shaderKey ~= 'laserbolt'
end

local function isMissile(proj)
    return proj.shaderKey == 'missile'
end

local function isLaserBolt(proj)
    return proj.shaderKey == 'laserbolt'
end

--- Draw `mesh` once per projectile `want` selects, in the additive pass: the
--- pipeline carries the shader and the additive scene state, the draw block
--- the transform (`drawUser[0]` color and alpha, `drawUser[1]` sprite size).
local function drawSprites(pass, shader, mesh, projectiles, want, fillSize)
    pass:setPipeline(Pipelines.get(shader, Pipelines.Additive))
    for i = 1, #projectiles do
        local proj = projectiles[i]
        if want(proj) then
            local pulse = proj.effect
            local d = pass:alloc(DrawBlock)
            ffi.copy(d.mWorld, pulse.matrix.m, 64)
            local user = d.drawUser
            user[0], user[1], user[2] = proj.pColorR, proj.pColorG, proj.pColorB
            user[3] = pulse.life / pulse.lifeMax
            fillSize(user, proj, pulse)
            pass:drawMesh(mesh)
        end
    end
end

function Pulse.Render(projectiles, state)
    if state.mode == BlendMode.Additive then
        do -- Recalculate matrices.
            for i = 1, #projectiles do
                local proj  = projectiles[i]
                local pulse = proj.effect
                pulse:refreshMatrix(state.eye)
            end
        end

        local pass = Renderer:currentPass()

        local function headSize(user, proj)
            user[4] = proj.pulseHeadSize or state.headSize or 16
        end
        local function tailSize(user, proj, pulse)
            user[4] = proj.pulseTailWidth or state.tailWidth or 16
            user[5] = min(proj.pulseTailLength or state.tailLength or Config.gen.compTurretPulseStats.size,
                1.5 * pulse.dist)
        end

        do -- Heads
            Profiler.Begin('Pulse.RenderAdditive.Head')
            drawSprites(pass, shaderHead, meshHead, projectiles, isPulse, headSize)
            drawSprites(pass, shaderMissileHead, meshHead, projectiles, isMissile, headSize)
            drawSprites(pass, shaderLaserBoltHead, meshHead, projectiles, isLaserBolt, headSize)
            Profiler.End()
        end

        do -- Tails
            Profiler.Begin('Pulse.RenderAdditive.Tail')
            drawSprites(pass, shaderTail, meshTail, projectiles, isPulse, tailSize)
            drawSprites(pass, shaderMissileTail, meshTail, projectiles, isMissile, tailSize)
            drawSprites(pass, shaderLaserBoltTail, meshTail, projectiles, isLaserBolt, tailSize)
            Profiler.End()
        end
    end
end

function Pulse.UpdatePrePhysics(system, projectiles, dt)
    Profiler.Begin('Pulse.UpdatePre')
    local t = 1.0 - exp(-dt)
    for i = #projectiles, 1, -1 do
        local proj  = projectiles[i]
        local pulse = proj.effect
        pulse.life  = pulse.life - dt
        if pulse.life <= 0 then
            --Log.Debug("PULSE: projectile delete on expiration = %s", projectiles[i]:getName())
            if proj then
                proj:deleteLight(proj)
            end
            projectiles[i] = projectiles[#projectiles]
            projectiles[#projectiles] = nil
            pulse:delete()
        else
            pulse.pos:imadds(pulse.vel, dt)
            pulse.dir:ilerp(pulse.vel:normalize(), t) -- not needed for dumb-fire projectiles, but retained
            pulse.dist = pulse.dist + dt * Config.gen.compTurretPulseStats.speed
        end
    end
    Profiler.End()
end

function Pulse.UpdatePostPhysics(system, projectiles, dt)
    Profiler.Begin('Pulse.UpdatePostPhysics')
    local restitution = 0.4 * Config.gen.compTurretPulseStats.size
    local ray = Ray()
    ray.tMin = 0
    ray.tMax = 1

    for i = #projectiles, 1, -1 do
        local pulse = projectiles[i].effect

        -- raycast
        ray.px = pulse.pos.x
        ray.py = pulse.pos.y
        ray.pz = pulse.pos.z
        ray.dirx = dt * pulse.vel.x
        ray.diry = dt * pulse.vel.y
        ray.dirz = dt * pulse.vel.z
        local hit = system.physics:rayCast(ray).body

        if hit ~= nil then
            -- Get parent rigid body
            while hit:getParentBody() ~= nil do hit = hit:getParentBody() end
            local hitEnt = Entity.fromRigidBody(hit)
            local source = Deref(pulse.source)
            -- TODO: This hitEnt nil check fixes a bug in PhysicsTest.lua. For some reason these two objects do not
            -- return anything fromRigidBody for the first few seconds. While this is a good check to do since
            -- we cannot confirm that the hit will have a rigidbody. This is a hotfix for a weird error.
            if (hitEnt ~= nil) then
                -- Don't collide with the socket that spawned me
                if hitEnt ~= source then
                    -- Do damage if the collidee has health
                    if hitEnt:isAlive() then
                        -- TODO: Get damage type and amount from the pulse
                        hitEnt:applyDamage(Config.gen.compTurretPulseStats.damage, source)
                    end

                    -- Remove projectile
                    --Log.Debug("PULSE: projectile delete on hit = %s", projectiles[i]:getName())
                    if projectiles[i] then
                        projectiles[i]:deleteLight(projectiles[i])
                    end
                    projectiles[i] = projectiles[#projectiles]
                    projectiles[#projectiles] = nil
                    pulse:delete()
                end
            end
        end
    end

    Profiler.End()
end

return Pulse
