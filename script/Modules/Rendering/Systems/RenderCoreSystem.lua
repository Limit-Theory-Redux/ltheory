local Registry           = require("Core.ECS.Registry")
local QuickProfiler      = require("Shared.Tools.QuickProfiler")
local CameraManager      = require("Modules.Cameras.Managers.CameraManager")
local RenderComp         = require("Modules.Rendering.Components").Render
local PointLightSystem   = require("Modules.Rendering.Systems.PointLightSystem")
local LightManager       = require("Modules.Rendering.Managers.LightManager")
local CameraComponent    = require("Modules.Cameras.Components.CameraDataComponent")
local RigidBodyComponent = require("Modules.Physics.Components.RigidBodyComponent")
local Cache              = require("Render.Cache")
local Pipelines          = require("Render.Pipelines")

-- Pipeline states of the fullscreen passes (post-processing, lighting,
-- present). The vertex shader (`fullscreen_ndc`, `fullscreen_ray`) maps the
-- built-in unit quad to the viewport, so no state besides blending matters.
---@type PipelineState
local FullscreenState = { vertex = VertexLayout.Fullscreen }
---@type PipelineState
local FullscreenAdditive = { vertex = VertexLayout.Fullscreen, blend = BlendMode.Additive }

-- Descriptors of the scene passes are cached per texture set. The buffers are
-- swapped between passes (buffer0/buffer1/buffer2), so a few distinct sets
-- recur; anything beyond this is a stale set from before a resize.
local MAX_CACHED_DESCS = 4

-- Scratch for `buildPassLists`: out-param of RigidBody:getPos (double precision),
-- reused across entities and frames to avoid per-entity allocation.
local scratchPos = Position()

---@class RenderCoreSystem
---@overload fun(self): RenderCoreSystem
---@overload fun(): RenderCoreSystem
local RenderCoreSystem = Class("RenderCoreSystem", function(self)
    require("Shared.Definitions.MaterialDefs")

    self:registerVars()
    self:registerPasses()
end)

function RenderCoreSystem:registerVars()
    self.profiler        = QuickProfiler("RenderCoreSystem", false, false)

    self.settings        = {
        superSampleRate = Config.render.general.superSampleRate,
        downSampleRate  = Config.render.general.downSampleRate,
        showBuffers     = Config.render.debug.showBuffers,
        cullFace        = Config.render.renderState.cullFace,
        deferredLighting = Config.render.general.deferredLighting ~= false,
        frustumCulling  = Config.render.general.frustumCulling,
    }

    -- What the scene passes draw: filled once per frame by `buildPassLists`
    -- (entries are reused in place), then culled, sorted and emitted per pass
    -- by `SceneList:submit`. Render-fn entities are kept apart in
    -- `passRenderFns` (tracked via an explicit `.n` count, since a reused
    -- table can have stale entries past the current frame's count).
    self.scene         = SceneList.Create()
    self.passRenderFns = { n = 0 }
    self.cullStats = { submitted = 0, visible = 0, culled = 0 }

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

    local win            = Window:size()
    self.resX, self.resY = win.x, win.y
    self.ssResX          = self.resX * self.settings.superSampleRate
    self.ssResY          = self.resY * self.settings.superSampleRate
    self.dsResX          = self.resX / self.settings.downSampleRate
    self.dsResY          = self.resY / self.settings.downSampleRate

    self.ds              = 4  -- downsample factor for bloom (matches old pipeline)

    self.buffers         = {}
    -- Single-color-attachment pass descriptors, cached per texture (weak, so
    -- a resized-away buffer takes its descriptors with it); see `colorPassDesc`.
    self.passDescCache   = setmetatable({}, { __mode = 'k' })
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

--- Toggle frustum culling at runtime (e.g. for A/B comparison).
---@param enable boolean
function RenderCoreSystem:setFrustumCulling(enable)
    self.settings.frustumCulling = enable
end

---@return { culled: integer, submitted: integer, visible: integer }
function RenderCoreSystem:getCullStats()
    return self.cullStats
end

function RenderCoreSystem:initializeBuffers()
    local function create(x, y, fmt)
        local t = Tex2D.Create(x, y, fmt)
        t:setMagFilter(TexFilter.Linear)
        t:setMinFilter(TexFilter.Linear)
        t:setWrapMode(TexWrapMode.Clamp)
        -- A depth buffer has nothing to clear here (the opaque pass clears it).
        if not TexFormat.IsDepth(fmt) then t:clear(0, 0, 0, 0) end
        t:genMipmap()
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
    }
