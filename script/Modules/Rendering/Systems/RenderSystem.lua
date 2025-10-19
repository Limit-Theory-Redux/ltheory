local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local RenderingPass = require("Shared.Rendering.RenderingPass")
local CameraSystem = require("Modules.Rendering.Systems.CameraSystem")
local RigidBodyComponent = require("Modules.Physics.Components.RigidBodyComponent")
local RenderComponent = require("Modules.Rendering.Components.RenderComponent")
local RenderPipeline = require("Render.RenderPipeline")

---@class Buffer : Tex2D

---@class RenderSettings
---@field superSampleRate integer
---@field downSampleRate integer
---@field showBuffers boolean
---@field cullFace boolean

---@class RenderSystem
---@field buffers table<BufferName, Buffer>
---@field settings RenderSettings
---@field passes RenderingPass
---@field level integer
---@field resX number
---@field resY number
---@field ssResX number
---@field ssResY number
---@field dsResX number
---@field dsResY number

-- TexFormats for Buffers
local colorFormat = TexFormat.RGBA16F
local depthFormat = TexFormat.Depth32F
local depthFormatL = TexFormat.R32F

local ssTable = { 1, 2, 4 }

---@class RenderSystem
---@overload fun(self: RenderSystem) class internal
---@overload fun() class external
local RenderSystem = Class("RenderSystem", function(self)
    self.profiler = QuickProfiler("RenderSystem", false, false)

    -- Set Variables based on Config.render
    self.settings = {
        superSampleRate = Config.render.general.superSampleRate,
        downSampleRate = Config.render.general.downSampleRate,
        showBuffers = Config.render.debug.showBuffers,
        cullFace = Config.render.renderState.cullFace
    }
    self.resX = Config.render.window.defaultResX
    self.resY = Config.render.window.defaultResY
    self.ssResX = self.resX * self.settings.superSampleRate
    self.ssResY = self.resY * self.settings.superSampleRate
    self.dsResX = self.resX / self.settings.downSampleRate
    self.dsResY = self.resY / self.settings.downSampleRate

    self.renderer = RenderPipeline()
    self.camera = nil
    self:setViewport(0, 0, self.resX, self.resY)

    -- Set Buffers
    self.buffers = {}
    self:initializeBuffers()

    -- Set Variables
    self.passes = {}
    self.level = 0
    
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
    EventBus:subscribe(Event.Render, self, self.onRender)
    EventBus:subscribe(Event.PostRender, self, self.onPostRender)

    self:registerRenderingPasses()
end)

---@private
function RenderSystem:initializeBuffers()
    -- If a buffer is currently set then reset all buffers
    -- if self.buffers[Enums.BufferName.buffer0] then self:resetBuffers() end
    -- self.buffers = {
    --     [Enums.BufferName.buffer0] = createBuffer(self.ssResX, self.ssResY, colorFormat),
    --     [Enums.BufferName.buffer1] = createBuffer(self.ssResX, self.ssResY, colorFormat),
    --     [Enums.BufferName.buffer2] = createBuffer(self.ssResX, self.ssResY, colorFormat),
    --     [Enums.BufferName.zBuffer] = createBuffer(self.ssResX, self.ssResY, depthFormat),
    --     [Enums.BufferName.zBufferL] = createBuffer(self.ssResX, self.ssResY, depthFormatL),
    --     [Enums.BufferName.dsBuffer0] = createBuffer(self.dsResX, self.dsResY, colorFormat),
    --     [Enums.BufferName.dsBuffer1] = createBuffer(self.dsResX, self.dsResY, colorFormat)
    -- }
end

---@private
function RenderSystem:resetBuffers()
    -- self.buffers[Enums.BufferName.buffer0] = nil
    -- self.buffers[Enums.BufferName.buffer1] = nil
    -- self.buffers[Enums.BufferName.buffer2] = nil
    -- self.buffers[Enums.BufferName.zBuffer] = nil
    -- self.buffers[Enums.BufferName.zBufferL] = nil
    -- self.buffers[Enums.BufferName.dsBuffer0] = nil
    -- self.buffers[Enums.BufferName.dsBuffer1] = nil
end

