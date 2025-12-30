local Registry           = require("Core.ECS.Registry")
local QuickProfiler      = require("Shared.Tools.QuickProfiler")
local RenderingPass      = require("Shared.Rendering.RenderingPass")
local CameraManager      = require("Modules.Cameras.Managers.CameraManager")
local RenderComp         = require("Modules.Rendering.Components").Render
local PointLightComp     = require("Modules.Rendering.Components").PointLight
local CameraComponent    = require("Modules.Cameras.Components.CameraDataComponent")
local UniformFuncs       = require("Shared.Rendering.UniformFuncs")
local Cache              = require("Render.Cache")
local Physics            = require("Modules.Physics.Components")
local RenderOverlay      = require("Shared.Tools.RenderOverlay")
local ShaderErrorOverlay = require("Shared.Tools.ShaderErrorOverlay")
local ShaderHotReload    = require("Render.ShaderHotReload")

---@class RenderCoreSystem
---@overload fun(self): RenderCoreSystem
---@overload fun(): RenderCoreSystem
local RenderCoreSystem = Class("RenderCoreSystem", function(self)
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    -- Initialize shader hot-reload (must be after MaterialDefs to register existing shaders)
    ShaderHotReload:init()

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
        panini     = Config.render.postFx.panini,
    }

    self.autoExposure    = {
        current = 1.0, -- current adapted exposure
        target  = 1.0, -- what we're adapting toward this frame
    }

    local win            = Window:size()
    self.resX, self.resY = win.x, win.y
    self.ssResX          = self.resX * self.settings.superSampleRate
    self.ssResY          = self.resY * self.settings.superSampleRate
    self.dsResX          = self.resX / self.settings.downSampleRate
    self.dsResY          = self.resY / self.settings.downSampleRate

    self.ds              = 4  -- downsample factor for bloom (matches old pipeline)

    self.materialCache   = {} -- material → { var = {type, values} }
    self.instanceCache   = {} -- entity  → { mat  = { var = {type, values} } }
    self.processedMats   = {}

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
        [Enums.BufferName.buffer0]    = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.buffer1]    = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.buffer2]    = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.zBuffer]    = create(self.ssResX, self.ssResY, TexFormat.Depth32F),
        [Enums.BufferName.zBufferL]   = create(self.ssResX, self.ssResY, TexFormat.R32F),
        [Enums.BufferName.dsBuffer0]  = create(self.dsResX, self.dsResY, TexFormat.RGBA16F),
        [Enums.BufferName.dsBuffer1]  = create(self.dsResX, self.dsResY, TexFormat.RGBA16F),
        [Enums.BufferName.lightAccum] = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
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

    -- Deferred lighting pass - accumulates point/spot light contributions
    pass(Enums.RenderingPasses.Lighting,
        BlendMode.Additive, CullFace.None, false, false,
        { Enums.BufferName.lightAccum },
        function()
            Draw.Clear(0, 0, 0, 0)
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

    -- Data cache
    self:cacheData()

    -- Opaque Pass (writes to G-buffer: albedo, normalMat, depth)
    self.currentPass = Enums.RenderingPasses.Opaque
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self:renderInOrder(BlendMode.Disabled)
    self.passes[self.currentPass]:stop()

    -- Deferred Lighting Pass (only if there are point lights)
    -- This avoids breaking scenes without deferred lighting (e.g., RenderingTest)
    local hasPointLights = false
    for _ in Registry:view(PointLightComp) do
        hasPointLights = true
        break
    end

    if hasPointLights then
        self.currentPass = Enums.RenderingPasses.Lighting
        self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
        self:renderGlobalLight()
        self:renderPointLights()
        self.passes[self.currentPass]:stop()

        -- Deferred Composite (albedo * lighting → buffer0)
        self:compositeDeferredLighting()
    end

    -- Additive Pass
    self.currentPass = Enums.RenderingPasses.Additive
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self:renderInOrder(BlendMode.Additive)
    self.passes[self.currentPass]:stop()

    -- Alpha Pass
    self.currentPass = Enums.RenderingPasses.Alpha
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self:renderInOrder(BlendMode.Alpha)
    self.passes[self.currentPass]:stop()

    -- UI Pass
    self.currentPass = Enums.RenderingPasses.UI
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self.passes[self.currentPass]:stop()

    -- Manual UI Composite: buffer0 (scene) + buffer1 (UI) → buffer2
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

    -- Post-processing chain
    self:downsampleForPost()

    -- HDR effects (before tonemapping)
    self:aberration(dt)
    self:bloom(dt)
    self:colorgrade(dt)
    self:tonemap(dt)         -- HDR → LDR conversion

    -- LDR effects (after tonemapping)
    self:fxaa(dt)            -- FXAA needs LDR values for edge detection
    self:sharpen(dt)
    self:dither(dt)
    self:panini(dt)          -- FOV distortion correction (before vignette)
    self:vignette(dt)
    self:radialBlur(dt)

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

    -- Update shader hot-reload (poll for file changes)
    ShaderHotReload:update()

    -- Draw performance overlay (Shift+O to toggle)
    RenderOverlay:draw()

    -- Draw shader error overlay (auto-shows on compile errors)
    ShaderErrorOverlay:draw()
end

--- Handle input for render overlays
function RenderCoreSystem:handleInput()
    -- Shader error overlay takes priority (can be dismissed with ESC/click)
    if ShaderErrorOverlay:handleInput() then
        return
    end
    RenderOverlay:handleInput()
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
    -- Use material-grouped batching when render thread is active
    -- This reduces shader switches by grouping entities by material
    if Engine:isRenderThreadActive() then
        self:renderBatched(blendMode)
    else
        self:renderDirect(blendMode)
    end
end

-- Direct rendering (traditional path - one shader switch per entity)
function RenderCoreSystem:renderDirect(blendMode)
    for entity in Registry:view(RenderComp) do
        local rend = entity:get(RenderComp)
        if rend:getRenderFn() then
            rend:getRenderFn()(entity, blendMode)
        else
            for meshmat in Iterator(rend:getMeshes()) do
                local mat = meshmat.material
                if (mat:getBlendMode() or BlendMode.Disabled) == blendMode then
                    local sh = mat:getShaderState()
                    sh:start()

                    self:applyCachedVars(mat, entity)

                    meshmat.mesh:draw()
                    sh:stop()
                end
            end
        end
    end
end

-- Batch rendering (material-grouped path)
-- Groups entities by material to minimize shader switches
function RenderCoreSystem:renderBatched(blendMode)
    -- Phase 1: Collect draw calls grouped by material
    local materialGroups = {} -- material -> { {entity, mesh}, ... }

    for entity in Registry:view(RenderComp) do
        local rend = entity:get(RenderComp)

        -- Custom render functions bypass batching
        if rend:getRenderFn() then
            rend:getRenderFn()(entity, blendMode)
        else
            for meshmat in Iterator(rend:getMeshes()) do
                local mat = meshmat.material
                if (mat:getBlendMode() or BlendMode.Disabled) == blendMode then
                    -- Group by material
                    if not materialGroups[mat] then
                        materialGroups[mat] = {}
                    end
                    table.insert(materialGroups[mat], {
                        entity = entity,
                        mesh = meshmat.mesh
                    })
                end
            end
        end
    end

    -- Phase 2: Render each material group
    for mat, drawCalls in pairs(materialGroups) do
        local sh = mat:getShaderState()
        sh:start() -- Binds shader, sets textures, auto vars

        -- Apply material-level cached vars (once per material)
        local matCache = self.materialCache[mat]
        if matCache then
            local shader = sh:shader()
            for varObj, entry in pairs(matCache) do
                if varObj.uniformInt then
                    local fn = UniformFuncs[entry.type]
                    if fn then fn(shader, varObj.uniformInt, table.unpack(entry.values)) end
                end
            end
        end

        -- Draw all entities with this material
        for _, drawCall in ipairs(drawCalls) do
            local entity = drawCall.entity
            local mesh = drawCall.mesh

            -- Apply per-instance cached vars
            local instCache = self.instanceCache[entity.id]
            if instCache then
                local matInst = instCache[mat]
                if matInst then
                    local shader = sh:shader()
                    for varObj, entry in pairs(matInst) do
                        if varObj.uniformInt then
                            local fn = UniformFuncs[entry.type]
                            if fn then fn(shader, varObj.uniformInt, table.unpack(entry.values)) end
                        end
                    end
                end
            end

            mesh:draw()
        end

        sh:stop()
    end
end

function RenderCoreSystem:cacheData()
    self.materialCache = {}
    self.instanceCache = {}
    local processedMats = {}

    local eye = CameraManager:getEye()

    for entity in Registry:view(RenderComp) do
        local rend = entity:get(RenderComp)
        if rend:getRenderFn() then goto next_entity end

        for meshmat in Iterator(rend:getMeshes()) do
            local mat = meshmat.material

            -- material cache (per material)
            if not processedMats[mat] then
                processedMats[mat] = true
                local matCache = {}

                for _, v in ipairs(mat.staticShaderVars or {}) do
                    matCache[v] = { type = v.uniformType, values = v:getValues(eye, entity) }
                end
                for _, v in ipairs(mat.constShaderVars or {}) do
                    matCache[v] = { type = v.uniformType, values = v:getValues(eye, entity) }
                end

                for _, v in ipairs(mat.autoShaderVars or {}) do
                    if not v.perInstance then
                        matCache[v] = { type = v.uniformType, values = v:getValues(eye, entity) }
                    end
                end

                self.materialCache[mat] = matCache
            end

            -- instance cache (per instance)
            local instCache = self.instanceCache[entity.id] or {}
            local matInstCache = instCache[mat] or {}

            for _, v in ipairs(mat.autoShaderVars or {}) do
                if v.perInstance then
                    matInstCache[v] = { type = v.uniformType, values = v:getValues(eye, entity) }
                end
            end

            instCache[mat] = matInstCache
            self.instanceCache[entity.id] = instCache
        end
        ::next_entity::
    end
end

function RenderCoreSystem:applyCachedVars(mat, entity)
    local shader = mat:getShaderState():shader()

    -- material level
    local matCache = self.materialCache[mat]
    if matCache then
        for varObj, entry in pairs(matCache) do
            if varObj.uniformInt then
                local fn = UniformFuncs[entry.type]
                if fn then fn(shader, varObj.uniformInt, table.unpack(entry.values)) end
            end
        end
    end

    -- instance level (per-entity)
    local instCache = self.instanceCache[entity.id]
    if instCache then
        local matInst = instCache[mat]
        if matInst then
            for varObj, entry in pairs(matInst) do
                if varObj.uniformInt then
                    local fn = UniformFuncs[entry.type]
                    if fn then fn(shader, varObj.uniformInt, table.unpack(entry.values)) end
                end
            end
        end
    end
end

-- =============================================================================
-- Deferred Lighting
-- =============================================================================

--- Render global lighting (environment/ambient) using G-buffer
function RenderCoreSystem:renderGlobalLight()
    local shader = Cache.Shader('worldray', 'light/global')
    shader:start()

    -- Bind G-buffer textures
    shader:setTex2D('texNormalMat', self.buffers[Enums.BufferName.buffer1])
    shader:setTex2D('texDepth', self.buffers[Enums.BufferName.zBufferL])

    -- Global light direction (star direction from skybox if available)
    shader:setFloat3('lightPos', 10000, 5000, 10000)
    shader:setFloat3('lightColor', 1, 1, 1)

    Draw.Rect(0, 0, self.ssResX, self.ssResY)
    shader:stop()
end

--- Render all point lights using deferred shading
function RenderCoreSystem:renderPointLights()
    local shader = Cache.Shader('worldray', 'light/point')
    local eye = CameraManager:getEye()
    local lightCount = 0

    -- Bind shader and textures ONCE before the loop (optimization)
    shader:start()
    shader:setTex2D('texNormalMat', self.buffers[Enums.BufferName.buffer1])
    shader:setTex2D('texDepth', self.buffers[Enums.BufferName.zBufferL])

    for entity in Registry:view(PointLightComp) do
        local light = entity:get(PointLightComp)
        local transform = entity:get(Physics.Transform)
        if not transform then goto continue end

        -- Use camera-relative position (camera at origin)
        -- worldray.glsl reconstructs fragment positions in camera-relative space
        local worldPos = transform:getPos()
        local pos = worldPos:relativeTo(eye)
        local color = light:getColor()
        local intensity = light:getIntensity()
        local radius = light:getRadius()

        lightCount = lightCount + 1
        if lightCount == 1 then
            Log.Debug("[Deferred] First light: camRelPos=(%f,%f,%f) color=(%f,%f,%f) intensity=%f radius=%f",
                pos.x, pos.y, pos.z, color.x, color.y, color.z, intensity, radius)
        end

        -- Update light UBO with camera-relative light position (only this changes per light)
        Engine:updateLightUBO(
            pos.x, pos.y, pos.z, radius,
            color.x, color.y, color.z, intensity
        )

        Draw.Rect(0, 0, self.ssResX, self.ssResY)

        ::continue::
    end

    shader:stop()
end

--- Composite deferred lighting: albedo * lighting → buffer0
function RenderCoreSystem:compositeDeferredLighting()
    local shader = Cache.Shader('worldray', 'light/composite')

    -- Save current albedo buffer
    local albedoBuffer = self.buffers[Enums.BufferName.buffer0]

    -- Render composite into buffer2 (scratch)
    local buffer2 = self.buffers[Enums.BufferName.buffer2]
    buffer2:push()
    Draw.Clear(0, 0, 0, 0)

    shader:start()
    shader:setTex2D('texAlbedo', albedoBuffer)
    shader:setTex2D('texDepth', self.buffers[Enums.BufferName.zBufferL])
    shader:setTex2D('texLighting', self.buffers[Enums.BufferName.lightAccum])
    Draw.Rect(0, 0, self.ssResX, self.ssResY)
    shader:stop()

    buffer2:pop()

    -- Swap: composite result becomes the new main buffer
    self.buffers[Enums.BufferName.buffer0], self.buffers[Enums.BufferName.buffer2] =
        self.buffers[Enums.BufferName.buffer2], self.buffers[Enums.BufferName.buffer0]
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

function RenderCoreSystem:bloom(radius)
    if not self.postSettings.bloom.enable then return end

    local A = self.buffers[Enums.BufferName.dsBuffer0]
    local B = self.buffers[Enums.BufferName.dsBuffer1]
    local iterations = self.postSettings.bloom.iterations or 1
    local useFastBlur = self.postSettings.bloom.fastBlur ~= false

    -- Bright extract (downsample to 1/4 resolution)
    do
        local shader = Cache.Shader('ui', 'filter/bloompre')
        A:push()
        shader:start()
        shader:setTex2D('src', self.buffers[Enums.BufferName.buffer0])
        Draw.Rect(0, 0, self.resX / self.ds, self.resY / self.ds)
        shader:stop()
        A:pop()
    end

    -- Blur passes (optimized: fewer iterations, fast blur shader)
    local intensity = self.postSettings.bloom.intensity or 0.15
    for i = 1, iterations do
        if useFastBlur then
            self:blurFast(B, A, 1, 0)
            self:blurFast(A, B, 0, 1)
        else
            local width = radius * 0.2
            self:blur(B, A, 1, 0, radius, width)
            self:blur(A, B, 0, 1, radius, width)
        end

        self:applyFilter('bloomcomposite', function(sh)
            sh:setTex2D('srcBlur', A)
            sh:setFloat('intensity', intensity)
        end)
    end
end

function RenderCoreSystem:blurFast(dst, src, dx, dy)
    local shader = Cache.Shader('ui', 'filter/blur_fast')
    local size = src:getSize()
    dst:push()
    shader:start()
    shader:setFloat2('dir', dx, dy)
    shader:setFloat2('size', size.x, size.y)
    shader:setTex2D('src', src)
    Draw.Rect(0, 0, size.x, size.y)
    shader:stop()
    dst:pop()
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

function RenderCoreSystem:fxaa()
    if not self.postSettings.fxaa.enable then return end

    local settings = self.postSettings.fxaa
    local shaderName = settings.fast and 'fxaa_fast' or 'fxaa'

    self:applyFilter(shaderName, function(sh)
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

function RenderCoreSystem:panini()
    if not self.postSettings.panini.enable then return end

    local settings = self.postSettings.panini
    local distance = settings.distance

    -- Skip if distance is effectively zero
    if distance <= 0.001 then return end

    -- Auto-calculate vertical scale based on FOV if not specified
    local scale = settings.scale
    if scale <= 0 then
        -- Compensate for FOV-induced vertical stretching
        local fov = Config.render.camera.fov
        scale = 1.0 + (fov - 60) * 0.002  -- Subtle adjustment based on FOV
    end

    self:applyFilter('panini', function(sh)
        sh:setFloat('distance', distance)
        sh:setFloat('scale', scale)
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