end

function RenderCoreSystem:registerPasses()
    -- Fixed-function state (blend, culling, depth) is not part of a pass: every
    -- scene draw brings its pipeline (see `Render.Pipelines` for the state of
    -- each pass), so a pass is only its attachments and load ops.
    ---@param clear { color: number[]|nil, depth: number|nil }|nil nil keeps the contents (LoadOp.Load)
    local function pass(name, bufs, clear)
        self.passes[name] = { name = tostring(name), bufferOrder = bufs, clear = clear or {}, descs = {} }
    end

    pass(Enums.RenderingPasses.Opaque,
        { Enums.BufferName.buffer0, Enums.BufferName.buffer1, Enums.BufferName.zBufferL, Enums.BufferName.zBuffer },
        { color = { 0, 0, 0, 0 }, depth = 1 })

    pass(Enums.RenderingPasses.Additive,
        { Enums.BufferName.buffer0, Enums.BufferName.zBuffer })

    pass(Enums.RenderingPasses.Alpha,
        { Enums.BufferName.buffer0, Enums.BufferName.zBuffer })

    pass(Enums.RenderingPasses.UI,
        { Enums.BufferName.buffer1, Enums.BufferName.zBuffer },
        { color = { 0, 0, 0, 0 } })

    -- The window: one backbuffer pass that draws the final image.
    self.presentDesc = RenderPassDesc.Create('Present')
end

