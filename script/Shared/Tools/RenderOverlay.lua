--[[
    RenderOverlay - NVIDIA-style performance overlay

    Advanced debug overlay with frame time graph and detailed stats.
    Toggle with Shift+O or via RenderOverlay:toggle()

    Usage:
        local RenderOverlay = require('Shared.Tools.RenderOverlay')

        -- In onInput:
        RenderOverlay:handleInput()

        -- In your render loop (after all other rendering):
        RenderOverlay:draw()
]]

local DrawEx = require('UI.DrawEx')

-- Lazy load to avoid circular dependency (RenderCoreSystem requires RenderOverlay)
local RenderCoreSystem = nil
local function getRenderCoreSystem()
    if not RenderCoreSystem then
        RenderCoreSystem = require('Modules.Rendering.Systems.RenderCoreSystem')
    end
    return RenderCoreSystem
end

---@class RenderOverlay
local RenderOverlay = {}

-- Configuration
RenderOverlay.visible = false
RenderOverlay.position = "top-right"
RenderOverlay.width = 280
RenderOverlay.graphHeight = 60
RenderOverlay.fontSize = 13
RenderOverlay.font = 'Unageo-Bold'
RenderOverlay.smallFont = 'Unageo-Medium'
RenderOverlay.lineHeight = 18
RenderOverlay.padding = 10
RenderOverlay.margin = 12

-- Colors (NVIDIA-inspired)
local colors = {
    bg = Color(0.05, 0.05, 0.05, 0.85),
    bgGraph = Color(0.1, 0.1, 0.1, 0.9),
    border = Color(0.3, 0.3, 0.3, 0.8),
    header = Color(0.4, 0.9, 0.2, 1.0),       -- Bright green
    label = Color(0.7, 0.7, 0.7, 0.9),        -- Gray
    value = Color(1.0, 1.0, 1.0, 1.0),        -- White
    fps = Color(0.4, 0.9, 0.2, 1.0),          -- Green
    fpsWarn = Color(1.0, 0.8, 0.2, 1.0),      -- Yellow
    fpsBad = Color(1.0, 0.3, 0.2, 1.0),       -- Red
    graphLine = Color(0.4, 0.9, 0.2, 0.9),    -- Green
    graphFill = Color(0.2, 0.5, 0.1, 0.4),    -- Dark green
    graphTarget = Color(0.5, 0.5, 0.5, 0.5),  -- Gray line for 16.67ms
    rtActive = Color(0.2, 0.8, 1.0, 1.0),     -- Cyan for RT
    rtInactive = Color(0.6, 0.6, 0.6, 1.0),   -- Gray for GL
}

-- Frame time history for graph
local frameHistory = {}
local maxHistorySize = 100
local maxFrameTime = 33.33  -- Cap at ~30fps for graph scale

-- Smoothed values
local smoothFPS = 0
local smoothFrameTime = 0
local smoothRTTime = 0
local smoothMainWork = 0
local smoothWait = 0
local smoothFactor = 0.15

function RenderOverlay:toggle()
    self.visible = not self.visible
end

function RenderOverlay:setVisible(visible)
    self.visible = visible
end

function RenderOverlay:isVisible()
    return self.visible
end

-- Get FPS color based on performance
local function getFPSColor(fps)
    if fps >= 55 then return colors.fps
    elseif fps >= 30 then return colors.fpsWarn
    else return colors.fpsBad end
end

-- Draw a filled rectangle
local function drawRect(x, y, w, h, color)
    DrawEx.Rect(x, y, w, h, color)
end

-- Draw text with specific styling
local function drawText(font, text, size, x, y, color, alignX)
    alignX = alignX or 0
    DrawEx.TextAdditive(font, text, size, x, y, 200, 16, color.r, color.g, color.b, color.a, alignX, 0.5)
end

