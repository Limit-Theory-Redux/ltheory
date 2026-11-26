local Component = require("Core.ECS.Component")

--- A ship's flight limits: the profile its flight systems read.
---@class FlightProfileComponent: Component
---@field private _profile FlightProfile
---@overload fun(profile: FlightProfile): FlightProfileComponent
local FlightProfileComponent = Subclass("FlightProfileComponent", Component, function(self, profile)
    self:setComponentName("FlightProfile")

    self._profile = profile
end)

---@return FlightProfile
function FlightProfileComponent:getProfile() return self._profile end

--- The profile of a ship that flies; a ship without one is a set-up error.
---@param shipEntity Entity
---@return FlightProfile
function FlightProfileComponent.ProfileOf(shipEntity)
    local component = shipEntity:get(FlightProfileComponent)
    assert(component, "ship has no FlightProfile: " .. tostring(shipEntity))
    return component:getProfile()
end

return FlightProfileComponent