--- The `RenderPassDesc` of a scene pass for the current buffers: color
--- attachments in `bufferOrder` order, the depth-format buffer as depth.
---@param spec table scene pass spec made by `registerPasses`
---@return RenderPassDesc
function RenderCoreSystem:scenePassDesc(spec)
    local buffers = self.buffers
    local order = spec.bufferOrder
    local descs = spec.descs
    for _, entry in ipairs(descs) do
        local same = true
        for i = 1, #order do
            if entry.textures[i] ~= buffers[order[i]] then
                same = false
                break
            end
        end
        if same then return entry.desc end
    end

    local desc = RenderPassDesc.Create(spec.name)
    local textures = {}
    local colorIndex = 0
    local c = spec.clear.color
    local d = spec.clear.depth
    for i = 1, #order do
        local tex = buffers[order[i]]
        textures[i] = tex
        if TexFormat.IsDepth(tex:getFormat()) then
            desc:depth(tex:view(), d and LoadOp.Clear or LoadOp.Load, d or 1.0)
        else
            if c then
                desc:color(colorIndex, tex:view(), LoadOp.Clear, c[1], c[2], c[3], c[4])
            else
                desc:color(colorIndex, tex:view(), LoadOp.Load, 0, 0, 0, 0)
            end
            colorIndex = colorIndex + 1
        end
    end

    if #descs >= MAX_CACHED_DESCS then
        table.remove(descs, 1)
    end
    descs[#descs + 1] = { textures = textures, desc = desc }
    return desc
end

--- Begin a scene pass on the current buffers.
---@param name RenderingPassName
---@return RenderPass
function RenderCoreSystem:beginScenePass(name)
    return Renderer:beginPass(self:scenePassDesc(self.passes[name]))
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

    -- Only the final present draws to the window, in its own backbuffer pass.
    ClipRect.PushDisabled()
    CameraManager:updateViewMatrix()
    CameraManager:updateProjectionMatrix(self.resX, self.resY)
    CameraManager:beginDraw()

    -- Refresh the generic ECS light snapshot once per frame. All render passes
    -- (including reusable diagnostics) consume this same snapshot.
    PointLightSystem:update(dt)

    -- Collect what the scene passes draw once (each pass submits its bucket).
    self:buildPassLists()

    -- Opaque Pass
    Profiler.Begin('Render.Opaque')
    self.currentPass = Enums.RenderingPasses.Opaque
    do
        local pass = self:beginScenePass(self.currentPass)
        self:renderInOrder(pass, BlendMode.Disabled)
        pass:finish()
    end
    Profiler.End() -- Render.Opaque

    -- Deferred Lighting Pass
    if self.settings.deferredLighting then
        Profiler.Begin('Render.Lighting.Deferred')
        self:deferredLighting()
        Profiler.End() -- Render.Lighting.Deferred
    end

    -- Additive Pass
    Profiler.Begin('Render.Additive')
    self.currentPass = Enums.RenderingPasses.Additive
    do
        local pass = self:beginScenePass(self.currentPass)
        self:renderInOrder(pass, BlendMode.Additive)
        pass:finish()
    end
    Profiler.End() -- Render.Additive

    -- Alpha Pass
    Profiler.Begin('Render.Alpha')
    self.currentPass = Enums.RenderingPasses.Alpha
    do
        local pass = self:beginScenePass(self.currentPass)
        self:renderInOrder(pass, BlendMode.Alpha)
        pass:finish()
    end
    Profiler.End() -- Render.Alpha

    local scene, st = self.scene, self.cullStats
    st.submitted, st.visible, st.culled = scene:getSubmitted(), scene:getVisible(), scene:getCulled()

    -- UI Pass
    Profiler.Begin('Render.UI')
    self.currentPass = Enums.RenderingPasses.UI
    self:beginScenePass(self.currentPass):finish()
    Profiler.End() -- Render.UI

    -- Manual UI Composite: buffer0 (scene) + buffer1 (UI) → buffer2
    Profiler.Begin('Render.UI.Composite')
    do
        local buffer2 = self.buffers[Enums.BufferName.buffer2]
        local pass = Renderer:beginPass(self:colorPassDesc('UI.Composite', buffer2, 0, true))

        local shader = Cache.Shader('fullscreen_ndc', 'ui/composite')
        pass:setPipeline(Pipelines.get(shader, FullscreenState))
        pass:setInputs(
            self.buffers[Enums.BufferName.buffer0]:mipView(0), Samplers.LinearClamp, -- srcBottom
            self.buffers[Enums.BufferName.buffer1]:mipView(0), Samplers.LinearClamp) -- srcTop
        pass:drawFullscreen()

        pass:finish()

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

    Profiler.Begin('Render.Present')
    self.presentDesc:backbuffer(self.resX, self.resY, LoadOp.Load, 0, 0, 0, 1)
    do
        local pass = Renderer:beginPass(self.presentDesc)
        if self.settings.showBuffers then
            self:presentAll(pass, self.resX, self.resY)
        else
            self:present(pass)
        end
        pass:finish()
    end
    Profiler.End()

    ClipRect.Pop()

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

    -- Post-processing samples explicit mip views (`downsampleForPost` picks the level)
    self.level = 0
end

--- Draw one scene pass: the custom render fns, then the scene list's bucket
--- of the pass's blend mode.
---@param pass RenderPass the open pass
---@param blendMode BlendMode
function RenderCoreSystem:renderInOrder(pass, blendMode)
    local eye = CameraManager:getEye()

    if blendMode == BlendMode.Additive then
        PointLightSystem:renderDiagnostics(blendMode, eye)
    end

    -- Custom render fns are called in every pass (their blend mode isn't
    -- queryable); they set their own pipeline. Mesh entities are in the scene
    -- list, so `submit` only touches the meshes that draw in this pass.
    local fns = self.passRenderFns
    Profiler.Begin('Render.Fns')
    for fi = 1, fns.n do
        local fnEntry = fns[fi]
        fnEntry.fn(fnEntry.entity, blendMode)
    end
    Profiler.End()

    -- Frustum cull, sort by (pipeline, material, mesh) (alpha keeps insertion
    -- order), per-draw callbacks of the survivors, then the draws.
    Profiler.Begin('Render.Scene')
    self.scene:submit(pass, blendMode, self.settings.frustumCulling)
    Profiler.End()
end

--- Collect what the scene passes draw, once per frame (the passes run 3x and
--- each submits its own bucket). One transform per mesh entity, written in
--- place by the rigid body, and one item per mesh. Culling, sorting and the
--- per-draw values happen in `SceneList:submit`.
function RenderCoreSystem:buildPassLists()
    local scene = self.scene
    local passRenderFns = self.passRenderFns

    scene:reset()
    passRenderFns.n = 0

    local eye = CameraManager:getEye()

    for entity in Registry:view(RenderComp) do
        local rend = entity:get(RenderComp)
        if not rend:isVisible() then goto next_entity end

        if rend:getRenderFn() then
            local n = passRenderFns.n + 1
            passRenderFns.n = n
            local fnEntry = passRenderFns[n]
            if not fnEntry then
                fnEntry = {}
                passRenderFns[n] = fnEntry
            end
            fnEntry.fn = rend:getRenderFn()
            fnEntry.entity = entity
        elseif rend:getMeshes() then
            -- All meshes of an entity (hull/turrets/thrusters) share the same
            -- rigid-body transform: one transform, written straight into the
            -- list, serves them all (mWorldIT is derived once per transform
            -- when it draws).
            --
            -- Cull sphere centre: the rigid body's position, camera-relative
            -- to match the camera-relative render (CameraManager:beginDraw
            -- pushes eye = (0,0,0)). Plain double subtraction rather than
            -- Position:relativeTo(), which returns a boxed Vec3 by value.
            --
            -- `scale` is the factor mWorld carries (RigidBody::get_to_world_matrix
            -- multiplies by get_scale), so mesh-local extents are scaled by it
            -- to get world extents. No rigid body -> `scale = -1`, the "never
            -- cull" sentinel, and an identity transform.
            local t = scene:addTransform()
            local rbc = entity:get(RigidBodyComponent)
            if rbc then
                local rb = rbc:getRigidBody()
                rb:getPos(scratchPos)
                t.cx, t.cy, t.cz = scratchPos.x - eye.x, scratchPos.y - eye.y, scratchPos.z - eye.z
                t.scale = rbc:getScale()
                rb:getToWorldMatrixInto(eye, t.world)
            else
                t.cx, t.cy, t.cz = 0, 0, 0
                t.scale = -1
                local m = t.world.m
                m[0], m[5], m[10], m[15] = 1, 1, 1, 1
            end

            local index = t.index
            local meshes = rend:getMeshes()
            for mi = 1, #meshes do
                local meshmat = meshes[mi]
                scene:addItem(index, meshmat.mesh, meshmat.material, entity)
            end
        end
        ::next_entity::
    end
end

-- Post-processing helpers
function RenderCoreSystem:swap()
    self.buffers[Enums.BufferName.buffer0], self.buffers[Enums.BufferName.buffer1] =
        self.buffers[Enums.BufferName.buffer1], self.buffers[Enums.BufferName.buffer0]
end

--- Pass descriptor for rendering into mip `level` of `tex`, cached per
--- (texture, label, level, clear). `clear` selects LoadOp.Clear to transparent
--- black, otherwise the contents are kept (LoadOp.Load).
---@param label string
---@param tex Tex2D
---@param level integer
---@param clear boolean
---@return RenderPassDesc
function RenderCoreSystem:colorPassDesc(label, tex, level, clear)
    local byTex = self.passDescCache[tex]
    if not byTex then
        byTex = {}
        self.passDescCache[tex] = byTex
    end
    local byLabel = byTex[label]
    if not byLabel then
        byLabel = {}
        byTex[label] = byLabel
    end
    local key = level * 2 + (clear and 1 or 0)
    local desc = byLabel[key]
    if not desc then
        desc = RenderPassDesc.Create(label)
        desc:color(0, tex:mipView(level), clear and LoadOp.Clear or LoadOp.Load, 0, 0, 0, 0)
        byLabel[key] = desc
    end
    return desc
end

-- `Params` struct types of the filters, by shader name.
local paramsTypes = {}

--- One post-processing filter pass: the current level of `buffer0` through
--- `filter/<fragName>` into the same level of `buffer1`, then the two swap.
--- `fill(p)` writes the filter's group-2 `Params` block (nil for a filter
--- without one). `extra` is a second sampled view (slot 1).
---@param fragName string
---@param fill fun(p: ffi.cdata*)|nil
---@param extra TexView|nil
function RenderCoreSystem:applyFilter(fragName, fill, extra)
    local shader = Cache.Shader('fullscreen_ndc', 'filter/' .. fragName)
    local level = self.level or 0
    local target = self.buffers[Enums.BufferName.buffer1]
    local pass = Renderer:beginPass(self:colorPassDesc('Post.' .. fragName, target, level, false))

    pass:setPipeline(Pipelines.get(shader, FullscreenState))
    local src = self.buffers[Enums.BufferName.buffer0]:mipView(level)
    if extra then
        pass:setInputs(src, Samplers.LinearClamp, extra, Samplers.LinearClamp)
    else
        pass:setInputs(src, Samplers.LinearClamp)
    end
    if fill then
        local T = paramsTypes[fragName]
        if not T then
            T = shader:blockType('Params')
            paramsTypes[fragName] = T
        end
        fill(pass:alloc(T))
    end
    pass:drawFullscreen()

    pass:finish()
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

    local currentLevel = 0
    local currentSizeX = self.ssResX
    local currentSizeY = self.ssResY

    while currentSizeX > self.resX or currentSizeY > self.resY do
        currentLevel = currentLevel + 1
        currentSizeX = math.floor(currentSizeX / 2)
        currentSizeY = math.floor(currentSizeY / 2)

        -- Downsample level N-1 of buffer0 into level N of buffer1 (a bilinear
        -- resolve over the whole level)
        local target = self.buffers[Enums.BufferName.buffer1]
        local pass = Renderer:beginPass(self:colorPassDesc('Post.downsample', target, currentLevel, false))

        local shader = Cache.Shader('fullscreen_ndc', 'filter/downsample')
        pass:setPipeline(Pipelines.get(shader, FullscreenState))
        pass:setInputs(self.buffers[Enums.BufferName.buffer0]:mipView(currentLevel - 1), Samplers.LinearClamp)
        pass:drawFullscreen()
        pass:finish()

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

