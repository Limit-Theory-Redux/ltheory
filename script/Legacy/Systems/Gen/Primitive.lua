local function Billboard(u0, v0, u1, v1)
    local self = Mesh.Create()
    self:addVertex(0, 0, 0, 0, 0, 0, u0, v0)
    self:addVertex(0, 0, 0, 0, 0, 0, u1, v0)
    self:addVertex(0, 0, 0, 0, 0, 0, u1, v1)
    self:addVertex(0, 0, 0, 0, 0, 0, u0, v1)
    self:addQuad(0, 3, 2, 1)
    return self
end

-- Sphere from tessellated icosahedron
-- n = number of tessellation passes; default is 1
-- NOTE : Mesh size is exponential in n; should never need more than 5 or 6!
local function IcoSphere(n, radius)
    radius = radius or 1.0
    local p = (1.0 + sqrt(5)) / 2.0
    local self = PolyMesh()
        :addVertex(0, p, 1):addVertex(0, p, -1):addVertex(0, -p, 1):addVertex(0, -p, -1)
        :addVertex(-1, 0, -p):addVertex(1, 0, -p):addVertex(-1, 0, p):addVertex(1, 0, p)
        :addVertex(-p, 1, 0):addVertex(p, 1, 0):addVertex(-p, -1, 0):addVertex(p, -1, 0)
        :addTri(1, 0, 9):addTri(1, 9, 5):addTri(1, 5, 4):addTri(1, 4, 8):addTri(1, 8, 0)
        :addTri(7, 6, 2):addTri(11, 7, 2):addTri(3, 11, 2):addTri(10, 3, 2):addTri(6, 10, 2)
        :addTri(9, 0, 7):addTri(9, 7, 11):addTri(5, 9, 11):addTri(5, 11, 3):addTri(5, 3, 4)
        :addTri(4, 3, 10):addTri(4, 10, 8):addTri(8, 10, 6):addTri(0, 8, 6):addTri(0, 6, 7)

    for i = 1, n or 1 do self:tessellate() end
    self:spherize()
    self:scale(radius, radius, radius)
    local mesh = self:getMesh()
    mesh:computeNormals()
    return mesh
end

local function Ring(innerRadius, outerRadius, segments)
    innerRadius = innerRadius or 1.0
    outerRadius = outerRadius or 2.0
    segments = segments or 64

    local self = Mesh.Create()
    local verts = {}
    local indices = {}

    -- Generate vertices
    for i = 0, segments do
        local a = i / segments * math.pi * 2
        local c, s = math.cos(a), math.sin(a)

        -- Outer vertex
        table.insert(verts, { c * outerRadius, 0, s * outerRadius })
        table.insert(verts, { c * innerRadius, 0, s * innerRadius })

        -- UVs: radial + angular
        local u = (i / segments)
        local vOuter = 0.0
        local vInner = 1.0
        table.insert(verts[#verts], u)
        table.insert(verts[#verts], vOuter)
        table.insert(verts[#verts - 1], u)
        table.insert(verts[#verts - 1], vInner)
    end

    -- Generate quads (as two triangles)
    for i = 1, segments do
        local base = (i - 1) * 2
        local next = i * 2

        -- Quad: (base, base+1, next+1, next)
        table.insert(indices, base); table.insert(indices, base + 1)
        table.insert(indices, next + 1); table.insert(indices, next)
        table.insert(indices, base); table.insert(indices, next + 1)
        table.insert(indices, next); table.insert(indices, next + 1)
    end

    -- Upload to mesh
    for _, v in ipairs(verts) do
        local x, y, z, u, v = table.unpack(v)
        self:addVertex(x, y, z, 0, 1, 0, u, v) -- normal = up
    end
    for _, idx in ipairs(indices) do
        self:addIndex(idx)
    end

    self:computeNormals()

    return self
end

return {
    Billboard = Billboard,
    IcoSphere = IcoSphere,
    Ring = Ring
}
