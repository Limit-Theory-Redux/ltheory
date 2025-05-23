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

return DrawEx