--- Enable or disable the deferred lighting pass for the active scene
---@param enabled boolean
function RenderCoreSystem:setDeferredLightingEnabled(enabled)
    self.settings.deferredLighting = enabled == true
end

-- Deferred lighting pass: global environment + directional + point lights
-- accumulate in one pass (additive pipelines after the global term), then
-- composite with albedo
function RenderCoreSystem:deferredLighting()
    local buffer0 = self.buffers[Enums.BufferName.buffer0]   -- albedo
    local buffer1 = self.buffers[Enums.BufferName.buffer1]   -- normals/material
    local buffer2 = self.buffers[Enums.BufferName.buffer2]   -- lighting accumulation
    local zBufferL = self.buffers[Enums.BufferName.zBufferL] -- linear depth

    local eye = CameraManager:getEye()
    local normalMat = buffer1:mipView(0)
    local depth = zBufferL:mipView(0)

    -- 1. Global lighting (environment from irMap/envMap)
    local pass = Renderer:beginPass(self:colorPassDesc('Lighting', buffer2, 0, true))
    local globalShader = Cache.Shader('fullscreen_ray', 'light/global')
    pass:setPipeline(Pipelines.get(globalShader, FullscreenState))
    pass:setInputs(normalMat, Samplers.LinearClamp, depth, Samplers.LinearClamp) -- texNormalMat, texDepth
    pass:drawFullscreen()

    -- 2. Directional lights (star - no distance falloff, like the sun)
    local directional = self.directionalLights
    if directional and #directional > 0 then
        local dirShader = Cache.Shader('fullscreen_ray', 'light/directional')
        pass:setPipeline(Pipelines.get(dirShader, FullscreenAdditive))
        local DirParams = dirShader:blockType('Params')
        for _, light in ipairs(directional) do
            local p = pass:alloc(DirParams)
            p.lightDir.x, p.lightDir.y, p.lightDir.z = light.dir.x, light.dir.y, light.dir.z
            p.lightColor.x, p.lightColor.y, p.lightColor.z = light.color.x, light.color.y, light.color.z
            pass:drawFullscreen()
        end
    end

    -- 3. Point lights (stations, engines, weapon effects, etc.)
    local pointLights = LightManager:getPointLights()
    if #pointLights > 0 then
        local pointShader = Cache.Shader('fullscreen_ray', 'light/point')
        pass:setPipeline(Pipelines.get(pointShader, FullscreenAdditive))
        local PointLight = pointShader:blockType('PointLight')
        for _, light in ipairs(pointLights) do
            local renderPos = light.pos:relativeTo(eye)
            local p = pass:alloc(PointLight)
            p.positionRadius.x, p.positionRadius.y, p.positionRadius.z = renderPos.x, renderPos.y, renderPos.z
            p.positionRadius.w = light.radius or 0.0
            p.colorIntensity.x, p.colorIntensity.y, p.colorIntensity.z = light.color.x, light.color.y, light.color.z
            p.colorIntensity.w = light.intensity or 1.0
            pass:drawFullscreen()
        end
    end
    pass:finish()

    -- 4. Composite: albedo * lighting -> buffer1 (reuse as temp)
    pass = Renderer:beginPass(self:colorPassDesc('Lighting.composite', buffer1, 0, false))
    local compShader = Cache.Shader('fullscreen_ray', 'light/composite')
    pass:setPipeline(Pipelines.get(compShader, FullscreenState))
    pass:setInputs(
        buffer0:mipView(0), Samplers.LinearClamp, -- texAlbedo
        depth, Samplers.LinearClamp,              -- texDepth
        buffer2:mipView(0), Samplers.LinearClamp) -- texLighting
    pass:drawFullscreen()
    pass:finish()

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
        local shader = Cache.Shader('fullscreen_ndc', 'filter/bloompre')
        local pass = Renderer:beginPass(self:colorPassDesc('Post.bloompre', A, 0, false))
        pass:setPipeline(Pipelines.get(shader, FullscreenState))
        pass:setInputs(self.buffers[Enums.BufferName.buffer0]:mipView(self.level or 0), Samplers.LinearClamp)
        pass:drawFullscreen()
        pass:finish()
    end

    for i = 1, 3 do
        self:blur(B, A, 1, 0, radius, width)
        self:blur(A, B, 0, 1, radius, width)

        self:applyFilter('bloomcomposite', nil, A:mipView(0))
    end
