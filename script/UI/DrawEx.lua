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

-- The batcher copies the color at the call, so one scratch color serves every
-- primitive (no allocation per call).
local tmp = Color(0, 0, 0, 0)

--- `color` with its alpha scaled by the pushed alpha (and replaced by `a`, if given).
local function tint(color, a)
    tmp.r, tmp.g, tmp.b = color.r, color.g, color.b
    tmp.a = (a or color.a) * (alphaStack:last() or 1)
    return tmp
end

function DrawEx.Arrow(p, n, color)
    local t = Vec2f(-n.y / 2, n.x / 2) -- divide by 2 to make directional arrow more clearly pointed
    DrawEx.TriV(p + n, p - n + t, p - n - t, color)
end

function DrawEx.Circle(x, y, r, color)
    local x, y, sx, sy = padAndCenter(padCircle, x, y, r, r)
    Imm.Shape(Shape.Circle, x, y, sx, sy, tint(color), r)
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
    Imm.Shape(Shape.Grid, x, y, sx, sy, tint(c))
end

function DrawEx.Hex(x, y, r, c)
    local x, y, sx, sy = padAndCenter(padRing, x, y, r, r)
    Imm.Shape(Shape.Hex, x, y, sx, sy, tint(c), r)
end

function DrawEx.Icon(icon, x, y, sx, sy, color)
    local x, y, sx, sy = padAndCenter(0, x, y, sx, sy)
    Imm.Icon(icon, Samplers.Point, x, y, sx, sy, tint(color))
end

function DrawEx.Line(x1, y1, x2, y2, color, fade)
    Imm.LineGlow(x1, y1, x2, y2, tint(color), fade and true or false, padLine)
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
    -- bevel: the panel shader's own default (the old uniform was never set here)
    Imm.Shape(Shape.Panel, x, y, sx, sy, tint(color), innerAlpha * alpha, 0)
end

function DrawEx.PanelGlow(x, y, sx, sy, color)
    local x, y, sx, sy = padOffCenter(padPanel, x, y, sx, sy)
    Imm.Shape(Shape.PanelGlow, x, y, sx, sy, tint(color))
end

function DrawEx.Point(x, y, r, color)
    local x, y, sx, sy = padAndCenter(padPoint, x, y, r, r)
    Imm.Shape(Shape.Point, x, y, sx, sy, tint(color))
end

function DrawEx.PointGlow(x, y, r, color)
    local x, y, sx, sy = padAndCenter(padPoint, x, y, r, r)
    Imm.Shape(Shape.PointGlow, x, y, sx, sy, tint(color))
end

function DrawEx.PushAlpha(a)
    alphaStack:append(a * (alphaStack:last() or 1))
end

function DrawEx.PopAlpha()
    alphaStack:pop()
end

--- A solid, alpha-blended rectangle (no glow, unlike `DrawEx.Rect`).
function DrawEx.SimpleRect(x, y, sx, sy, color)
    Imm.Rect(x, y, sx, sy, tint(color))
end

function DrawEx.SimpleBorder(s, x, y, sx, sy, color)
    Imm.Border(s, x, y, sx, sy, tint(color))
end

--- A solid line `width` pixels wide.
function DrawEx.SimpleLine(x1, y1, x2, y2, color, width)
    Imm.Line(x1, y1, x2, y2, tint(color), width or 1)
end

function DrawEx.SimpleTri(x1, y1, x2, y2, x3, y3, color)
    Imm.Tri(x1, y1, x2, y2, x3, y3, tint(color))
end

function DrawEx.SimplePoint(x, y, size, color)
    Imm.Point(x, y, size, tint(color))
end

function DrawEx.Rect(x, y, sx, sy, color)
    local x, y, sx, sy = padOffCenter(padBox, x, y, sx, sy)
    Imm.Shape(Shape.Box, x, y, sx, sy, tint(color))
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
    local x, y, sx, sy = padAndCenter(padRing, x, y, r, r)
    Imm.Shape(glow and Shape.RingGlow or Shape.Ring, x, y, sx, sy, tint(c), r)
end

function DrawEx.RingDim(x, y, r, c)
    local x, y, sx, sy = padAndCenter(padRing, x, y, r, r)
    Imm.Shape(Shape.RingDim, x, y, sx, sy, tint(c), r)
end

function DrawEx.Tri(x1, y1, x2, y2, x3, y3, color)
    Imm.TriGlow(x1, y1, x2, y2, x3, y3, tint(color), padTri)
end

function DrawEx.TriV(p1, p2, p3, color)
    DrawEx.Tri(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, color)
end

function DrawEx.Wedge(x, y, r1, r2, to, tw, c, a)
    local x, y, sx, sy = padAndCenter(padWedge, x, y, 2.0 * r2, 2.0 * r2)
    Imm.Shape(Shape.Wedge, x, y, sx, sy, tint(c, a), r1, r2, to, tw)
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
