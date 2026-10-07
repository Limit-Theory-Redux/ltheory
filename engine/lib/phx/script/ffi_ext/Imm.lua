local libphx = require('libphx').lib

-- The immediate batcher draws into the open render pass of the current
-- Renderer (see doc/engine/render-api-v2.md, section 3d); inject the global
-- `Renderer` set by SetEngine so call sites don't pass it.
function onDef_Imm(t, mt)
    t.Rect = function(x, y, w, h, color)
        libphx.Imm_Rect(Renderer, x, y, w, h, color)
    end

    t.Border = function(s, x, y, w, h, color)
        libphx.Imm_Border(Renderer, s, x, y, w, h, color)
    end

    --- `Imm.Image(tex, sampler, x, y, w, h, u0, v0, u1, v1, color)`
    t.Image = function(tex, sampler, x, y, w, h, u0, v0, u1, v1, color)
        libphx.Imm_Image(Renderer, tex, sampler, x, y, w, h, u0, v0, u1, v1, color)
    end

    t.Icon = function(tex, sampler, x, y, w, h, color)
        libphx.Imm_Icon(Renderer, tex, sampler, x, y, w, h, color)
    end

    --- `Imm.Shape(Shape.Circle, x, y, w, h, color, a, b, c, d)`: `a..d` are the
    --- shape's parameters (radius, ...), see the `Shape` enum in imm.rs.
    t.Shape = function(shape, x, y, w, h, color, a, b, c, d)
        libphx.Imm_Shape(Renderer, shape, x, y, w, h, color, a or 0, b or 0, c or 0, d or 0)
    end

    t.Tri = function(x1, y1, x2, y2, x3, y3, color)
        libphx.Imm_Tri(Renderer, x1, y1, x2, y2, x3, y3, color)
    end

    t.TriGlow = function(x1, y1, x2, y2, x3, y3, color, pad)
        libphx.Imm_TriGlow(Renderer, x1, y1, x2, y2, x3, y3, color, pad)
    end

    t.Line = function(x1, y1, x2, y2, color, width)
        libphx.Imm_Line(Renderer, x1, y1, x2, y2, color, width)
    end

    t.LineGlow = function(x1, y1, x2, y2, color, fade, pad)
        libphx.Imm_LineGlow(Renderer, x1, y1, x2, y2, color, fade, pad)
    end

    t.Point = function(x, y, size, color)
        libphx.Imm_Point(Renderer, x, y, size, color)
    end

    t.Box3 = function(b)
        libphx.Imm_Box3(Renderer, b)
    end

    t.Line3 = function(p1, p2, color, width)
        libphx.Imm_Line3(Renderer, p1, p2, color, width)
    end

    t.Point3 = function(p, color, size)
        libphx.Imm_Point3(Renderer, p, color, size)
    end
end
