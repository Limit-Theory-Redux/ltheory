-- AUTO GENERATED. DO NOT MODIFY!
-- Draw ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Draw'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Draw

    do -- C Definitions
        ffi.cdef [[
            void Draw_Clear        (Renderer* r, float red, float green, float blue, float alpha);
            void Draw_ClearDepth   (Renderer* r, float d);
            void Draw_Color        (Renderer* r, float red, float green, float blue, float alpha);
            void Draw_Flush        (Renderer* r);
            void Draw_PushAlpha    (Renderer* r, float a);
            void Draw_PopAlpha     (Renderer* r);
            void Draw_LineWidth    (Renderer* r, float width);
            void Draw_PointSize    (Renderer* r, float size);
            void Draw_Axes         (Renderer* r, Vec3f const* pos, Vec3f const* x, Vec3f const* y, Vec3f const* z, float scale, float alpha);
            void Draw_Border       (Renderer* r, float s, float x, float y, float w, float h);
            void Draw_Box3         (Renderer* r, Box3f const* b);
            void Draw_Line         (Renderer* r, float x1, float y1, float x2, float y2);
            void Draw_Line3        (Renderer* r, Vec3f const* p1, Vec3f const* p2);
            void Draw_Plane        (Renderer* r, Vec3f const* p, Vec3f const* n, float scale);
            void Draw_Point        (Renderer* r, float x, float y);
            void Draw_Point3       (Renderer* r, float x, float y, float z);
            void Draw_Quad         (Renderer* r, Vec2f const* p1, Vec2f const* p2, Vec2f const* p3, Vec2f const* p4);
            void Draw_Quad3        (Renderer* r, Vec3f const* p1, Vec3f const* p2, Vec3f const* p3, Vec3f const* p4);
            void Draw_Rect         (Renderer* r, float x1, float y1, float xs, float ys);
            void Draw_RectEx       (Renderer* r, float x1, float y1, float xs, float ys, float u1, float v1, float u2, float v2);
            void Draw_SmoothPoints (bool enable);
            void Draw_Sphere       (Renderer* r, Vec3f const* p, float radius);
            void Draw_Tri          (Renderer* r, Vec2f const* v1, Vec2f const* v2, Vec2f const* v3);
            void Draw_Tri3         (Renderer* r, Vec3f const* v1, Vec3f const* v2, Vec3f const* v3);
            void Draw_Poly         (Renderer* r, Vec2f const* points, uint64 points_size);
            void Draw_Poly3        (Renderer* r, Vec3f const* points, uint64 points_size);
        ]]
    end

    do -- Global Symbol Table
        Draw = {
            Clear        = libphx.Draw_Clear,
            ClearDepth   = libphx.Draw_ClearDepth,
            Color        = libphx.Draw_Color,
            Flush        = libphx.Draw_Flush,
            PushAlpha    = libphx.Draw_PushAlpha,
            PopAlpha     = libphx.Draw_PopAlpha,
            LineWidth    = libphx.Draw_LineWidth,
            PointSize    = libphx.Draw_PointSize,
            Axes         = libphx.Draw_Axes,
            Border       = libphx.Draw_Border,
            Box3         = libphx.Draw_Box3,
            Line         = libphx.Draw_Line,
            Line3        = libphx.Draw_Line3,
            Plane        = libphx.Draw_Plane,
            Point        = libphx.Draw_Point,
            Point3       = libphx.Draw_Point3,
            Quad         = libphx.Draw_Quad,
            Quad3        = libphx.Draw_Quad3,
            Rect         = libphx.Draw_Rect,
            RectEx       = libphx.Draw_RectEx,
            SmoothPoints = libphx.Draw_SmoothPoints,
            Sphere       = libphx.Draw_Sphere,
            Tri          = libphx.Draw_Tri,
            Tri3         = libphx.Draw_Tri3,
            Poly         = libphx.Draw_Poly,
            Poly3        = libphx.Draw_Poly3,
        }

        if onDef_Draw then onDef_Draw(Draw, mt) end
        Draw = setmetatable(Draw, mt)
    end

    return Draw
end

return Loader
