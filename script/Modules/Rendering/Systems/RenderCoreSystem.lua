local Registry         = require("Core.ECS.Registry")
local QuickProfiler    = require("Shared.Tools.QuickProfiler")
local RenderingPass    = require("Shared.Rendering.RenderingPass")
local CameraManager    = require("Modules.Cameras.Managers.CameraManager")
local RenderComp       = require("Modules.Rendering.Components").Render
local CameraComponent  = require("Modules.Cameras.Components.CameraDataComponent")
local UniformFuncs     = require("Shared.Rendering.UniformFuncs")
local Cache            = require("Render.Cache")

---@class RenderCoreSystem
---@overload fun(self): RenderCoreSystem
---@overload fun(): RenderCoreSystem
local RenderCoreSystem = Class("RenderCoreSystem", function(self)
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    self:registerVars()
    self:registerPasses()
end)

function RenderCoreSystem:registerVars()
    self.profiler        = QuickProfiler("RenderCoreSystem", false, false)

    self.settings        = {
        superSampleRate = Config.render.general.superSampleRate,
        downSampleRate  = Config.render.general.downSampleRate,
        showBuffers     = Config.render.debug.showBuffers,
        cullFace        = Config.render.renderState.cullFace
    }

    self.postSettings    = {
        aberration = Config.render.postFx.aberration,
        bloom      = Config.render.postFx.bloom,
        sharpen    = Config.render.postFx.sharpen,
        radialblur = Config.render.postFx.radialblur,
        tonemap    = Config.render.postFx.tonemap,
        vignette   = Config.render.postFx.vignette,
        fxaa       = Config.render.postFx.fxaa,
        dither     = Config.render.postFx.dither,
        colorgrade = Config.render.postFx.colorgrade,
    }

    self.autoExposure    = {
        current = 1.0, -- current adapted exposure
        target  = 1.0, -- what we're adapting toward this frame
    }

    -- Menu background blur strength (set per frame by the state; nil = off)
    self.menuBlurStrength = nil

    local win            = Window:size()
    self.resX, self.resY = win.x, win.y
    self.ssResX          = self.resX * self.settings.superSampleRate
    self.ssResY          = self.resY * self.settings.superSampleRate
    self.dsResX          = self.resX / self.settings.downSampleRate
    self.dsResY          = self.resY / self.settings.downSampleRate

    self.ds              = 4  -- downsample factor for bloom (matches old pipeline)

    -- Half-res ping-pong for the menu blur (smoother than the 4x ds buffers)
    self.hbResX          = math.floor(self.resX / 2)
    self.hbResY          = math.floor(self.resY / 2)

    self.buffers         = {}
    self:initializeBuffers()
    self.passes = {}
    self.level = 0

    -- FPS tracking
    self.frameTimes = {}          -- table storing recent frame times
    self.frameHistoryLength = 100 -- track last 100 frames
    self.currentFPS = 0
    self.currentFrameTime = 0
    self.smoothFPS = 0
    self.smoothFrameTime = 0
    self.smoothFactor = 0.035 -- smaller = smoother, slower to react

    -- For injection
    self.currentPass = nil
end

function RenderCoreSystem:initializeBuffers()
    local function create(x, y, fmt)
        local t = Tex2D.Create(x, y, fmt)
        t:setMagFilter(TexFilter.Linear)
        t:setMinFilter(TexFilter.Linear)
        t:setWrapMode(TexWrapMode.Clamp)
        t:push(); Draw.Clear(0, 0, 0, 0); t:pop(); t:genMipmap()
        return t
    end

    self.buffers = {
        [Enums.BufferName.buffer0]   = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.buffer1]   = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.buffer2]   = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.zBuffer]   = create(self.ssResX, self.ssResY, TexFormat.Depth32F),
        [Enums.BufferName.zBufferL]  = create(self.ssResX, self.ssResY, TexFormat.R32F),
        [Enums.BufferName.dsBuffer0] = create(self.dsResX, self.dsResY, TexFormat.RGBA16F),
        [Enums.BufferName.dsBuffer1] = create(self.dsResX, self.dsResY, TexFormat.RGBA16F),
        [Enums.BufferName.hbBuffer0] = create(self.hbResX, self.hbResY, TexFormat.RGBA16F),
        [Enums.BufferName.hbBuffer1] = create(self.hbResX, self.hbResY, TexFormat.RGBA16F),
    }
end

