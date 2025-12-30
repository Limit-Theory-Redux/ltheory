local DrawEx = {}

local function padOffCenter(pad, x, y, sx, sy)
    return x - pad, y - pad, sx + 2 * pad, sy + 2 * pad
end

local function padAndCenter(pad, x, y, sx, sy)
    return x - 0.5 * sx - pad, y - 0.5 * sy - pad, sx + 2 * pad, sy + 2 * pad
end

-- TODO : Push the paddings down as far as possible without clipping halos
local padBox = 32
local padLine = 64
local padPanel = 64
local padCircle = 64 -- limits circle size to a maximum radius of about 110 without clipping box border
local padPoint = 32
local padRing = 256  -- 128 clips rings when zooming; 1024 doesn't clip at 3840x2160, but murders the frame rate
local padTri = 32
local padWedge = 32
local alphaStack = List()

function DrawEx.Arrow(p, n, color)
    local t = Vec2f(-n.y / 2, n.x / 2) -- divide by 2 to make directional arrow more clearly pointed
    DrawEx.TriV(p + n, p - n + t, p - n - t, color)
end

function DrawEx.Circle(x, y, r, color)
    local x, y, sx, sy = padAndCenter(padCircle, x, y, r, r)
    local shader = Cache.Shader('ui', 'ui/circle')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat('radius', r)
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Cross(x, y, r, color)
    DrawEx.Line(x - r, y - r, x + r, y + r, color, false)
    DrawEx.Line(x - r, y + r, x + r, y - r, color, false)
end

function DrawEx.GetAlpha()
    return alphaStack:last() or 1
end

function DrawEx.Grid(x, y, sx, sy, c)
    local x, y, sx, sy = padOffCenter(padPanel, x, y, sx, sy)
    local shader = Cache.Shader('ui', 'ui/grid')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', c.r, c.g, c.b, c.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Hex(x, y, r, c)
    local x, y, sx, sy = padAndCenter(padRing, x, y, r, r)
    local shader = Cache.Shader('ui', 'ui/hex')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat('radius', r)
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', c.r, c.g, c.b, c.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Hologram(mesh, x, y, sx, sy, color, radius, yaw, pitch)
    local center = mesh:getCenter()
    local eye = center + Math.Spherical(radius, pitch, yaw)
    local mView = Matrix.LookAt(eye, center, Vec3f(0, -1, 0))
    local mProj = Matrix.Perspective(70, sx / sy, 0.1, 1e6)
    local shader = Cache.Shader('ui3D', 'ui/hologram')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setMatrix('mView', mView)
    shader:setMatrix('mProj', mProj)
    shader:setFloat('time', Engine:getTime())
    shader:setFloat3('eye', eye.x, eye.y, eye.z)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    shader:setFloat4('viewport', x, y, x + sx, y + sy)
    mesh:draw()
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Icon(icon, x, y, sx, sy, color)
    -- Log.Debug(icon)
    local x, y, sx, sy = padAndCenter(0, x, y, sx, sy)
    local shader = Cache.Shader('ui', 'ui/icon')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    shader:setTex2D('icon', icon)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Line(x1, y1, x2, y2, color, fade)
    local fadeval = 0
    if fade then fadeval = 1 end -- insure we pass the correct Int to the shader (expects bool)
    local xMin = min(x1, x2) - padLine
    local yMin = min(y1, y2) - padLine
    local xMax = max(x1, x2) + padLine
    local yMax = max(y1, y2) + padLine
    local sx = xMax - xMin
    local sy = yMax - yMin
    local shader = Cache.Shader('ui', 'ui/line')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat2('origin', xMin, yMin)
    shader:setFloat2('size', sx, sy)
    shader:setFloat2('p1', x1, y1)
    shader:setFloat2('p2', x2, y2)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    shader:setInt('fade', fadeval)
    Draw.Rect(xMin, yMin, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Meter(x, y, sx, sy, color, spacing, total, level, overcharge, overchargeColor, direction)
    -- NOTE: There must be a more elegant way to do this, but brain will not brain today
    local filled = level
    if direction == -1 then
        filled = total - level
        for i = 1, total do
            if i <= filled then
                DrawEx.PanelGlow(x, y, sx, sy, color)
            else
                if overcharge and i == 1 then
                    DrawEx.Rect(x, y, sx, sy, overchargeColor)
                else
                    DrawEx.Rect(x, y, sx, sy, color)
                end
            end
            x = x + sx + spacing
        end
    else
        for i = 1, total do
            if i <= filled then
                if overcharge and i == total then
                    DrawEx.Rect(x, y, sx, sy, overchargeColor)
                else
                    DrawEx.Rect(x, y, sx, sy, color)
                end
            else
                DrawEx.PanelGlow(x, y, sx, sy, color)
            end
            x = x + sx + spacing
        end
    end
end

function DrawEx.MeterV(x, y, sx, sy, color, spacing, total, level)
    for i = 1, total do
        if i <= level then
            DrawEx.Rect(x, y, sx, sy, color)
        else
            DrawEx.PanelGlow(x, y, sx, sy, color)
        end
        y = y - (sy + spacing)
    end
end

function DrawEx.Panel(x, y, sx, sy, color, innerAlpha)
    local color = color or Color(0.2, 0.2, 0.2, 1.0)
    local innerAlpha = innerAlpha or 1
    local alpha = alphaStack:last() or 1
    local x, y, sx, sy = padOffCenter(padPanel, x, y, sx, sy)
    local shader = Cache.Shader('ui', 'ui/panel')
    RenderState.PushBlendMode(BlendMode.Alpha)
    shader:start()
    shader:setFloat('padding', padPanel)
    shader:setFloat('innerAlpha', innerAlpha * alpha)
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.PanelGlow(x, y, sx, sy, color)
    local x, y, sx, sy = padOffCenter(padPanel, x, y, sx, sy)
    local shader = Cache.Shader('ui', 'ui/panelglow')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat('padding', padPanel)
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Point(x, y, r, color)
    local x, y, sx, sy = padAndCenter(padPoint, x, y, r, r)
    local shader = Cache.Shader('ui', 'ui/point') -- previously used 'ui/circle-old' shader
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Alpha)
    shader:start()
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.PointGlow(x, y, r, color)
    local x, y, sx, sy = padAndCenter(padPoint, x, y, r, r)
    local shader = Cache.Shader('ui', 'ui/point') -- previously used 'ui/circle-old' shader
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.PushAlpha(a)
    alphaStack:append(a * (alphaStack:last() or 1))