end

function RenderCoreSystem:blur(dst, src, dx, dy, radius, variance)
    local shader = Cache.Shader('fullscreen_ndc', 'filter/blur')
    local size = src:getSize()
    local pass = Renderer:beginPass(self:colorPassDesc('Post.blur', dst, 0, false))
    pass:setPipeline(Pipelines.get(shader, FullscreenState))
    pass:setInputs(src:mipView(0), Samplers.LinearClamp)
    local T = paramsTypes['blur']
    if not T then
        T = shader:blockType('Params')
        paramsTypes['blur'] = T
    end
    local p = pass:alloc(T)
    p.variance = variance
    p.dir.x, p.dir.y = dx, dy
    p.size.x, p.size.y = size.x, size.y
    p.radius = radius
    pass:drawFullscreen()
    pass:finish()
end

function RenderCoreSystem:fxaa()
    if not self.postSettings.fxaa.enable then return end

    local settings = self.postSettings.fxaa

    self:applyFilter('fxaa', function(p)
        p.fxaaQualitySubpix = settings.strength
        p.fxaaQualityEdgeThreshold = settings.edgeThreshold or 0.125
        p.fxaaQualityEdgeThresholdMin = settings.edgeThresholdMin or 0.0312
        p.size.x, p.size.y = self.resX, self.resY
    end)
