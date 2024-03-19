--[[-- Controls ----------------------------------------------------------------

  There are 2 asteroids you can control. Use Ctrl + <button> to control the
  first and Shift + <button> to control the second.

  Return      : Attach to/detach from your ship
  -/=         : Change asteroid's scale
  I/J/K/L/U/O : Change asteroid's position
  T/F/G/H/R/Y : Change asteroid's rotation

  TODO
  In the distance 2 collisions will occur soon after running: asteroid vs
  asteroid, ship vs ship and asteroid vs ship.

  A stationary asteroid and ship are placed in front of the ship to run into.

  A stationary ship is placed in front of the ship to test compound bounding
  boxes.

  A massive stationary asteroid is placed behind the ship to test small vs large
  collisions.

  A trigger box is positioned at the origin and content count is printed.
----------------------------------------------------------------------------]]
--

local compoundTest        = true
local collisionTest       = true
local boundingTest        = true
local scaleTest           = true
local worldTriggerTest    = true
local attachedTriggerTest = true
local rayCastTest         = true
local printCounts         = false

local print_              = print
local print               = function(...) if printCounts then print_(...) end end

-- This requireAll seems to be against normal conventions.
-- TODO: Potentially instead of requireAll Entities Only Require each entity needed. Otherwise we might need a way to require specific entities into a file in a more seemless way.
-- Could theoretically have a function does like.
-- local Entites = RequireEach('GameObjects.Entites', [(System, Test.System), (Asteroid, Objects.Asteroid)]). Then we can call them by Entities.Objects.Asteroid and Entities.System
-- Might be Faulty logic but should be investigated.
local Entities            = requireAll('GameObjects.Entities')
local DebugControl        = require('Systems.Controls.Controls.DebugControl')
local MasterControl       = require('Systems.Controls.Controls.MasterControl')
local GameView            = require('Systems.Overlay.GameView')

local LTheory             = require('States.Application')
local rng                 = RNG.FromTime()

function LTheory:generate()
    if Config.gen.seedGlobal then
        rng:free()
        rng = RNG.Create(Config.gen.seedGlobal)
    end
    self.seed = rng:get64()
    Log.Debug('Seed: %s', self.seed)

    if self.system then self.system:delete() end
    self.system = Entities.Test.System(self.seed)
    GameState.world.currentSystem = self.system
    GameState.gen.uniqueShips = true
    GameState:SetState(Enums.GameStates.InGame)

    local ship
    do -- Player Ship
        ship = self.system:spawnShip(Enums.ShipHulls.Solo, self.player)
        ship:setPos(Config.gen.origin)
        ship:setFriction(0)
        ship:setSleepThreshold(0, 0)
        ship:setOwner(self.player, true)
        self.system:addChild(ship)
        self.player:setControlling(ship)

        if compoundTest then
            self.asteroid1 = Entities.Objects.Asteroid(1234, 5)
            self.asteroid1:setPos(Vec3f(-10, 0, 10))
            self.system:addChild(self.asteroid1)
            self.asteroid1.pos = Vec3f(1, 0, 0)

            self.asteroid2 = Entities.Objects.Asteroid(1234, 5)
            self.asteroid2:setPos(Vec3f(10, 0, 10))
            self.system:addChild(self.asteroid2)
            self.asteroid2.pos = Vec3f(-1, 0, 0)
        end

        if collisionTest then
            local asteroid = Entities.Objects.Asteroid(1234, 20)
            asteroid:setPos(Vec3f(20, 0, -100))
            self.system:addChild(asteroid)
            local ship = self.system:spawnShip(Enums.ShipHulls.Solo, nil)
            ship:setPos(Vec3f(-20, 0, -100))

            local ship = self.system:spawnShip(Enums.ShipHulls.Solo, nil)
            local mat = Matrix.YawPitchRoll(0, 0, math.pi / 4)
            local rot = mat:toQuat()
            mat:free()
            ship:setPos(Vec3f(0, 40, -100))
            ship:setRot(rot)
            if boundingTest then
                ship:attach(Entities.Objects.Asteroid(1234, 5), Vec3f(10, 0, 0), Quat.Identity())
                ship:attach(Entities.Objects.Asteroid(1234, 5), Vec3f(-10, 0, 0), Quat.Identity())
            end
        end

        if scaleTest then
            local asteroid = Entities.Objects.Asteroid(1234, 10000)
            asteroid:setPos(Vec3f(0, 0, 10500))
            self.system:addChild(asteroid)
        end
    end

    if worldTriggerTest then
        self.trigger1 = Entities.Trigger(Vec3f(20, 20, 20))
        self.trigger1:triggerSetPos(Vec3f(9, 0, 0))
        self.system:addChild(self.trigger1)
    end

    if attachedTriggerTest then
        self.trigger2 = Entities.Trigger(Vec3f(20, 20, 20))
        self.system:addChild(self.trigger2)
        self.trigger2:triggerAttach(self.player:getControlling().body, Vec3f())
    end
end

function LTheory:onInit()
    Systems.SFX.SoundManager:init()

    self.player                                    = Entities.Player()
    GameState.player.humanPlayer                   = self.player

    GameState.debug.physics.drawBoundingBoxesLocal = false
    GameState.debug.physics.drawBoundingBoxesWorld = false
    GameState.debug.physics.drawWireframes         = true

    self:generate()

    DebugControl.ltheory = self
    self.gameView = GameView(self.player, self.audio)
    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(MasterControl(self.gameView, self.player)))
