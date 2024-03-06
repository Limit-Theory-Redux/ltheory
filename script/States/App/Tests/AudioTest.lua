-- DOES NOT WORK USES DEPRICATED WAY OF GETTING KEY
local Bindings = require('States.ApplicationBindings')

local AudioTest = require('States.Application')
local SoundManager = require('Systems.SFX.SoundManager')

local Music = {
    MainTheme = './res/sound/system/audio/music/LTR_Surpassing_The_Limit_Redux_Ambient_Long_Fade.ogg',
    AltTheme  = './res/sound/system/audio/music/LTR_Parallax_Universe.ogg'
}

local kMoveSpeed = 100.0

function AudioTest:getTitle()
    return 'Audio Test'
end

function AudioTest:onInit()
    self.emitters = {
        -- { file = 'cantina', image = 'image/cantinaband', x = 128, y = 100 },
        -- { file = 'Imperial_March', image = 'image/vader', x = 256, y = 600 },
    }

    self.ambiances = {
        --[[     '900years',
    'breath',
    'chewy',
    'chosenone',
    'comeback',
    'dont',
    'jabbalaugh',
    'jedi',
    'nerfherder',
    'thybidding',
    'traintheboy',
    'yesmaster',
    'yodalaugh', ]]
    }

    self.rng = RNG.FromTime()

    -- Sound.Load('cantina', true, true)

    SoundManager:init()

    self.musicToggle = 0
    self.music = {}
    self.music[0] = Sound.Load(Music.MainTheme, true)
    self.music[1] = Sound.Load(Music.AltTheme, true)

    self.currentlyPlaying = self.audio:play(self.music[self.musicToggle], 1.0, 5000)

    for i = 1, #self.emitters do
        local e = self.emitters[i]
        e.tex = Tex2D.Load(e.image)
        e.tex:genMipmap()
        e.tex:setMagFilter(TexFilter.Linear)
        e.tex:setMinFilter(TexFilter.LinearMipLinear)

        e.sound = Sound.Load(e.file, true)
        e.sound:set3DPos(Vec3f(e.x, 0, e.y), Vec3f(0, 0, 0))
        --e.sound:setPlayPos(e.sound:getDuration() - 10*i)
        self.audio:play(e.sound)
    end

    self.lastFireTime = TimeStamp.Now()
    self.pos = Vec3f(self.resX / 2, 0, self.resY / 2)
    self.vel = Vec3f(0, 0, 0)
    self.ambianceTimer = 1.0 + self.rng:getExp()
    self.particles = {}

    -- self.onKeyDown = {
    -- [Key.S] = function () self.vel.z = self.vel.z + kMoveSpeed * self.dt end,
    -- [Key.W] = function () self.vel.z = self.vel.z - kMoveSpeed * self.dt end,
    -- [Key.D] = function () self.vel.x = self.vel.x + kMoveSpeed * self.dt end,
    -- [Key.A] = function () self.vel.x = self.vel.x - kMoveSpeed * self.dt end,
    -- }

    -- self.onKeyPress = {
    -- [Key.N1]    = function () Audio.Prepare(Audio.Load(SFX.Gun, true), true, false):play() end,
    -- [Key.N2]    = function () Audio.Prepare(Audio.Load(SFX.Hero, true), false, false):play() end,
    -- [Key.Left]  = function () self.pos = Vec3f( 10,  0,   0) end,
    -- [Key.Right] = function () self.pos = Vec3f(-10,  0,   0) end,
    -- [Key.Up]    = function () self.pos = Vec3f(  0,  0, -10) end,
    -- [Key.Down]  = function () self.pos = Vec3f(  0,  0,  10) end,
    -- [Key.Space] = function () self.pos = Vec3f(  0,  2,   0) end,
    -- }
end