end

function DrawEx.PopAlpha()
    alphaStack:pop()
end

function DrawEx.SimpleShaderStart(color)
    local shader = Cache.Shader('ui', 'simple_color')
    local alpha = alphaStack:last() or 1
    shader:start()
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
end

function DrawEx.SimpleShaderStop()
    local shader = Cache.Shader('ui', 'simple_color')
    shader:start()
end

function DrawEx.SimpleRect(x, y, sx, sy, color)
    DrawEx.SimpleShaderStart(color)
    Draw.Rect(x, y, sx, sy)
    DrawEx.SimpleShaderStop()
end

function DrawEx.Rect(x, y, sx, sy, color)
    local x, y, sx, sy = padOffCenter(padBox, x, y, sx, sy)
    local shader = Cache.Shader('ui', 'ui/box')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.RectOutline(x, y, sx, sy, color)
    local p = 1.5
    local lx, rx = x, x + sx
    local ty, by = y, y + sy
    DrawEx.Line(lx + p, ty, rx - p, ty, color, false)
    DrawEx.Line(rx, ty + p, rx, by - p, color, false)
    DrawEx.Line(rx - p, by, lx + p, by, color, false)
    DrawEx.Line(lx, by - p, lx, ty + p, color, false)
end

function DrawEx.Ring(x, y, r, c, glow)
    local glowval = 0
    if glow then glowval = 1 end -- insure we pass the correct Int to the shader (expects bool)
    local x, y, sx, sy = padAndCenter(padRing, x, y, r, r)
    local shader = Cache.Shader('ui', 'ui/ring')
    local alpha = alphaStack:last() or 1
    if glow then
        RenderState.PushBlendMode(BlendMode.Additive)
    else
        RenderState.PushBlendMode(BlendMode.Alpha)
    end
    shader:start()
    shader:setFloat('radius', r)
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', c.r, c.g, c.b, c.a * alpha)
    shader:setInt('glow', glowval)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.RingDim(x, y, r, c)
    local glowval = 0
    local x, y, sx, sy = padAndCenter(padRing, x, y, r, r)
    local shader = Cache.Shader('ui', 'ui/ringdim')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat('radius', r)
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', c.r, c.g, c.b, c.a * alpha)
    shader:setInt('glow', 1)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.Tri(x1, y1, x2, y2, x3, y3, color)
    local xMin = min(x1, min(x2, x3)) - padTri
    local yMin = min(y1, min(y2, y3)) - padTri
    local xMax = max(x1, max(x2, x3)) + padTri
    local yMax = max(y1, max(y2, y3)) + padTri
    local shader = Cache.Shader('ui', 'ui/triangle')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat2('p1', x1, y1)
    shader:setFloat2('p2', x2, y2)
    shader:setFloat2('p3', x3, y3)
    shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
    Draw.Rect(xMin, yMin, xMax - xMin, yMax - yMin)
    shader:stop()
    RenderState.PopBlendMode()
