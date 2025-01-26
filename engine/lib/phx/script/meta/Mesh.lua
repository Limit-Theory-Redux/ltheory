-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Mesh
Mesh = {}

---@return Mesh
function Mesh.Create() end

---@return Mesh
function Mesh:clone() end

---@param name string
---@return Mesh
function Mesh.Load(name) end

---@return Bytes
function Mesh:toBytes() end

---@param buf Bytes
---@return Mesh
function Mesh.FromBytes(buf) end

---@param bytes string
---@return Mesh
function Mesh.FromObj(bytes) end

---@param res integer
---@return Mesh
function Mesh.Box(res) end

---@param res integer
---@return Mesh
function Mesh.BoxSphere(res) end

---@param origin Vec3f
---@param du Vec3f
---@param dv Vec3f
---@param resU integer
---@param resV integer
---@return Mesh
function Mesh.Plane(origin, du, dv, resU, resV) end

---@param newIndex integer
function Mesh:addIndex(newIndex) end

---@param other Mesh
function Mesh:addMesh(other) end

---@param i1 integer
---@param i2 integer
---@param i3 integer
---@param i4 integer
function Mesh:addQuad(i1, i2, i3, i4) end

---@param i1 integer
---@param i2 integer
---@param i3 integer
function Mesh:addTri(i1, i2, i3) end

---@param px number
---@param py number
---@param pz number
---@param nx number
---@param ny number
---@param nz number
---@param u number
---@param v number
function Mesh:addVertex(px, py, pz, nx, ny, nz, u, v) end

---@param vertex Vertex
function Mesh:addVertexRaw(vertex) end

function Mesh:drawBind() end

function Mesh:drawBound() end

function Mesh:drawUnbind() end

function Mesh:draw() end

---@param scale number
function Mesh:drawNormals(scale) end

---@param out Box3f
function Mesh:getBound(out) end

---@param out Vec3f
function Mesh:getCenter(out) end

---@return integer
function Mesh:getIndexCount() end

---@param f fun(arg1: integer[]): nil
function Mesh:lockIndexData(f) end

---@return number
function Mesh:getRadius() end

---@return integer
function Mesh:getVersion() end

function Mesh:incVersion() end

---@return integer
function Mesh:validate() end

---@param index integer
---@return Vertex
function Mesh:getVertex(index) end

---@return integer
function Mesh:getVertexCount() end

---@param f fun(arg1: Vertex[]): nil
function Mesh:lockVertexData(f) end

---@param capacity integer
function Mesh:reserveIndexData(capacity) end

---@param capacity integer
function Mesh:reserveVertexData(capacity) end

---@return Mesh
function Mesh:center() end

---@return Mesh
function Mesh:invert() end

---@param rads number
---@return Mesh
function Mesh:rotateX(rads) end

---@param rads number
---@return Mesh
function Mesh:rotateY(rads) end

---@param rads number
---@return Mesh
function Mesh:rotateZ(rads) end

---@param yaw number
---@param pitch number
---@param roll number
---@return Mesh
function Mesh:rotateYPR(yaw, pitch, roll) end

---@param x number
---@param y number
---@param z number
---@return Mesh
function Mesh:scale(x, y, z) end

---@param s number
---@return Mesh
function Mesh:scaleUniform(s) end

---@param x number
---@param y number
---@param z number
---@return Mesh
function Mesh:translate(x, y, z) end

---@param matrix Matrix
function Mesh:transform(matrix) end

function Mesh:computeNormals() end

---@param minDot number
function Mesh:splitNormals(minDot) end

---@param radius number
function Mesh:computeAO(radius) end

---@param sdf Tex3D
---@param radius number
function Mesh:computeOcclusion(sdf, radius) end

