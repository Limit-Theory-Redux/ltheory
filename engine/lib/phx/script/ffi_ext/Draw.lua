local libphx = require('libphx').lib

-- Draw's C functions (except SmoothPoints, a no-op stub) all take the
-- current Renderer as their first argument now (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_Draw(t, mt)
    t.Clear = function(red, green, blue, alpha)
        libphx.Draw_Clear(Renderer, red, green, blue, alpha)
    end

    t.ClearDepth = function(d)
        libphx.Draw_ClearDepth(Renderer, d)
    end

    t.Color = function(red, green, blue, alpha)
        libphx.Draw_Color(Renderer, red, green, blue, alpha)
    end

    t.Flush = function()
        libphx.Draw_Flush(Renderer)
    end

    t.PushAlpha = function(a)
        libphx.Draw_PushAlpha(Renderer, a)
    end

    t.PopAlpha = function()
        libphx.Draw_PopAlpha(Renderer)
    end

    t.LineWidth = function(width)
        libphx.Draw_LineWidth(Renderer, width)
    end

    t.PointSize = function(size)
        libphx.Draw_PointSize(Renderer, size)
    end

    t.Axes = function(pos, x, y, z, scale, alpha)
        libphx.Draw_Axes(Renderer, pos, x, y, z, scale, alpha)
    end

    t.Border = function(s, x, y, w, h)
        libphx.Draw_Border(Renderer, s, x, y, w, h)
    end

    t.Box3 = function(b)
        libphx.Draw_Box3(Renderer, b)
    end

    t.Line = function(x1, y1, x2, y2)
        libphx.Draw_Line(Renderer, x1, y1, x2, y2)
    end

    t.Line3 = function(p1, p2)
        libphx.Draw_Line3(Renderer, p1, p2)
    end

    t.Plane = function(p, n, scale)
        libphx.Draw_Plane(Renderer, p, n, scale)
    end

    t.Point = function(x, y)
        libphx.Draw_Point(Renderer, x, y)
    end

    t.Point3 = function(x, y, z)
        libphx.Draw_Point3(Renderer, x, y, z)
    end

    t.Quad = function(p1, p2, p3, p4)
        libphx.Draw_Quad(Renderer, p1, p2, p3, p4)
    end

    t.Quad3 = function(p1, p2, p3, p4)
        libphx.Draw_Quad3(Renderer, p1, p2, p3, p4)
    end

    t.Rect = function(x1, y1, xs, ys)
        libphx.Draw_Rect(Renderer, x1, y1, xs, ys)
    end

    t.RectEx = function(x1, y1, xs, ys, u1, v1, u2, v2)
        libphx.Draw_RectEx(Renderer, x1, y1, xs, ys, u1, v1, u2, v2)
    end

    t.Sphere = function(p, radius)
        libphx.Draw_Sphere(Renderer, p, radius)
    end

    t.Tri = function(v1, v2, v3)
        libphx.Draw_Tri(Renderer, v1, v2, v3)
    end

    t.Tri3 = function(v1, v2, v3)
        libphx.Draw_Tri3(Renderer, v1, v2, v3)
    end

    t.Poly = function(points, pointsSize)
        libphx.Draw_Poly(Renderer, points, pointsSize)
    end

    t.Poly3 = function(points, pointsSize)
        libphx.Draw_Poly3(Renderer, points, pointsSize)
    end
end