end

function DrawEx.TriV(p1, p2, p3, color)
    DrawEx.Tri(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, color)
end

function DrawEx.Wedge(x, y, r1, r2, to, tw, c, a)
    local x, y, sx, sy = padAndCenter(padWedge, x, y, 2.0 * r2, 2.0 * r2)
    local shader = Cache.Shader('ui', 'ui/wedge')
    local alpha = alphaStack:last() or 1
    RenderState.PushBlendMode(BlendMode.Additive)
    shader:start()
    shader:setFloat('r1', r1)
    shader:setFloat('r2', r2)
    shader:setFloat('to', to)
    shader:setFloat('tw', tw)
    shader:setFloat2('size', sx, sy)
    shader:setFloat4('color', c.r, c.g, c.b, (a or c.a) * alpha)
    Draw.Rect(x, y, sx, sy)
    shader:stop()
    RenderState.PopBlendMode()
end

local function drawText(font, text, size, x, y, sx, sy, cr, cg, cb, ca, alignX, alignY)
    local ax = alignX or 0.0
    local ay = alignY or 1.0
    local font = Cache.Font(font, size)
    local bound = font:getSize(text)
    local alpha = alphaStack:last() or 1
    font:draw(text,
        x + ax * (sx - bound.z) - bound.x,
        y + ay * (sy - bound.w) + bound.w,
        Color(cr, cg, cb, ca * alpha)
    )
end

function DrawEx.TextAdditive(...)
    RenderState.PushBlendMode(BlendMode.Additive)
    drawText(...)
    RenderState.PopBlendMode()
end

function DrawEx.TextAlpha(...)
    RenderState.PushBlendMode(BlendMode.Alpha)
    drawText(...)
    RenderState.PopBlendMode()
end

-- Extended UI primitives for space game HUD

--- Draw a circular gauge (like a speedometer or power meter)
--- @param x number Center X position
--- @param y number Center Y position
--- @param r number Radius
--- @param value number Current value (0 to 1)
--- @param color Color Gauge color
--- @param startAngle number? Start angle in radians (default: -PI*0.75)
--- @param sweepAngle number? Total sweep angle (default: PI*1.5)
function DrawEx.Gauge(x, y, r, value, color, startAngle, sweepAngle)
    local startAngle = startAngle or (-math.pi * 0.75)
    local sweepAngle = sweepAngle or (math.pi * 1.5)
    local endAngle = startAngle + sweepAngle * math.min(1, math.max(0, value))

    -- Background arc (dim)
    local dimColor = Color(color.r * 0.2, color.g * 0.2, color.b * 0.2, color.a * 0.5)
    DrawEx.Arc(x, y, r, startAngle, startAngle + sweepAngle, dimColor, 32)

    -- Foreground arc (bright)
    if value > 0 then
        DrawEx.Arc(x, y, r, startAngle, endAngle, color, 32)
    end
