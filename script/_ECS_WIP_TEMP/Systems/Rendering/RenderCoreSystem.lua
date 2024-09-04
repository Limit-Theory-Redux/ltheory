-- Storage
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage") --!temp path
local MeshStorage = require("_ECS_WIP_TEMP.Systems.Storage.MeshStorage") --!temp path
-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler") --!temp path
-- Rendering
local RenderingPass = require("_ECS_WIP_TEMP.Systems.Rendering.RenderingPass") --!temp path
local CameraSystem = require("_ECS_WIP_TEMP.Systems.Rendering.CameraSystem") --!temp path

---@class Buffer : Tex2D

---@class RenderSettings
---@field superSampleRate integer
---@field downSampleRate integer
---@field showBuffers boolean
---@field cullFace boolean

---@class RenderCoreSystem
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

--[[
    Buffers are just Tex2D's 
    Should CreateBuffer be moved over to a different file?
]]
---@param x number
---@param y number
---@param format TexFormat
---@return Buffer
local function createBuffer(x, y, format)
    local buffer = Tex2D.Create(x, y, format)
    buffer:setMagFilter(TexFilter.Linear)
    buffer:setMinFilter(TexFilter.Linear)
    buffer:setWrapMode(TexWrapMode.Clamp)
    buffer:push()
    Draw.Clear(0, 0, 0, 0)
    buffer:pop()
    buffer:genMipmap()
    return buffer
end

---@class RenderCoreSystem
---@overload fun(self: RenderCoreSystem) class internal
---@overload fun() class external
local RenderCoreSystem = Class(function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
    ---@diagnostic disable-next-line: invisible
    self:registerRenderingPasses()
end)

---@private
function RenderCoreSystem:registerVars()
    self.profiler = QuickProfiler("RenderCoreSystem", false, false)

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

    -- Set Buffers
    self.buffers = {}
    self:initializeBuffers()

    -- Set Variables
    self.passes = {}
    self.level = 0
end

---@private
function RenderCoreSystem:initializeBuffers()
    -- If a buffer is currently set then reset all buffers
    if self.buffers[Enums.BufferName.buffer0] then self:resetBuffers() end
    self.buffers = {
        [Enums.BufferName.buffer0] = createBuffer(self.ssResX, self.ssResY, colorFormat),
        [Enums.BufferName.buffer1] = createBuffer(self.ssResX, self.ssResY, colorFormat),
        [Enums.BufferName.buffer2] = createBuffer(self.ssResX, self.ssResY, colorFormat),
        [Enums.BufferName.zBuffer] = createBuffer(self.ssResX, self.ssResY, depthFormat),
        [Enums.BufferName.zBufferL] = createBuffer(self.ssResX, self.ssResY, depthFormatL),
        [Enums.BufferName.dsBuffer0] = createBuffer(self.dsResX, self.dsResY, colorFormat),
        [Enums.BufferName.dsBuffer1 ]= createBuffer(self.dsResX, self.dsResY, colorFormat)
    }
end

---@private
function RenderCoreSystem:resetBuffers()
    self.buffers[Enums.BufferName.buffer0]:free()
    self.buffers[Enums.BufferName.buffer1]:free()
    self.buffers[Enums.BufferName.buffer2]:free()
    self.buffers[Enums.BufferName.zBuffer]:free()
    self.buffers[Enums.BufferName.zBufferL]:free()
    self.buffers[Enums.BufferName.dsBuffer0]:free()
    self.buffers[Enums.BufferName.dsBuffer1]:free()
end

