local Component = require("Core.ECS.Component")

---@class SensorSignatureArgs
---@field kind string a key of `SensorModel.kinds`
---@field label string
---@field radius number|nil metres; nil = the entity's bounding radius
---@field mw number|nil power draw, MW
---@field activity string|nil ships: idle (default), thrust, firing or cloak
---@field fixedEm number|nil EM given off, instead of its power draw's
---@field fixedHeat number|nil heat given off, instead of its power draw's
---@field glow number|nil light of its own (a star)
---@field name string|nil what a T3 read shows
---@field details string|nil a second T3 line
---@field contents string[]|nil what a T4 read shows
---@field shield number|nil its shields, 0 to 1, as a T3 read shows them; nil for none
---@field standing string|nil owned, friendly, neutral (default) or hostile; known once the object is classed (T2)

--- What sensors can pick up of an entity: the sensor model's three facts
--- (what it gives off, what it reflects, its size) and what a scan reads.
--- Holds a `SensorObject` without a position: each sensor sees it from its
--- own place (see SensorSystem).
---@class SensorSignatureComponent: Component
---@overload fun(args: SensorSignatureArgs): SensorSignatureComponent
local SensorSignatureComponent = Subclass("SensorSignatureComponent", Component, function(self, args)
    self:setComponentName("SensorSignature")
    self.object = {
        kind = args.kind,
        label = args.label,
        radius = args.radius,
        mw = args.mw,
        activity = args.activity or "idle",
        hot = 0,
        fixedEm = args.fixedEm,
        fixedHeat = args.fixedHeat,
        glow = args.glow,
        name = args.name,
        details = args.details,
        contents = args.contents,
        shield = args.shield,
        standing = args.standing or "neutral",
    }
end)

--- The signature as the sensor model reads it (without a position).
---@return SensorObject
function SensorSignatureComponent:getObject()
    return self.object
end

---@return string
function SensorSignatureComponent:getLabel()
    return self.object.label
end

---@return number|nil metres; nil until it is known
function SensorSignatureComponent:getRadius()
    return self.object.radius
end

---@param radius number metres
function SensorSignatureComponent:setRadius(radius)
    self.object.radius = radius
end

---@return number
function SensorSignatureComponent:getPowerDraw()
    return self.object.mw or 0
end

---@param mw number
function SensorSignatureComponent:setPowerDraw(mw)
    self.object.mw = mw
end

---@return string
function SensorSignatureComponent:getActivity()
    return self.object.activity
end

---@param activity string idle, thrust, firing or cloak
function SensorSignatureComponent:setActivity(activity)
    self.object.activity = activity
end

---@return number how hot it runs, 0 to 1
function SensorSignatureComponent:getHeat()
    return self.object.hot or 0
end

return SensorSignatureComponent
