local Material = require('Legacy.GameObjects.Material')
local Registry = require('Core.ECS.Registry')
local RenderComponent = require('Modules.Rendering.Components.RenderComponent')
local RigidBodyComponent = require('Modules.Physics.Components.RigidBodyComponent')
local AsteroidInstancedRenderer = require('Legacy.Systems.Overlay.AsteroidInstancedRenderer')

local GameView = {}
GameView.__index = GameView
setmetatable(GameView, UI.Container)

GameView.name = 'Game View'

local ssTable = { 1, 2, 4 }

function GameView:draw(focus, active)
    self._frame = (self._frame or 0) + 1
    self.camera:push()

    local ss = ssTable[Settings.get('render.superSample')]
    local x, y, sx, sy = self:getRectGlobal()
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()
    self.camera:setViewport(x, y, sx, sy)
    self.camera:beginDraw()

    local system = GameState.world.currentSystem
    -- local system = self.player:getRoot()
    local eye = self.camera.pos
    system:beginRender()

    do -- Opaque Pass
        Profiler.Begin('Render.Opaque')
        AsteroidInstancedRenderer.beginFrame()
        self.renderer:start(self.sx, self.sy, ss)
        Profiler.Begin('Opaque.DrawScene')
        self:drawScene(BlendMode.Disabled, eye) -- significant performance point with ss
        Profiler.End()
        -- Flush instanced asteroids: one DrawInstancedWithData per
        -- (mesh variant, LOD level) group, bound with the instanced shader
        -- + rock diffuse texture (replaces ~600 individual asteroid draws).
        Profiler.Begin('Opaque.InstancedFlush')
        local instShader = Cache.Shader('wvp_instanced', 'material/asteroid_instanced')
        instShader:start()
        instShader:setTex2D('texDiffuse', Cache.Texture('rock'))
        AsteroidInstancedRenderer.flush()
        instShader:stop()
        Profiler.End()
        self.renderer:stop()
        Profiler.End()
    end

    do -- Lighting
        -- Gather light sources
        -- Note: Scan only objects with lights attached
        local lights = {}
        if #system.lightList > 0 then
            --Log.Debug("---------")
            for _, v in ipairs(system.lightList) do
                insert(lights, { pos = v:getPos(), color = v:getLight() })
                --Log.Debug("light '%s' @ %s, %s", v:getName(), v:getPos(), v:getLight())
            end
        end

        do -- Global lighting (environment)
            Profiler.Begin('Render.Lighting.Global')
            self.renderer.buffer2:push()
            Draw.Clear(0, 0, 0, 0)
            local shader = Cache.Shader('worldray', 'light/global')
            shader:start()
            shader:setTex2D('texDepth', self.renderer.zBufferL)
            shader:setTex2D('texNormalMat', self.renderer.buffer1)
            Draw.Rect(-1, -1, 2, 2)
            shader:stop()
            self.renderer.buffer2:pop()
            Profiler.End()
        end

        do -- Local lighting (TODO: performance issues?)
            Profiler.Begin('Render.Lighting.Local')
            self.renderer.buffer2:push()
            RenderState.PushBlendMode(BlendMode.Additive)
            local shader = Cache.Shader('worldray', 'light/point')
            shader:start()
            for i, v in ipairs(lights) do
                -- TODO : Batching
                local renderPos = v.pos:relativeTo(eye)
                Renderer:updateLightUbo(
                    renderPos.x, renderPos.y, renderPos.z, 0.0,
                    v.color.x, v.color.y, v.color.z, 1.0
                )
                shader:setTex2D('texDepth', self.renderer.zBufferL)
                shader:setTex2D('texNormalMat', self.renderer.buffer1)
                Draw.Rect(-1, -1, 2, 2)
            end
            shader:stop()
            RenderState.PopBlendMode()
            self.renderer.buffer2:pop()
            Profiler.End()
        end

        do -- Composite albedo & accumulated light buffer
            Profiler.Begin('Render.Lighting.Albedo')
            self.renderer.buffer1:push()
            local shader = Cache.Shader('worldray', 'light/composite')
            shader:start()
            shader:setTex2D('texAlbedo', self.renderer.buffer0)
            shader:setTex2D('texDepth', self.renderer.zBufferL)
            shader:setTex2D('texLighting', self.renderer.buffer2)
            Draw.Rect(-1, -1, 2, 2)
            shader:stop()
            self.renderer.buffer1:pop()
            Profiler.End()
        end

        Profiler.Begin('Render.Lighting.BufferExchange')
        self.renderer.buffer0, self.renderer.buffer1 = self.renderer.buffer1, self.renderer.buffer0
        Profiler.End()
    end

    if true then -- Alpha (Additive) Pass
        Profiler.Begin('Render.Additive')
        self.renderer:startAlpha(BlendMode.Additive)
        self:drawScene(BlendMode.Additive, eye)
        self.renderer:stopAlpha()
        Profiler.End()
    end

    if true then -- Alpha Pass
        Profiler.Begin('Render.AlphaDebug')
        self.renderer:startAlpha(BlendMode.Alpha)
        self:drawScene(BlendMode.Alpha, eye)

        -- TODO : This should be moved into a render pass
        if GameState.debug.physics.drawBoundingBoxesLocal or
            GameState.debug.physics.drawBoundingBoxesWorld or
            GameState.debug.physics.drawWireframes
        then
            local mat = Material.DebugColorA()
            mat:start()
            local shader = mat.state:shader()
            if GameState.debug.physics.drawBoundingBoxesLocal then
                shader:setFloat4('color', 0, 0, 1, 0.5)
                system.physics:drawBoundingBoxesLocal()
            end
            if GameState.debug.physics.drawBoundingBoxesWorld then
                shader:setMatrix('mWorld', Matrix.Identity())
                shader:setMatrixT('mWorldIT', Matrix.Identity())
                shader:setFloat('scale', 1)
                shader:setFloat4('color', 1, 0, 0, 0.5)
                system.physics:drawBoundingBoxesWorld()
            end
            if GameState.debug.physics.drawWireframes then
                shader:setMatrix('mWorld', Matrix.Identity())
                shader:setMatrixT('mWorldIT', Matrix.Identity())
                shader:setFloat('scale', 1)
                -- drawWireframes will set the 'color' shader variable.
                system.physics:drawWireframes(shader, eye)
            end
            mat:stop()
        end
        self.renderer:stopAlpha()
        Profiler.End()
    end

    Profiler.Begin('Render.endDraw')
    system:endRender()
    self.camera:endDraw() -- now go perform all the deferred rendering operations
    Profiler.End()

    if true then -- Composited UI Pass (becomes slow with many asteroids)
        Profiler.Begin('Render.CompositedUI.start')
        self.renderer:startUI()
        Viewport.Push(0, 0, ss * self.sx, ss * self.sy, true)
        ClipRect.PushTransform(0, 0, ss, ss)
        ShaderVar.PushMatrix("mWorldViewUI", Matrix.Scaling(ss, ss, 1.0))
        Profiler.End()
        Profiler.Begin('Render.CompositedUI.draw')
        for i = 1, #self.children do self.children[i]:draw(focus, active) end
        Profiler.End()
        Profiler.Begin('Render.CompositedUI.stop')
        ShaderVar.Pop("mWorldViewUI")
        ClipRect.PopTransform()
        Viewport.Pop()
        self.renderer:stopUI()
        Profiler.End()
    end

    if false or Settings.get('render.showBuffers') then
        self.renderer:presentAll(x, y, sx, sy)
    else
        Profiler.Begin('Render.PostEffects')
        self.renderer:startPostEffects()
        if Settings.get('postfx.bloom.enable') then self.renderer:bloom(Settings.get('postfx.bloom.radius')) end
        if Settings.get('postfx.tonemap.enable') then self.renderer:tonemap() end
        if Settings.get('postfx.aberration.enable') then
            self.renderer:applyFilter('aberration', function()
                shader:setFloat('strength', Settings.get('postfx.aberration.strength'))
            end)
        end
        if Settings.get('postfx.radialblur.enable') then
            self.renderer:applyFilter('radialblur', function()
                shader:setFloat('strength', Settings.get('postfx.radialblur.strength'))
            end)
        end
        if Settings.get('postfx.sharpen.enable') then
            self.renderer:sharpen(2, 1, 1)
        end
        self.renderer:present(x, y, sx, sy, ss > 2)
        Profiler.End()
    end

    --[[
    Unclear what this is referencing will need to investigate later

    if GUI.DrawHmGui then
        GUI.DrawHmGui(self.sx, self.sy)
    end
    --]]

    RenderState.PopAll()
    ClipRect.Pop()
    self.camera:pop()