---@private
function RenderCoreSystem:registerRenderingPasses()
    do -- < Opaque Pass Definition > --
        local stateSettings = {
            blendMode = BlendMode.Disabled,
            cullFace = self.settings.cullFace and CullFace.Back or CullFace.None,
            depthTest = true,
            depthWritable = true
        }
        local bufferOrder = {}
        insert(bufferOrder, Enums.BufferName.buffer0)
        insert(bufferOrder, Enums.BufferName.buffer1)
        insert(bufferOrder, Enums.BufferName.zBufferL)
        insert(bufferOrder, Enums.BufferName.zBuffer)
        local drawFunc = function() 
            Draw.Clear(0, 0, 0, 0)
            Draw.ClearDepth(1)
            Draw.Color(1, 1, 1, 1)
        end
        self.passes[Enums.RenderingPasses.Opaque] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    end
    do -- < Additive Pass Definition > --
        local stateSettings = {
            blendMode = BlendMode.Additive,
            cullFace = Cullface.None,
            depthTest = true,
            depthWritable = false
        }
        local bufferOrder = {} -- Reset BufferOrder
        insert(bufferOrder, Enums.BufferName.buffer0)
        insert(bufferOrder, Enums.BufferName.zBuffer)
        local drawFunc = nil
        self.passes[Enums.RenderingPasses.Additive] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    end
    do -- < Alpha Pass Definition > --
        local stateSettings = {
            blendMode = BlendMode.Alpha,
            cullFace = Cullface.None,
            depthTest = true,
            depthWritable = false
        }
        local bufferOrder = {}
        insert(bufferOrder, Enums.BufferName.buffer0)
        insert(bufferOrder, Enums.BufferName.zBuffer)
        local drawFunc = nil
        self.passes[Enums.RenderingPasses.Alpha] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    end
    do -- < UI Pass Definition > --
        local stateSettings = {
            blendMode = BlendMode.Alpha,
            cullFace = Cullface.None,
            depthTest = false,
            depthWritable = false
        }
        local bufferOrder = {}
        insert(bufferOrder, Enums.BufferName.buffer1)
        insert(bufferOrder, Enums.BufferName.zBuffer)
        local drawFunc = function() Draw.Clear(0, 0, 0, 0) end
        self.passes[Enums.RenderingPasses.Alpha] = RenderingPass(bufferOrder, stateSettings, drawFunc)
    end
end

---@private
function RenderCoreSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
    EventBus:subscribe(Event.Render, self, self.onRender)
    EventBus:subscribe(Event.PostRender, self, self.onPostRender)
end