-- Draw the frame time graph
local function drawGraph(x, y, w, h)
    -- Background
    drawRect(x, y, w, h, colors.bgGraph)

    -- Target line at 16.67ms (60fps)
    local targetY = y + h - (16.67 / maxFrameTime) * h
    if targetY > y and targetY < y + h then
        for i = 0, w - 4, 8 do
            drawRect(x + 2 + i, targetY, 4, 1, colors.graphTarget)
        end
    end

    -- Draw frame time bars
    local barWidth = w / maxHistorySize
    for i, ft in ipairs(frameHistory) do
        local barHeight = math.min(ft / maxFrameTime, 1.0) * (h - 4)
        local barX = x + 2 + (i - 1) * barWidth
        local barY = y + h - 2 - barHeight

        -- Color based on frame time
        local barColor
        if ft <= 16.67 then
            barColor = colors.graphLine
        elseif ft <= 33.33 then
            barColor = colors.fpsWarn
        else
            barColor = colors.fpsBad
        end

        drawRect(barX, barY, math.max(barWidth - 1, 1), barHeight, barColor)
    end

    -- Border
    DrawEx.RectOutline(x, y, w, h, colors.border)
end

-- Draw a stat row with label and value
local function drawStatRow(x, y, label, value, labelColor, valueColor, w)
    labelColor = labelColor or colors.label
    valueColor = valueColor or colors.value
    w = w or 110
    drawText(RenderOverlay.smallFont, label, 11, x, y, labelColor, 0)
    drawText(RenderOverlay.font, value, 11, x + w, y, valueColor, 0)
end