end

--- Draw an arc using line segments
--- @param x number Center X
--- @param y number Center Y
--- @param r number Radius
--- @param startAngle number Start angle in radians
--- @param endAngle number End angle in radians
--- @param color Color Arc color
--- @param segments number? Number of segments (default: 16)
function DrawEx.Arc(x, y, r, startAngle, endAngle, color, segments)
    local segments = segments or 16
    local alpha = alphaStack:last() or 1
    local angleStep = (endAngle - startAngle) / segments

    for i = 0, segments - 1 do
        local a1 = startAngle + i * angleStep
        local a2 = startAngle + (i + 1) * angleStep
        local x1 = x + r * math.cos(a1)
        local y1 = y + r * math.sin(a1)
        local x2 = x + r * math.cos(a2)
        local y2 = y + r * math.sin(a2)
        DrawEx.Line(x1, y1, x2, y2, Color(color.r, color.g, color.b, color.a * alpha), false)
    end
end

--- Draw a horizontal progress bar
--- @param x number Left position
--- @param y number Top position
--- @param width number Total width
--- @param height number Height
--- @param value number Progress value (0 to 1)
--- @param fgColor Color Foreground (filled) color
--- @param bgColor Color? Background color (optional)
function DrawEx.ProgressBar(x, y, width, height, value, fgColor, bgColor)
    value = math.min(1, math.max(0, value))

    -- Background
    if bgColor then
        DrawEx.Rect(x, y, width, height, bgColor)
    end

    -- Foreground (filled portion)
    if value > 0 then
        DrawEx.Rect(x, y, width * value, height, fgColor)
    end
end

--- Draw a vertical progress bar (fills from bottom to top)
--- @param x number Left position
--- @param y number Bottom position
--- @param width number Width
--- @param height number Total height
--- @param value number Progress value (0 to 1)
--- @param fgColor Color Foreground color
--- @param bgColor Color? Background color (optional)
function DrawEx.ProgressBarV(x, y, width, height, value, fgColor, bgColor)
    value = math.min(1, math.max(0, value))

    -- Background
    if bgColor then
        DrawEx.Rect(x, y, width, height, bgColor)
    end

    -- Foreground (filled portion from bottom)
    if value > 0 then
        local filledHeight = height * value
        DrawEx.Rect(x, y + height - filledHeight, width, filledHeight, fgColor)
    end
end

--- Draw corner brackets (for selection/targeting)
--- @param x number Left position
--- @param y number Top position
--- @param width number Box width
--- @param height number Box height
--- @param bracketSize number Size of bracket arms
--- @param color Color Bracket color
function DrawEx.Bracket(x, y, width, height, bracketSize, color)
    local s = bracketSize

    -- Top-left corner
    DrawEx.Line(x, y, x + s, y, color, false)
    DrawEx.Line(x, y, x, y + s, color, false)

    -- Top-right corner
    DrawEx.Line(x + width - s, y, x + width, y, color, false)
    DrawEx.Line(x + width, y, x + width, y + s, color, false)

    -- Bottom-left corner
    DrawEx.Line(x, y + height - s, x, y + height, color, false)
    DrawEx.Line(x, y + height, x + s, y + height, color, false)

    -- Bottom-right corner
    DrawEx.Line(x + width, y + height - s, x + width, y + height, color, false)
    DrawEx.Line(x + width - s, y + height, x + width, y + height, color, false)
end

--- Draw a 2D crosshair
--- @param x number Center X
--- @param y number Center Y
--- @param size number Crosshair size
--- @param gap number? Gap in center (default: 0)
--- @param color Color Crosshair color
function DrawEx.Crosshair(x, y, size, gap, color)
    local gap = gap or 0
    local half = size / 2

    -- Horizontal lines
    DrawEx.Line(x - half, y, x - gap, y, color, false)
    DrawEx.Line(x + gap, y, x + half, y, color, false)

    -- Vertical lines
    DrawEx.Line(x, y - half, x, y - gap, color, false)
    DrawEx.Line(x, y + gap, x, y + half, color, false)
end

