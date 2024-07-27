local Cache = require('Render.Cache')
local ImageFilter = {}

function ImageFilter.IAdd(dst, src1, src2, mult1, mult2)
    local shader = Cache.Shader('identity', 'filter/add')
    dst:push()
    shader:start()
    shader:setFloat('mult1', mult1)
    shader:setFloat('mult2', mult2)
    shader:setTex2D('src1', src1)
    shader:setTex2D('src2', src2)
    Draw.Color(1, 1, 1, 1)
    Draw.Rect(-1, -1, 2, 2)
    shader:stop()
    dst:pop()
end

function ImageFilter.IComposite(dst, src1, src2)
    local shader = Cache.Shader('identity', 'ui/composite')
    dst:push()
    shader:start()
    shader:setTex2D('srcBottom', src1)
    shader:setTex2D('srcTop', src2)
    Draw.Color(1, 1, 1, 1)
    Draw.Rect(-1, -1, 2, 2)
    shader:stop()
    dst:pop()
end

function ImageFilter.IGlitch(dst, src, strength)
    local shader = Cache.Shader('identity', 'filter/glitch')
    local size = src:getSize()
    dst:push()
    shader:start()
    shader:setFloat('strength', strength)
    shader:setFloat('scroll', 0.0)
    shader:setFloat2('size', size.x, size.y)
    shader:setTex2D('src', src)
    Draw.Color(1, 1, 1, 1)
    Draw.Rect(-1, -1, 2, 2)
    shader:stop()
    dst:pop()
end

function ImageFilter.ITonemap(dst, src)
    local shader = Cache.Shader('identity', 'filter/tonemap')
    dst:push()
    shader:start()
    shader:setTex2D('src', src)
    Draw.Color(1, 1, 1, 1)
    Draw.Rect(-1, -1, 2, 2)
    shader:stop()
    dst:pop()
end

function ImageFilter.IVignette(dst, src, strength, hardness)
    local shader = Cache.Shader('identity', 'filter/vignette')
    local size = src:getSize()
    dst:push()
    shader:start()
    shader:setFloat('strength', strength)
    shader:setFloat('hardness', hardness)
    shader:setTex2D('src', src)
    Draw.Color(1, 1, 1, 1)
    Draw.Rect(-1, -1, 2, 2)
    shader:stop()
    dst:pop()
end

function ImageFilter.Blur(src, dx, dy, radius)
    local size = src:getSize()
    local self = Tex2D.Create(size.x, size.y, src:getFormat())
    ImageFilter.IBlur(self, src, dx, dy, radius)
    return self
end

function ImageFilter.IBlur(dst, src, dx, dy, radius)
    local shader = Cache.Shader('identity', 'filter/blur')
    local size = dst:getSize()
    dst:push()
    shader:start()
    shader:setFloat('variance', 0.2 * radius)
    shader:setFloat2('dir', dx, dy)
    shader:setFloat2('size', size.x, size.y)
    shader:setInt('radius', radius)
    shader:setTex2D('src', src)
    Draw.Color(1, 1, 1, 1)
    Draw.Rect(-1, -1, 2, 2)
    shader:stop()
    dst:pop()
end

function ImageFilter.ILerp(dst, src1, src2, t)
    local shader = Cache.Shader('identity', 'filter/add')
    dst:push()
    shader:start()
    shader:setFloat('mult1', 1 - t)
    shader:setFloat('mult2', t)
    shader:setTex2D('src1', src1)
    shader:setTex2D('src2', src2)
    Draw.Color(1, 1, 1, 1)
    Draw.Rect(-1, -1, 2, 2)
    shader:stop()
    dst:pop()
end

function ImageFilter.Lerp(src1, src2, t)
    local size = src1:getSize()
    local self = Tex2D.Create(size.x, size.y, src1:getFormat())
    ImageFilter.ILerp(self, src1, src2, t)
    return self
end

return ImageFilter
