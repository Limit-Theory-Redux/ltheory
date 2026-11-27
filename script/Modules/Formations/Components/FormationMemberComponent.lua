local Component = require("Core.ECS.Component")

---A ship's place in a formation: the formation's entity id, and the Follow order that keeps it on
---its slot (nil for the leader, which flies its own orders).
---@class FormationMemberComponent: Component
---@field formationId integer
---@field order Entity|nil
---@field offset Vec3d|nil The slot the order holds, in metres along the leader's axes
---@field spacingM number|nil The formation's spacing, in metres
---@overload fun(self: FormationMemberComponent, formationId: integer): FormationMemberComponent subclass internal
---@overload fun(formationId: integer): FormationMemberComponent subclass external
local FormationMemberComponent = Subclass("FormationMemberComponent", Component, function(self, formationId)
    self:setComponentName("FormationMember")

    self.formationId = formationId
    self.order = nil
    self.offset = nil
end)

---@return integer
function FormationMemberComponent:getFormation()
    return self.formationId
end

return FormationMemberComponent
