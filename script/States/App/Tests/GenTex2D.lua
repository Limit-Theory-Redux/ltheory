local GenTex2D = require('States.Application')

-- Pure-Lua helper; `Gen` is the global of the Legacy generator namespace, which this state does not load.
local MathUtil = require('Legacy.Systems.Gen.MathUtil')
local Pipelines = require('Render.Pipelines')

local kTexSize = 1024
local rng = RNG.FromTime()

local vs = Resource.LoadString(ResourceType.Shader, 'vertex/fullscreen_ndc')

-- Main generating fragment shader
local fs = [[
#include fragment
#include noise
#include math
#include bezier

/* Declare inputs. */
#group 2
layout(std140) uniform Params {
  vec2 size;
  float seed;
  float borderThreshold;
};

void main() {
  vec3 c = vec3(0.0);

  /* Compute frag color. */ {

    vec2 t = cellNoiseMH(7.0 * uv, seed);
    c = vec3(1.0, 1.0, 1.0) * exp(-32.0 * abs(t.x - t.y));
    //c = vec3(t.x, t.x, t.x);
  }

  /* Tonemap, just for the purpose of viewing. This is roughly the same
     tonemap under which an in-game texture runs. */ {
    c = 1.0 - exp(-2.3 * pow(c, 1.25 + c));
  }

  /* Output frag color. */ {
    outColor.xyz = c;
    outColor.w = 1.0;
  }
}

]]

local white = Color(1, 1, 1, 1)
local light = Color(0.8, 0.8, 0.8, 1)
local dark = Color(0.5, 0.5, 0.5, 1)

function GenTex2D:onGenerate()
    do -- Free old texture
        if self.texture ~= nil then
            self.texture:free()
            self.texture = nil
        end
    end

    do -- Generate new texture
        local tex = Tex2D.Create(kTexSize, kTexSize, TexFormat.RGBA16F)

        local desc = RenderPassDesc.Create('GenTex2D')
        desc:color(0, tex:view(), LoadOp.Clear, 0, 0, 0, 1)
        local pass = Renderer:beginPass(desc)

        self:DrawWorn(tex)

        pass:finish()

        tex:genMipmap()
        self.texture = tex
    end
end

function GenTex2D:DrawWorn(tex)
    -- blank grey texture
    Imm.Rect(0, 0, kTexSize, kTexSize, light)

    -- rect plates with line details
    local n = 10
    local w = kTexSize / n
    for i = 0, n - 1 do
        -- outer rect
        local x = i * w
        Imm.Border(5, x, 0, x + w, kTexSize, dark)
        -- inner detail lines
        local nd = rng:getInt(1, 3)
        local dist = MathUtil.GenerateNumsThatAddToSum(nd, w, rng)
        local dx = x
        for j = 0, nd - 1 do
            local y = rng:getUniformRange(0, kTexSize)
            Imm.Line(dx, 0, dx, y, dark, 2)
            dx = dx + dist[j + 1]
        end
    end
end

--- Fill the open pass with the cel shader (one fullscreen draw).
function GenTex2D:DrawCel(tex)
    local pass = Renderer:currentPass()
    pass:setPipeline(Pipelines.get(self.genShader, { vertex = VertexLayout.Fullscreen }))
    local p = pass:alloc(self.CelParams)
    p.seed = rng:getUniformRange(0, 1000.0)
    p.size.x, p.size.y = kTexSize, kTexSize
    p.borderThreshold = 0.01
    pass:drawFullscreen()
end

function GenTex2D:DrawRect1(tex)
    local kHalfTS = kTexSize * 0.5

    -- blank grey texture
    Imm.Rect(0, 0, kTexSize, kTexSize, light)

    -- buncha random dark grey boxes
    local lineWidth = 2
    local numRows = 10
    local rowHeight = kTexSize / numRows
    local numCols = 0
    local columnWidths = {}
    local x, y
    for i = 0, numRows - 1 do
        x = 0
        y = rowHeight * i
        numCols = rng:getInt(5, 20)
        columnWidths = MathUtil.GenerateNumsThatAddToSum(numCols, kTexSize, rng)
        for j = 1, numCols do
            Imm.Border(lineWidth, x, y, columnWidths[j], rowHeight, dark)
            -- vertical box subdivision
            local sub = rng:choose({ 0, 0, 0, 1, 2, 3, 4, 5 })
            local subHeight = rowHeight / sub
            for k = 0, sub - 1 do
                Imm.Line(x, y + k * subHeight, x + columnWidths[j], y + k * subHeight, dark, lineWidth)
            end
            -- increment x-pos
            x = x + columnWidths[j]
        end
    end

    -- buncha random small boxes
    --[[
  local numBoxes = rng:getInt(50, 100)
  local width, length
  for i = 0, numBoxes do
    width = rng:getUniformRange(kTexSize/100, kTexSize/50)
    length = rng:getUniformRange(kTexSize/100, kTexSize/50)
    x = rng:getUniformRange(0, kTexSize)
    y = rng:getUniformRange(0, kTexSize)
    Imm.Rect(x, y, width, length, Color(0.2, 0.2, 0.2, 1))
  end--]]
end

function GenTex2D:onInit()
    self.genShader = Shader.Create(vs, fs)
    self.CelParams = self.genShader:blockType('Params')
    self.zoom = 1
    self.zoomT = 1
    self.panX = 0
    self.panY = 0
    self:onGenerate()
end

function GenTex2D:onUpdate(dt)
    if Input:isDown(Button.KeyboardControlLeft) and Input:isPressed(Button.KeyboardW) then self:quit() end
    if Input:isPressed(Button.KeyboardSpace) then self:onGenerate() end
    if Input:isDown(Button.MouseLeft) then
        local dp = Input:mouse():delta()
        self.panX = self.panX + dp.x / self.zoom
        self.panY = self.panY + dp.y / self.zoom
    end
    self.zoomT = self.zoomT * exp(0.1 * Input:getValue(Button.MouseScrollY))
    self.zoom = Math.Lerp(self.zoom, self.zoomT, 1.0 - exp(-16.0 * dt))
end

function GenTex2D:onRender()
    local sx = self.zoom * kTexSize
    local sy = self.zoom * kTexSize
    local x = (self.resX - sx) / 2 + self.panX * self.zoom
    local y = (self.resY - sy) / 2 + self.panY * self.zoom
    local desc = RenderPassDesc.Create('GenTex2D.draw')
    desc:backbuffer(self.resX, self.resY, LoadOp.Clear, 0.1, 0.1, 0.1, 1.0)
    local pass = Renderer:beginPass(desc)
    Imm.Image(self.texture, Samplers.LinearMipClamp, x, y, sx, sy, 0, 0, 1, 1, white)
    pass:finish()
end

return GenTex2D