end

function GameView:onInputChildren(state)
    self.camera:push()
    for i = 1, #self.children do
        local child = self.children[i]
        if not child.removed then child:input(state) end
    end
    self.camera:pop()
end

function GameView:onUpdate(state)
    --[[ TODO : This may be one frame delayed since onUpdateChildren happens later
              and one of them is responsible for updating the camera position.
              Further reason to invert the current Camera-Control relationship. ]]
    self.camera:onUpdate(state.dt)

    do -- Compute Eye Velocity EMA
        local eye = self.camera.pos
        local v = (eye - self.eyeLast):scale(1.0 / max(1e-10, state.dt))
        self.eyeVel:setv(self.player:getControlling():getVelocity())
        self.eyeLast:setv(eye)
    end

    self.audio:setListenerPos(self.camera.pos)
    self.audio:setListenerRot(self.camera.rot)

    self.camera:pop()
end

function GameView:onUpdateChildren(state)
    self.camera:push()
    for i = 1, #self.children do
        local child = self.children[i]
        if not child.removed then child:update(state) end
    end
    self.camera:pop()
end

function GameView:onLayoutSizeChildren()
    self.camera:push()
    for i = 1, #self.children do self.children[i]:layoutSize() end
    self.camera:pop()
end

function GameView:setCameraMode(cameraMode)
    local lastCamera = self.camera
    GameState.player.lastCamera = GameState.player.currentCamera

    if cameraMode == Enums.CameraMode.FirstPerson then
        self.camera = self.cameraFirstPerson

        if GameState.player.currentShip then
            -- hide ship mesh
            GameState.player.currentShip:setRenderVisibleMesh(false, true)
        end
    elseif cameraMode == Enums.CameraMode.Chase then
        self.camera = self.cameraChase

        if GameState.player.currentShip then
            -- hide ship mesh
            GameState.player.currentShip:setRenderVisibleMesh(true, false)
        end
    elseif cameraMode == Enums.CameraMode.Orbit then
        self.camera = self.cameraOrbit
        self.camera:setRelative(true)

        if GameState.player.currentShip then
            -- hide ship mesh
            GameState.player.currentShip:setRenderVisibleMesh(true, false)
        end
    else
        error("Invalid camera mode passed")
    end

    GameState.player.currentCamera = cameraMode
    self.camera:setTarget(self.player:getControlling())

    -- NOTE : We're assuming that no one else could have pushed a camera
    local camera = Systems.Camera.Camera.get()
    if camera and camera == lastCamera then
        lastCamera:pop()
        self.camera:push()
    end
    return self.camera