function RenderCoreSystem:registerPasses()
    local function pass(name, blend, cull, dt, dw, bufs, onStart)
        self.passes[name] = RenderingPass(bufs, {
            blendMode = blend, cullFace = cull, depthTest = dt, depthWritable = dw
        }, onStart)
    end

    pass(Enums.RenderingPasses.Opaque,
        BlendMode.Disabled, self.settings.cullFace and CullFace.Back or CullFace.None,
        true, true,
        { Enums.BufferName.buffer0, Enums.BufferName.buffer1, Enums.BufferName.zBufferL, Enums.BufferName.zBuffer },
        function()
            Draw.Clear(0, 0, 0, 0); Draw.ClearDepth(1); Draw.Color(1, 1, 1, 1)
        end)

    pass(Enums.RenderingPasses.Additive,
        BlendMode.Additive, CullFace.None, true, false,
        { Enums.BufferName.buffer0, Enums.BufferName.zBuffer })

    pass(Enums.RenderingPasses.Alpha,
        BlendMode.Alpha, CullFace.None, true, false,
        { Enums.BufferName.buffer0, Enums.BufferName.zBuffer })

    pass(Enums.RenderingPasses.UI,
        BlendMode.Alpha, CullFace.None, false, false,
        { Enums.BufferName.buffer1, Enums.BufferName.zBuffer },
        function() Draw.Clear(0, 0, 0, 0) end)
end

