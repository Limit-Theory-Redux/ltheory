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
            void Draw_Clear        (float r, float g, float b, float a);
            void Draw_ClearDepth   (float d);
            void Draw_Color        (float r, float g, float b, float a);
            void Draw_Flush        ();
            void Draw_PushAlpha    (float a);
            void Draw_PopAlpha     ();
            void Draw_LineWidth    (float width);
            void Draw_PointSize    (float size);
            void Draw_Axes         (Vec3f const* pos, Vec3f const* x, Vec3f const* y, Vec3f const* z, float scale, float alpha);
            void Draw_Border       (float s, float x, float y, float w, float h);
            void Draw_Box3         (Box3f const* b);
            void Draw_Line         (float x1, float y1, float x2, float y2);
            void Draw_Line3        (Vec3f const* p1, Vec3f const* p2);
            void Draw_Plane        (Vec3f const* p, Vec3f const* n, float scale);
            void Draw_Point        (float x, float y);
            void Draw_Point3       (float x, float y, float z);
            void Draw_Quad         (Vec2f const* p1, Vec2f const* p2, Vec2f const* p3, Vec2f const* p4);
            void Draw_Quad3        (Vec3f const* p1, Vec3f const* p2, Vec3f const* p3, Vec3f const* p4);
            void Draw_Rect         (float x1, float y1, float xs, float ys);
            void Draw_RectEx       (float x1, float y1, float xs, float ys, float u1, float v1, float u2, float v2);
            void Draw_SmoothPoints (bool enable);
            void Draw_Sphere       (Vec3f const* p, float r);
            void Draw_Tri          (Vec2f const* v1, Vec2f const* v2, Vec2f const* v3);
            void Draw_Tri3         (Vec3f const* v1, Vec3f const* v2, Vec3f const* v3);
            void Draw_Poly         (Vec2f const* points, uint64 points_size);
            void Draw_Poly3        (Vec3f const* points, uint64 points_size);
            void Draw_Circle       (float x, float y, float r, int segments);
            void Draw_CircleFilled (float x, float y, float r, int segments);
            void Draw_Circle3      (Vec3f const* center, Vec3f const* normal, float r, int segments);
            void Draw_Arc          (float x, float y, float r, float start, float end, int segments);
            void Draw_LineStrip    (Vec2f const* points, uint64 points_size);
            void Draw_LineStrip3   (Vec3f const* points, uint64 points_size);
            void Draw_Arrow        (float x1, float y1, float x2, float y2, float headSize);
            void Draw_Arrow3       (Vec3f const* p1, Vec3f const* p2, float headSize);
            void Draw_Grid         (float x, float y, float width, float height, int cellsX, int cellsY);
            void Draw_Grid3        (Vec3f const* center, float size, int cells);
            void Draw_Cylinder     (Vec3f const* base, Vec3f const* axis, float radius, float height, int segments);
            void Draw_Cone         (Vec3f const* base, Vec3f const* axis, float radius, float height, int segments);
            void Draw_Capsule      (Vec3f const* p1, Vec3f const* p2, float radius, int segments);
            void Draw_WireBox3     (Box3f const* b);
            void Draw_Crosshair3   (Vec3f const* pos, float size);
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
            Circle       = libphx.Draw_Circle,
            CircleFilled = libphx.Draw_CircleFilled,
            Circle3      = libphx.Draw_Circle3,
            Arc          = libphx.Draw_Arc,
            LineStrip    = libphx.Draw_LineStrip,
            LineStrip3   = libphx.Draw_LineStrip3,
            Arrow        = libphx.Draw_Arrow,
            Arrow3       = libphx.Draw_Arrow3,
            Grid         = libphx.Draw_Grid,
            Grid3        = libphx.Draw_Grid3,
            Cylinder     = libphx.Draw_Cylinder,
            Cone         = libphx.Draw_Cone,
            Capsule      = libphx.Draw_Capsule,
            WireBox3     = libphx.Draw_WireBox3,
            Crosshair3   = libphx.Draw_Crosshair3,
        }

        if onDef_Draw then onDef_Draw(Draw, mt) end
        Draw = setmetatable(Draw, mt)
    end

    return Draw
end

return Loader