--- Draw a diamond shape
--- @param x number Center X
--- @param y number Center Y
--- @param size number Diamond size (distance from center to point)
--- @param color Color Diamond color
function DrawEx.Diamond(x, y, size, color)
    DrawEx.Line(x, y - size, x + size, y, color, false)
    DrawEx.Line(x + size, y, x, y + size, color, false)
    DrawEx.Line(x, y + size, x - size, y, color, false)
    DrawEx.Line(x - size, y, x, y - size, color, false)
end

--- Draw a filled diamond
--- @param x number Center X
--- @param y number Center Y
--- @param size number Diamond size
--- @param color Color Diamond color
function DrawEx.DiamondFilled(x, y, size, color)
    DrawEx.TriV(
        Vec2f(x, y - size),
        Vec2f(x + size, y),
        Vec2f(x, y + size),
        color
    )
    DrawEx.TriV(
        Vec2f(x, y - size),
        Vec2f(x, y + size),
        Vec2f(x - size, y),
        color
    )
end

--- Draw a radar sweep indicator
--- @param x number Center X
--- @param y number Center Y
--- @param r number Radius
--- @param angle number Current sweep angle
--- @param color Color Sweep color
function DrawEx.RadarSweep(x, y, r, angle, color)
    -- Draw ring
    DrawEx.Ring(x, y, r, Color(color.r * 0.3, color.g * 0.3, color.b * 0.3, color.a * 0.5), false)

    -- Draw sweep line
    local dx = r * math.cos(angle)
    local dy = r * math.sin(angle)
    DrawEx.Line(x, y, x + dx, y + dy, color, true)

    -- Draw trailing fade (previous positions)
    for i = 1, 5 do
        local fadeAngle = angle - i * 0.1
        local fadeDx = r * math.cos(fadeAngle)
        local fadeDy = r * math.sin(fadeAngle)
        local fadeAlpha = color.a * (1 - i * 0.18)
        DrawEx.Line(x, y, x + fadeDx, y + fadeDy, Color(color.r, color.g, color.b, fadeAlpha), true)
    end
end

--- Draw a connector line with curve (for node graphs or UI connections)
--- @param x1 number Start X
--- @param y1 number Start Y
--- @param x2 number End X
--- @param y2 number End Y
--- @param color Color Line color
--- @param segments number? Number of curve segments (default: 16)
function DrawEx.ConnectorLine(x1, y1, x2, y2, color, segments)
    local segments = segments or 16
    local dx = x2 - x1

    -- Use cubic bezier-like curve (horizontal tangents at endpoints)
    for i = 0, segments - 1 do
        local t1 = i / segments
        local t2 = (i + 1) / segments

        -- Cubic ease: smooth S-curve
        local ease1 = t1 * t1 * (3 - 2 * t1)
        local ease2 = t2 * t2 * (3 - 2 * t2)

        local px1 = x1 + dx * t1
        local py1 = y1 + (y2 - y1) * ease1
        local px2 = x1 + dx * t2
        local py2 = y1 + (y2 - y1) * ease2

        DrawEx.Line(px1, py1, px2, py2, color, false)
    end
end

--- Draw a notched bar (like health bar with notches for each segment)
--- @param x number Left position
--- @param y number Top position
--- @param width number Total width
--- @param height number Height
--- @param value number Current value (0 to max)
--- @param maxValue number Maximum value
--- @param segments number Number of segments
--- @param fgColor Color Filled segment color
--- @param bgColor Color Empty segment color
--- @param spacing number? Gap between segments (default: 2)
function DrawEx.NotchedBar(x, y, width, height, value, maxValue, segments, fgColor, bgColor, spacing)
    local spacing = spacing or 2
    local segWidth = (width - (segments - 1) * spacing) / segments
    local filledSegments = math.floor((value / maxValue) * segments + 0.5)

    for i = 0, segments - 1 do
        local segX = x + i * (segWidth + spacing)
        if i < filledSegments then
            DrawEx.Rect(segX, y, segWidth, height, fgColor)
        else
            DrawEx.PanelGlow(segX, y, segWidth, height, bgColor)
        end
    end
end

return DrawEx
