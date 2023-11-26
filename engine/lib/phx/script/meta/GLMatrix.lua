---@meta

---@class GLMatrix
GLMatrix = {}

function GLMatrix.Clear() end

---@param matrix Matrix
function GLMatrix.Load(matrix) end

---@param eye Vec3d
---@param at Vec3d
---@param up Vec3d
function GLMatrix.LookAt(eye, at, up) end

function GLMatrix.ModeP() end

function GLMatrix.ModeWV() end

---@param matrix Matrix
function GLMatrix.Mult(matrix) end

---@param fovy number
---@param aspect number
---@param z0 number
---@param z1 number
function GLMatrix.Perspective(fovy, aspect, z0, z1) end

function GLMatrix.Pop() end

function GLMatrix.Push() end

function GLMatrix.PushClear() end

---@return Matrix
function GLMatrix.Get() end

---@param angle number
function GLMatrix.RotateX(angle) end

---@param angle number
function GLMatrix.RotateY(angle) end

---@param angle number
function GLMatrix.RotateZ(angle) end

---@param x number
---@param y number
---@param z number
function GLMatrix.Scale(x, y, z) end

---@param x number
---@param y number
---@param z number
function GLMatrix.Translate(x, y, z) end

