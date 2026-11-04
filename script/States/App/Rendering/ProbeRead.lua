--- Pixel probes of the validation scenes (render API v2, S8). A probe reads
--- a texel back with `Renderer:readSync`, which stalls until the GPU is done:
--- fine for a test that looks at a handful of pixels once, never for a frame.
local ProbeRead = {}

--- RGB of one texel of `tex` (any 2D render target), each in [0, 1], through
--- an RGBA8 read (the conversion rounds as a draw into an RGBA8 target would).
--- `x`, `y` are clamped to the texture; `y` counts from the top of the image
--- as drawn, so `y = 0` is the last texel row.
---@param tex Tex2D
---@param x integer
---@param y integer
---@return Vec3f
function ProbeRead.sample(tex, x, y)
    local size = tex:getSize()
    x = math.max(0, math.min(size.x - 1, x))
    y = math.max(0, math.min(size.y - 1, y))
    local bytes = Renderer:readSync(tex:view(), x, size.y - 1 - y, 1, 1, TexFormat.RGBA8)
    return Vec3f(bytes:readU8() / 255, bytes:readU8() / 255, bytes:readU8() / 255)
end

return ProbeRead
