local Entity = require('GameObjects.Entity')

local rng = RNG.Create(50123)

local Explosion = subclass(Entity, function(self, pos, age)
    self.age = 0
    self.pos = pos
    self.seed = rng:getUniform()

    self:register(Event.Render, self.render)
    self:register(Event.Update, self.update)
end)

local cache
local mesh
local rng
local shader

Preload.Add(function()
    mesh = Gen.Primitive.Billboard(-1, -1, 1, 1)
    rng = RNG.FromTime()
    shader = Cache.Shader('billboard/quadpos', 'effect/explosion')
    cache = ShaderVarCache(shader, { 'color', 'origin', 'up', 'age', 'seed', 'size' })
end)

function Explosion:render(state)
    if state.mode == BlendMode.Additive then
        if self.age >= 0 then
            local up = Systems.Camera.Camera.get().rot:getUp()
            shader:start()
            shader:iSetFloat3(cache.color, 2.0, 1.5, 1.0)
            shader:iSetFloat3(cache.up, up.x, up.y, up.z)
            shader:iSetFloat(cache.size, Config.game.explosionSize)
            shader:iSetFloat(cache.age, self.age)
            shader:iSetFloat(cache.seed, self.seed)
            shader:iSetFloat3(cache.origin, self.pos.x, self.pos.y, self.pos.z)
            mesh:draw()
            shader:stop()
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