end

function RenderCoreSystem:sharpen()
    if not self.postSettings.sharpen.enable then return end

    local settings = self.postSettings.sharpen

    -- Single-pass CAS
    self:applyFilter('sharpen_cas', function(p)
        p.casSharpness = settings.strength
        p.size.x, p.size.y = self.resX, self.resY -- pixel size for offsets
    end)
end

function RenderCoreSystem:radialBlur()
    if not self.postSettings.radialblur.enable or self.postSettings.radialblur.strength <= 0 then return end

    local rb = self.postSettings.radialblur

    -- Slot 1 is the linear depth the filter weighs its taps with
    self:applyFilter('radialblur', function(p)
        p.strength = rb.strength
        p.center.x, p.center.y = rb.center[1], rb.center[2]
    end, self.buffers[Enums.BufferName.zBufferL]:mipView(0))
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
        self:applyFilter('tonemap_limittheory', function(p)
            p.exposure = exposure
            p.size.x, p.size.y = self.resX, self.resY
        end)
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

    self:applyFilter('tonemap', function(p)
        p.mode = modeId
        p.exposure = exposure
        p.size.x, p.size.y = self.resX, self.resY
    end)
end

function RenderCoreSystem:vignette()
    if not self.postSettings.vignette.enable then return end
    self:applyFilter('vignette', function(p)
        p.strength = self.postSettings.vignette.strength
        p.hardness = self.postSettings.vignette.hardness
    end)