---@param data EventData
function RenderCoreSystem:render(data)
    Profiler.Begin('RenderCore.render')
    -- Track frame time
    local dt = data:deltaTime() -- already in your code
    table.insert(self.frameTimes, dt)

    -- Keep only last N frames
    if #self.frameTimes > self.frameHistoryLength then
        table.remove(self.frameTimes, 1)
    end

    self:handleResize()

    Window:beginDraw()
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()

    CameraManager:updateViewMatrix()
    CameraManager:updateProjectionMatrix(self.resX, self.resY)
    CameraManager:beginDraw()

    -- Sort visible meshes into per-pass lists once (renderInOrder runs 3×).
    self:buildPassLists()

    -- Opaque Pass
    Profiler.Begin('Render.Opaque')
    self.currentPass = Enums.RenderingPasses.Opaque
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self:renderInOrder(BlendMode.Disabled)
    self.passes[self.currentPass]:stop()
    Profiler.End() -- Render.Opaque

    -- Deferred Lighting Pass
    Profiler.Begin('Render.Lighting.Deferred')
    self:deferredLighting()
    Profiler.End() -- Render.Lighting.Deferred

    -- Additive Pass
    Profiler.Begin('Render.Additive')
    self.currentPass = Enums.RenderingPasses.Additive
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self:renderInOrder(BlendMode.Additive)
    self.passes[self.currentPass]:stop()
    Profiler.End() -- Render.Additive

    -- Alpha Pass
    Profiler.Begin('Render.Alpha')
    self.currentPass = Enums.RenderingPasses.Alpha
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self:renderInOrder(BlendMode.Alpha)
    self.passes[self.currentPass]:stop()
    Profiler.End() -- Render.Alpha

    -- UI Pass
    Profiler.Begin('Render.UI')
    self.currentPass = Enums.RenderingPasses.UI
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self.passes[self.currentPass]:stop()
    Profiler.End() -- Render.UI

    -- Manual UI Composite: buffer0 (scene) + buffer1 (UI) → buffer2
    Profiler.Begin('Render.UI.Composite')
    do
        local buffer2 = self.buffers[Enums.BufferName.buffer2]
        buffer2:push()

        Draw.Clear(0, 0, 0, 0) -- Recommended

        local shader = Cache.Shader('ui', 'ui/composite')
        shader:start()
        shader:setTex2D('srcBottom', self.buffers[Enums.BufferName.buffer0])
        shader:setTex2D('srcTop', self.buffers[Enums.BufferName.buffer1])
        Draw.Rect(0, 0, self.ssResX, self.ssResY)
        shader:stop()

        buffer2:pop()

        -- Swap: make composited result the new main buffer
        self.buffers[Enums.BufferName.buffer0], self.buffers[Enums.BufferName.buffer2] =
            self.buffers[Enums.BufferName.buffer2], self.buffers[Enums.BufferName.buffer0]
    end
    Profiler.End() -- Render.UI.Composite

    -- Post-processing chain
    Profiler.Begin('Render.Post.Downsample')
    self:downsampleForPost()
    Profiler.End()

    Profiler.Begin('Render.Post.Aberration')
    self:aberration(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.Bloom')
    self:bloom(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.FXAA')
    self:fxaa(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.Sharpen')
    self:sharpen(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.ColorGrade')
    self:colorgrade(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.Tonemap')
    self:tonemap(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.Dither')
    self:dither(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.Vignette')
    self:vignette(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.RadialBlur')
    self:radialBlur(dt)
    Profiler.End()

    Profiler.Begin('Render.Post.MenuBlur')
    self:menuBlurPass(self.menuBlurStrength)
    Profiler.End()

    CameraManager:endDraw()

    if self.settings.showBuffers then
        self:presentAll(0, 0, self.resX, self.resY)
    else
        self:present(0, 0, self.resX, self.resY, false)
    end

    RenderState.PopAll()
    ClipRect.Pop()
    Window:endDraw()

    self.currentPass = nil

    -- Compute average frametime and FPS
    local sum = 0
    for _, t in ipairs(self.frameTimes) do
        sum = sum + t
    end
    self.currentFrameTime = sum / #self.frameTimes
    self.currentFPS       = math.floor(1 / self.currentFrameTime)

    -- Smooth with exponential moving average
    self.smoothFrameTime  = self.smoothFrameTime + (self.currentFrameTime - self.smoothFrameTime) * self.smoothFactor
    self.smoothFPS        = self.smoothFPS + (self.currentFPS - self.smoothFPS) * self.smoothFactor

    Profiler.End() -- RenderCore.render
end

function RenderCoreSystem:handleResize()
    local win = Window:size()
    local rx, ry = win.x, win.y
    local ssx = rx * self.settings.superSampleRate
    local dsx = rx / self.settings.downSampleRate

    if self.resX ~= rx or self.ssResX ~= ssx then
        self.resX, self.resY = rx, ry
        self.ssResX, self.ssResY = ssx, ry * self.settings.superSampleRate
        self.dsResX, self.dsResY = dsx, ry / self.settings.downSampleRate
        self:initializeBuffers()
    end

    -- Reset mip settings
    for _, buf in pairs(self.buffers) do
        if buf.setMipRange then
            buf:setMipRange(0, 0)
            buf:setMinFilter(TexFilter.Linear)
        end
    end
    self.level = 0
end

function RenderCoreSystem:renderInOrder(blendMode)
    local lastMaterial = nil
    local eye = CameraManager:getEye()

    -- Custom render fns are called in every pass (their blend mode isn't
    -- queryable). Mesh entities are pre-sorted into per-pass lists by
    -- buildPassLists, so this loop only touches entities that draw here.
    local fns = self.passRenderFns
    if fns then
        Profiler.Begin('Render.Fns')
        for fi = 1, #fns do
            local fnEntry = fns[fi]
            fnEntry.fn(fnEntry.entity, blendMode)
            lastMaterial = nil
        end
        Profiler.End()
    end

    local list = self.passMeshes[blendMode]
    if not list then return end

    for i = 1, #list do
        local entry = list[i]
        local mat = entry.mat
        local sh = entry.sh

        -- start() resets the texture-unit counter, so it must
        -- run per mesh (cannot be skipped for shared shaders);
        -- its auto-var re-application is already skipped inside
        -- by the var-stack revision check, and the redundant
        -- BindShader command is suppressed on the main thread.
        sh:start()

        -- The uniform funcs (UniformFuncs, keyed by UniformType)
        -- call iSetFloat/iSetFloat3/... which live on the raw
        -- Shader, not the ShaderState wrapper. Resolve it once
        -- (sh:shader() - colon form passes the ShaderState as self).
        local shader = sh:shader()

        -- Material-level vars only change when the material
        -- changes (they're constant across instances of the
        -- same material), so apply them once per material.
        if mat ~= lastMaterial then
            self:applyMaterialVars(mat, shader, eye, entry.entity)
            lastMaterial = mat
        end

        self:applyInstanceVars(mat, shader, eye, entry.entity, entry.instCache)

        entry.mesh:draw()
    end
end

--- Sort visible render entities into per-pass mesh lists once per frame.
--- renderInOrder runs 3× (Opaque/Additive/Alpha passes); the old code
--- re-iterated every entity × mesh in each pass just to filter on blend
--- mode. Building the lists once turns that into one iteration + three
--- walks of only the meshes that actually draw.
function RenderCoreSystem:buildPassLists()
    local passMeshes = {}
    local passRenderFns = {}
    local entityInstCache = {}

    for entity in Registry:view(RenderComp) do
        local rend = entity:get(RenderComp)
        if not rend:isVisible() then goto next_entity end

        if rend:getRenderFn() then
            table.insert(passRenderFns, { fn = rend:getRenderFn(), entity = entity })
        elseif rend:getMeshes() then
            -- Per-entity instance-var cache. All meshes of an entity
            -- (hull/turrets/thrusters) share the same rigid-body transform,
            -- so perInstance vars (mWorld/mWorldIT/scale) are identical
            -- across them. Previously each mesh recomputed + reallocated
            -- the matrices via getToWorldMatrix()/getToLocalMatrix() (each
            -- a managed Matrix* with a finalizer); the cache computes each
            -- var once per entity per frame and reuses the values.
            local instCache = {}
            entityInstCache[entity.id] = instCache

            local meshes = rend:getMeshes()
            for mi = 1, #meshes do
                local meshmat = meshes[mi]
                local mat = meshmat.material
                local bm = mat:getBlendMode() or BlendMode.Disabled
                local list = passMeshes[bm]
                if not list then
                    list = {}
                    passMeshes[bm] = list
                end
                table.insert(list, {
                    mesh = meshmat.mesh,
                    mat = mat,
                    sh = mat:getShaderState(),
                    entity = entity,
                    instCache = instCache,
                })
            end
        end
        ::next_entity::
    end

    self.passMeshes = passMeshes
    self.passRenderFns = passRenderFns
    self.entityInstCache = entityInstCache
end

function RenderCoreSystem:applyMaterialVars(mat, shader, eye, entity)
    -- material level (constant across all instances of the material)
    local vars = mat.staticShaderVars
    if vars then
        for i = 1, #vars do
            vars[i]:setShaderVar(eye, shader, entity)
        end
    end
    vars = mat.constShaderVars
    if vars then
        for i = 1, #vars do
            vars[i]:setShaderVar(eye, shader, entity)
        end
    end
    vars = mat.autoShaderVars
    if vars then
        for i = 1, #vars do
            local v = vars[i]
            if not v.perInstance then
                v:setShaderVar(eye, shader, entity)
            end
        end
    end
end

function RenderCoreSystem:applyInstanceVars(mat, shader, eye, entity, instCache)
    -- instance level (per-entity): values are computed once per entity per
    -- frame (see buildPassLists) and reused across all of the entity's
    -- meshes. Missing uniformInt vars fall through to setShaderVar, which
    -- warns once and skips.
    local vars = mat.autoShaderVars
    if vars then
        for i = 1, #vars do
            local v = vars[i]
            if v.perInstance then
                if not v.uniformInt then
                    v:setShaderVar(eye, shader, entity)
                else
                    -- Key by the var OBJECT, not its name: two materials could
                    -- have perInstance vars with the same name but different
                    -- value functions. Same material on multiple meshes of the
                    -- same entity -> same var object -> cache hit.
                    local values = instCache and instCache[v]
                    if not values then
                        values = v:getValues(eye, entity)
                        if instCache then instCache[v] = values end
                    end
                    local func = UniformFuncs[v.uniformType]
                    if func then func(shader, v.uniformInt, table.unpack(values)) end
                end
            end
        end
    end
end

-- Post-processing helpers
function RenderCoreSystem:swap()
    self.buffers[Enums.BufferName.buffer0], self.buffers[Enums.BufferName.buffer1] =
        self.buffers[Enums.BufferName.buffer1], self.buffers[Enums.BufferName.buffer0]
end

function RenderCoreSystem:applyFilter(fragName, onSetVars)
    local shader = Cache.Shader('ui', 'filter/' .. fragName)
    local target = self.buffers[Enums.BufferName.buffer1]
    target:pushLevel(self.level or 0)

    shader:start()
    shader:setTex2D('src', self.buffers[Enums.BufferName.buffer0])
    if onSetVars then onSetVars(shader) end
    local scale = 2 ^ (self.level or 0)
    Draw.Rect(0, 0, self.ssResX / scale, self.ssResY / scale)
    shader:stop()

    target:pop()
    self:swap()
end

function RenderCoreSystem:downsampleForPost()
    if self.settings.superSampleRate <= 1 then
        self.level = 0
        return
    end

    -- We need to resolve the supersampled buffer0 (ssResX x ssResY) down to screen res
    -- and optionally generate lower mips for post effects that might use them
    -- We'll do this in log2(superSampleRate) steps, building mips progressively

    local ssFactor = self.settings.superSampleRate -- e.g., 2, 4, etc. (assumed power of 2)
    local currentLevel = 0
    local currentSizeX = self.ssResX
    local currentSizeY = self.ssResY

    while currentSizeX > self.resX or currentSizeY > self.resY do
        currentLevel = currentLevel + 1
        currentSizeX = math.floor(currentSizeX / 2)
        currentSizeY = math.floor(currentSizeY / 2)

        -- Downsample from previous level (or original) into current mip level of buffer1
        local target = self.buffers[Enums.BufferName.buffer1]
        target:pushLevel(currentLevel)

        local shader = Cache.Shader('ui', 'filter/downsample') -- simple bilinear downsample
        shader:start()
        shader:setTex2D('src', self.buffers[Enums.BufferName.buffer0])

        -- If this is the first downsample (full res → half), draw full screen quad at half size
        -- Otherwise, we're downsampling from previous mip
        if currentLevel == 1 then
            Draw.Rect(0, 0, self.ssResX / 2, self.ssResY / 2)
        else
            Draw.Rect(0, 0, currentSizeX * 2, currentSizeY * 2) -- draw from previous larger mip
        end

        shader:stop()
        target:pop()

        -- Set all main buffers to use this mip level for sampling in post
        for _, key in pairs({ Enums.BufferName.buffer0, Enums.BufferName.buffer1, Enums.BufferName.buffer2 }) do
            local b = self.buffers[key]
            if b.setMipRange then
                b:setMipRange(currentLevel, currentLevel)
                b:setMinFilter(TexFilter.Linear) -- Linear for smooth resolve
            end
        end

        -- Make the downsampled result the new "current" buffer0 for next post passes
        self:swap()
    end

    -- Final level is the one matching screen res
    self.level = currentLevel
end

--- Set directional lights for the scene (call before render)
---@param lights table[] Array of { dir: Vec3f (normalized, toward scene), color: Vec3f }
function RenderCoreSystem:setDirectionalLights(lights)
    self.directionalLights = lights
end

--- Set point lights for the scene (call before render)
---@param lights table[] Array of { pos: Position, color: Vec3f }
function RenderCoreSystem:setPointLights(lights)
    self.pointLights = lights
end

--- Deferred lighting pass: global environment + point lights → composite with albedo
function RenderCoreSystem:deferredLighting()
    local buffer0 = self.buffers[Enums.BufferName.buffer0]   -- albedo
    local buffer1 = self.buffers[Enums.BufferName.buffer1]   -- normals/material
    local buffer2 = self.buffers[Enums.BufferName.buffer2]   -- lighting accumulation
    local zBufferL = self.buffers[Enums.BufferName.zBufferL] -- linear depth

    local eye = CameraManager:getEye()

    -- 1. Global lighting (environment from irMap/envMap)
    buffer2:push()
    Draw.Clear(0, 0, 0, 0)
    local globalShader = Cache.Shader('worldray', 'light/global')
    globalShader:start()
    globalShader:setTex2D('texDepth', zBufferL)
    globalShader:setTex2D('texNormalMat', buffer1)
    Draw.Rect(-1, -1, 2, 2)
    globalShader:stop()
    buffer2:pop()


    -- 2. Directional lights (star — no distance falloff, like the sun)
    if self.directionalLights and #self.directionalLights > 0 then
        buffer2:push()
        RenderState.PushBlendMode(BlendMode.Additive)
        local dirShader = Cache.Shader('worldray', 'light/directional')
        dirShader:start()
        for _, light in ipairs(self.directionalLights) do
            dirShader:setFloat3('lightDir', light.dir.x, light.dir.y, light.dir.z)
            dirShader:setFloat3('lightColor', light.color.x, light.color.y, light.color.z)
            dirShader:setTex2D('texDepth', zBufferL)
            dirShader:setTex2D('texNormalMat', buffer1)
            Draw.Rect(-1, -1, 2, 2)
        end
        dirShader:stop()
        RenderState.PopBlendMode()
        buffer2:pop()
    end

    -- 3. Point lights (stations, engines, etc.)
    if self.pointLights and #self.pointLights > 0 then
        buffer2:push()
        RenderState.PushBlendMode(BlendMode.Additive)
        local pointShader = Cache.Shader('worldray', 'light/point')
        pointShader:start()
        for _, light in ipairs(self.pointLights) do
            local renderPos = light.pos:relativeTo(eye)
            Renderer:updateLightUbo(
                renderPos.x, renderPos.y, renderPos.z, 0.0,
                light.color.x, light.color.y, light.color.z, 1.0
            )
            pointShader:setTex2D('texDepth', zBufferL)
            pointShader:setTex2D('texNormalMat', buffer1)
            Draw.Rect(-1, -1, 2, 2)
        end
        pointShader:stop()
        RenderState.PopBlendMode()
        buffer2:pop()
    end

    -- 3. Composite: albedo * lighting → buffer1 (reuse as temp)
    buffer1:push()
    local compShader = Cache.Shader('worldray', 'light/composite')
    compShader:start()
    compShader:setTex2D('texAlbedo', buffer0)
    compShader:setTex2D('texDepth', zBufferL)
    compShader:setTex2D('texLighting', buffer2)
    Draw.Rect(-1, -1, 2, 2)
    compShader:stop()
    buffer1:pop()

    -- Swap buffer1 (lit result) into buffer0 (main scene buffer)
    self.buffers[Enums.BufferName.buffer0], self.buffers[Enums.BufferName.buffer1] =
        self.buffers[Enums.BufferName.buffer1], self.buffers[Enums.BufferName.buffer0]
end

function RenderCoreSystem:bloom(radius)
    if not self.postSettings.bloom.enable then return end

    local width = radius * 0.2
    local A = self.buffers[Enums.BufferName.dsBuffer0]
    local B = self.buffers[Enums.BufferName.dsBuffer1]

    -- Bright extract
    do
        local shader = Cache.Shader('ui', 'filter/bloompre')
        A:push()
        shader:start()
        shader:setTex2D('src', self.buffers[Enums.BufferName.buffer0])
        Draw.Rect(0, 0, self.resX / self.ds, self.resY / self.ds)
        shader:stop()
        A:pop()
    end

    for i = 1, 3 do
        self:blur(B, A, 1, 0, radius, width)
        self:blur(A, B, 0, 1, radius, width)

        self:applyFilter('bloomcomposite', function(sh)
            sh:setTex2D('srcBlur', A)
        end)
    end
end

function RenderCoreSystem:blur(dst, src, dx, dy, radius, variance)
    local shader = Cache.Shader('ui', 'filter/blur')
    local size = src:getSize()
    dst:push()
    shader:start()
    shader:setFloat('variance', variance)
    shader:setFloat2('dir', dx, dy)
    shader:setFloat2('size', size.x, size.y)
    shader:setInt('radius', radius)
    shader:setTex2D('src', src)
    Draw.Rect(0, 0, size.x, size.y)
    shader:stop()
    dst:pop()
end

function RenderCoreSystem:menuBlurPass(strength)
    if not strength or strength <= 0 then return end

    local A = self.buffers[Enums.BufferName.hbBuffer0]
    local B = self.buffers[Enums.BufferName.hbBuffer1]
    local src = self.buffers[Enums.BufferName.buffer0]

    -- Downsample the scene into A (half res — keeps the gaussian smooth
    -- when upscaled, unlike a 4x ds buffer which shows blocky edges).
    A:push()
    local dsShader = Cache.Shader('ui', 'filter/downsample')
    dsShader:start()
    dsShader:setTex2D('src', src)
    Draw.Rect(0, 0, self.hbResX, self.hbResY)
    dsShader:stop()
    A:pop()

    -- Tight small gaussian (2-tap radius); a wide blur reads as "bad sight".
    self:blur(B, A, 1, 0, 2, 3)
    self:blur(A, B, 0, 1, 2, 3)

    -- Mix the sharp scene with the blurred copy.
    self:applyFilter('menublur', function(sh)
        sh:setTex2D('srcBlur', A)
        sh:setFloat('strength', strength)
    end)
end

--- Enable/disable the menu background blur (nil or 0 disables).
--- The state calls this per frame based on the active menu view.
---@param strength number|nil
function RenderCoreSystem:setMenuBlur(strength)
    self.menuBlurStrength = (strength and strength > 0) and strength or nil
end

function RenderCoreSystem:fxaa()
    if not self.postSettings.fxaa.enable then return end

    local settings = self.postSettings.fxaa

    self:applyFilter('fxaa', function(sh)
        sh:setFloat('fxaaQualitySubpix', settings.strength)
        sh:setFloat('fxaaQualityEdgeThreshold', settings.edgeThreshold or 0.125)
        sh:setFloat('fxaaQualityEdgeThresholdMin', settings.edgeThresholdMin or 0.0312)
        sh:setFloat2('size', self.resX, self.resY)
    end)
end

function RenderCoreSystem:sharpen()
    if not self.postSettings.sharpen.enable then return end

    local settings = self.postSettings.sharpen

    -- Single-pass CAS
    self:applyFilter('sharpen_cas', function(sh)
        sh:setFloat('casSharpness', settings.strength)

        sh:setFloat2('size', self.resX, self.resY) -- pixel size for offsets
    end)
end

function RenderCoreSystem:radialBlur()
    if not self.postSettings.radialblur.enable or self.postSettings.radialblur.strength <= 0 then return end

    local rb = self.postSettings.radialblur

    self:applyFilter('radialblur', function(sh)
        sh:setFloat('strength', rb.strength)
        sh:setFloat2('center', rb.center[1], rb.center[2])
    end)
end

---@param dt number
function RenderCoreSystem:tonemap(dt)
    if not self.postSettings.tonemap.enable then return end

    local settings = self.postSettings.tonemap
    local exposure = settings.exposure

    -- Space-game optimized auto-exposure: extremely stable, ignores bright stars/sun, very slow adaptation
    if settings.autoExpose.enable then
        local src = self.buffers[Enums.BufferName.buffer0]
        src:setMinFilter(TexFilter.Linear)
        src:genMipmap()

        -- Strong downsampling
        local targetMipSize = 512
        local mip = 0
        local size = src:getSize()
        while size.x > targetMipSize or size.y > targetMipSize do
            mip = mip + 1
            size.x = math.floor(size.x / 2)
            size.y = math.floor(size.y / 2)
        end
        mip = math.max(mip, 2)

        src:setMipRange(mip, mip)

        local smallSize = src:getSizeLevel(mip)
        local w, h = smallSize.x, smallSize.y

        -- Continuous random sampling: 128 samples
        local numSamples = 128
        local lumSamples = {}
        local maxLumCap = 0.05

        local seed = (self.frameCounter or 0) + dt * 1000
        math.randomseed(math.floor(seed * 1000))

        for i = 1, numSamples do
            local u = math.random()
            local v = math.random()

            local x = math.floor(u * (w - 1) + 0.5)
            local y = math.floor(v * (h - 1) + 0.5)

            local color = src:sample(x, y)

            local lum = color.x * 0.2126 + color.y * 0.7152 + color.z * 0.0722
            lum = math.min(lum, maxLumCap)
            table.insert(lumSamples, math.max(lum, 0.000001))
        end

        table.sort(lumSamples)

        -- Keep lowest 65%
        local validFraction = 0.65
        local validCount = math.max(1, math.floor(#lumSamples * validFraction))
        local logSum = 0.0
        for i = 1, validCount do
            logSum = logSum + math.log(lumSamples[i])
        end

        local logAvgLum          = logSum / validCount
        local avgLum             = math.exp(logAvgLum)

        -- Base target
        local targetExposure     = 0.0005 / avgLum

        -- Slight dark bias
        targetExposure           = targetExposure * 0.8

        local minTarget          = settings.autoExpose.minTarget
        local maxTarget          = settings.autoExpose.maxTarget
        targetExposure           = Math.Clamp(targetExposure, minTarget, maxTarget)

        self.autoExposure.target = targetExposure

        -- Extremely slow adaptation
        local ae                 = self.autoExposure
        local speedUp            = settings.autoExpose.speedUp
        local speedDown          = settings.autoExpose.speedDown
        local speed              = (targetExposure > ae.current) and speedUp or speedDown

        local lerpFactor         = dt * speed
        ae.current               = ae.current + (targetExposure - ae.current) * math.min(lerpFactor, 1.0)

        local minMultiplier      = 0.15 -- darkest allowed (relative to manual exposure setting)
        local maxMultiplier      = 5.0  -- brightest allowed
        ae.current               = Math.Clamp(ae.current, minMultiplier, maxMultiplier)

        exposure                 = exposure * ae.current

        -- Optional extra safety floor (can keep or remove)
        -- exposure = math.max(exposure, settings.exposure * 0.05)

        -- Restore
        src:setMipRange(0, 0)
    end

    -- Legacy path
    if settings.mode == Enums.Tonemappers.Legacy then
        local shader = Cache.Shader('ui', 'filter/tonemap_legacy')
        local target = self.buffers[Enums.BufferName.buffer1]
        target:pushLevel(self.level or 0)

        shader:start()
        shader:setTex2D('src', self.buffers[Enums.BufferName.buffer0])
        shader:setFloat('exposure', exposure)
        shader:setFloat2('size', self.resX, self.resY)
        local scale = 2 ^ (self.level or 0)
        Draw.Rect(0, 0, self.ssResX / scale, self.ssResY / scale)
        shader:stop()

        target:pop()
        self:swap()
        return
    end

    -- Modern tonemappers
    local modeId = 0
    if settings.mode == Enums.Tonemappers.Linear then
        modeId = 0
    elseif settings.mode == Enums.Tonemappers.Reinhard then
        modeId = 1
    elseif settings.mode == Enums.Tonemappers.ACES then
        modeId = 2
    elseif settings.mode == Enums.Tonemappers.Filmic then
        modeId = 3
    elseif settings.mode == Enums.Tonemappers.Uncharted2 then
        modeId = 4
    elseif settings.mode == Enums.Tonemappers.Lottes then
        modeId = 5
    elseif settings.mode == Enums.Tonemappers.Uchimura then
        modeId = 6
    elseif settings.mode == Enums.Tonemappers.GranTurismo then
        modeId = 7
    elseif settings.mode == Enums.Tonemappers.NarkowiczACES then
        modeId = 8
    elseif settings.mode == Enums.Tonemappers.ReinhardExt then
        modeId = 9
    elseif settings.mode == Enums.Tonemappers.ReinhardLum then
        modeId = 10
    elseif settings.mode == Enums.Tonemappers.AgX then
        modeId = 11
    elseif settings.mode == Enums.Tonemappers.Illustris then
        modeId = 12
    end

    self:applyFilter('tonemap', function(sh)
        sh:setInt('mode', modeId)
        sh:setFloat('exposure', exposure)
        sh:setFloat2('size', self.resX, self.resY)
    end)
end

function RenderCoreSystem:vignette()
    if not self.postSettings.vignette.enable then return end
    self:applyFilter('vignette', function(sh)
        sh:setFloat('strength', self.postSettings.vignette.strength)
        sh:setFloat('hardness', self.postSettings.vignette.hardness)
    end)
end

function RenderCoreSystem:aberration()
    if not self.postSettings.aberration.enable then return end
    self:applyFilter('aberration', function(sh)
        sh:setFloat('strength', self.postSettings.aberration.strength)
    end)
end

function RenderCoreSystem:dither()
    if not self.postSettings.dither.enable then return end

    self:applyFilter('dither', function(sh)
        sh:setFloat('strength', self.postSettings.dither.strength)
    end)
end

function RenderCoreSystem:colorgrade()
    if not self.postSettings.colorgrade.enable then return end

    local settings = self.postSettings.colorgrade

    local modeId = 0
    if settings.mode == Enums.ColorGrades.Neutral then
        modeId = 0
    elseif settings.mode == Enums.ColorGrades.Cinematic then
        modeId = 1
    elseif settings.mode == Enums.ColorGrades.Space then
        modeId = 2
    elseif settings.mode == Enums.ColorGrades.Warm then
        modeId = 3
    elseif settings.mode == Enums.ColorGrades.Cool then
        modeId = 4
    elseif settings.mode == Enums.ColorGrades.Vibrant then
        modeId = 5
    elseif settings.mode == Enums.ColorGrades.Bleach then
        modeId = 6
    end

    self:applyFilter('colorgrade', function(sh)
        sh:setInt('mode', modeId)
        sh:setFloat('preExposure', settings.preExposure)
        sh:setFloat('temperature', settings.temperature)
        sh:setFloat('tint', settings.tint)
        sh:setFloat('saturation', settings.saturation)
        sh:setFloat('contrast', settings.contrast)
        sh:setFloat('brightness', settings.brightness)
        sh:setFloat('vibrance', settings.vibrance)
        sh:setFloat3('lift', settings.lift[1], settings.lift[2], settings.lift[3])
        sh:setFloat3('gamma', settings.gamma[1], settings.gamma[2], settings.gamma[3])
        sh:setFloat3('gain', settings.gain[1], settings.gain[2], settings.gain[3])
    end)
end

function RenderCoreSystem:present(x, y, sx, sy, useMips)
    RenderState.PushAllDefaults()
    local sh = Cache.Shader('ui', 'filter/identity')
    sh:start()
    sh:setTex2D("src", self.buffers[Enums.BufferName.buffer0])
    Draw.Rect(x, y + sy, sx, -sy)
    sh:stop()
    RenderState.PopAll()
end

function RenderCoreSystem:presentAll(x, y, sx, sy)
    RenderState.PushAllDefaults()
    local sh = Cache.Shader('ui', 'filter/identity')
    sh:start()
    local function draw(bufKey, px, py)
        sh:setTex2D("src", self.buffers[bufKey])
        Draw.Rect(px, py, sx / 2, -sy / 2)
    end
    draw(Enums.BufferName.buffer0, x, y + sy / 2)
    draw(Enums.BufferName.buffer1, x + sx / 2, y + sy / 2)
    draw(Enums.BufferName.buffer2, x, y)
    draw(Enums.BufferName.zBufferL, x + sx / 2, y)
    sh:stop()
    RenderState.PopAll()
end

function RenderCoreSystem:getFPS()
    return self.currentFPS
end

---@param inMs boolean
function RenderCoreSystem:getFrameTime(inMs)
    if inMs then
        return self.currentFrameTime * 1000
    else
        return self.currentFrameTime
    end
end

---@param inMs boolean
function RenderCoreSystem:getSmoothFrameTime(inMs)
    if inMs then
        return self.smoothFrameTime * 1000
    else
        return self.smoothFrameTime
    end
end

function RenderCoreSystem:getSmoothFPS()
    return self.smoothFPS
end

return RenderCoreSystem()
