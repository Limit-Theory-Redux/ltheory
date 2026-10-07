-- AUTO GENERATED. DO NOT MODIFY!
-- Imm -------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Imm'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Imm

    do -- C Definitions
        ffi.cdef [[
            void Imm_Rect     (Renderer* r, float x, float y, float w, float h, Color const* color);
            void Imm_Border   (Renderer* r, float s, float x, float y, float w, float h, Color const* color);
            void Imm_Image    (Renderer* r, Tex2D const* tex, uint32 sampler, float x, float y, float w, float h, float u0, float v0, float u1, float v1, Color const* color);
            void Imm_Icon     (Renderer* r, Tex2D const* tex, uint32 sampler, float x, float y, float w, float h, Color const* color);
            void Imm_Shape    (Renderer* r, Shape shape, float x, float y, float w, float h, Color const* color, float a, float b, float c, float d);
            void Imm_Tri      (Renderer* r, float x1, float y1, float x2, float y2, float x3, float y3, Color const* color);
            void Imm_TriGlow  (Renderer* r, float x1, float y1, float x2, float y2, float x3, float y3, Color const* color, float pad);
            void Imm_Line     (Renderer* r, float x1, float y1, float x2, float y2, Color const* color, float width);
            void Imm_LineGlow (Renderer* r, float x1, float y1, float x2, float y2, Color const* color, bool fade, float pad);
            void Imm_Point    (Renderer* r, float x, float y, float size, Color const* color);
            void Imm_Box3     (Renderer* r, Box3f const* b);
            void Imm_Line3    (Renderer* r, Vec3f const* p1, Vec3f const* p2, Color const* color, float width, bool depth);
            void Imm_Point3   (Renderer* r, Vec3f const* p, Color const* color, float size, bool depth);
        ]]
    end

    do -- Global Symbol Table
        Imm = {
            Rect     = libphx.Imm_Rect,
            Border   = libphx.Imm_Border,
            Image    = libphx.Imm_Image,
            Icon     = libphx.Imm_Icon,
            Shape    = libphx.Imm_Shape,
            Tri      = libphx.Imm_Tri,
            TriGlow  = libphx.Imm_TriGlow,
            Line     = libphx.Imm_Line,
            LineGlow = libphx.Imm_LineGlow,
            Point    = libphx.Imm_Point,
            Box3     = libphx.Imm_Box3,
            Line3    = libphx.Imm_Line3,
            Point3   = libphx.Imm_Point3,
        }

        if onDef_Imm then onDef_Imm(Imm, mt) end
        Imm = setmetatable(Imm, mt)
    end

    return Imm
end

return Loader
