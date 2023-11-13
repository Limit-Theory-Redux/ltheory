local Action = require('GameObjects.Action')

local rng = RNG.FromTime()

-- TODO : Dock range should be specified by the dockable component
local kDockRange = 2000 -- ships are getting "stuck" at 250

local DockAt = subclass(Action, function(self, target)
    self.target = target
end)

function DockAt:clone()
    return DockAt(self.target)
end

function DockAt:getName()
    local typename = Config:getObjectInfo("object_types", self.target:getType())
    return format("DockAt %s '%s'", typename, self.target:getName())
end

local function getTargetPos(e, target)
    local tp = target:getPos()
    local tr = target:getRadius()
    local tu = target:getUp()
    local er = e:getRadius()
    return tp - tu:muls(1.25 * tr + er)
end

-- TODO CHANGE TO USE MOVETO OR IMPLEMENT TRAVEL DRIVE

function DockAt:onUpdateActive(e, dt)
    -- Move to within docking range of the dockable target object
    local tp = getTargetPos(e, self.target)

    -- Within range of the target object?
    if (e:getPos() - tp):length() <= kDockRange then
        if self.target:hasDockable() and self.target:isDockable() and not self.target:isBanned(e) then
            e:getThrustController():clear()
            self.target:addDocked(e)
        end
        e:popAction()
    else
        -- Use the "target" metaphor to store where this ship is moving to
        if self.target:hasDockable() and self.target:isDockable() and not self.target:isBanned(e) then
            e:setTarget(self.target)

            if GameState.debug.instantJobs then
                local p = e:getPos()
                local dp = tp - p
                e:setPos(p + dp:normalize():scale(rng:getUniform() * min(dp:length(), dt * GameState.debug.jobSpeed)))
            else
                local tf = self.target:getForward()
                local tu = self.target:getUp()
                self:flyToward(e, tp, -tf, tu)
            end
        else
            -- Station is no longer available for docking, so stop this DockAt action
            e:popAction()
        end
    end
end

-- TODO : Update this when we have real dock positions

return DockAt
