local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")
local DrawEx              = require("UI.DrawEx")
local Primitive           = require("Legacy.Systems.Gen.Primitive")
local CameraEntity        = require("Modules.Cameras.Entities").Camera
local CameraManager       = require("Modules.Cameras.Managers.CameraManager")
local CameraDataComponent = require("Modules.Cameras.Components.CameraDataComponent")
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")

--- 3D System Map — renders the solar system using the proper render pipeline
--- with its own camera entity.
---@class SystemMap3D
local SystemMap3D = {}

function SystemMap3D:create(config)
    config = config or {}

    -- Create a dedicated camera entity for the map
    local cam = CameraEntity()
    CameraManager:registerCamera("SystemMap3D", cam)

    return {
        enabled      = false,
        yaw          = -0.5,
        pitch        = 0.6,
        radius       = 3.0,
        radiusTarget = 3.0,
        minRadius    = 0.0000001,
        maxRadius    = 20.0,
        zoomSpeed    = 0.15,
        rotateSens   = 0.005,
        dragging     = false,
        lastMouseX   = 0,
        lastMouseY   = 0,
        entities     = {},
        selected     = nil,
        shipEntity   = nil,
        maxOrbit     = 1,
        focusX       = 0,
        focusY       = 0,
        focusZ       = 0,
        focusTargetX = 0,
        focusTargetY = 0,
        focusTargetZ = 0,
        trails       = {},
        trailMeshes  = {},
        trailDirty   = false,
        trailLen     = 300,
        trailTimer   = 0,
        trailInterval = 0.033,
        -- Camera
        cameraEntity = cam,
        prevCamera   = nil, -- to restore when map closes
    }
end

--- Set the skybox entity for background rendering
function SystemMap3D:setSkybox(state, skyboxEntity)
    state.skyboxEntity = skyboxEntity
end

function SystemMap3D:collectEntities(state, rootEntity, shipEntity)
    state.entities = {}
    state.shipEntity = shipEntity
    state._planetIdx = 0
    state.maxOrbit = 1
    self:_walk(state, rootEntity, nil)

    if shipEntity then
        local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            table.insert(state.entities, {
                entity = shipEntity, label = "Ship",
                pos = rbCmp:getRigidBody():getPos(),
                scale = rbCmp:getRigidBody():getBoundingRadius(),
                orbitRadius = 0, inclination = 0, isMoon = false,
                parentPos = nil, color = Color(0.2, 1.0, 0.2, 0.9),
                trailWidth = 0.3,
            })
        end
    end
end

function SystemMap3D:_walk(state, entity, parentPos)
    local name = tostring(entity)
    local transform = entity:get(PhysicsComponents.Transform)
    if not transform then
        local cc = entity:get(CoreComponents.Children)
        if cc then for child in cc:iterChildren() do self:_walk(state, child, parentPos) end end
        return
    end

    local pos = transform:getPos()
    local label, color, isMoon, trailWidth = nil, nil, false, 1.0

    if name:find("StarEntity") then
        label = "Star"; color = Color(1.0, 0.9, 0.3, 0.5); trailWidth = 3.0
    elseif name:find("PlanetEntity") then
        state._planetIdx = state._planetIdx + 1
        label = "Planet " .. state._planetIdx; color = Color(0.3, 0.6, 1.0, 0.5); trailWidth = 2.0
    elseif name:find("MoonEntity") then
        label = "Moon"; color = Color(0.5, 0.5, 0.6, 0.4); isMoon = true; trailWidth = 0.5
    elseif name:find("SpaceStationEntity") then
        label = "Station"; color = Color(1.0, 0.5, 0.1, 0.4); trailWidth = 0.3
    elseif name:find("AsteroidBeltEntity") then
        label = "Asteroid Belt"; color = Color(0.7, 0.5, 0.3, 0.3); trailWidth = 0
    end

    if label then
        local rbCmp = entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then pos = rbCmp:getRigidBody():getPos() end

        local orbitCmp = entity:get(SpatialComponents.Orbit)
        local orbitRadius = orbitCmp and orbitCmp:getOrbitRadius() or 0
        local incCmp = entity:get(SpatialComponents.Inclination)
        local inclination = incCmp and math.rad(incCmp:getInclination() or 0) or 0
        if orbitRadius > state.maxOrbit then state.maxOrbit = orbitRadius end

        table.insert(state.entities, {
            entity = entity, label = label, pos = pos,
            scale = transform:getScale(), orbitRadius = orbitRadius,
            inclination = inclination, isMoon = isMoon,
            parentPos = parentPos, color = color, trailWidth = trailWidth,
        })
    end

    local cc = entity:get(CoreComponents.Children)
    if cc then for child in cc:iterChildren() do self:_walk(state, child, pos) end end
