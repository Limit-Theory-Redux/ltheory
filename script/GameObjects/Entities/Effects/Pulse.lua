local Entity = require('GameObjects.Entity')

local Pulse = CType.Struct('Pulse')
Pulse:add(CType.Int32, 'source')
Pulse:add(CType.Int32, 'type')
Pulse:add(CType.Vec3f, 'pos')
Pulse:add(CType.Vec3f, 'vel')
Pulse:add(CType.Vec3f, 'dir')
Pulse:add(CType.Float32, 'lifeMax')
Pulse:add(CType.Float32, 'life')
Pulse:add(CType.Float32, 'dist')
Pulse:add(CType.Pointer(CType.Matrix), 'matrix')

local cacheHead
local cacheTail
local meshHead
local meshTail
local shaderHead
local shaderTail
local onAddedToParent

Preload.Add(function()
    meshHead = Gen.Primitive.Billboard(-1, -1, 1, 1)
    meshTail = Gen.Primitive.Billboard(-1, -1, 1, 0)
    shaderHead = Cache.Shader('billboard/quad', 'effect/pulsehead')
    shaderTail = Cache.Shader('billboard/axis', 'effect/pulsetail')
    cacheHead = ShaderVarCache(shaderHead, { 'size', 'alpha', 'mWorld' })
    cacheTail = ShaderVarCache(shaderTail, { 'alpha', 'size', 'axis', 'mWorld' })
end)

Pulse:setInitializer(function(self)
    self.matrix = Matrix.Identity()
    self:register(Event.AddedToParent, onAddedToParent)
end)

Pulse:addOnDestruct(function(self)
    self.matrix:free()
    DecRef(self.source)
end)

Pulse:define()

onAddedToParent = function(self, parent)
    self:refreshMatrix()
end

function Pulse:refreshMatrix()
    self.matrix:free()
    self.matrix = Matrix.LookUp(self.pos, -self.dir, Math.OrthoVector(self.dir))
end

function Pulse.Render(projectiles, state)
    if state.mode == BlendMode.Additive then
        do -- Heads
            Profiler.Begin('Pulse.RenderAdditive.Head')
            local shader = shaderHead
            shader:start()
            meshHead:drawBind()
            for i = 1, #projectiles do
                local proj  = projectiles[i]
                local pulse = proj.effect
                if proj then
                    Shader.SetFloat3('color', proj.pColorR,
                        proj.pColorG,
                        proj.pColorB)
                else
                    Shader.SetFloat3('color', Config.game.pulseColorBodyR,
                        Config.game.pulseColorBodyG,
                        Config.game.pulseColorBodyB)
                end
                Shader.ISetFloat(cacheHead.size, 16)
                Shader.ISetFloat(cacheHead.alpha, pulse.life / pulse.lifeMax)
                Shader.ISetMatrix(cacheHead.mWorld, pulse.matrix)
                meshHead:drawBound()
            end
            meshHead:drawUnbind()
            shader:stop()
            Profiler.End()
        end

        do -- Tails
            Profiler.Begin('Pulse.RenderAdditive.Tail')
            local shader = shaderTail
            shader:start()
            meshTail:drawBind()
            for i = 1, #projectiles do
                local proj  = projectiles[i]
                local pulse = proj.effect
                if proj then
                    Shader.SetFloat3('color', proj.pColorR,
                        proj.pColorG,
                        proj.pColorB)
                else
                    Shader.SetFloat3('color', Config.game.pulseColorBodyR,
                        Config.game.pulseColorBodyG,
                        Config.game.pulseColorBodyB)
                end
                Shader.ISetFloat(cacheTail.alpha, pulse.life / pulse.lifeMax)
                Shader.ISetFloat2(cacheTail.size, 16, min(Config.gen.compTurretPulseStats.size, 1.5 * pulse.dist))
                Shader.ISetFloat3(cacheTail.axis, pulse.dir.x, pulse.dir.y, pulse.dir.z)
                Shader.ISetMatrix(cacheTail.mWorld, pulse.matrix)
                meshTail:drawBound()
            end
            meshTail:drawUnbind()
            shader:stop()
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
            --printf("PULSE: projectile delete on expiration = %s", projectiles[i]:getName())
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
            pulse:refreshMatrix()
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
            --       return anything fromRigidBody for the first few seconds. While this is a good check to do since
            --       we cannot confirm that the hit will have a rigidbody. This is a hotfix for a weird error.
            if (hitEnt ~= nil) then
                -- Don't collide with the socket that spawned me
                if hitEnt ~= source then
                    -- Do damage if the collidee has health
                    if hitEnt:isAlive() then
                        -- TODO: Get damage type and amount from the pulse
                        hitEnt:applyDamage(Config.gen.compTurretPulseStats.damage, source)
                    end

                    -- Remove projectile
                    --printf("PULSE: projectile delete on hit = %s", projectiles[i]:getName())
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
