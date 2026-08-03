-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Draw
Draw = {}

---@param r Renderer
---@param red number
---@param green number
---@param blue number
---@param alpha number
function Draw.Clear(r, red, green, blue, alpha) end

---@param r Renderer
---@param d number
function Draw.ClearDepth(r, d) end

---@param r Renderer
---@param red number
---@param green number
---@param blue number
---@param alpha number
function Draw.Color(r, red, green, blue, alpha) end

---@param r Renderer
function Draw.Flush(r) end

---@param r Renderer
---@param a number
function Draw.PushAlpha(r, a) end

---@param r Renderer
function Draw.PopAlpha(r) end

---@param r Renderer
---@param width number
function Draw.LineWidth(r, width) end

---@param r Renderer
---@param size number
function Draw.PointSize(r, size) end

---@param r Renderer
---@param pos Vec3f
---@param x Vec3f
---@param y Vec3f
---@param z Vec3f
---@param scale number
---@param alpha number
function Draw.Axes(r, pos, x, y, z, scale, alpha) end

---@param r Renderer
---@param s number
---@param x number
---@param y number
---@param w number
---@param h number
function Draw.Border(r, s, x, y, w, h) end

---@param r Renderer
---@param b Box3f
function Draw.Box3(r, b) end

---@param r Renderer
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
function Draw.Line(r, x1, y1, x2, y2) end

---@param r Renderer
---@param p1 Vec3f
---@param p2 Vec3f
function Draw.Line3(r, p1, p2) end

---@param r Renderer
---@param p Vec3f
---@param n Vec3f
---@param scale number
function Draw.Plane(r, p, n, scale) end

---@param r Renderer
---@param x number
---@param y number
function Draw.Point(r, x, y) end

---@param r Renderer
---@param x number
---@param y number
---@param z number
function Draw.Point3(r, x, y, z) end

---@param r Renderer
---@param p1 Vec2f
---@param p2 Vec2f
---@param p3 Vec2f
---@param p4 Vec2f
function Draw.Quad(r, p1, p2, p3, p4) end

---@param r Renderer
---@param p1 Vec3f
---@param p2 Vec3f
---@param p3 Vec3f
---@param p4 Vec3f
function Draw.Quad3(r, p1, p2, p3, p4) end

---@param r Renderer
---@param x1 number
---@param y1 number
---@param xs number
---@param ys number
function Draw.Rect(r, x1, y1, xs, ys) end

---@param r Renderer
---@param x1 number
---@param y1 number
---@param xs number
---@param ys number
---@param u1 number
---@param v1 number
---@param u2 number
---@param v2 number
function Draw.RectEx(r, x1, y1, xs, ys, u1, v1, u2, v2) end

---@param enable boolean
function Draw.SmoothPoints(enable) end

---@param r Renderer
---@param p Vec3f
---@param radius number
function Draw.Sphere(r, p, radius) end

---@param r Renderer
---@param v1 Vec2f
---@param v2 Vec2f
---@param v3 Vec2f
function Draw.Tri(r, v1, v2, v3) end

---@param r Renderer
---@param v1 Vec3f
---@param v2 Vec3f
---@param v3 Vec3f
function Draw.Tri3(r, v1, v2, v3) end

---@param r Renderer
---@param points Vec2f[]
---@param points_size integer
function Draw.Poly(r, points, points_size) end

---@param r Renderer
---@param points Vec3f[]
---@param points_size integer
function Draw.Poly3(r, points, points_size) end

