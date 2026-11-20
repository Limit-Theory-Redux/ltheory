---Fixture kernel: out = a * x + y over `n` elements of `double[?]` buffers.
local Axpy = {}

---@param a number
---@param x ffi.cdata*
---@param y ffi.cdata*
---@param out ffi.cdata*
---@param n integer
function Axpy.run(a, x, y, out, n)
    for i = 0, n - 1 do
        out[i] = a * x[i] + y[i]
    end
end

return Axpy