---@private
function RenderSystem:registerRenderingPasses()
    -- do -- < Opaque Pass Definition > --
    --     local stateSettings = {
    --         blendMode = BlendMode.Disabled,
    --         cullFace = self.settings.cullFace and CullFace.Back or CullFace.None,
    --         depthTest = true,
    --         depthWritable = true
    --     }
    --     local bufferOrder = {}
    --     insert(bufferOrder, Enums.BufferName.buffer0)
    --     insert(bufferOrder, Enums.BufferName.buffer1)
    --     insert(bufferOrder, Enums.BufferName.zBufferL)
    --     insert(bufferOrder, Enums.BufferName.zBuffer)
    --     local drawFunc = function()
    --         Draw.Clear(0, 0, 0, 0)
    --         Draw.ClearDepth(1)
    --         Draw.Color(1, 1, 1, 1)
    --     end
    --     self.passes[Enums.RenderingPasses.Opaque] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    -- end
    -- do -- < Additive Pass Definition > --
    --     local stateSettings = {
    --         blendMode = BlendMode.Additive,
    --         cullFace = CullFace.None,
    --         depthTest = true,
    --         depthWritable = false
    --     }
    --     local bufferOrder = {} -- Reset BufferOrder
    --     insert(bufferOrder, Enums.BufferName.buffer0)
    --     insert(bufferOrder, Enums.BufferName.zBuffer)
    --     local drawFunc = nil
    --     self.passes[Enums.RenderingPasses.Additive] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    -- end
    -- do -- < Alpha Pass Definition > --
    --     local stateSettings = {
    --         blendMode = BlendMode.Alpha,
    --         cullFace = CullFace.None,
    --         depthTest = true,
    --         depthWritable = false
    --     }
    --     local bufferOrder = {}
    --     insert(bufferOrder, Enums.BufferName.buffer0)
    --     insert(bufferOrder, Enums.BufferName.zBuffer)
    --     local drawFunc = nil
    --     self.passes[Enums.RenderingPasses.Alpha] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    -- end
    -- do -- < UI Pass Definition > --
    --     local stateSettings = {
    --         blendMode = BlendMode.Alpha,
    --         cullFace = CullFace.None,
    --         depthTest = false,
    --         depthWritable = false
    --     }
    --     local bufferOrder = {}
    --     insert(bufferOrder, Enums.BufferName.buffer1)
    --     insert(bufferOrder, Enums.BufferName.zBuffer)
    --     local drawFunc = function() Draw.Clear(0, 0, 0, 0) end
    --     self.passes[Enums.RenderingPasses.Alpha] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    -- end
end

function RenderSystem:setCamera(camera)
    self.camera = camera
end

function RenderSystem:setViewport(x, y, sx, sy)
    self.viewport = {x = x, y = y, sx = sx, sy = sy}
end