end

function LTheory:onInput()
    self.canvas:input()

    if compoundTest then
        local asteroids = List()
        if InputInstance:isDown(Button.KeyboardControlLeft) then asteroids:append(self.asteroid1) end
        if InputInstance:isDown(Button.KeyboardShiftLeft) then asteroids:append(self.asteroid2) end

        local ship = self.player:getControlling()
        for i = 1, #asteroids do
            local asteroid = asteroids[i]

            -- Attach/detach
            if InputInstance:isPressed(Button.KeyboardReturn) then
                local parent = asteroid:getParentBody()
                if parent == nil then
                    self.system:removeChild(asteroid)
                    ship:attach(asteroid, asteroid.pos:muls(5), Quat.Identity())
                else
                    ship:detach(asteroid)
                    self.system:addChild(asteroid)
                end
            end

            -- Scale
            if InputInstance:isPressed(Button.KeyboardMinus) then
                local scale = asteroid:getScale()
                if scale > 1 then asteroid:setScale(scale - 1) end
            end
            if InputInstance:isPressed(Button.KeyboardEquals) then
                local scale = asteroid:getScale()
                asteroid:setScale(scale + 1)
            end

            -- Position
            local pos = Vec3f(0, 0, 0)
            if InputInstance:isPressed(Button.KeyboardI) then pos.z = pos.z - 1 end
            if InputInstance:isPressed(Button.KeyboardK) then pos.z = pos.z + 1 end
            if InputInstance:isPressed(Button.KeyboardL) then pos.x = pos.x + 1 end
            if InputInstance:isPressed(Button.KeyboardJ) then pos.x = pos.x - 1 end
            if InputInstance:isPressed(Button.KeyboardO) then pos.y = pos.y + 1 end
            if InputInstance:isPressed(Button.KeyboardU) then pos.y = pos.y - 1 end
            local parent = asteroid:getParentBody()
            if parent == nil then
                asteroid:setPos(pos + asteroid:getPos());
            else
                asteroid:setPosLocal(pos + asteroid:getPosLocal());
            end

            local ypr = Vec3f(0, 0, 0)
            if InputInstance:isPressed(Button.KeyboardT) then ypr.y = ypr.y - math.pi / 10 end
            if InputInstance:isPressed(Button.KeyboardG) then ypr.y = ypr.y + math.pi / 10 end
            if InputInstance:isPressed(Button.KeyboardH) then ypr.z = ypr.z - math.pi / 10 end
            if InputInstance:isPressed(Button.KeyboardF) then ypr.z = ypr.z + math.pi / 10 end
            if InputInstance:isPressed(Button.KeyboardY) then ypr.x = ypr.x - math.pi / 10 end
            if InputInstance:isPressed(Button.KeyboardR) then ypr.x = ypr.x + math.pi / 10 end
            local mat = Matrix.YawPitchRoll(ypr.y, ypr.x, ypr.z)
            local rot = mat:toQuat()
            mat:free()
            local parent = asteroid:getParentBody()
            if parent == nil then
                asteroid:setRot(rot * asteroid:getRot());
            else
                asteroid:setRotLocal(rot * asteroid:getRotLocal());
            end
        end
    end
end

function LTheory:onUpdate(dt)
    self.player:getRoot():update(dt)
    self.canvas:update(dt)

    local collision = Collision()
    local collisions = {}
    while (self.system.physics:getNextCollision(collision)) do
        table.insert(collisions,
            string.format('Collision %d between %s and %s', collision.index, tostring(collision.body0),
                tostring(collision.body1)))
    end

    if rayCastTest then
        local camera = self.gameView.camera
        local ray = camera:mouseToRay(50000)

        local rayCastResult = self.system.physics:rayCast(ray)
        local hit = rayCastResult.body
        local entity = Entity.fromRigidBody(hit)

        if hit and entity and entity:getName() then
            print_(ray, entity:getName())
        end
    end

    Gui:beginGui(self.resX, self.resY, InputInstance)
    Gui:beginVerticalContainer()

    Gui:textEx(Cache.Font('Iceland', 32), string.format('Collision Count: %d', collision.count),
        Color(1.0, 1.0, 1.0, 1.0))

    if worldTriggerTest then
        local triggerCount = self.trigger1:getContentsCount()
        Gui:textEx(Cache.Font('Iceland', 32), string.format('World Trigger Count: %d', triggerCount),
            Color(1.0, 1.0, 1.0, 1.0))
        for i = 1, triggerCount do
            self.trigger1:getContents(i - 1)
        end
    end

    if attachedTriggerTest then
        local triggerCount = self.trigger2:getContentsCount()
        Gui:textEx(Cache.Font('Iceland', 32), string.format('Attached Trigger Count: %d', triggerCount),
            Color(1.0, 1.0, 1.0, 1.0))
        for i = 1, triggerCount do
            self.trigger2:getContents(i - 1)
        end
    end

    for k, v in ipairs(collisions) do
        Gui:textEx(Cache.Font('Iceland', 32), v, Color(1.0, 1.0, 1.0, 1.0))
    end

    Gui:endContainer()
    Gui:endGui(InputInstance)
end

function LTheory:onDraw()
    self.canvas:draw(self.resX, self.resY)
    Gui:draw()
end

return LTheory
