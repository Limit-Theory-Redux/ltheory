local DrawTest = require('States.Application')

local vs = [[
#version 440 core

void main() {
  float x = float(1 - int(gl_VertexIndex)) * 0.5;
  float y = float(int(gl_VertexIndex & 1) * 2 - 1) * 0.5;
  gl_Position = vec4(x, y, 0, 1);
}
]]

local fs = [[
#version 440 core
    
layout(location = 0) out vec4 color;

void main() {
    color = vec4(0.3, 0.2, 0.1, 1.0);
}
]]

function DrawTest:onInit()
    -- self.renderer = Renderer()
    -- self.shader = Shader.Create(vs, fs)
end

function DrawTest:onUpdate(dt)
end

function DrawTest:onDraw()
    Draw.Clear(0.1, 0.2, 0.3, 1)
    -- self.renderer:start(self.resX, self.resY)
    -- self.renderer:stop()
    -- self.renderer:present(0, 0, self.resX, self.resY)
end

return DrawTest
