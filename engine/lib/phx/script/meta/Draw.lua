-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Draw
Draw = {}

---@param r number
---@param g number
---@param b number
---@param a number
function Draw.Clear(r, g, b, a) end

---@param d number
function Draw.ClearDepth(d) end

---@param r number
---@param g number
---@param b number
---@param a number
function Draw.Color(r, g, b, a) end

function Draw.Flush() end

---@param a number
function Draw.PushAlpha(a) end

function Draw.PopAlpha() end

---@param width number
function Draw.LineWidth(width) end

---@param size number
function Draw.PointSize(size) end

---@param pos Vec3f
---@param x Vec3f
---@param y Vec3f
---@param z Vec3f
---@param scale number
---@param alpha number
function Draw.Axes(pos, x, y, z, scale, alpha) end

---@param s number
---@param x number
---@param y number
---@param w number
---@param h number
function Draw.Border(s, x, y, w, h) end

---@param b Box3f
function Draw.Box3(b) end

---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
function Draw.Line(x1, y1, x2, y2) end

---@param p1 Vec3f
---@param p2 Vec3f
function Draw.Line3(p1, p2) end

---@param p Vec3f
---@param n Vec3f
---@param scale number
function Draw.Plane(p, n, scale) end

---@param x number
---@param y number
function Draw.Point(x, y) end

---@param x number
---@param y number
---@param z number
function Draw.Point3(x, y, z) end

---@param p1 Vec2f
---@param p2 Vec2f
---@param p3 Vec2f
---@param p4 Vec2f
function Draw.Quad(p1, p2, p3, p4) end

---@param p1 Vec3f
---@param p2 Vec3f
---@param p3 Vec3f
---@param p4 Vec3f
function Draw.Quad3(p1, p2, p3, p4) end

---@param x1 number
---@param y1 number
---@param xs number
---@param ys number
function Draw.Rect(x1, y1, xs, ys) end

---@param x1 number
---@param y1 number
---@param xs number
---@param ys number
---@param u1 number
---@param v1 number
---@param u2 number
---@param v2 number
function Draw.RectEx(x1, y1, xs, ys, u1, v1, u2, v2) end

---@param enable boolean
function Draw.SmoothPoints(enable) end

---@param p Vec3f
---@param r number
function Draw.Sphere(p, r) end

---@param v1 Vec2f
---@param v2 Vec2f
---@param v3 Vec2f
function Draw.Tri(v1, v2, v3) end

---@param v1 Vec3f
---@param v2 Vec3f
---@param v3 Vec3f
function Draw.Tri3(v1, v2, v3) end

---@param points Vec2f[]
---@param points_size integer
function Draw.Poly(points, points_size) end

---@param points Vec3f[]
---@param points_size integer
function Draw.Poly3(points, points_size) end

