local Component = require("Core.ECS.Component")

---Where a person serves now and in which role: a post (a ship, later a station) and a `CrewRoleDefs`
---id. The post's `CrewComponent` lists them too; `PeopleManager` keeps the two in step.
---@class AssignmentComponent: Component
---@field postId integer
---@field role string
---@overload fun(self: AssignmentComponent, postId: integer, role: string): AssignmentComponent subclass internal
---@overload fun(postId: integer, role: string): AssignmentComponent subclass external
local AssignmentComponent = Subclass("AssignmentComponent", Component, function(self, postId, role)
    self:setComponentName("Assignment")

    self.postId = postId
    self.role = role
end)

---@return integer
function AssignmentComponent:getPost()
    return self.postId
end

---@return string
function AssignmentComponent:getRole()
    return self.role
end

---@param role string
function AssignmentComponent:setRole(role)
    self.role = role
end

return AssignmentComponent
