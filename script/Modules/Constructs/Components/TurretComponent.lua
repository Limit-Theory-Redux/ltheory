local Component = require("Core.ECS.Component")

---@class TurretComponent: Component
---@overload fun(mountId: string, localPosition: Position, config?: table): TurretComponent
local TurretComponent = Subclass("TurretComponent", Component, function(self, mountId, localPosition, config)
    self:setComponentName("Turret")

    assert(type(mountId) == "string" and #mountId > 0)
    self.mountId = mountId
    self.localPosition = localPosition or Position()
    self.localRotation = config and config.localRotation or Quat.Identity()
    assert(type(config) == "table"
        and ((type(config.weaponId) == "number" and config.weaponId > 0)
            or (type(config.weaponRef) == "table" and config.weaponRef.canonicalKey)),
        "turret " .. mountId .. " requires an explicit weaponId or procedural weaponRef")
    self.weaponId = config.weaponId
    self.weaponRef = config.weaponRef
        or { kind = "builtin", id = self.weaponId }
    self.trackingModuleRef = config.trackingModuleRef
    self.trackingModuleStats = config.trackingModuleStats or {}
    self.burstRemaining = 0
    self.burstGap = 0

    self.yaw = 0
    self.pitch = 0
    self.desiredYaw = 0
    self.desiredPitch = 0
    self.yawMin = config and config.yawMin or -math.pi
    self.yawMax = config and config.yawMax or math.pi
    self.pitchMin = config and config.pitchMin or -math.pi / 4
    self.pitchMax = config and config.pitchMax or math.pi / 4
    self.traverseRate = config and config.traverseRate or math.pi
    self.cooldown = 0
    self.ready = false
end)

function TurretComponent:getMountId()
    return self.mountId
end

function TurretComponent:getLocalPosition()
    return self.localPosition
end

function TurretComponent:setLocalPosition(position)
    self.localPosition = position
end

function TurretComponent:getLocalRotation()
    return self.localRotation
end

function TurretComponent:getWeaponId()
    return self.weaponId
end

function TurretComponent:getYaw()
    return self.yaw
end

function TurretComponent:setYaw(yaw)
    self.yaw = yaw
end

function TurretComponent:getPitch()
    return self.pitch
end

function TurretComponent:setPitch(pitch)
    self.pitch = pitch
end

return TurretComponent