function AudioTest:onInput()
    if InputInstance:isPressed(Bindings.Exit) then
        self:quit()
    end

    if InputInstance:isPressed(Button.MouseLeft) then
        -- Fade out currently playing music
        self.currentlyPlaying:stop(500)
        -- Fade in alternate music
        self.musicToggle = (self.musicToggle + 1) % 2

        self.currentlyPlaying = self.audio:play(self.music[self.musicToggle], 1.0, 5000)
    end

    if InputInstance:isDown(Button.MouseLeft) then
        --if self.lastFireTime:getElapsed() > 0.12 then
        --    self.lastFireTime = self.lastUpdate
        --    local sound = Sound.Load(SFX.Gun, false, true)
        --    sound:setFreeOnFinish(true)
        --    sound:set3DPos(Vec3f(0, 0, 0), Vec3f(0, 0, 0))
        --    sound:setVolume(Math.Lerp(0.2, 0.6, self.rng:getUniform() ^ 2.0))
        --    sound:play()
        --end
    end

    if InputInstance:isDown(Button.MouseRight) then
        local is = InputInstance:mouse():position()
        self.pos.x = is.x
        self.pos.z = is.y

        if self.lastFireTime:getElapsed() > 0.12 then
            self.lastFireTime = TimeStamp.Now()

            if Config.audio.sounds.pulseFire then
                local instance = Config.audio.sounds.pulseFire:Play(self.pos, 1.0)
                instance:setEmitterPos(self.pos)
            end
        end
    end

    -- for k, v in pairs(self.onKeyDown) do
    -- if InputInstance:isDown(k) then v() end
    -- end

    -- for k, v in pairs(self.onKeyPress) do
    -- if InputInstance:isPressed(k) then v() end
    -- end
end

function AudioTest:onDraw()
    BlendMode.PushAlpha()
    Draw.Clear(0.1, 0.1, 0.1, 1.0)
    for i = 1, #self.emitters do
        Draw.Color(1, 1, 1, 1)
        local e = self.emitters[i]
        local sz = e.tex:getSize()
        e.tex:draw(e.x - 96, e.y - 96, 192, 192)
        local d = Vec3f(e.x, 0, e.y):distance(self.pos)
        local c = Vec3f():lerp(Vec3f(1.0, 0.0, 0.2), exp(-max(0, d / 128 - 1.0)))
        Draw.Color(c.x, c.y, c.z, 1)
        Draw.Border(8, e.x - 96, e.y - 96, 192, 192)
    end

    Draw.PointSize(2.0)
    Draw.SmoothPoints(true)
    for i = 1, #self.particles do
        local p = self.particles[i]
        local alpha = p.life / 5
        Draw.Color(0.25, 1.0, 0.25, alpha * 0.8)
        Draw.Point(p.x, p.y)
    end

    Draw.Color(0.1, 0.6, 1.0, 1.0)
    Draw.Rect(self.pos.x - 4, self.pos.z - 4, 8, 8)
    BlendMode.Pop()
end

function AudioTest:onUpdate(dt)
    self.pos = self.pos + self.vel:scale(dt)
    self.vel:iscale(exp(-dt))

    SoundManager:clean(dt)

    do -- Play 'ambient' sound effects in a cloud around the listener
        -- WARNING : May cause extreme annoyance, nightmares, and/or euphoria.
        -- Josh hereby absolves himself of all responsibility.
        self.ambianceTimer = self.ambianceTimer - dt
        if self.ambianceTimer <= 0 then
            self.ambianceTimer = self.ambianceTimer + 0.25 * self.rng:getExp()
            -- local sound = Sound.Load(self.rng:choose(self.ambiances), false, true)
            local dp = self.rng:getDir2():scale(100.0 * (1.0 + self.rng:getExp()))
            -- sound:setFreeOnFinish(true)
            -- sound:setPitch(Math.Clamp(1.0 + 0.1 * self.rng:getGaussian(), 0.6, 1.0 / 0.6))
            -- sound:set3DPos(self.pos + Vec3f(dp.x, 0, dp.y), Vec3f(0, 0, 0))
            -- sound:setVolume(0.5 + 0.5 * self.rng:getExp())
            -- sound:play()
            self.particles[#self.particles + 1] = { life = 5, x = self.pos.x + dp.x, y = self.pos.z + dp.y }
        end
    end

    do -- Particle update
        local i = 1
        while i <= #self.particles do
            local p = self.particles[i]
            p.life = p.life - dt
            if p.life < 0 then
                self.particles[i] = self.particles[#self.particles]
                self.particles[#self.particles] = nil
            else
                i = i + 1
            end
        end
    end

    self.audio:setListenerPos(Vec3f(self.resX / 2, 0, self.resY / 2), Quat(0, 0, 0, 1))

    --[[
  for i = 1, #self.emitters do
    local s = self.emitters[i].sound
    --Log.Debug("%20s\t%.2f\t%s\t%s", tostring(s:getName()), s:getDuration(), s:isPlaying(), s:isFinished())
    Log.Debug("%20s\t%.2f\t%s", tostring(s:getName()), s:getDuration(), s:isFinished())
  end
--]]
end

function AudioTest:onExit()
    --self.audio:free()
end

return AudioTest

-- TODO : Push Audio handling from LTheory up into Appliction?
-- TODO : Where is CoInitialize being called? I don't see a warning from FMOD
-- TODO : Pool of sounds with pitch variation
