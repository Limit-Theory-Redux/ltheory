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

---@return Shield

local Entity      = require('GameObjects.Entity')
local BasicShapes = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType  = require('GameObjects.Entities.Ship.SocketType')
local Bindings    = require('States.ApplicationBindings')

local shared
local rng         = RNG.FromTime()

local Shield      = Subclass("Shield", Entity, function(self)
    -- All of this crap is completely worthless, but updateShield() will not be called without it
    if not shared then
        shared = {}
        shared.mesh = BasicShapes.Prism(2, 3):finalize()
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
    end
    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, Material.Debug())

    -- OK, back now to what Shield actually requires
    self.name         = Config.gen.compShieldStats.name
    self.healthCurr   = Config.gen.compShieldStats.healthCurr
    self.healthMax    = Config.gen.compShieldStats.healthMax
    self.strengthCurr = Config.gen.compShieldStats.strengthCurr
    self.strengthMax  = Config.gen.compShieldStats.strengthMax
    self.reviveRate   = Config.gen.compShieldStats.reviveRate
    self.resistances  = Config.gen.compShieldStats.resistances
    self.colorR       = Config.gen.compShieldStats.colorR
    self.colorG       = Config.gen.compShieldStats.colorG
    self.colorB       = Config.gen.compShieldStats.colorB
    --Log.Debug("Register: Shield type = %s, handler = %s", OldEvent.Update, self.updateShield)
    self:register(OldEvent.Update, self.updateShield)
end)

---@return SocketType.Shield
function Shield:getSocketType()
    return SocketType.Shield
end

---@return string
function Shield:getName()
    return self.name
end

---@param newName string
---@return string
function Shield:setName(newName)
    self.name = newName
end

---@param amount number
function Shield:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --Log.Debug("Vessel %s shield takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)

    -- Reduce maximum possible shield strength due to damage
    local maxStrength = self.strengthMax * (self.healthCurr / self.healthMax)
    if self.strengthCurr > maxStrength then self.strengthCurr = maxStrength end
    Shield:setShield(self.strengthCurr, maxStrength, self.reviveRate)
end

---@return number|0.0
function Shield:getHealth()
    return self.healthCurr or 0.0
end

---@return number|0.0
function Shield:getHealthMax()
    return self.healthMax or 0.0
end

---@return number
function Shield:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

---@param value number
---@param max number
function Shield:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = floor(max)
end

---@return integer|0
function Shield:getReviveRate()
    return self.reviveRate or 0
end

---@return number|0.0
function Shield:getStrength()
    return self.strengthCurr or 0.0
end

---@return number|0.0
function Shield:getStrengthMax()
    return self.strengthMax or 0.0
end

---@return number
function Shield:getStrengthPercent()
    if self.strengthMax < 1e-6 then return 0.0 end
    return 100.0 * self.strengthCurr / self.strengthMax
end

---@param value number
function Shield:reduceStrength(value)
    -- TODO: Modify shield reduction by its resistance versus incoming damage type
    local reducedValue = value

    if self.strengthCurr - reducedValue < 1e-6 then
        self.strengthCurr = 0.0
    else
        self.strengthCurr = self.strengthCurr - reducedValue
    end
    --Log.Debug("Vessel %s shield reduced by %s, %s remaining", self:getName(), reducedValue, self.strengthCurr)
end

---@param value number
---@param max number
---@param rate number
function Shield:setStrength(value, max, rate)
    self.strengthCurr = value
    self.strengthMax = floor(max)
    self.reviveRate = rate
end

---@param state OldEvent
function Shield:updateShield(state)
    if not self:getParent():isDestroyed() then
        local oldStrength = self.strengthCurr
        if oldStrength < self.strengthMax then
            local timeScale = 1.0
            if GameState.paused then
                timeScale = 0.0
            end
            if Input:isDown(Bindings.TimeAccel) then
                timeScale = GameState.debug.timeAccelFactor
            end

            local newStrength = min(self.strengthMax, oldStrength + (timeScale * state.dt) * self.reviveRate)
            local diffStrength = newStrength - oldStrength

            -- This shield generator draws as much recharge energy from capacitor as is available
            local undischarged = self:getParent():mgrCapacitorDischarge(diffStrength)

            self.strengthCurr = newStrength - undischarged
            --Log.Debug("SHIELD: %s - curr = %s, max = %s, rate = %s, diff = %s, undischarged = %s",
            --self:getName(), self.strengthCurr, self.strengthMax, self.reviveRate, diffStrength, undischarged)
        end
    end
end

return Shield
