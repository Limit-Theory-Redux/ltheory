local Component = require("Core.ECS.Component")

---@class WeaponTrackingComponent: Component
---@overload fun(config?: table): WeaponTrackingComponent
local WeaponTrackingComponent = Subclass("WeaponTrackingComponent", Component, function(self, config)
    self:setComponentName("WeaponTracking")

    config = config or {}
    self.moduleRef = config.moduleRef
    self.moduleStats = config.moduleStats or {}
    self.track = nil
    self.targetKey = nil
    self.sampleTime = 0
    self.confidence = 0
    self.mountSolutions = {}
    self.targetPointByMount = {}
    self.lastTargetPoint = nil
end)

function WeaponTrackingComponent:getModuleRef()
    return self.moduleRef
end

function WeaponTrackingComponent:setModuleRef(moduleRef)
    self.moduleRef = moduleRef
end

function WeaponTrackingComponent:getTrack()
    return self.track
end

function WeaponTrackingComponent:reset(targetKey)
    self.track = nil
    self.targetKey = targetKey
    self.sampleTime = 0
    self.confidence = 0
    self.mountSolutions = {}
    self.targetPointByMount = {}
    self.lastTargetPoint = nil
end

return WeaponTrackingComponent
