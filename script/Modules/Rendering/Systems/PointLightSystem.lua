local Registry = require("Core.ECS.Registry")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local LightManager = require("Modules.Rendering.Managers.LightManager")

local pointLightMarkerMesh
local pointLightMarkerShader

local function getEntityPosition(entity)
    if not entity or (entity.isValid and not entity:isValid()) then
        return nil
    end

    local rigidBodyComponent = entity:get(Physics.RigidBody)
    local rigidBody = rigidBodyComponent and rigidBodyComponent:getRigidBody()
    if rigidBody and rigidBody.getPos then
        return rigidBody:getPos()
    end

    local transform = entity:get(Physics.Transform)
    return transform and transform.getPos and transform:getPos() or nil
end

local function copyColor(color)
    if not color then
        return nil
    end
    local red = color.r ~= nil and color.r or color.x
    local green = color.g ~= nil and color.g or color.y
    local blue = color.b ~= nil and color.b or color.z
    if red == nil or green == nil or blue == nil then
        return nil
    end
    return Vec3f(red, green, blue)
end

local function copyPosition(position)
    if not position then
        return nil
    end
    return Position(position.x, position.y, position.z)
end

---@class PointLightSystem
---@overload fun(): PointLightSystem
local PointLightSystem = Class("PointLightSystem", function() end)

---@param lights table[]
---@param entity Entity
---@param component PointLightComponent
---@param position Vec3f|nil
---@param source table|nil
---@param sourceIndex integer
---@param sourceCount integer
local function appendLight(lights, entity, component, position, source, sourceIndex, sourceCount)
    if not position or (source and source.enabled == false) then
        return
    end

    local radius = math.max(0, (source and source.radius) or component:getRadius())
    local intensity = math.max(0, (source and source.intensity) or component:getIntensity())
    local color = copyColor((source and source.color) or component:getColor())
    if not color or intensity <= 0 then
        return
    end

    table.insert(lights, {
        entityId = entity.id,
        sourceIndex = sourceIndex,
        sourceCount = sourceCount,
        pos = copyPosition(position),
        color = color,
        radius = radius,
        intensity = intensity,
    })
end

---@param dt number
function PointLightSystem:advance(dt)
    if not dt or dt <= 0 then
        return
    end

    local expired = {}
    for entity, effect in Registry:view(Rendering.LightEffect) do
        if entity:isValid() then
            effect.remaining = effect.remaining - dt
            local pointLight = entity:get(Rendering.PointLight)
            if pointLight then
                effect.baseIntensity = effect.baseIntensity
                    or pointLight:getIntensity()
                local fade = 1
                if effect.fadeOutDuration > 0 then
                    fade = math.min(1, math.max(0,
                        effect.remaining / effect.fadeOutDuration))
                end
                pointLight:setIntensity(effect.baseIntensity * fade)
            end
            if effect.remaining <= 0 then
                table.insert(expired, entity)
            end
        end
    end

    for _, entity in ipairs(expired) do
        if entity:isValid() then
            Registry:destroyEntity(entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
        end
    end
end

---@return table[]
function PointLightSystem:collect()
    local lights = {}
    for entity, component in Registry:view(Rendering.PointLight) do
        local enabled = component.isEnabled
            and component:isEnabled()
            or component.enabled == true
        if enabled then
            local sources = component.getSources and component:getSources() or nil
            if sources and #sources > 0 then
                local sourceCount = #sources
                for sourceIndex, source in ipairs(sources) do
                    local position = source.position or source.pos or getEntityPosition(entity)
                    appendLight(lights, entity, component, position,
                        source, sourceIndex, sourceCount)
                end
            else
                appendLight(lights, entity, component, getEntityPosition(entity),
                    nil, 1, 1)
            end
        end
    end
    return lights
end

---@param dt number|nil
---@return table[]
function PointLightSystem:update(dt)
    self:advance(dt or 0)
    local lights = self:collect()
    LightManager:setPointLights(lights)
    return lights
end

---@param blendMode any
---@param eye Position|nil
function PointLightSystem:renderDiagnostics(blendMode, eye)
    if blendMode ~= BlendMode.Additive
        or not LightManager:isDiagnosticsEnabled()
    then
        return
    end

    local lights = LightManager:getPointLights()
    if #lights == 0 then
        return
    end

    if not pointLightMarkerMesh then
        pointLightMarkerMesh = Gen.Primitive.Billboard(-1, -1, 1, 1)
        pointLightMarkerShader = Cache.Shader("billboard/quad", "effect/pulsehead")
    end

    eye = eye or CameraManager:getEye()
    pointLightMarkerShader:start()
    pointLightMarkerMesh:drawBind()
    for _, light in ipairs(lights) do
        local position = light.pos:relativeTo(eye)
        local markerSize = math.max(0.12, math.min(0.35, (light.radius or 0.18) * 0.85))
        pointLightMarkerShader:setFloat3(
            "color",
            light.color.x,
            light.color.y,
            light.color.z)
        pointLightMarkerShader:setMatrix(
            "mWorld",
            Matrix.Translation(position.x, position.y, position.z))
        pointLightMarkerShader:setFloat("alpha", 0.35)
        pointLightMarkerShader:setFloat("size", markerSize * 2.2)
        pointLightMarkerMesh:drawBound()
        pointLightMarkerShader:setFloat("alpha", 1.0)
        pointLightMarkerShader:setFloat("size", markerSize)
        pointLightMarkerMesh:drawBound()
    end
    pointLightMarkerMesh:drawUnbind()
    pointLightMarkerShader:stop()
end

return PointLightSystem()
