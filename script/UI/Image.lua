local Widget = require('UI.Widget')

local Image = {}
Image.__index = Image
setmetatable(Image, Widget)

Image.name = 'Image'
Image:setStretch(0, 0)

local default

function Image:onDraw(focus, active)
    local shader = Cache.Shader('ui', 'simple_image')
    shader:start()
    Shader.SetTex2D("image", self.tex)
    local x, y, sx, sy = self:getRectGlobal()
    Draw.Rect(x, y, sx, sy)
    shader:stop()
end

function Image:setTex(tex)
    self.tex = tex or default
    local sz = self.tex:getSize()
    self.desiredSX = sz.x
    self.desiredSY = sz.y
    return self
end

function Image.Create(tex)
    if not Image.init then
        Image.init = true
        default = Tex2D.Load('null')
    end

    local self = setmetatable({}, Image)
    self:setTex(tex)
    return self
end

return Image
