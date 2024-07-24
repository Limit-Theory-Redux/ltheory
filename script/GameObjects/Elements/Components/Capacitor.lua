--[[
    TODO:   Seems like there's a unification opportunity here as well. Certainly
            integrity and capacitor should be unified: all the 'energies' of an
            entity should be stored in a kind of 'energy inventory' -- to be used
            by shields, armor, hull, capacitor, and anything else that uses a
            similar 'virtual currency'
    NOTE:   The above "TODO" came from Josh -- we may want to rethink it. It makes sense
            that things using energy -- energy weapons, shield generators -- should
            take that energy from a capacitor. But physical objects, such as hull and
            armor, don't need to take energy and should have their own "health" values.
--]]
local Entity      = require('GameObjects.Entity')
local BasicShapes = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType  = require('GameObjects.Entities.Ship.SocketType')
local Bindings    = require('States.ApplicationBindings')

local shared
local rng         = RNG.FromTime()

local Capacitor   = subclass(Entity, function(self)
    -- All of this crap is completely worthless, but updateCapacitor() will not be called without it
    if not shared then
        shared = {}
        shared.mesh = BasicShapes.Prism(2, 3):finalize()
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
    end
    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, Material.Debug())

    -- OK, back now to what Capacitor actually requires
    self.name       = Config.gen.compCapacitorStats.name
    self.healthCurr = Config.gen.compCapacitorStats.healthCurr
    self.healthMax  = Config.gen.compCapacitorStats.healthMax
    self.chargeCurr = Config.gen.compCapacitorStats.chargeCurr
    self.chargeMax  = Config.gen.compCapacitorStats.chargeMax
    self.chargeRate = Config.gen.compCapacitorStats.chargeRate
    --Log.Debug("Register: Capacitor name = '%s', type = %s, handler = %s", self.name, Event.Update, self.updateCapacitor)
    self:register(Event.Update, self.updateCapacitor)
end)

function Capacitor:getSocketType()
    return SocketType.Capacitor
end

function Capacitor:getName()
    return self.name
end

function Capacitor:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --Log.Debug("Vessel %s capacitor takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)

    -- Reduce maximum possible charge due to damage
    -- We could also reduce the charge rate, but let's be nice for now
    local maxCharge = self.chargeMax * (self.health / self.healthMax)
    if self.chargeCurr > maxCharge then self.chargeCurr = maxCharge end
    Capacitor:setCharge(self.chargeCurr, maxCharge, self.chargeRate)
end

function Capacitor:discharge(value)
    local undischargedAmount = 0
    local chargeRemaining = self.chargeCurr
    local newCharge = chargeRemaining - value

    if newCharge < 0 then
        undischargedAmount = 0 - newCharge
        newCharge = 0.0
    else
        undischargedAmount = 0
    end

    self.chargeCurr = newCharge
    --Log.Debug("Entity %s discharges %s, %s charge remaining, %s undischarged",
    --self:getName(), value, self.chargeCurr, undischargedAmount)
    return undischargedAmount
end

function Capacitor:getHealth()
    return self.healthCurr or 0.0
end

function Capacitor:getHealthMax()
    return self.healthMax or 0.0
end

function Capacitor:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function Capacitor:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = floor(max)
end

function Capacitor:getCharge()
    return self.chargeCurr or 0
end

function Capacitor:getChargeMax()
    return self.chargeMax or 0
end

function Capacitor:getChargeRate()
    return self.chargeRate or 0
end

function Capacitor:getChargePercent()
    return 100.0 * self.chargeCurr / self.chargeMax
end

function Capacitor:setCharge(value, max, rate)
    self.chargeCurr = value
    self.chargeMax = max
    self.chargeRate = rate
end

function Capacitor:setName(newName)
    self.name = newName
end

function Capacitor:updateCapacitor(state)
    if not self:getParent():isDestroyed() then
        if self.chargeCurr < self.chargeMax then
            local timeScale = 1.0
            if GameState.paused then
                timeScale = 0.0
            end
            if Input:isDown(Bindings.TimeAccel) then
                timeScale = GameState.debug.timeAccelFactor
            end

            self.chargeCurr = min(self.chargeMax, self.chargeCurr + (timeScale * state.dt) * self.chargeRate)
            --Log.Debug("CAPACITOR: %s - curr = %s, max = %s, rate = %s", self:getName(), self.chargeCurr, self.chargeMax, self.chargeRate)
        end
    end
end

return Capacitor