end

function GameView.Create(player, audioInstance)
    if not player then
        Log.Error("No player passed")
    end

    if not audioInstance then
        Log.Error("No audioInstance passed")
    end

    local self = setmetatable({
        player            = player,
        renderer          = RenderPipeline(),
        cameraFirstPerson = Systems.Camera.CameraFirstPerson(),
        cameraChase       = Systems.Camera.CameraChase(),
        cameraOrbit       = Systems.Camera.CameraOrbit(),
        camera            = nil,
        eyeLast           = nil,
        eyeVel            = nil,
        children          = List(),
        audio             = audioInstance
    }, GameView)

    self:setCameraMode(GameState.player.currentCamera)
    self.eyeLast = self.camera.pos:clone()
    self.eyeVel  = self.player:getControlling():getVelocity():clone()
    return self
end

function GameView:drawScene(blendMode, eye)
    -- Render all entities with a RenderComponent.
    --
    -- Per-frame pass lists: iterEntities is a coroutine.wrap that creates a
    -- new coroutine + Entity wrapper tables per call, and we run 3 passes
    -- (Opaque/Additive/Alpha) per frame. Building the lists ONCE per frame
    -- (instead of re-iterating the registry per pass) turns 3 coroutines +
    -- per-entity table churn into 1 iteration, and each pass then walks
    -- only the meshes that actually draw in it.
    Profiler.Begin('Opaque.BuildLists')
    if not self.frameLists or self.frameListsFrame ~= self._frame then
        local lists = self.frameLists
        if not lists then
            lists = { [BlendMode.Disabled] = {}, [BlendMode.Additive] = {}, [BlendMode.Alpha] = {} }
            self.frameLists = lists
        end
        -- Reuse the entry tables across frames (clear + refill) instead of
        -- allocating ~1,200 fresh {rb,entity,mesh} tables per frame - the
        -- remaining GC churn in the draw loop. Frame N+1 overwrites frame
        -- N's entries in place.
        local pool = self._entryPool or {}
        self._entryPool = pool
        local poolIdx = 0
        lists[BlendMode.Disabled] = {}
        lists[BlendMode.Additive] = {}
        lists[BlendMode.Alpha] = {}
        self.frameListsFrame = self._frame
        for entity, rigidBody, renderComponent in Registry:iterEntities(RigidBodyComponent, RenderComponent) do
            if not renderComponent:isVisible() then
                goto continue
            end

            local meshes = renderComponent:getMeshes()
            for mi = 1, #meshes do
                local mesh = meshes[mi]
                local list = lists[mesh.material.blendMode]
                if list then
                    poolIdx = poolIdx + 1
                    local entry = pool[poolIdx]
                    if not entry then
                        entry = {}
                        pool[poolIdx] = entry
                    end
                    entry.rb = rigidBody.rigidBody
                    entry.entity = entity
                    entry.mesh = mesh
                    list[#list + 1] = entry
                end
            end
            ::continue::
        end
    end
    Profiler.End()

    local list = self.frameLists[blendMode]
    Profiler.Begin('DrawScene.ECS')
    for i = 1, #list do
        local entry = list[i]
        local mesh = entry.mesh
        mesh.material:start()
        mesh.material:updateState(entry.rb, entry.entity, eye)
        mesh.mesh:draw()
        mesh.material:stop()
    end
    Profiler.End()

    -- Start a recursive render of the scene.
    Profiler.Begin('DrawScene.Recursive')
    GameState.world.currentSystem:send(OldEvent.Broadcast(OldEvent.Render(blendMode, eye)))
    GameState.world.currentSystem:render(OldEvent.Render(blendMode, eye))
    Profiler.End()
end

return GameView
