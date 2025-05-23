-- TODO : Split materials in subdir + flesh them out more as a first-class
-- asset (maybe we can even put the glsl inline in the material files?)
-- TODO : Separate the concept of 'shading model' from 'material'
-- Metal shader is a specific shading model. Metal shader + metal/01_*
-- is a specific 'type' of metal
-- NOTE : Really they're just nested ShaderStates.

local Material = Class("Material", function(self) end)

local allMaterials = {}

OnEvent('Engine.Reload', function()
    for i = 1, #allMaterials do
        allMaterials[i]:reload()
    end
end)

local function setTextureState(tex)
    tex:genMipmap()
    tex:setMagFilter(TexFilter.Linear)
    tex:setMinFilter(TexFilter.LinearMipLinear)
    tex:setAnisotropy(16)
    tex:setWrapMode(TexWrapMode.Repeat)
end

function Material.Create(name, diffuse, normal, spec)
    local self = Material()
    self.name = name
    self.texDiffuse = diffuse
    self.texNormal = normal
    self.texSpec = spec
    self.state = nil

    if diffuse then
        setTextureState(diffuse)
    end

    if normal then
        setTextureState(normal)
    end

    if spec then
        setTextureState(spec)
    end

    self:reload()
    table.insert(allMaterials, self)
    return self
end

function Material:free()
    self.texDiffuse = nil
    self.texNormal = nil
    self.texSpec = nil
    self.state = nil
    remove(allMaterials, self)
end

function Material:reload()
    local shader = Cache.Shader('wvp', self.name)
    self.state = ShaderState.Create(shader)

    if self.texDiffuse and shader:hasVariable('texDiffuse') then
        self.state:setTex2D('texDiffuse', self.texDiffuse)
    end

    if self.texNormal and shader:hasVariable('texNormal') then
        self.state:setTex2D('texNormal', self.texNormal)
    end

    if self.texSpec and shader:hasVariable('texSpec') then
        self.state:setTex2D('texSpec', self.texSpec)
    end

    self.imWorld   = shader:hasVariable('mWorld') and shader:getVariable('mWorld')
    self.imWorldIT = shader:hasVariable('mWorldIT') and shader:getVariable('mWorldIT')
    self.iScale    = shader:hasVariable('scale') and shader:getVariable('scale')
end

function Material:setState(e, eye)
    if self.imWorld then self.state:shader():iSetMatrix(self.imWorld, e:getToWorldMatrix(eye)) end
    if self.imWorldIT then self.state:shader():iSetMatrixT(self.imWorldIT, e:getToLocalMatrix(eye)) end
    if self.iScale then self.state:shader():iSetFloat(self.iScale, e:getScale()) end
end

function Material:start()
    self.state:start()
    if self.onStart then self.onStart() end
end

function Material:stop()
    self.state:stop()
end

local cache = {}

function Material.Debug()
    if not cache.debug then
        cache.debug = Material.Create('material/devmat')
    end
    return cache.debug
end

function Material.DebugColor()
    if not cache.debugColor then
        cache.debugColor = Material.Create('material/solidcolor')
    end
    return cache.debugColor
end

function Material.DebugColorA()
    if not cache.debugColorA then
        cache.debugColorA = Material.Create('material/alphacolor')
    end
    return cache.debugColorA
end

function Material.Metal()
    if not cache.metal then
        cache.metal = Material.Create(
            'material/metal',
            Cache.Texture('metal/01_d'),
            Cache.Texture('metal/01_n'),
            Cache.Texture('metal/01_s'))
    end
    return cache.metal
end

function Material.Rock()
    if not cache.rock then
        cache.rock = Material.Create(
            'material/asteroid',
            Cache.Texture('rock'))
    end
    return cache.rock
end

return Material