function RenderSystem:onPreRender(data)
    -- -- Can the Changes to SSR/DSR and Resolution be done in PreRender?
    -- -- How do we communicate SuperSampleRate/DownSampleRate changes to RenderSystem?
    -- local ssr = self.settings.superSampleRate -- data.ssr or
    -- local dsr = self.settings.downSampleRate
    -- local ssResX, ssResY = ssr * Window:size().x, ssr * Window:size().y
    -- local dsResX, dsResY = dsr * Window:size().x, dsr * Window:size().y
    -- if self.ssResX ~= ssResX or self.ssResY ~= ssResY or self.ssr ~= ssr then
    --     self.ssResX = ssResX
    --     self.ssResY = ssResY
    --     self.dsResX = dsResX
    --     self.dsResY = dsResY
    --     self.settings.superSampleRate = ssr
    --     --self.settings.downSampleRate = dsr -- Only if we add the option to change DSR
    --     self.resX = Window:size().x
    --     self.resY = Window:size().y

    --     -- Buffers must be reinitialized when SuperSampleRate/DownSampleRate is changed.
    --     self:initializeBuffers()
    -- end

    -- -- < Prepare Buffers for Primary Render Pass > --
    -- -- Is there a better place to put this?
    -- self.buffers[Enums.BufferName.buffer0]:setMipRange(0, 0)
    -- self.buffers[Enums.BufferName.buffer1]:setMipRange(0, 0)
    -- self.buffers[Enums.BufferName.buffer2]:setMipRange(0, 0)
    -- self.buffers[Enums.BufferName.buffer0]:setMinFilter(TexFilter.Linear)
    -- self.buffers[Enums.BufferName.buffer1]:setMinFilter(TexFilter.Linear)
    -- self.buffers[Enums.BufferName.buffer2]:setMinFilter(TexFilter.Linear)
    -- self.level = 0

    -- < Cache Entities (EntityId's?) for Rendering Passes > --
    -- Necessary because previously we obtained Entites from 'StarSystem'
    --[[ Need Method to pull by BlendMode in RenderPass and change cache based on Culling
        OpaqueCache = AllEntitiesWithComponent(RenderComponent):filter(BlendMode.Opaque)
        AdditiveCache = AllEntitiesWithComponent(RenderComponent):filter(BlendMode.Additive)
        AlphaCache = AllEntitiesWithComponent(RenderComponent):filter(BlendMode.Alpha)
        LightCache = AllEntitiesWithComponent(LightComponent)
    --]]
end

function RenderSystem:onRender(data)
    -- Build the render pass lists.
    local passes = {
        [BlendMode.Disabled] = {},
        [BlendMode.Additive] = {},
        [BlendMode.Alpha] = {},
        [BlendMode.PreMultAlpha] = {}
    }
    for entity, rigidBody, renderComponent in Registry:iterEntities(RigidBodyComponent, RenderComponent) do
        if renderComponent:isVisible() then
            for _, renderable in ipairs(renderComponent:getMeshes()) do
                local bm = renderable.material.blendMode
                passes[bm][#passes[bm] + 1] = {renderable, rigidBody, entity}
            end
        end
    end

    self.camera:push()

    local ss = ssTable[Settings.get('render.superSample')]
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()
    self.camera:setViewport(self.viewport.x, self.viewport.y, self.viewport.sx, self.viewport.sy)
    self.camera:beginDraw()

    local system = GameState.world.currentSystem
    -- local system = self.player:getRoot()
    local eye = self.camera.pos
    system:beginRender()

    do -- Opaque Pass
        Profiler.Begin('Render.Opaque')
        self.renderer:start(self.viewport.sx, self.viewport.sy, ss)
        self:doRenderPass(passes, BlendMode.Disabled, eye)
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
                shader:setFloat3('lightPos', renderPos.x, renderPos.y, renderPos.z)
                shader:setFloat3('lightColor', v.color.x, v.color.y, v.color.z)
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
        self:doRenderPass(passes, BlendMode.Additive, eye)
        self.renderer:stopAlpha()
        Profiler.End()
    end

    if true then -- Alpha Pass
        Profiler.Begin('Render.AlphaDebug')
        self.renderer:startAlpha(BlendMode.Alpha)
        self:doRenderPass(passes, BlendMode.Alpha, eye)

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

    if false then -- Composited UI Pass (becomes slow with many asteroids)
        Profiler.Begin('Render.CompositedUI.start')
        self.renderer:startUI()
        Viewport.Push(0, 0, ss * self.viewport.sx, ss * self.viewport.sy, true)
        ClipRect.PushTransform(0, 0, ss, ss)
        ShaderVar.PushMatrix("mWorldViewUI", Matrix.Scaling(ss, ss, 1.0))
        Profiler.End()
        Profiler.Begin('Render.CompositedUI.draw')
        GameState.render.gameView:draw(true, true)
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
        self.renderer:present(self.viewport.x, self.viewport.y, self.viewport.sx, self.viewport.sy, ss > 2)
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

    -- -- Begin Drawing Window
    -- Window:beginDraw()

    -- -- Reset RenderState and ClipRect at Start of Render
    -- ClipRect.PushDisabled()
    -- RenderState.PushAllDefaults()

    -- do -- < Camera > --
    --     -- Add Camera Stack?
    --     -- Should we remove updateViewMatrix/updateProjectionMatrix and use a RefreshMatrices to do this?
    --     CameraSystem:updateViewMatrix()
    --     CameraSystem:updateProjectionMatrix(self.resX, self.resY) -- Do we use ssRes or res?
    --     CameraSystem:beginDraw()                                  -- Push Camera ShaderVars
    --     --[[Original Order for Camera
    --         In GameView:
    --             local x, y, sx, sy = self:getRectGlobal()
    --             self.camera:setViewport(x, y, sx, sy) -- just sets those vars
    --             self.camera:beginDraw()
    --             eye = self.camera.pos
    --         In Camera:beginDraw()
    --             camera:push()
    --             camera:refreshMatrixes
    --             ShaderVar.PushMatrix('varname', self.matrix) - for mView, mProj, eye, ...
    --     ]]
    -- end

    -- do -- < Push Skybox/Nebula Global ShaderVars > --
    --     --[["StarSystem:beginRender()"
    --         self.nebula:forceLoad() -- Generates Nebula
    --         ShaderVar.PushFloat3('starDir', self.starDir.x, self.starDir.y, self.starDir.z)
    --         ShaderVar.PushTexCube('envMap', self.nebula.envMap)
    --         ShaderVar.PushTexCube('irMap', self.nebula.irMap)
    --     "]]
    -- end

    -- do -- < Opaque Pass > --
    --     self.passes[Enums.RenderingPasses.Opaque]:start(self.buffers, self.ssResX, self.ssResY)
    --     self:renderInOrder(BlendMode.Alpha)
    --     self.passes[Enums.RenderingPasses.Opaque]:stop()
    -- end

    -- do -- < Lighting Pass > --
    --     -- TODO: Needs a different solution than other passes, as it's structured differently currently.
    --     -- < Global Lighting > --
    --     -- Use Cached World Light Material ?
    --     -- < Local Lighting > --
    --     -- Use Cached Light Material
    --     -- < Aldebo & accumulated light buffer > --
    --     -- Use Cached Light Material
    -- end

    -- do -- < Alpha (Additive Pass) > --
    --     self.passes[Enums.RenderingPasses.Additive]:start(self.buffers, self.ssResX, self.ssResY)
    --     self:renderInOrder(BlendMode.Alpha)
    --     self.passes[Enums.RenderingPasses.Additive]:stop()
    -- end

    -- do -- < Alpha Pass > --
    --     self.passes[Enums.RenderingPasses.Alpha]:start(self.buffers, self.ssResX, self.ssResY)
    --     self:renderInOrder(BlendMode.Alpha)
    --     self.passes[Enums.RenderingPasses.Alpha]:stop()
    -- end

    -- do -- < Pop Skybox and Camera Global ShaderVars > --
    --     --[[ "StarSystem:endRender()"
    --         ShaderVar.Pop('starDir')
    --         ShaderVar.Pop('envMap')
    --         ShaderVar.Pop('irMap')
    --     ]]--
    --     CameraSystem:endDraw() -- Pop Camera ShaderVars
    -- end

    -- do -- < UI Pass > --
    --     self.passes[Enums.RenderingPasses.UI]:start(self.buffers, self.ssResX, self.ssResY)
    --     -- TODO: Do UI Rendering Here or Pass off to a UIRenderer?
    --     self.passes[Enums.RenderingPasses.UI]:stop()
    --     -- TODO: Do UI Equivalent of RenderPipeline:present() also why self.buffer1:pop()
    --     -- Reference: RenderPipeline:stopUI()
    -- end

    -- do -- < PostFX and Draw Rendered Frame to Screen > --
    --     if self.settings.showBuffers then
    --         -- < PresentAll Buffers / RenderPipeline.presentAll(...) > --
    --         self:presentAll(Window:position().x, Window:position().y, self.resX, self.resY)
    --     else
    --         -- < PostFX Pass > --
    --         -- < Present Frame / Present buffer0 / RenderPipeline.present(...) > -
    --         self:present(Window:position().x, Window:position().y, self.resX, self.resY, false)
    --     end
    -- end

    -- RenderState.PopAll()
    -- ClipRect.Pop()
    -- --self.camera:pop() -- TODO: Implement Camera Stack
end

function RenderSystem:onPostRender(data)
    -- Window:endDraw()
end

function RenderSystem:doRenderPass(passes, blendMode, eye)
    Log.Warn("doRenderPass - blendMode: %d", blendMode)
    for _, op in ipairs(passes[blendMode]) do
        local renderable = op[1]
        local rigidBody = op[2]
        local entity = op[3]

        renderable.material:start()
        renderable.material:updateState(rigidBody, entity, eye)
        renderable.mesh:draw()
        renderable.material:stop()
    end
        
    -- Start a recursive render of the scene using the old render code.
    GameState.world.currentSystem:send(OldEvent.Broadcast(OldEvent.Render(blendMode, eye)))
    GameState.world.currentSystem:render(OldEvent.Render(blendMode, eye))
    
    --[[ Original Order
        self:send(OldEvent.Broadcast(blendMode))
        self:renderProjectiles(blendMode)
        self.dust:render(blendMode)
        self.nebula:render(blendMode)
    ]]
    -- < Render General Materials > --
    -- if blendMode == BlendMode.Disabled then
    --     -- < Render All 'RenderComponent' Opaque Pass > --
    --     -- < Render Nebula 'Skybox' Opaque Pass > --
    -- elseif blendMode == BlendMode.Additive then
    --     -- < Render All 'RenderComponent' Additive Pass > --
    --     -- < Render All Projectiles > --
    --     -- < Render Dust Additive Pass > --
    --     -- < Render Nebula 'StarBg' Additive Pass > --
    -- elseif blendMode == BlendMode.Alpha then
    --     -- < Render All 'RenderComponent' Alpha Pass> --
    --     -- < Render Dust Alpha Pass > --
    -- elseif blendMode == BlendMode.PreMultAlpha then
    --     -- Only used currently by 'Planet.lua' during Alpha Pass
    -- end
end

return RenderSystem()
