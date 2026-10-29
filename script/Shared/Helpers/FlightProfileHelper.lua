---Applies a flight profile to a rigid body.
---@class FlightProfileHelper
local FlightProfileHelper = {}

---Set the body's mass, drag and soft-CCD distance from the profile. The body is frictionless
---and never sleeps, because thrust is the only thing that moves a ship.
---@param profile FlightProfile
---@param rb RigidBody
function FlightProfileHelper.applyToRigidBody(profile, rb)
    rb:setMass(profile.mass)
    rb:setDrag(profile.linearDrag, profile.angularDrag)
    rb:setFriction(0)
    rb:setSleepThreshold(0, 0)
    rb:setSoftCcdPrediction(Config.game.physics.softCcdDistance(profile.maxSpeed))
end

return FlightProfileHelper