end

function SystemMap3D:updateTrails(state, dt)
    if not state.enabled then return end
    state.trailTimer = state.trailTimer + dt
    if state.trailTimer < state.trailInterval then return end
    state.trailTimer = 0
    state.trailDirty = true

    local norm = state.maxOrbit
    for _, entry in ipairs(state.entities) do
        if entry.orbitRadius and entry.orbitRadius > 0 then
            local key = entry.entity
            if not state.trails[key] then state.trails[key] = {} end
            local trail = state.trails[key]
            local rbCmp = entry.entity:get(PhysicsComponents.RigidBody)
            local pos = entry.pos
            if rbCmp and rbCmp:getRigidBody() then pos = rbCmp:getRigidBody():getPos() end
            table.insert(trail, { x = pos.x / norm, y = pos.y / norm, z = pos.z / norm })
            while #trail > state.trailLen do table.remove(trail, 1) end
        end
    end
end

function SystemMap3D:updateInput(state, dt)
    if not state.enabled then return end

    local MapActions = require("Input.ActionBindings.MapActions")
    MapActions.Zoom:update(dt)
    MapActions.MoveX:update(dt)
    MapActions.MoveZ:update(dt)
    MapActions.Drag:update(dt)
    MapActions.Pan:update(dt)
    MapActions.Select:update(dt)

    -- Zoom
    local scrollY = MapActions.Zoom:get()
    if math.abs(scrollY) > 0.001 then
        state.radiusTarget = Math.Clamp(
            state.radiusTarget * math.exp(-state.zoomSpeed * scrollY),
            state.minRadius, state.maxRadius)
    end
    local logCur = math.log(state.radius)
    local logTgt = math.log(state.radiusTarget)
    local diff = logTgt - logCur
    if math.abs(diff) > 0.0001 then
        state.radius = math.exp(logCur + diff * math.min(1, 12 * dt))
    end

    -- Compute camera axes for panning
    local cosYaw = math.cos(state.yaw)
    local sinYaw = math.sin(state.yaw)
    local camRightX = cosYaw
    local camRightZ = -sinYaw
    local camFwdX = sinYaw
    local camFwdZ = cosYaw

    -- WASD panning via ActionBindings
    local panSpeed = state.radius * 0.5 * dt
    local moveX = MapActions.MoveX:get()
    local moveZ = MapActions.MoveZ:get()
    local panning = false
    if math.abs(moveZ) > 0.001 then
        state.manualFocusX = (state.manualFocusX or 0) + camFwdX * panSpeed * moveZ
        state.manualFocusZ = (state.manualFocusZ or 0) + camFwdZ * panSpeed * moveZ
        panning = true
    end
    if math.abs(moveX) > 0.001 then
        state.manualFocusX = (state.manualFocusX or 0) + camRightX * panSpeed * moveX
        state.manualFocusZ = (state.manualFocusZ or 0) + camRightZ * panSpeed * moveX
        panning = true
    end
    if panning then state.selected = nil end

    -- Rotate with right mouse, pan with middle mouse
    local mp = Input:mouse():position()
    if MapActions.Drag:isDown() then
        if state.dragging then
            state.yaw   = state.yaw + (mp.x - state.lastMouseX) * state.rotateSens
            state.pitch = Math.Clamp(state.pitch - (mp.y - state.lastMouseY) * state.rotateSens, -1.4, 1.4)
        end
        state.dragging = true
    elseif MapActions.Pan:isDown() then
        if state.dragging then
            local dx = (mp.x - state.lastMouseX) * state.radius * 0.001
            local dy = (mp.y - state.lastMouseY) * state.radius * 0.001
            state.manualFocusX = (state.manualFocusX or 0) - camRightX * dx - camFwdX * dy
            state.manualFocusZ = (state.manualFocusZ or 0) - camRightZ * dx - camFwdZ * dy
            state.selected = nil
        end
        state.dragging = true
    else
        state.dragging = false
    end
    state.lastMouseX = mp.x
    state.lastMouseY = mp.y

    -- Click to select
    if MapActions.Select:isPressed() then
        local scrW = Window:width()
        local scrH = Window:height()
        local mV = state._mView
        local mP = state._mProj
        local focusPos = state._focusPos or Position(0, 0, 0)

        if not mV or not mP then goto skip_click end

        local bestDist = 900
        local bestEntry = nil
        for _, entry in ipairs(state.entities) do
            if entry.isMoon and state.radius > 0.02 then goto next_sel end
            local rel = entry.pos:relativeTo(focusPos)
            local vp = mV:mulVec(Vec4f(rel.x, rel.y, rel.z, 1.0))
            local cp = mP:mulVec(vp)
            if cp.w <= 0 then goto next_sel end
            local screenX = (cp.x / cp.w + 1) * 0.5 * scrW
            local screenY = (1 - cp.y / cp.w) * 0.5 * scrH
            local dx = mp.x - screenX
            local dy = mp.y - screenY
            local d = dx * dx + dy * dy
            if d < bestDist then bestDist = d; bestEntry = entry end
            ::next_sel::
        end
        state.selected = bestEntry
        ::skip_click::
    end