end

function RenderCoreSystem:aberration()
    if not self.postSettings.aberration.enable then return end
    self:applyFilter('aberration', function(p)
        p.strength = self.postSettings.aberration.strength
    end)
end

function RenderCoreSystem:dither()
    if not self.postSettings.dither.enable then return end

    self:applyFilter('dither', function(p)
        p.strength = self.postSettings.dither.strength
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

    self:applyFilter('colorgrade', function(p)
        p.mode = modeId
        p.preExposure = settings.preExposure
        p.temperature = settings.temperature
        p.tint = settings.tint
        p.saturation = settings.saturation
        p.contrast = settings.contrast
        p.brightness = settings.brightness
        p.vibrance = settings.vibrance
        p.lift.x, p.lift.y, p.lift.z = settings.lift[1], settings.lift[2], settings.lift[3]
        p.gamma.x, p.gamma.y, p.gamma.z = settings.gamma[1], settings.gamma[2], settings.gamma[3]
        p.gain.x, p.gain.y, p.gain.z = settings.gain[1], settings.gain[2], settings.gain[3]
    end)
end

--- Draw the final image over the whole window. The `fullscreen_ndc` quad has
--- uv.y = 0 at the bottom row, which is how the window is y-up in GL: no flip
--- is needed (the old y-down flipped rectangle did exactly this).
---@param pass RenderPass the open backbuffer pass
function RenderCoreSystem:present(pass)
    local sh = Cache.Shader('fullscreen_ndc', 'filter/identity')
    pass:setPipeline(Pipelines.get(sh, FullscreenState))
    pass:setInputs(self.buffers[Enums.BufferName.buffer0]:mipView(self.level or 0), Samplers.LinearClamp)
    pass:drawFullscreen()
end

--- Debug view: the four main buffers in one quadrant each (buffer0 top left,
--- buffer1 top right, buffer2 bottom left, linear depth bottom right).
---@param pass RenderPass the open backbuffer pass
---@param sx integer window width
---@param sy integer window height
function RenderCoreSystem:presentAll(pass, sx, sy)
    local sh = Cache.Shader('fullscreen_ndc', 'filter/identity')
    pass:setPipeline(Pipelines.get(sh, FullscreenState))
    local hx, hy = math.floor(sx / 2), math.floor(sy / 2)
    local level = self.level or 0
    local function draw(bufKey, bufLevel, x, y)
        pass:setViewport(x, y, hx, hy)
        pass:setInputs(self.buffers[bufKey]:mipView(bufLevel), Samplers.LinearClamp)
        pass:drawFullscreen()
    end
    draw(Enums.BufferName.buffer0, level, 0, hy)
    draw(Enums.BufferName.buffer1, level, hx, hy)
    draw(Enums.BufferName.buffer2, level, 0, 0)
    draw(Enums.BufferName.zBufferL, 0, hx, 0)
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
