local Pipelines = require('Render.Pipelines')
local Entity = require('Legacy.GameObjects.Entity')

local rng = RNG.Create(50123)

local Explosion = Subclass("Explosion", Entity, function(self, pos, age)
    self.age = 0
    self.pos = pos
    self.seed = rng:getUniform()

    self:register(OldEvent.Render, self.render)
    self:register(OldEvent.Update, self.update)
end)

local mesh
local rng
local shader
local DrawBlock

Preload.Add(function()
    mesh = Gen.Primitive.Billboard(-1, -1, 1, 1)
    rng = RNG.FromTime()
    shader = Cache.Shader('billboard/quadpos_draw', 'effect/explosion_draw')
    DrawBlock = shader:blockType('DrawBlock')
end)

function Explosion:render(state)
    if state.mode == BlendMode.Additive then
        if self.age >= 0 then
            local up = Systems.Camera.Camera.get().rot:getUp()
            local pass = Renderer:currentPass()
            pass:setPipeline(Pipelines.get(shader, Pipelines.Additive))
            local d = pass:alloc(DrawBlock)
            local m = d.mWorld
            m[0], m[5], m[10], m[15] = 1, 1, 1, 1
            local user = d.drawUser
            user[0], user[1] = self.age, self.seed
            user[4], user[5], user[6], user[7] = self.pos.x, self.pos.y, self.pos.z, Config.game.explosionSize
            user[8], user[9], user[10] = up.x, up.y, up.z
            pass:drawMesh(mesh)
        end
    end
end

function Explosion:update(state)
    self.age = self.age + state.dt
    if self.age >= 10 then
        self:delete()
    end
end

return Explosion