end

--- Update the map camera entity transform — uses REAL game coordinates, not normalized
function SystemMap3D:updateCamera(state)
    local norm = state.maxOrbit

    -- Track focus in REAL f64 position
    if state.selected then
        local rbCmp = state.selected.entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            state.focusPos = rbCmp:getRigidBody():getPos()
        end
        -- Clear manual offset when following an entity
        state.manualFocusX = 0
        state.manualFocusZ = 0
    else
        -- Apply manual pan offset
        local mx = state.manualFocusX or 0
        local mz = state.manualFocusZ or 0
        if not state.focusPos then state.focusPos = Position(0, 0, 0) end
        state.focusPos = Position(
            state.focusPos.x + mx * norm,
            state.focusPos.y,
            state.focusPos.z + mz * norm
        )
        state.manualFocusX = 0
        state.manualFocusZ = 0
    end
    if not state.focusPos then state.focusPos = Position(0, 0, 0) end

    -- Camera distance in real game units
    local camDist = state.radius * norm
    local focusReal = state.focusPos
    local sphereOffset = Math.Spherical(camDist, state.pitch, state.yaw)
    local camEyePos = Position(
        focusReal.x + sphereOffset.x,
        focusReal.y + sphereOffset.y,
        focusReal.z + sphereOffset.z
    )

    local transform = state.cameraEntity:get(PhysicsComponents.Transform)
    transform:setPos(camEyePos)

    -- Look direction: negate the spherical offset (always valid)
    local lookDir = Vec3f(-sphereOffset.x, -sphereOffset.y, -sphereOffset.z):normalize()
    transform:setRot(Quat.FromLook(lookDir, Vec3f(0, 1, 0)))

    -- Cache for overlay pass
    local sx, sy = Window:width(), Window:height()
    -- View matrix relative to camera position (f32-safe)
    local camEyeVec = Vec3f(sphereOffset.x, sphereOffset.y, sphereOffset.z)
    state._mView = Matrix.LookAt(camEyeVec, Vec3f(0, 0, 0), Vec3f(0, 1, 0))
    state._mProj = Matrix.Perspective(60, sx / sy, camDist * 0.001, camDist * 1000)
    state._camDist = camDist
    state._norm = norm
    state._focusPos = focusReal
