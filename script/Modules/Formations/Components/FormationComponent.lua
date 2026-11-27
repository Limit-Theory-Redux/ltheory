local Component = require("Core.ECS.Component")

---A formation: its shape (`FormationDefs`), its ships in slot order (the first leads), and the
---spacing of its slots in the bounding radii of its largest ship. `FormationSystem` keeps the
---wingmen on their slots and the leader at the wing's pace.
---@class FormationComponent: Component
---@field shape string
---@field members integer[] Ship entity ids; [1] leads
---@field spacingRadii number
---@field paceShare number The share of the slowest member's top speed the leader flies at
---@field dirty boolean Whether the slots must be given out again
---@overload fun(self: FormationComponent, shape: string, spacingRadii: number|nil, paceShare: number|nil): FormationComponent subclass internal
---@overload fun(shape: string, spacingRadii: number|nil, paceShare: number|nil): FormationComponent subclass external
local FormationComponent = Subclass("FormationComponent", Component, function(self, shape, spacingRadii, paceShare)
    self:setComponentName("Formation")

    self.shape = shape
    self.members = {}
    self.spacingRadii = spacingRadii or 6
    self.paceShare = paceShare or 0.5
    self.dirty = true
end)

---@return integer|nil leaderId
function FormationComponent:getLeader()
    return self.members[1]
end

---@return integer[]
function FormationComponent:getMembers()
    return self.members
end

---@return string
function FormationComponent:getShape()
    return self.shape
end

---@param shape string
function FormationComponent:setShape(shape)
    self.shape = shape
    self.dirty = true
end

return FormationComponent
