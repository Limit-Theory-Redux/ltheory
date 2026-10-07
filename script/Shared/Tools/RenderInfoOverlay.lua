--[[
    RenderInfoOverlay - compact renderer statistics panel (top-right).

    Toggled with Ctrl+Shift+F3 (GeneralActions.RenderInfoOverlay); the env var
    LTHEORY_OVERLAY=1 forces it visible (also in capture mode, for checking it).

    Usage (Application does this):
        RenderInfoOverlay:tick(dt)   -- every frame, even when hidden (frame history)
        RenderInfoOverlay:draw()     -- inside Application:immediateUI

    Cost: the frame history is a ring of numbers; the text is rebuilt 4x per
    second and only while visible; GPU times come from the renderer's stats
    snapshot (already produced every frame) and a cached startup description.
]]

local HISTORY = 120
local NA = 1.8e19 -- Rust side reports u64::MAX (about 1.8446e19) for "backend can't provide it"

---@class RenderInfoOverlay
local RenderInfoOverlay = {
    visible = false,
    frames = {}, -- ring of frame times, ms
    head = 0,
    count = 0,
    last = nil,
    info = nil, -- parsed Renderer:backendInfo()
    lines = {}, -- cached text lines { text, colorKey }
    nextText = 0,
    fontSize = 12,
    gpuHist = {}, -- ring of GPU frame times (ms), see tick
    gpuHead = 0,
    gpuCount = 0,
    gpuFrame = -1,
}

local colors = {
    bg = Color(0.02, 0.02, 0.04, 0.72),
    border = Color(0.35, 0.55, 0.85, 0.9),
    header = Color(1.0, 0.85, 0.3, 1.0),
    section = Color(0.45, 0.8, 1.0, 1.0),
    text = Color(0.9, 0.9, 0.9, 1.0),
    good = Color(0.4, 0.95, 0.45, 1.0),
    warn = Color(1.0, 0.85, 0.3, 1.0),
    bad = Color(1.0, 0.4, 0.35, 1.0),
}

function RenderInfoOverlay:init()
    self.forced = os.getenv('LTHEORY_OVERLAY') == '1'
    self.visible = self.forced
    self.font = Font.Load('DejaVuSansMono', self.fontSize)
    self.lineH = math.floor(self.font:getLineHeight()) + 1
    self.charW = self.font:getSize2("M").x
    -- Font:draw takes the baseline: shift so rows sit inside their boxes
    self.textOff = math.floor(self.lineH * 0.7)
    self.last = TimeStamp.Now()
end

function RenderInfoOverlay:toggle()
    self.visible = not self.visible
    self.nextText = 0
end

--- Record the wall-clock time of the frame that just ended.
function RenderInfoOverlay:tick()
    if not self.last then return end
    local ms = self.last:getElapsed() * 1000
    self.last = TimeStamp.Now()
    -- startup frames (asset loading, shader compiles) would swamp the window
    self.warmup = (self.warmup or 0) + 1
    if self.warmup <= 10 then return end
    self.head = self.head % HISTORY + 1
    self.frames[self.head] = ms
    if self.count < HISTORY then self.count = self.count + 1 end

    -- GPU frame times (one per new measurement, only while the panel shows)
    if self.visible then
        local gf = tonumber(Renderer:statsGpuFrames())
        if gf ~= self.gpuFrame then
            self.gpuFrame = gf
            if gf > 0 then
                self.gpuHead = (self.gpuHead or 0) % HISTORY + 1
                self.gpuHist[self.gpuHead] = tonumber(Renderer:statsGpuTotalUs()) / 1000
                if self.gpuCount < HISTORY then self.gpuCount = self.gpuCount + 1 end
            end
        end
    end
end

local function parseInfo(s)
    local t = {}
    for k, v in s:gmatch("([%w_]+)=([^\n]*)") do t[k] = v end
    return t
end

local function num(v) return tonumber(v) end
local function isNA(v) return v >= NA end

local function fmtCount(v)
    v = tonumber(v)
    if isNA(v) then return "n/a" end
    return string.format("%d", v)
end

local function fmtBytes(v)
    v = tonumber(v)
    if isNA(v) then return "n/a" end
    if v >= 1048576 then return string.format("%.1f MB", v / 1048576) end
    if v >= 1024 then return string.format("%.1f KB", v / 1024) end
    return string.format("%d B", v)
end

