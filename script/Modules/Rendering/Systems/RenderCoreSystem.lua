local Registry         = require("Core.ECS.Registry")
local QuickProfiler    = require("Shared.Tools.QuickProfiler")
local RenderingPass    = require("Shared.Rendering.RenderingPass")
local CameraSystem     = require("Modules.Rendering.Systems.CameraSystem")
local RenderComp       = require("Modules.Rendering.Components").Render
local UniformFuncs     = require("Shared.Rendering.UniformFuncs")

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

    local win            = Window:size()
    self.resX, self.resY = win.x, win.y
    self.ssResX          = self.resX * self.settings.superSampleRate
    self.ssResY          = self.resY * self.settings.superSampleRate
    self.dsResX          = self.resX / self.settings.downSampleRate
    self.dsResY          = self.resY / self.settings.downSampleRate

    self.materialCache   = {} -- material → { var = {type, values} }   -- static + const + entity-independent auto
    self.instanceCache   = {} -- entity  → { mat  = { var = {type, values} } } -- per-entity auto vars
    self.processedMats   = {}

    self.buffers         = {}
    self:initializeBuffers()
    self.passes = {}
    self.level = 0

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
        [Enums.BufferName.buffer0] = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.buffer1] = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.buffer2] = create(self.ssResX, self.ssResY, TexFormat.RGBA16F),
        [Enums.BufferName.zBuffer] = create(self.ssResX, self.ssResY, TexFormat.Depth32F),
        [Enums.BufferName.zBufferL] = create(self.ssResX, self.ssResY, TexFormat.R32F),
        [Enums.BufferName.dsBuffer0] = create(self.dsResX, self.dsResY, TexFormat.RGBA16F),
        [Enums.BufferName.dsBuffer1] = create(self.dsResX, self.dsResY, TexFormat.RGBA16F),
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

function RenderCoreSystem:render(data)
    self:handleResize()

    Window:beginDraw()
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()

    CameraSystem:updateViewMatrix()
    CameraSystem:updateProjectionMatrix(self.resX, self.resY)
    CameraSystem:beginDraw()

    -- Data cache
    self:cacheData()

    -- Opaque Pass
    self.currentPass = Enums.RenderingPasses.Opaque
    self.passes[self.currentPass]:start(self.buffers, self.ssResX, self.ssResY)
    self:renderInOrder(BlendMode.Disabled)
    self.passes[self.currentPass]:stop()

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

    CameraSystem:endDraw()

    if self.settings.showBuffers then
        self:presentAll(0, 0, self.resX, self.resY)
    else
        self:present(0, 0, self.resX, self.resY, false)
    end

    RenderState.PopAll()
    ClipRect.Pop()
    Window:endDraw()

    self.currentPass = nil
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

    local b = self.buffers
    for _, buf in pairs({ b.buffer0, b.buffer1, b.buffer2 }) do
        buf:setMipRange(0, 0); buf:setMinFilter(TexFilter.Linear)
    end
    self.level = 0
end

function RenderCoreSystem:renderInOrder(blendMode)
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

function RenderCoreSystem:cacheData()
    self.materialCache = {}
    self.instanceCache = {}
    local processedMats = {}

    local eye = CameraSystem:getCurrentCameraEye()

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
                    ---@cast v AutoShaderVar
                    matCache[v] = { type = v.uniformType, values = { v:getValue(eye, entity) } }
                end
                for _, v in ipairs(mat.constShaderVars or {}) do
                    ---@cast v AutoShaderVar
                    matCache[v] = { type = v.uniformType, values = { v:getValue(eye, entity) } }
                end

                for _, v in ipairs(mat.autoShaderVars or {}) do
                    ---@cast v AutoShaderVar
                    if not v.perInstance then
                        matCache[v] = { type = v.uniformType, values = { v:getValue(eye, entity) } }
                    end
                end

                self.materialCache[mat] = matCache
            end

            -- instance cache (per instance)
            local instCache = self.instanceCache[entity.id] or {}
            local matInstCache = instCache[mat] or {}

            for _, v in ipairs(mat.autoShaderVars or {}) do
                if v.perInstance then
                    matInstCache[v] = { type = v.uniformType, values = { v:getValue(eye, entity) } }
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
    local function draw(buf, px, py)
        sh:setTex2D("src", buf); Draw.Rect(px, py, sx / 2, -sy / 2)
    end
    draw(self.buffers[Enums.BufferName.buffer0], x, y + sy / 2)
    draw(self.buffers[Enums.BufferName.buffer1], x + sx / 2, y + sy / 2)
    draw(self.buffers[Enums.BufferName.buffer2], x, y)
    draw(self.buffers[Enums.BufferName.zBufferL], x + sx / 2, y)
    sh:stop()
    RenderState.PopAll()
end

return RenderCoreSystem()
