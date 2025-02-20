local Nebula = class(function(self, seed, starDir)
    self.seed = seed
    self.starDir = starDir
end)

function Nebula:forceLoad()
    if self.envMap then return end
    local rng = RNG.Create(self.seed + 0xC0104FULL)
    self.envMap = Gen.Generator.Get('Nebula', rng)(rng, Config.gen.nebulaRes, self.starDir)
    self.irMap = self.envMap:genIRMap(256)
    self.stars = Gen.Starfield(rng, Config.gen.nStars(rng))
end

function Nebula:render(state)
    self:forceLoad()
    if state.mode == BlendMode.Disabled then
        RenderState.PushDepthWritable(false)
        local shader = Cache.Shader('farplane', 'skybox')
        RenderState.PushCullFace(CullFace.None)
        shader:start()
        Draw.Box3(Box3f(-1, -1, -1, 1, 1, 1))
        shader:stop()
        RenderState.PopCullFace()
        RenderState.PopDepthWritable()
    elseif state.mode == BlendMode.Additive then
        local shader = Cache.Shader('farplane', 'starbg')
        shader:start()
        shader:setFloat('brightnessScale', 0.15)
        shader:setTexCube('irMap', self.irMap)
        shader:setTexCube('envMap', self.envMap)
        self.stars:draw()
        shader:stop()
    end
end

return Nebula