function RenderCoreSystem:onPreRender(data)
    -- TODO: Use 'data' or a global 'RenderState' to set proper superSampleRate, resX, resY
    -- Can this be in PreRender or must SuperSampleRate, Resolution, and all that be set in Render?
    --[[
    local ssr = data.ssr or self.settings.superSampleRate
    local ssResX, ssResY = ssr * data.resX, ssr * data.resY
    if self.ssResX ~= ssResX or self.ssResY ~= ssResY or self.ssr ~= ssr then
        self.ssResX = ssResX
        self.ssResY = ssResY
        self.settings.superSampleRate = ssr
        self.resX = data.resX
        self.resY = data.resY

        -- Buffers must be reinitialized when SuperSampleRate/DownSampleRate is changed.
        self:initializeBuffers()
    ]]--
    -- < Prepare Buffers for Primary Render Pass > --
    self.buffers[Enums.BufferName.buffer0]:setMipRange(0, 0)
    self.buffers[Enums.BufferName.buffer1]:setMipRange(0, 0)
    self.buffers[Enums.BufferName.buffer2]:setMipRange(0, 0)
    self.buffers[Enums.BufferName.buffer0]:setMinFilter(TexFilter.Linear)
    self.buffers[Enums.BufferName.buffer1]:setMinFilter(TexFilter.Linear)
    self.buffers[Enums.BufferName.buffer2]:setMinFilter(TexFilter.Linear)
    self.level = 0

    -- < Cache Entities (EntityInfo's?) for Rendering Passes > --
    -- Necessary because previously we obtained Entites from 'StarSystem'
    --[[ Need Method to pull by BlendMode in RenderPass and change cache based on Culling
        OpaqueCache = AllEntitiesWithComponent(RenderComponent):filter(BlendMode.Opaque)
        AdditiveCache = AllEntitiesWithComponent(RenderComponent):filter(BlendMode.Additive)
        AlphaCache = AllEntitiesWithComponent(RenderComponent):filter(BlendMode.Alpha)
        LightCache = AllEntitiesWithComponent(LightComponent)
    --]]
end

function RenderCoreSystem:onRender(data)
    -- Reset RenderState and ClipRect at Start of Render
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()
    
    do -- < Camera > --
        -- Add Camera Stack?
        -- Should we remove updateViewMatrix/updateProjectionMatrix and use a RefreshMatrices to do this?
        CameraSystem:updateViewMatrix()
        CameraSystem:updateProjectionMatrix(self.resX,self.resY)
        CameraSystem:beginDraw() -- Push Camera ShaderVars
        --[[Original Order for Camera 
            In GameView:
                local x, y, sx, sy = self:getRectGlobal()
                self.camera:setViewport(x, y, sx, sy) -- just sets those vars
                self.camera:beginDraw()
                eye = self.camera.pos 
            In Camera:beginDraw()
                camera:push()
                camera:refreshMatrixes
                ShaderVar.PushMatrix('varname', self.matrix) - for mView, mProj, eye, ...
        ]]
    end

    do -- < Push Skybox/Nebula Global ShaderVars > --
        --[["StarSystem:beginRender()"
            self.nebula:forceLoad() -- Generates Nebula
            ShaderVar.PushFloat3('starDir', self.starDir.x, self.starDir.y, self.starDir.z)
            ShaderVar.PushTexCube('envMap', self.nebula.envMap)
            ShaderVar.PushTexCube('irMap', self.nebula.irMap)
        "]]
    end
    
    do -- < Opaque Pass > --
        self.passes[Enums.RenderingPasses.Opaque]:start(self.buffers, self.ssResX, self.ssResY)
        self:renderInOrder(BlendMode.Alpha)
        self.passes[Enums.RenderingPasses.Opaque]:stop()
    end
    
    do -- < Lighting Pass > --
        -- TODO: Needs a different solution than other passes, as it's structured differently currently.
            -- < Global Lighting > --
            -- Use Cached World Light Material ?
            -- < Local Lighting > --
            -- Use Cached Light Material
            -- < Aldebo & accumulated light buffer > --
            -- Use Cached Light Material 
    end

    do -- < Alpha (Additive Pass) > --
        self.passes[Enums.RenderingPasses.Additive]:start(self.buffers, self.ssResX, self.ssResY)
        self:renderInOrder(BlendMode.Alpha)
        self.passes[Enums.RenderingPasses.Additive]:stop()
    end

    do -- < Alpha Pass > --
        self.passes[Enums.RenderingPasses.Alpha]:start(self.buffers, self.ssResX, self.ssResY)
        self:renderInOrder(BlendMode.Alpha)
        self.passes[Enums.RenderingPasses.Alpha]:stop()
    end

    do -- < Pop Skybox and Camera Global ShaderVars > --
        --[[ "StarSystem:endRender()"
            ShaderVar.Pop('starDir')
            ShaderVar.Pop('envMap')
            ShaderVar.Pop('irMap')
        ]]--
        CameraSystem:endDraw() -- Pop Camera ShaderVars
    end

    do -- < UI Pass > --
        self.passes[Enums.RenderingPasses.UI]:start(self.buffers, self.ssResX, self.ssResY)
        -- TODO: Do UI Rendering Here or Pass off to a UIRenderer?
        self.passes[Enums.RenderingPasses.UI]:stop()
        -- TODO: Do UI Equivalent of RenderPipeline:present() also why self.buffer1:pop()
        -- Reference: RenderPipeline:stopUI()
    end

    do -- < PostFX and Draw Rendered Frame to Screen > --
        if self.settings.showBuffers then
            -- self:presentAll(x,y,sx,sy)
            -- < PresentAll Buffers / RenderPipeline.presentAll(...) > --
        else
            -- < PostFX Pass > --
            -- self:present(x,y,sx,sy,false)
            -- < Present Frame / Present buffer0 / RenderPipeline.present(...) > -
        end
    end

    RenderState.PopAll()
    ClipRect.Pop()
    --self.camera:pop() -- TODO: Implement Camera Stack
end

function RenderCoreSystem:onPostRender(data)
    --[[
        
    ]]--
end

function RenderCoreSystem:renderInOrder(blendMode)
    --[[ Original Order
        self:send(OldEvent.Broadcast(blendMode))
        self:renderProjectiles(blendMode)
        self.dust:render(blendMode)
        self.nebula:render(blendMode)
    ]]
    -- < Render General Materials > --
    if blendMode == BlendMode.Disabled then
        -- < Render All 'RenderComponent' Opaque Pass > --
        -- < Render Nebula 'Skybox' Opaque Pass > --
    elseif blendMode == BlendMode.Additive then
        -- < Render All 'RenderComponent' Additive Pass > --
        -- < Render All Projectiles > --
        -- < Render Dust Additive Pass > --
        -- < Render Nebula 'StarBg' Additive Pass > --
    elseif blendMode == BlendMode.Alpha then
        -- < Render All 'RenderComponent' Alpha Pass> --
        -- < Render Dust Alpha Pass > --
    elseif blendMode == BlendMode.PreMultAlpha then
        -- Only used currently by 'Planet.lua' during Alpha Pass
    end
end

---Update the screen with any rendering performed since the previous call
---@param x integer x position of screen?
---@param y integer y position of screen?
---@param sx integer SuperSampleRate * Resolution?
---@param sy integer SuperSampleRate * Resolution?
---@param useMips boolean Use MipMap Rendering
function RenderCoreSystem:present(x, y, sx, sy, useMips)
    --[[
        Directly from RenderPipeline.
        Do we ever use MipMap for Rendering?
        Need Clarity on where x/y/sx/sy should be coming from. Window? SuperSampled Resolution?
    ]]--
    RenderState.PushAllDefaults()

    local shader = Cache.Shader('ui', 'filter/identity')
    shader:start()

    shader:setTex2D("src", self.buffer0)
    if false and useMips then
        self.buffer0:genMipmap()
        self.buffer0:setMinFilter(TexFilter.LinearMipLinear)
        Draw.Rect(x, y + sy, sx, -sy)
        self.buffer0:setMinFilter(TexFilter.Linear)
    else
        Draw.Rect(x, y + sy, sx, -sy)
    end

    shader:stop()
    RenderState.PopAll()
end

---Update the screen with deferred rendering buffers, 1 in each quadrant of the screen.
---@param x integer x position of screen?
---@param y integer y position of screen?
---@param sx integer SuperSampleRate * Resolution?
---@param sy integer SuperSampleRate * Resolution?
function RenderCoreSystem:presentAll(x, y, sx, sy)
    RenderState.PushAllDefaults()

    local shader = Cache.Shader('ui', 'filter/identity')
    shader:start()

    shader:setTex2D("src", self.buffers[Enums.BufferName.buffer0])
    Draw.Rect(x, y + sy / 2, sx / 2, -sy / 2)

    shader:resetTexIndex()
    shader:setTex2D("src", self.buffers[Enums.BufferName.buffer1])
    Draw.Rect(x + sx / 2, y + sy / 2, sx / 2, -sy / 2)

    shader:resetTexIndex()
    shader:setTex2D("src", self.buffers[Enums.BufferName.buffer2])
    Draw.Rect(x, y + sy, sx / 2, -sy / 2)

    shader:resetTexIndex()
    shader:setTex2D("src", self.buffers[Enums.BufferName.zBufferL])
    Draw.Rect(x + sx / 2, y + sy, sx / 2, -sy / 2)

    shader:stop()
    RenderState.PopAll()
end

return RenderCoreSystem()