function RenderInfoOverlay:buildLines()
    if not self.info then
        local s = ffi.string(Renderer:backendInfo())
        if s ~= "" then self.info = parseInfo(s) end
    end
    local info = self.info or {}
    local L = {}
    local function add(text, c) L[#L + 1] = { text, c or 'text' } end

    -- Frame statistics over the history window
    local sum, mx, n = 0, 0, self.count
    for i = 1, n do
        local f = self.frames[i]
        sum = sum + f
        if f > mx then mx = f end
    end
    local avg = n > 0 and sum / n or 0
    local fps = avg > 0 and 1000 / avg or 0
    local size = Window:size()
    local pm = Window:presentMode() == PresentMode.Vsync and "Vsync" or "NoVsync"

    add(string.format("Render info (Ctrl+Shift+F3)"), 'header')
    add("Backend", 'section')
    local isGL = (info.backend or ""):find("OpenGL") ~= nil
    if isGL then
        add(string.format("API       %s", info.backend))
        add(string.format("Renderer  %s", info.gl_renderer or "?"))
        add(string.format("Vendor    %s", info.gl_vendor or "?"))
        add(string.format("Version   %s", info.gl_version or "?"))
        add(string.format("Present   %s", pm))
    elseif info.backend then
        add(string.format("API       wgpu / %s", info.wgpu_backend or "?"))
        add(string.format("Adapter   %s", info.adapter or "?"))
        add(string.format("Device    %s", info.device_type or "?"))
        add(string.format("Driver    %s %s", info.driver or "", info.driver_info or ""))
        add(string.format("Surface   %s", info.surface_format or "?"))
        add(string.format("Present   %s (%s)", pm, pm == "Vsync" and "Fifo" or "Immediate"))
    else
        add("API       (starting...)")
    end
    local threaded = info.renderer_mode == "threaded"
    add(string.format("Mode      %s renderer, %dx%d", info.renderer_mode or "?", size.x, size.y))

    add("Frame", 'section')
    local fpsColor = fps >= 55 and 'good' or (fps >= 30 and 'warn' or 'bad')
    add(string.format("FPS %6.1f   avg %5.2f ms   max %5.2f ms", fps, avg, mx), fpsColor)
    self.graphLine = #L + 1
    add("") -- graph is drawn over these reserved lines
    add("")
    add("")

    -- Threads
    local frameUs = tonumber(Renderer:statsFrameTimeUs())
    local recvUs = tonumber(Renderer:statsRecvWaitUs())
    local presentUs = tonumber(Renderer:statsPresentWaitUs())
    local mainWaitUs = tonumber(Renderer:statsMainWaitUs())
    local execMs = math.max(0, frameUs - recvUs) / 1000
    local presentMs = presentUs / 1000
    local mainMs = math.max(0, avg - mainWaitUs / 1000)
    -- GPU (timestamp queries; a few frames old, smoothed)
    local gpuOn = Renderer:statsGpuAvailable() and tonumber(Renderer:statsGpuFrames()) > 0
    local gpuMs = gpuOn and tonumber(Renderer:statsGpuTotalSmoothUs()) / 1000 or nil

    -- Which of main / render thread / GPU limits the frame: the largest of
    -- the three, unless none of them fills the frame (vsync / frame cap).
    local function limiter(mainT, renderT, gpuT)
        local best, name = mainT, "main-bound"
        if renderT and renderT > best then best, name = renderT, "render-bound" end
        if gpuT and gpuT > best then best, name = gpuT, "GPU-bound" end
        if avg > 0 and best < 0.6 * avg then return "capped (vsync/idle)", 'good' end
        return name, 'warn'
    end

    add("Threads", 'section')
    if threaded then
        local idle = avg > 0 and math.max(0, 100 * (1 - (execMs + presentMs) / avg)) or 0
        local bound = idle >= 15 and "producer-bound (main/Lua)" or "consumer-bound (render)"
        add(string.format("Main       %6.2f ms  (waited %.2f)", mainMs, mainWaitUs / 1000))
        add(string.format("Render     %6.2f ms exec  %5.2f present", execMs, presentMs))
        add(string.format("Idle       %5.1f %%  %s", idle, bound), idle >= 15 and 'warn' or 'good')
        local name, c = limiter(mainMs, execMs, gpuMs)
        add(string.format("Limit      %s", name), c)
    else
        add(string.format("Main       %6.2f ms  (render inline)", avg))
        add(string.format("Present    %6.2f ms", presentMs))
        add("Idle       n/a")
        local name, c = limiter(mainMs, nil, gpuMs)
        add(string.format("Limit      %s", name), c)
    end

    add("GPU", 'section')
    if gpuOn then
        local sum, mxg = 0, 0
        for i = 1, self.gpuCount do
            local g = self.gpuHist[i]
            sum = sum + g
            if g > mxg then mxg = g end
        end
        local avgG = self.gpuCount > 0 and sum / self.gpuCount or gpuMs
        add(string.format("Total  avg %5.2f ms  max %5.2f ms  (busy %.2f)",
            avgG, mxg, tonumber(Renderer:statsGpuBusyUs()) / 1000), avgG <= 16.7 and "good" or (avgG <= 33 and "warn" or "bad"))
        local n = math.min(5, tonumber(Renderer:statsGpuPassCount()))
        for i = 0, n - 1 do
            add(string.format("%-28s %6.2f ms", ffi.string(Renderer:statsGpuPassLabel(i)):sub(1, 28),
                tonumber(Renderer:statsGpuPassSmoothUs(i)) / 1000))
        end
    else
        add(Renderer:statsGpuAvailable() and "Total  (measuring...)" or "n/a (no timestamp queries / LTHEORY_GPU_TIMING=0)")
    end

    -- Work
    local r = Renderer
    add("Work (per frame)", 'section')
    add(string.format("Draws %-7s Verts %-9s Imm %s",
        fmtCount(r:statsDrawCalls()), fmtCount(r:statsVertices()), fmtCount(r:statsImmVertices())))
    add(string.format("Cmds  %-7s Passes %-7s", fmtCount(r:statsCommands()), fmtCount(r:statsPasses())))
    add(string.format("Pipeline sw %-6s BindGroup sw %s",
        fmtCount(r:statsPipelineSwitches()), fmtCount(r:statsBindGroupSwitches())))
    add(string.format("Uniform ring %-10s Vertex ring %s",
        fmtBytes(r:statsUniformBytes()), fmtBytes(r:statsVertexBytes())))
    add("Resources", 'section')
    add(string.format("Pipelines %-6s Samplers %-6s BindGrp %s",
        fmtCount(r:statsPipelines()), fmtCount(r:statsSamplers()), fmtCount(r:statsBindGroups())))
    add(string.format("Textures  %-6s Meshes   %-6s TexMem  %s",
        fmtCount(r:statsTextures()), fmtCount(r:statsMeshes()), fmtBytes(r:statsTextureBytes())))
    add("Lua", 'section')
    add(string.format("Heap      %.1f MB", GC.GetMemory() / 1024))

    self.lines = L
end

local function frameColor(ms)
    if ms <= 17.5 then return colors.good end
    if ms <= 34 then return colors.warn end
    return colors.bad
end

--- Draw the panel. The backbuffer must already be open (Application:immediateUI).
function RenderInfoOverlay:draw()
    if not self.visible or not self.font then return end

    local now = os.clock()
    if now >= self.nextText then
        self.nextText = now + 0.25
        self:buildLines()
    end

    local pad = 8
    local maxChars = 60
    local w = maxChars * self.charW + pad * 2
    local h = #self.lines * self.lineH + pad * 2
    local size = Window:size()
    local x0 = size.x - w - 10
    local y0 = 10

    Imm.Rect(x0, y0, w, h, colors.bg)
    Imm.Border(1, x0, y0, w, h, colors.border)

    local y = y0 + pad
    for i, line in ipairs(self.lines) do
        local text = line[1]
        if #text > maxChars then text = text:sub(1, maxChars - 1) .. "~" end
        if text ~= "" then self.font:draw(text, x0 + pad, y + self.textOff, colors[line[2]]) end
        y = y + self.lineH
    end

    -- Frame-time graph over the three reserved lines
    if self.graphLine and self.count > 0 then
        local gx = x0 + pad
        local gy = y0 + pad + (self.graphLine - 1) * self.lineH
        local gw = w - pad * 2
        local gh = self.lineH * 3 - 3
        Imm.Rect(gx, gy, gw, gh, Color(0, 0, 0, 0.45))
        local mx = 20 -- scale is capped so one hitch does not flatten the rest
        for i = 1, self.count do
            local f = self.frames[(self.head - self.count + i - 1) % HISTORY + 1]
            if f > mx then mx = f end
        end
        mx = math.min(mx, 100)
        local barW = gw / HISTORY
        for i = 1, self.count do
            local f = self.frames[(self.head - self.count + i - 1) % HISTORY + 1]
            local bh = math.min(gh, math.max(1, gh * f / mx))
            local bx = gx + (HISTORY - self.count + i - 1) * barW
            Imm.Rect(bx, gy + gh - bh, math.max(1, barW - 0.5), bh, frameColor(f))
        end
        -- 16.7 ms and 33.3 ms reference lines
        for _, ref in ipairs({ 16.7, 33.3 }) do
            local ly = gy + gh - gh * ref / mx
            if ly >= gy then Imm.Line(gx, ly, gx + gw, ly, Color(1, 1, 1, 0.3), 1) end
        end
        self.font:draw(string.format("%.0f ms", mx), gx + gw - 6 * self.charW, gy + self.textOff, colors.text)
    end
end

return RenderInfoOverlay
