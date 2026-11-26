local Component = require("Core.ECS.Component")

---@class CostProfileComponent: Component
---@field private _profile CostProfile
---@field private _revision integer
---@overload fun(self: CostProfileComponent, profile: CostProfile): CostProfileComponent subclass internal
---@overload fun(profile: CostProfile): CostProfileComponent subclass external
--- How a ship weighs the zones it could cross. The revision rises with every change of the
--- profile, so a plan made under an earlier one is known to be stale by comparing numbers.
local CostProfileComponent = Subclass("CostProfileComponent", Component, function(self, profile)
    self:setComponentName("CostProfile")
    self._revision = 0
    self._profile = nil
    self:setProfile(profile)
end)

---@param profile CostProfile
function CostProfileComponent:setProfile(profile)
    assert(rawtype(profile) == "table" and profile.weightOf, "CostProfile component: not a cost profile")
    self._profile = profile
    self._revision = self._revision + 1
end

---@return CostProfile
function CostProfileComponent:getProfile()
    return self._profile
end

---@return integer Starts at one and rises by one with each `setProfile`
function CostProfileComponent:getRevision()
    return self._revision
end

--- The profile of a ship, nil when it has none.
---@param shipEntity Entity
---@return CostProfile|nil
function CostProfileComponent.ProfileOf(shipEntity)
    local component = shipEntity:get(CostProfileComponent)
    return component and component:getProfile() or nil
end

return CostProfileComponent
