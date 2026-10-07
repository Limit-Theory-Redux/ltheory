-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Imm
Imm = {}

-- A solid rectangle in pixel coordinates (y down).
---@param r Renderer
---@param x number
---@param y number
---@param w number
---@param h number
---@param color Color
function Imm.Rect(r, x, y, w, h, color) end

-- A solid rectangle outline of thickness `s` inside the rectangle.
---@param r Renderer
---@param s number
---@param x number
---@param y number
---@param w number
---@param h number
---@param color Color
function Imm.Border(r, s, x, y, w, h, color) end

-- A textured rectangle, multiplied by `color`. `sampler` is a
-- `Samplers.*` value.
---@param r Renderer
---@param tex Tex2D
---@param sampler integer
---@param x number
---@param y number
---@param w number
---@param h number
---@param u0 number
---@param v0 number
---@param u1 number
---@param v1 number
---@param color Color
function Imm.Image(r, tex, sampler, x, y, w, h, u0, v0, u1, v1, color) end

-- An icon: the texture's alpha, tinted by `color`, added to the target.
---@param r Renderer
---@param tex Tex2D
---@param sampler integer
---@param x number
---@param y number
---@param w number
---@param h number
---@param color Color
function Imm.Icon(r, tex, sampler, x, y, w, h, color) end

-- One of the `ui/*` shapes in the rectangle `x, y, w, h` (which includes
-- the shape's own padding); `a..d` are the shape's parameters (see
-- `Shape`).
---@param r Renderer
---@param shape Shape
---@param x number
---@param y number
---@param w number
---@param h number
---@param color Color
---@param a number
---@param b number
---@param c number
---@param d number
function Imm.Shape(r, shape, x, y, w, h, color, a, b, c, d) end

-- A solid triangle.
---@param r Renderer
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param x3 number
---@param y3 number
---@param color Color
function Imm.Tri(r, x1, y1, x2, y2, x3, y3, color) end

-- The soft-edged triangle (`ui/triangle`), additive.
---@param r Renderer
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param x3 number
---@param y3 number
---@param color Color
---@param pad number
function Imm.TriGlow(r, x1, y1, x2, y2, x3, y3, color, pad) end

-- A solid line of `width` pixels.
---@param r Renderer
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param color Color
---@param width number
function Imm.Line(r, x1, y1, x2, y2, color, width) end

-- The soft-edged line of `ui/line`, additive.
---@param r Renderer
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param color Color
---@param fade boolean
---@param pad number
function Imm.LineGlow(r, x1, y1, x2, y2, color, fade, pad) end

-- A solid square point of `size` pixels centered on `x, y`.
---@param r Renderer
---@param x number
---@param y number
---@param size number
---@param color Color
function Imm.Point(r, x, y, size, color) end

-- The faces of `b` with the pass's current pipeline (it must use
-- `VertexLayout.Imm3D`).
---@param r Renderer
---@param b Box3f
function Imm.Box3(r, b) end

-- A debug line (camera-relative positions, depth tested, alpha blended).
---@param r Renderer
---@param p1 Vec3f
---@param p2 Vec3f
---@param color Color
---@param width number
function Imm.Line3(r, p1, p2, color, width) end

-- A debug point of `size` pixels.
---@param r Renderer
---@param p Vec3f
---@param color Color
---@param size number
function Imm.Point3(r, p, color, size) end

