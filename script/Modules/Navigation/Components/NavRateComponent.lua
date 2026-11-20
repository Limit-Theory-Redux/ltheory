local Component = require("Core.ECS.Component")

---@class NavRateComponent: Component
---@overload fun(self: NavRateComponent, class: NavRateClassId, nextJudgeAt: number): NavRateComponent subclass internal
---@overload fun(class: NavRateClassId, nextJudgeAt: number): NavRateComponent subclass external
--- On a ship that is an agent of the navigation world: its rate class, and the time on the rate
--- system's clock, in seconds, at which the class is judged again.
local NavRateComponent = Subclass("NavRateComponent", Component, function(self, class, nextJudgeAt)
    self:setComponentName("NavRate")
    self.class = class
    self.nextJudgeAt = nextJudgeAt
end)

return NavRateComponent
