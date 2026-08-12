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

---@param path string
function Mesh:save(path) end

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

---@param r Renderer
function Mesh:drawBind(r) end

-- The mesh's GPU resource id (as a plain scalar - see
-- `Renderer::add_entity`'s `mesh_id`/`shader_id` params for why this
-- isn't `ResourceId` itself), lazily creating (or recreating, if the
-- mesh changed) the executor-owned resource just like `draw_bind` does
-- - without also drawing. For code that needs to reference the mesh
-- instead of calling `draw`/`drawBind` itself (e.g. the batch API,
-- `Renderer:addEntity`).
---@param r Renderer
---@return integer
function Mesh:resourceId(r) end

---@param r Renderer
function Mesh:drawBound(r) end

-- No-op: `DrawMeshByResource` binds/draws/unbinds in one self-contained
-- command (see `draw_bound`), so there is nothing left to unbind here.
-- Kept as a method - and still takes `r` - so `drawBind`/`drawBound`/
-- `drawUnbind` stay a matched FFI triple for existing Lua call sites
-- that interleave shader uniform changes between multiple `drawBound`
-- calls (e.g. per-instance rendering without true GPU instancing).
---@param r Renderer
function Mesh:drawUnbind(r) end

---@param r Renderer
function Mesh:draw(r) end

---@param r Renderer
---@param scale number
function Mesh:drawNormals(r, scale) end

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

---@param r Renderer
---@param radius number
function Mesh:computeAO(r, radius) end

---@param r Renderer
---@param sdf Tex3D
---@param radius number
function Mesh:computeOcclusion(r, sdf, radius) end

