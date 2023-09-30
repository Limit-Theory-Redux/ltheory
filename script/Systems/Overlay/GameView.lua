local Material   = require('GameObjects.Material')

local GameView   = {}
GameView.__index = GameView
setmetatable(GameView, UI.Container)

GameView.name = 'Game View'

local ssTable = { 1, 2, 4 }

function GameView:draw(focus, active)
    self.camera:push()

    local ss = ssTable[Settings.get('render.superSample')]
    local x, y, sx, sy = self:getRectGlobal()
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()
    self.camera:setViewport(x, y, sx, sy)
    self.camera:beginDraw()

    local system = GameState.world.currentSystem
    --  local system = self.player:getRoot()
    local eye = self.camera.pos
    system:beginRender()

    do -- Opaque Pass
        Profiler.Begin('Render.Opaque')
        self.renderer:start(self.sx, self.sy, ss)
        system:render(Event.Render(BlendMode.Disabled, eye)) -- significant performance point with ss
        self.renderer:stop()
        Profiler.End()
    end

    do -- Lighting
        -- Gather light sources
        -- Note: Scan only objects with lights attached
        local lights = {}
        if #system.lightList > 0 then
            --print("---------")
            for _, v in ipairs(system.lightList) do
                insert(lights, { pos = v:getPos(), color = v:getLight() })
                --printf("light '%s' @ %s, %s", v:getName(), v:getPos(), v:getLight())
            end
        end

        do -- Global lighting (environment)
            Profiler.Begin('Render.Lighting.Global')
            self.renderer.buffer2:push()
            Draw.Clear(0, 0, 0, 0)
            local shader = Cache.Shader('worldray', 'light/global')
            shader:start()
            Shader.SetTex2D('texDepth', self.renderer.zBufferL)
            Shader.SetTex2D('texNormalMat', self.renderer.buffer1)
            Draw.Rect(-1, -1, 2, 2)
            shader:stop()
            self.renderer.buffer2:pop()
            Profiler.End()
        end

        do -- Local lighting (TODO: performance issues?)
            Profiler.Begin('Render.Lighting.Local')
            self.renderer.buffer2:push()
            BlendMode.PushAdditive()
            local shader = Cache.Shader('worldray', 'light/point')
            shader:start()
            for i, v in ipairs(lights) do
                -- TODO : Batching
                Shader.SetFloat3('lightColor', v.color.x, v.color.y, v.color.z)
                Shader.SetFloat3('lightPos', v.pos.x, v.pos.y + 5, v.pos.z)
                Shader.SetTex2D('texDepth', self.renderer.zBufferL)
                Shader.SetTex2D('texNormalMat', self.renderer.buffer1)
                Draw.Rect(-1, -1, 2, 2)
            end
            shader:stop()
            BlendMode.Pop()
            self.renderer.buffer2:pop()
            Profiler.End()
        end

        do -- Composite albedo & accumulated light buffer
            Profiler.Begin('Render.Lighting.Albedo')
            self.renderer.buffer1:push()
            local shader = Cache.Shader('worldray', 'light/composite')
            shader:start()
            Shader.SetTex2D('texAlbedo', self.renderer.buffer0)
            Shader.SetTex2D('texDepth', self.renderer.zBufferL)
            Shader.SetTex2D('texLighting', self.renderer.buffer2)
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
        system:render(Event.Render(BlendMode.Additive, eye)) -- significant performance point
        self.renderer:stopAlpha()
        Profiler.End()
    end

    if true then -- Alpha Pass
        Profiler.Begin('Render.AlphaDebug')
        self.renderer:startAlpha(BlendMode.Alpha)
        system:render(Event.Render(BlendMode.Alpha, eye))

        -- TODO : This should be moved into a render pass
        if Config.debug.physics.drawBoundingBoxesLocal or
            Config.debug.physics.drawBoundingBoxesWorld or
            Config.debug.physics.drawWireframes or
            Config.debug.physics.drawTriggers
        then
            local mat = Material.DebugColorA()
            mat:start()
            if Config.debug.physics.drawBoundingBoxesLocal then
                Shader.SetFloat4('color', 0, 0, 1, 0.5)
                system.physics:drawBoundingBoxesLocal()
            end
            if Config.debug.physics.drawBoundingBoxesWorld then
                Shader.SetMatrix('mWorld', Matrix.Identity())
                Shader.SetMatrixT('mWorldIT', Matrix.Identity())
                Shader.SetFloat('scale', 1)
                Shader.SetFloat4('color', 1, 0, 0, 0.5)
                system.physics:drawBoundingBoxesWorld()
            end
            if Config.debug.physics.drawTriggers then
                Shader.SetMatrix('mWorld', Matrix.Identity())
                Shader.SetMatrixT('mWorldIT', Matrix.Identity())
                Shader.SetFloat('scale', 1)
                Shader.SetFloat4('color', 1, 0.5, 0, 0.5)
                system.physics:drawTriggers()
            end
            if Config.debug.physics.drawWireframes then
                Shader.SetMatrix('mWorld', Matrix.Identity())
                Shader.SetMatrixT('mWorldIT', Matrix.Identity())
                Shader.SetFloat('scale', 1)
                Shader.SetFloat4('color', 0, 1, 0, 0.5)
                system.physics:drawWireframes()
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
        GLMatrix.ModeWV()
        GLMatrix.Push()
        GLMatrix.Scale(ss, ss, 1.0)
        Profiler.End()
        Profiler.Begin('Render.CompositedUI.draw')
        for i = 1, #self.children do self.children[i]:draw(focus, active) end
        Profiler.End()
        Profiler.Begin('Render.CompositedUI.stop')
        GLMatrix.ModeWV()
        GLMatrix.Pop()
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
            self.renderer:applyFilter('aberration', function ()
                Shader.SetFloat('strength', Settings.get('postfx.aberration.strength'))
            end)
        end
        if Settings.get('postfx.radialblur.enable') then
            self.renderer:applyFilter('radialblur', function ()
                Shader.SetFloat('strength', Settings.get('postfx.radialblur.strength'))
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
  ]]
    --  if GUI.DrawHmGui then
    --    GUI.DrawHmGui(self.sx, self.sy)
    --  end

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

    if LTheoryRedux.audio then
        LTheoryRedux.audio:setListenerPos(
            self.camera.pos,
            self.camera.rot)
    else
        LTheoryRedux.audio = Audio.Create()
        Log.Warning("[GameView.lua Update] Audio not initialized at this point. This should not happen.")
    end

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

function GameView.Create(player)
    local self = setmetatable({
        player            = player,
        renderer          = Renderer(),
        cameraFirstPerson = Systems.Camera.CameraFirstPerson(),
        cameraChase       = Systems.Camera.CameraChase(),
        cameraOrbit       = Systems.Camera.CameraOrbit(),
        camera            = nil,
        eyeLast           = nil,
        eyeVel            = nil,
        children          = List(),
    }, GameView)

    self:setCameraMode(GameState.player.currentCamera)
    self.eyeLast = self.camera.pos:clone()
    self.eyeVel  = self.player:getControlling():getVelocity():clone()
    return self
end

return GameView
