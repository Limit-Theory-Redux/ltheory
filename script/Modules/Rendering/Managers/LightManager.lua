---@class LightManager
---@overload fun(): LightManager
local LightManager = Class("LightManager", function(self)
    self.pointLights = {}
    self.diagnosticsEnabled = false
end)

---@param lights table[]
function LightManager:setPointLights(lights)
    self.pointLights = lights or {}
end

---@return table[]
function LightManager:getPointLights()
    return self.pointLights
end

function LightManager:clearPointLights()
    self.pointLights = {}
end

---@param enabled boolean
function LightManager:setDiagnosticsEnabled(enabled)
    self.diagnosticsEnabled = enabled == true
end

---@return boolean
function LightManager:isDiagnosticsEnabled()
    return self.diagnosticsEnabled == true
end

return LightManager()
