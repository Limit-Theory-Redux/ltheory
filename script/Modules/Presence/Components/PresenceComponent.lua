local Component = require("Core.ECS.Component")

---Whether an entity is in the scene as a physics construct (materialized) or only as data (its
---`VirtualBodyComponent`), and who makes it either: `kind` names the materializer registered with
---`PresenceSystem`. `drive` says which body is the truth while it is materialized: "virtual" (the
---virtual body moves, the construct is placed on it: a drifting wreck) or "physical" (the construct
---flies, and its state is taken back when it dematerializes).
---@class PresenceComponent: Component
---@field kind string
---@field drive "virtual"|"physical"
---@field materialized boolean
---@overload fun(self: PresenceComponent, kind: string, drive: "virtual"|"physical"|nil): PresenceComponent subclass internal
---@overload fun(kind: string, drive: "virtual"|"physical"|nil): PresenceComponent subclass external
local PresenceComponent = Subclass("PresenceComponent", Component, function(self, kind, drive)
    self:setComponentName("Presence")

    self.kind = kind
    self.drive = drive or "virtual"
    self.materialized = false
end)

---@return string
function PresenceComponent:getKind()
    return self.kind
end

---@return "virtual"|"physical"
function PresenceComponent:getDrive()
    return self.drive
end

---@return boolean
function PresenceComponent:isMaterialized()
    return self.materialized
end

---@param materialized boolean
function PresenceComponent:setMaterialized(materialized)
    self.materialized = materialized == true
end

return PresenceComponent
