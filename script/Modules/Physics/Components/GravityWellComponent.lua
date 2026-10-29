local Component = require("Core.ECS.Component")

local OPEN_SPACE = "openSpace"

---@class GravityWellComponent: Component
---@overload fun(): GravityWellComponent
--- The gravity well a ship is in, kept by `GravityWellSystem`. A ship outside every well is in open space.
local GravityWellComponent = Subclass("GravityWellComponent", Component, function(self)
    self:setComponentName("GravityWell")

    self._bodyEntity   = nil        -- Entity the well belongs to, nil in open space
    self._bodyName     = nil        -- display name of that body
    self._zoneType     = OPEN_SPACE -- "star", "planet", "moon" or OPEN_SPACE
    self._prevBodyPos  = nil        -- Position of the body on the previous update, nil on entry
end)

--- The zone type of open space.
GravityWellComponent.OPEN_SPACE = OPEN_SPACE

-- GETTERS
---@return Entity|nil
function GravityWellComponent:getBodyEntity() return self._bodyEntity end
---@return string
function GravityWellComponent:getZoneType() return self._zoneType end
---@return Position|nil
function GravityWellComponent:getPrevBodyPos() return self._prevBodyPos end

--- Name for display: the body and its zone type, or "Open Space".
---@return string
function GravityWellComponent:getZoneName()
    if not self._bodyEntity then return "Open Space" end
    return self._bodyName .. " (" .. self._zoneType .. ")"
end

-- SETTERS
---@param v Position|nil
function GravityWellComponent:setPrevBodyPos(v) self._prevBodyPos = v end

--- Put the ship in the well of `bodyEntity`.
---@param bodyEntity Entity
---@param bodyName string
---@param zoneType string
function GravityWellComponent:enter(bodyEntity, bodyName, zoneType)
    self._bodyEntity = bodyEntity
    self._bodyName = bodyName
    self._zoneType = zoneType
end

--- Put the ship in open space and forget the body it followed.
function GravityWellComponent:leave()
    self._bodyEntity = nil
    self._bodyName = nil
    self._zoneType = OPEN_SPACE
    self._prevBodyPos = nil
end

return GravityWellComponent
