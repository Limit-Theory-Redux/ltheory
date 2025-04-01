local Entity            = require('GameObjects.Entity')
local SocketType        = require('GameObjects.Entities.Ship.SocketType')

local thrustMult        = 1
local thrustForwardMax  = 8e6 * thrustMult
local thrustBackwardMax = 2e6 * thrustMult
local thrustRightMax    = 8e6 * thrustMult
local thrustUpMax       = 2e6 * thrustMult
local thrustPitchMax    = 1e7 * thrustMult
local thrustYawMax      = 1e7 * thrustMult
local thrustRollMax     = 2e7 * thrustMult

--------------------------------------------------------------------------------

local ThrustController = Class("ThrustController", function(self)
    self:clear()
end)

function ThrustController:updateThrustMax(newThrustMult)
    thrustMult        = newThrustMult

    thrustForwardMax  = 8e6 * thrustMult / 2
    thrustBackwardMax = 2e6 * thrustMult / 2
    thrustRightMax    = 8e6 * thrustMult
    thrustUpMax       = 2e6 * thrustMult
    thrustPitchMax    = 1e7 * thrustMult
    thrustYawMax      = 1e7 * thrustMult
    thrustRollMax     = 2e7 * thrustMult
end

function ThrustController:clear()
    self.forward = 0
    self.right   = 0
    self.up      = 0
    self.yaw     = 0
    self.pitch   = 0
    self.roll    = 0
    self.boost   = 0
end

function ThrustController:setThrust(forward, right, up, yaw, pitch, roll, boost)
    self.forward = forward
    self.right   = right
    self.up      = up
    self.yaw     = yaw
    self.pitch   = pitch
    self.roll    = roll
    self.boost   = boost
end

function ThrustController:update(e, dt)
    local boost         = 0.0
    -- if self.boost > 0 and e:mgrCapacitorDischarge(dt * self.boost * Config.game.boostCost) then -- disabled for now
    boost               = self.boost
    -- end

    -- Large ships should have much less lateral/vertical movement and somewhat less maneuverability
    local mult          = 1.0 + 2.0 * boost
    local translateMult = Config.gen.shipHullTranM[e:getHull()] or 1
    local maneuverMult  = Config.gen.shipHullManuM[e:getHull()] or 1

    assert(e.countThruster)
    ThrustController:updateThrustMax(e.countThruster)

    -- TODO : Push this branching into the physics engine instead; engine
    -- should ignore impulses / torques below certain threshold
    -- BUG  : This does not respect thrustBackwardMax
    -- NOTE: applyForce() and applyTorque() seem to be resisted by a ship's radius, rather than its mass. WHY?!
    if abs(self.forward) > 1e-6 then
        e:applyForce(e:getForward():scale(self.forward * thrustForwardMax * mult))
    end

    if abs(self.right) > 1e-6 then
        e:applyForce(e:getRight():scale(self.right * thrustRightMax * translateMult))
    end

    if abs(self.up) > 1e-6 then
        e:applyForce(e:getUp():scale(self.up * thrustUpMax * translateMult))
    end

    if max(max(abs(self.pitch), abs(self.yaw)), abs(self.roll)) > 1e-6 then
        e:applyTorqueLocal(Vec3f(
            self.pitch * thrustPitchMax * maneuverMult,
            -self.yaw * thrustYawMax * maneuverMult,
            -self.roll * thrustRollMax * maneuverMult))
    end

    -- TODO : This is terrible.
    for thruster in e:iterSocketsByType(SocketType.Thruster) do
        thruster.activationT = self.forward
        thruster.boostT = boost
    end
end

--------------------------------------------------------------------------------

local function killThrust(self)
    for thruster in self:iterSocketsByType(SocketType.Thruster) do
        thruster.activationT = 0
        thruster.boostT = 0
    end
end

function Entity:addThrustController()
    assert(not self.thrustController)
    self.thrustController = ThrustController()
    self:register(OldEvent.Update, Entity.updateThrustController)
    self:register(OldEvent.Destroyed, killThrust)
end

function Entity:getThrustController()
    assert(self.thrustController)
    return self.thrustController
end

function Entity:hasThrustController()
    return self.thrustController ~= nil
end

function Entity:updateThrustController(state)
    if self:isDestroyed() then return end
    self.thrustController:update(self, state.dt)
end

--------------------------------------------------------------------------------