end

--- Activate the map camera (call once when opening map)
function SystemMap3D:activate(state)
    state.prevCamera = CameraManager:getActiveCameraName()
    CameraManager:setActiveCamera("SystemMap3D")
end

--- Deactivate the map camera (call once when closing map)
function SystemMap3D:deactivate(state)
    if state.prevCamera then
        CameraManager:setActiveCamera(state.prevCamera)
        state.prevCamera = nil
    end
end

--- Render the 3D map using the proper render pipeline
---@param state table
---@param data EventData
function SystemMap3D:render(state, data)
    if not state.enabled then return end

    -- Update camera transform each frame
    self:updateCamera(state)

    -- Render through the full pipeline
    RenderCoreSystem:render(data)
end

--- Render 2D overlay (labels, trails, info) on top of the 3D scene
function SystemMap3D:renderOverlay(state, x, y, sx, sy)
    if not state.enabled then return end

    local norm = state._norm or state.maxOrbit
    local camDist = state._camDist or (state.radius * state.maxOrbit)
    local mView = state._mView
    local mProj = state._mProj
    if not mView or not mProj then return end

    -- Draw trails in overlay (additive, using trail shader)
    RenderState.PushBlendMode(BlendMode.Additive)
    local trailShader = Cache.Shader('hologram3d', 'ui/trail3d')
    local mWorldIdentity = Matrix.Identity()
    local maxThickness = camDist * 0.0008

    for _, entry in ipairs(state.entities) do
        local key = entry.entity
        local trail = state.trails[key]
        if not trail or #trail < 3 then goto next_trail end

        if state.trailDirty or not state.trailMeshes[key] then
            local len = #trail
            local mesh = Mesh.Create()
            -- Trail positions are normalized — scale to real coords
            local tw = maxThickness * (entry.trailWidth or 1.0)

            for i = 1, len - 1 do
                local p1 = trail[i]
                local p2 = trail[i + 1]
                -- Convert normalized trail positions to camera-relative coordinates
                local fp = state._focusPos or Position(0, 0, 0)
                local r1x = p1.x * norm - fp.x
                local r1y = p1.y * norm - fp.y
                local r1z = p1.z * norm - fp.z
                local r2x = p2.x * norm - fp.x
                local r2y = p2.y * norm - fp.y
                local r2z = p2.z * norm - fp.z

                local t1 = (i - 1) / (len - 1)
                local t2 = i / (len - 1)
                local w1 = tw * t1
                local w2 = tw * t2

                local dx = r2x - r1x
                local dz = r2z - r1z
                local pl = math.sqrt(dx * dx + dz * dz)
                local nx, nz
                if pl > 1e-10 then nx = -dz / pl; nz = dx / pl
                else nx = 1; nz = 0 end

                local b = (i - 1) * 4
                mesh:addVertex(r1x - nx * w1, r1y, r1z - nz * w1, 0, 1, 0, t1, 0)
                mesh:addVertex(r1x + nx * w1, r1y, r1z + nz * w1, 0, 1, 0, t1, 0)
                mesh:addVertex(r2x + nx * w2, r2y, r2z + nz * w2, 0, 1, 0, t2, 0)
                mesh:addVertex(r2x - nx * w2, r2y, r2z - nz * w2, 0, 1, 0, t2, 0)
                mesh:addQuad(b, b + 1, b + 2, b + 3)
            end
            state.trailMeshes[key] = mesh
        end

        local c = entry.isMoon and { 0.5, 0.5, 0.7 } or { 0.7, 0.7, 0.9 }
        trailShader:start()
        trailShader:setMatrix('mWorld', mWorldIdentity)
        trailShader:setMatrix('mView', mView)
        trailShader:setMatrix('mProj', mProj)
        trailShader:setFloat4('color', c[1], c[2], c[3], entry.isMoon and 0.5 or 0.7)
        state.trailMeshes[key]:draw()
        trailShader:stop()

        ::next_trail::
    end
    state.trailDirty = false
    RenderState.PopBlendMode()

    -- Labels
    RenderState.PushBlendMode(BlendMode.Additive)
    for _, entry in ipairs(state.entities) do
        if entry.isMoon and state.radius > 0.02 then goto next_lbl end

        local rbCmp = entry.entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then entry.pos = rbCmp:getRigidBody():getPos() end

        -- Position relative to camera focus (f64 subtraction, then f32)
        local focusPos = state._focusPos or Position(0, 0, 0)
        local rel = entry.pos:relativeTo(focusPos)

        local vp = mView:mulVec(Vec4f(rel.x, rel.y, rel.z, 1.0))
        local cp = mProj:mulVec(vp)
        if cp.w <= 0 then goto next_lbl end

        local screenX = x + (cp.x / cp.w + 1) * 0.5 * sx
        local screenY = y + (1 - cp.y / cp.w) * 0.5 * sy

        if screenX > x and screenX < x + sx and screenY > y and screenY < y + sy then
            local c = entry.color
            local isSelected = state.selected and state.selected.entity == entry.entity
            DrawEx.TextAdditive('Unageo-Medium', entry.label, 11,
                screenX + 10, screenY - 6, 160, 14,
                c.r, c.g, c.b, isSelected and 1.0 or 0.8, 0.0, 0.5)
        end
        ::next_lbl::
    end

    DrawEx.TextAdditive('Unageo-Medium',
        "3D System Map [M to cycle] | RMB=Rotate | Scroll=Zoom", 12,
        x + 10, y + 10, 600, 16, 0.6, 0.7, 1.0, 0.9, 0.0, 0.5)
    RenderState.PopBlendMode()

    -- Selected info
    if state.selected then
        RenderState.PushBlendMode(BlendMode.Alpha)
        self:drawSelectedInfo(state, x + sx - 250, y + 40, 240)
        RenderState.PopBlendMode()
    end