function RenderOverlay:draw()
    if not self.visible then return end

    -- Set up render state for immediate UI
    Window:beginDraw()
    RenderState.PushAllDefaults()
    ClipRect.PushDisabled()

    local win = Window:size()
    local resX, resY = win.x, win.y
    local rtActive = Engine:isRenderThreadActive()

    -- Collect stats
    local rcs = getRenderCoreSystem()
    local fps = rcs:getSmoothFPS()
    local frameTime = rcs:getSmoothFrameTime(true)

    -- Update history
    table.insert(frameHistory, frameTime)
    if #frameHistory > maxHistorySize then
        table.remove(frameHistory, 1)
    end

    -- Smooth values
    smoothFPS = smoothFPS + (fps - smoothFPS) * smoothFactor
    smoothFrameTime = smoothFrameTime + (frameTime - smoothFrameTime) * smoothFactor

    -- Calculate panel dimensions
    local panelWidth = self.width
    local panelHeight = self.graphHeight + 20 -- Start with graph + padding

    -- Count content lines
    local numLines = 5 -- Base stats (FPS, frame, min/max, mem, cores)
    if rtActive then
        numLines = numLines + 14 -- RT stats (timing + render stats sections)
    else
        numLines = numLines + 2 -- Direct GL mode text
    end
    panelHeight = panelHeight + numLines * self.lineHeight + self.padding * 2

    -- Position (top-right)
    local panelX = resX - panelWidth - self.margin
    local panelY = self.margin

    -- Draw main background
    drawRect(panelX, panelY, panelWidth, panelHeight, colors.bg)
    DrawEx.RectOutline(panelX, panelY, panelWidth, panelHeight, colors.border)

    local contentX = panelX + self.padding
    local contentY = panelY + self.padding
    local contentWidth = panelWidth - self.padding * 2

    -- Header with FPS
    local fpsColor = getFPSColor(smoothFPS)
    local modeText = rtActive and "[RT]" or "[GL]"
    local modeColor = rtActive and colors.rtActive or colors.rtInactive

    drawText(self.font, string.format("%.0f FPS", smoothFPS), 14, contentX, contentY, fpsColor, 0)
    drawText(self.font, modeText, 12, contentX + contentWidth - 35, contentY, modeColor, 0)
    contentY = contentY + 20

    -- Frame time graph
    drawGraph(contentX, contentY, contentWidth, self.graphHeight)
    contentY = contentY + self.graphHeight + 8

    -- Frame time stats
    drawStatRow(contentX, contentY, "Frame", string.format("%.2f ms", smoothFrameTime), colors.label, colors.value)
    contentY = contentY + self.lineHeight

    -- Min/Max/Avg from history
    if #frameHistory > 0 then
        local minFT, maxFT, sumFT = frameHistory[1], frameHistory[1], 0
        for _, ft in ipairs(frameHistory) do
            minFT = math.min(minFT, ft)
            maxFT = math.max(maxFT, ft)
            sumFT = sumFT + ft
        end
        local avgFT = sumFT / #frameHistory

        drawStatRow(contentX, contentY, "Min/Max", string.format("%.1f / %.1f ms", minFT, maxFT), colors.label, colors.value)
        contentY = contentY + self.lineHeight
    end

    -- Timing breakdown
    if rtActive then
        -- Frame Timing Section
        contentY = contentY + 5
        drawText(self.font, "Frame Timing", 12, contentX, contentY, colors.header, 0)
        contentY = contentY + self.lineHeight + 2

        local rtTime = Engine:getRenderThreadFrameTimeMs()
        local waitTime = Engine:getMainThreadWaitTimeMs()
        local mainWork = math.max(0, smoothFrameTime - rtTime)
        local framesInFlight = Engine:getFramesInFlight()

        -- Smooth RT values
        smoothRTTime = smoothRTTime + (rtTime - smoothRTTime) * smoothFactor
        smoothMainWork = smoothMainWork + (mainWork - smoothMainWork) * smoothFactor
        smoothWait = smoothWait + (waitTime - smoothWait) * smoothFactor

        drawStatRow(contentX, contentY, "Total Frame", string.format("%.2f ms", smoothFrameTime))
        contentY = contentY + self.lineHeight

        drawStatRow(contentX, contentY, "Render Thread", string.format("%.2f ms", smoothRTTime))
        contentY = contentY + self.lineHeight

        drawStatRow(contentX, contentY, "Main Thread", string.format("%.2f ms", smoothMainWork))
        contentY = contentY + self.lineHeight

        drawStatRow(contentX, contentY, "Main Wait", string.format("%.2f ms", smoothWait))
        contentY = contentY + self.lineHeight

        drawStatRow(contentX, contentY, "Frames Queued", string.format("%d / 3", framesInFlight))
        contentY = contentY + self.lineHeight

        -- Render Stats Section
        contentY = contentY + 5
        drawText(self.font, "Render Stats", 12, contentX, contentY, colors.header, 0)
        contentY = contentY + self.lineHeight + 2

        local cmdsPerFrame = Engine:getRenderThreadCommandsPerFrame()
        local drawsPerFrame = Engine:getRenderThreadDrawCallsPerFrame()
        local totalCmds = Engine:getRenderThreadCommands()
        local totalDraws = Engine:getRenderThreadDrawCalls()
        local stateChanges = Engine:getRenderThreadStateChanges()

        drawStatRow(contentX, contentY, "Commands/Frame", string.format("%d", cmdsPerFrame))
        contentY = contentY + self.lineHeight

        drawStatRow(contentX, contentY, "Draw Calls/Frame", string.format("%d", drawsPerFrame))
        contentY = contentY + self.lineHeight

        drawStatRow(contentX, contentY, "State Changes", string.format("%d", stateChanges))
        contentY = contentY + self.lineHeight

        -- Workers
        local workers = Engine:getActiveWorkerCount()
        if workers > 0 then
            drawStatRow(contentX, contentY, "Active Workers", string.format("%d", workers))
            contentY = contentY + self.lineHeight
        end
    else
        contentY = contentY + 5
        drawText(self.font, "Direct GL Mode", 12, contentX, contentY, colors.rtInactive, 0)
        contentY = contentY + self.lineHeight
        drawText(self.smallFont, "Press R to enable Render Thread", 10, contentX, contentY, colors.label, 0)
        contentY = contentY + self.lineHeight
    end

    -- Memory
    local mem = GC.GetMemory()
    contentY = contentY + 3
    drawStatRow(contentX, contentY, "Lua Mem", string.format("%.1f KB", mem))
    contentY = contentY + self.lineHeight

    -- GPU info (if available)
    drawStatRow(contentX, contentY, "Cores", string.format("%d", Engine:getCpuCount()))

    -- Clean up render state
    ClipRect.Pop()
    RenderState.PopAll()
    Window:endDraw()
end

-- Handle Shift+O toggle
function RenderOverlay:handleInput()
    if Input:keyboard():isDown(Button.KeyboardShiftLeft) and
       Input:keyboard():isPressed(Button.KeyboardO) then
        self:toggle()
    end
end

return RenderOverlay