end

function SystemMap3D:drawSelectedInfo(state, x, y, w)
    local entry = state.selected
    if not entry then return end
    local lineH = 18; local ty = y

    DrawEx.SimpleRect(x - 5, y - 5, w + 10, 120, Color(0.03, 0.03, 0.05, 0.9))
    DrawEx.TextAlpha('Unageo-Medium', entry.label, 14, x, ty, w, lineH,
        entry.color.r, entry.color.g, entry.color.b, 1.0, 0.0, 0.5)
    ty = ty + lineH + 4
    DrawEx.TextAlpha('Unageo-Medium', "Radius: " .. UniverseScaleConfig:formatDistance(entry.scale or 0),
        10, x, ty, w, lineH, 0.8, 0.8, 0.8, 0.8, 0.0, 0.5)
    ty = ty + lineH
    if state.shipEntity then
        local shipRb = state.shipEntity:get(PhysicsComponents.RigidBody)
        if shipRb and shipRb:getRigidBody() then
            local dist = shipRb:getRigidBody():getPos():distance(entry.pos)
            DrawEx.TextAlpha('Unageo-Medium', "Distance: " .. UniverseScaleConfig:formatDistance(dist),
                10, x, ty, w, lineH, 0.8, 0.8, 0.8, 0.8, 0.0, 0.5)
            ty = ty + lineH
        end
    end
    if entry.orbitRadius and entry.orbitRadius > 0 then
        DrawEx.TextAlpha('Unageo-Medium',
            string.format("Orbit: %s | Inc: %.1f", UniverseScaleConfig:formatDistance(entry.orbitRadius), math.deg(entry.inclination or 0)),
            10, x, ty, w, lineH, 0.8, 0.8, 0.8, 0.8, 0.0, 0.5)
    end
end

return SystemMap3D
