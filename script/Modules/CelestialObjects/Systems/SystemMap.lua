local CoreComponents        = require("Modules.Core.Components")
local PhysicsComponents     = require("Modules.Physics.Components")
local SpatialComponents     = require("Modules.Spatial.Components")
local CelestialComponents   = require("Modules.CelestialObjects.Components")
local UniverseScaleConfig   = require("Config.Gen.UniverseScaleConfig")
local DrawEx                = require("UI.DrawEx")

local function formatDistance(gameUnits)
    return UniverseScaleConfig:formatDistance(gameUnits)
end

--- ECS System Map — top-down 2D view of the star system with mouse interaction.
---@class SystemMap
local SystemMap = {}

---@param config table|nil
---@return table state
function SystemMap:create(config)
    config = config or {}
    return {
        enabled    = false,
        panX       = 0,
        panY       = 0,
        zoom       = config.zoom or 0.001,
        zoomTarget = config.zoom or 0.001,
        minZoom    = config.minZoom or 0.0000001,
        maxZoom    = config.maxZoom or 1.0,
        zoomSpeed  = config.zoomSpeed or 0.15,
        panSpeed   = config.panSpeed or 500,
        entities   = {},
        shipEntity = nil,
        selected   = nil,    -- currently selected entity entry
        autopilotPos = nil,  -- current autopilot destination (Position or nil)
        dragging   = false,
        lastMouseX = 0,
        lastMouseY = 0,
    }
end

--- Collect all entities with positions from the star system hierarchy
---@param state table
---@param rootEntity Entity
---@param shipEntity Entity|nil
function SystemMap:collectEntities(state, rootEntity, shipEntity)
    state.entities = {}
    state.shipEntity = shipEntity
    state._planetIdx = 0
    self:_walk(state, rootEntity, nil)

    -- Add ship
    if shipEntity then
        local rbCmp = shipEntity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            local pos = rbCmp:getRigidBody():getPos()
            table.insert(state.entities, {
                entity = shipEntity,
                label  = "Player Ship",
                pos    = pos,
                scale  = rbCmp:getRigidBody():getBoundingRadius(),
                color  = { 0.2, 1.0, 0.2 },
                size   = 6,
            })
        end
    end

    -- Auto-fit on first open only (not every collectEntities call)
    if not state.hasAutoFit and #state.entities > 0 then
        state.hasAutoFit = true
        local minX, maxX, minZ, maxZ = math.huge, -math.huge, math.huge, -math.huge
        for _, entry in ipairs(state.entities) do
            local p = entry.pos
            if p.x < minX then minX = p.x end
            if p.x > maxX then maxX = p.x end
            if p.z < minZ then minZ = p.z end
            if p.z > maxZ then maxZ = p.z end
        end
        local cx = (minX + maxX) / 2
        local cz = (minZ + maxZ) / 2
        local spanX = maxX - minX
        local spanZ = maxZ - minZ
        local span = math.max(spanX, spanZ, 1)

        state.panX = -cx
        state.panY = -cz
        -- Fit with 20% padding
        local screenSize = math.min(Window:width(), Window:height()) * 0.8
        state.zoom = screenSize / span
        state.zoomTarget = state.zoom
    end
end

---@param state table
---@param entity Entity
---@param parentPos Position|nil
---@param parentPos Position|nil
function SystemMap:_walk(state, entity, parentPos)
    local name = tostring(entity)
    local transform = entity:get(PhysicsComponents.Transform)

    if transform then
        local pos = transform:getPos()
        local scale = transform:getScale()
        local label, color, pointSize, isMoon, isBelt = nil, nil, nil, false, false

        if name:find("StarEntity") then
            label = "Star"
            color = { 1.0, 0.9, 0.3 }
            pointSize = 8
        elseif name:find("PlanetEntity") then
            state._planetIdx = state._planetIdx + 1
            label = "Planet " .. state._planetIdx
            color = { 0.3, 0.6, 1.0 }
            pointSize = 5
        elseif name:find("MoonEntity") then
            label = "Moon"
            color = { 0.6, 0.6, 0.6 }
            pointSize = 3
            isMoon = true
        elseif name:find("SpaceStationEntity") then
            label = "Station"
            color = { 1.0, 0.5, 0.1 }
            pointSize = 4
        elseif name:find("AsteroidBeltEntity") then
            label = "Asteroid Belt"
            color = { 0.7, 0.5, 0.3 }
            pointSize = 3
            isBelt = true
        elseif name:find("AsteroidEntity") then
            label = "Asteroid"
            color = { 0.7, 0.5, 0.3 }
            pointSize = 2
        end

        if label then
            local rbCmp = entity:get(PhysicsComponents.RigidBody)
            if rbCmp and rbCmp:getRigidBody() then
                pos = rbCmp:getRigidBody():getPos()
                scale = rbCmp:getRigidBody():getBoundingRadius()
            end

            local orbitCmp = entity:get(SpatialComponents.Orbit)
            local orbitRadius = orbitCmp and orbitCmp:getOrbitRadius() or 0

            table.insert(state.entities, {
                entity      = entity,
                label       = label,
                pos         = pos,
                scale       = scale,
                orbitRadius = orbitRadius,
                color       = color,
                size        = pointSize,
                isMoon      = isMoon,
                isBelt      = isBelt,
                beltWidth   = isBelt and (entity:get(SpatialComponents.Width) and entity:get(SpatialComponents.Width):getWidth() or 0) or nil,
                parentPos   = parentPos,
            })
        end

        -- Children use this entity's position as parent
        local childrenCmp = entity:get(CoreComponents.Children)
        if childrenCmp then
            for child in childrenCmp:iterChildren() do
                self:_walk(state, child, pos)
            end
        end
    else
        local childrenCmp = entity:get(CoreComponents.Children)
        if childrenCmp then
            for child in childrenCmp:iterChildren() do
                self:_walk(state, child, parentPos)
            end
        end
    end
end

--- Handle input for the map (pan, zoom, click)
---@param state table
---@param dt number
function SystemMap:updateInput(state, dt)
    if not state.enabled then return end

    local MapActions = require("Input.ActionBindings.MapActions")
    MapActions.Zoom:update(dt)
    MapActions.Pan:update(dt)
    MapActions.Drag:update(dt)
    MapActions.Select:update(dt)

    -- Zoom with mouse wheel
    local scrollY = MapActions.Zoom:get()
    if math.abs(scrollY) > 0.001 then
        state.zoomTarget = Math.Clamp(
            state.zoomTarget * math.exp(state.zoomSpeed * scrollY),
            state.minZoom, state.maxZoom)
    end

    -- Smooth interpolation in log space
    local logCurrent = math.log(state.zoom)
    local logTarget = math.log(state.zoomTarget)
    local diff = logTarget - logCurrent
    if math.abs(diff) > 0.0001 then
        state.zoom = math.exp(logCurrent + diff * math.min(1, 12 * dt))
    else
        state.zoom = state.zoomTarget
    end

    -- Pan with middle mouse or right mouse drag
    local mp = Input:mouse():position()
    if MapActions.Pan:isDown() or MapActions.Drag:isDown() then
        if state.dragging then
            local dx = mp.x - state.lastMouseX
            local dy = mp.y - state.lastMouseY
            state.panX = state.panX + dx / state.zoom
            state.panY = state.panY + dy / state.zoom
            state.selected = nil
        end
        state.dragging = true
    else
        state.dragging = false
    end
    state.lastMouseX = mp.x
    state.lastMouseY = mp.y

    -- Follow selected: use clickPos (belt nav target) or entity position
    if state.selected and not state.dragging then
        local targetPanX, targetPanY
        if state.selected.clickPos then
            targetPanX = -state.selected.clickPos.x
            targetPanY = -state.selected.clickPos.z
        else
            local pos = state.selected.pos
            local rbCmp = state.selected.entity:get(PhysicsComponents.RigidBody)
            if rbCmp and rbCmp:getRigidBody() then
                pos = rbCmp:getRigidBody():getPos()
                state.selected.pos = pos
            end
            targetPanX = -pos.x
            targetPanY = -pos.z
        end
        local f = math.min(1, 6 * dt)
        state.panX = state.panX + (targetPanX - state.panX) * f
        state.panY = state.panY + (targetPanY - state.panY) * f
    end

    -- Click to select nearest entity
    if MapActions.Select:isPressed() then
        local sx = Window:width()
        local sy = Window:height()
        local hx, hy = sx / 2, sy / 2
        local bestDist = 400
        local bestEntry = nil
        local bestClickPos = nil

        for _, entry in ipairs(state.entities) do
            -- Skip hidden moons
            if entry.isMoon and entry.orbitRadius then
                local moonScreenRadius = entry.orbitRadius * state.zoom
                if moonScreenRadius < 20 then goto next_click end
            end

            if entry.isBelt and entry.orbitRadius and entry.orbitRadius > 0 then
                -- Check individual asteroid dots first (higher priority)
                local beltCmp = entry.entity:get(CelestialComponents.AsteroidBelt)
                local beltScreenR = entry.orbitRadius * state.zoom
                if beltCmp and beltScreenR > math.min(Window:width(), Window:height()) * 2 then
                    local asteroids = beltCmp:getAsteroidData()
                    local dotOcx = hx + state.panX * state.zoom
                    local dotOcy = hy + state.panY * state.zoom
                    -- Use angular buckets for fast lookup near mouse
                    local mdxB = (mp.x - dotOcx)
                    local mdyB = (mp.y - dotOcy)
                    local mouseAngle = math.atan2(mdyB, mdxB)
                    local indices = beltCmp:getAsteroidsInAngleRange(mouseAngle - 0.1, mouseAngle + 0.1)
                    local bestAsteroidDist = 400
                    for _, i in ipairs(indices) do
                        local a = asteroids[i]
                        local ax = dotOcx + a.px * state.zoom
                        local ay = dotOcy + a.pz * state.zoom
                        local ad = (mp.x - ax)^2 + (mp.y - ay)^2
                        if ad < bestAsteroidDist and ad < bestDist then
                            bestDist = ad
                            bestEntry = entry
                            bestClickPos = Position(a.px, a.py, a.pz)
                            bestAsteroidDist = ad
                        end
                    end
                end

                -- Belt ring fallback (lower priority)
                local ocx = hx + state.panX * state.zoom
                local ocy = hy + state.panY * state.zoom
                local mdx = mp.x - ocx
                local mdy = mp.y - ocy
                local mouseDistFromCenter = math.sqrt(mdx * mdx + mdy * mdy)
                local beltScreenW = (entry.beltWidth or entry.orbitRadius * 0.1) * state.zoom
                local innerR = beltScreenR - beltScreenW * 0.5
                local outerR = beltScreenR + beltScreenW * 0.5
                local clickPad = 10
                if mouseDistFromCenter >= innerR - clickPad and mouseDistFromCenter <= outerR + clickPad then
                    local d = 300
                    if d < bestDist then
                        bestDist = d
                        local angle = math.atan2(mdy, mdx)
                        local clickWorldX = math.cos(angle) * entry.orbitRadius
                        local clickWorldZ = math.sin(angle) * entry.orbitRadius
                        bestEntry = entry
                        bestClickPos = Position(clickWorldX, 0, clickWorldZ)
                    end
                end
            else
                local screenX = hx + (entry.pos.x + state.panX) * state.zoom
                local screenY = hy + (entry.pos.z + state.panY) * state.zoom
                local dx = mp.x - screenX
                local dy = mp.y - screenY
                local d = dx * dx + dy * dy
                if d < bestDist then
                    bestDist = d
                    bestEntry = entry
                    bestClickPos = nil  -- non-belt entity: clear belt click position
                end
            end
            ::next_click::
        end
        if bestEntry then
            bestEntry.clickPos = bestClickPos  -- world position for belt clicks
        end
        state.selected = bestEntry
    end
end

--- Draw the system map overlay
---@param state table
---@param x number
---@param y number
---@param sx number
---@param sy number
function SystemMap:draw(state, x, y, sx, sy)
    if not state.enabled then return end

    local hx, hy = sx / 2, sy / 2
    local cx, cy = x + hx, y + hy

    -- Background (use SimpleRect — DrawEx.Rect uses additive blend which can't darken)
    RenderState.PushBlendMode(BlendMode.Alpha)
    DrawEx.SimpleRect(x, y, sx, sy, Color(0.02, 0.02, 0.03, 0.97))

    -- Title
    DrawEx.TextAlpha('Unageo-Medium', "System Map [M to close] | Scroll=Zoom | RMB=Pan | LMB=Select", 12,
        x + 10, y + 10, 600, 16, 1.0, 1.0, 1.0, 0.8, 0.0, 0.5)

    -- Draw orbit rings using the ring shader with correctly sized quad
    for _, entry in ipairs(state.entities) do
        if entry.orbitRadius and entry.orbitRadius > 0 then
            local ringScreenRadius = entry.orbitRadius * state.zoom
            if ringScreenRadius > 2 then
                -- Orbit center: parent position or star (origin)
                local ocx, ocy
                if entry.parentPos then
                    ocx = cx + (entry.parentPos.x + state.panX) * state.zoom
                    ocy = cy + (entry.parentPos.z + state.panY) * state.zoom
                else
                    ocx = cx + state.panX * state.zoom
                    ocy = cy + state.panY * state.zoom
                end

                local c = entry.isMoon
                    and { r = 0.6, g = 0.6, b = 0.7, a = 0.2 }
                    or  { r = 0.8, g = 0.8, b = 0.9, a = 0.35 }

                -- Use ring shader directly with properly sized quad (2*r, not r)
                local r = ringScreenRadius
                local pad = 8
                local shader = Cache.Shader('ui', 'ui/ring')
                RenderState.PushBlendMode(BlendMode.Alpha)
                shader:start()
                shader:setFloat('radius', r)
                shader:setFloat2('size', 2 * r + 2 * pad, 2 * r + 2 * pad)
                shader:setFloat4('color', c.r, c.g, c.b, c.a)
                shader:setInt('glow', 0)
                Draw.Rect(ocx - r - pad, ocy - r - pad, 2 * r + 2 * pad, 2 * r + 2 * pad)
                shader:stop()
                RenderState.PopBlendMode()
            end
        end
    end

    -- Draw asteroid belt as solid filled annulus
    for _, entry in ipairs(state.entities) do
        if entry.isBelt and entry.orbitRadius and entry.orbitRadius > 0 then
            local beltScreenRadius = entry.orbitRadius * state.zoom
            local beltWidth = (entry.beltWidth or entry.orbitRadius * 0.1) * state.zoom
            if beltScreenRadius > 2 then
                local ocx = cx + state.panX * state.zoom
                local ocy = cy + state.panY * state.zoom

                local innerR = math.max(1, beltScreenRadius - beltWidth * 0.5)
                local outerR = beltScreenRadius + beltWidth * 0.5
                local pad = 4
                local totalSize = 2 * outerR + 2 * pad

                local shader = Cache.Shader('ui', 'ui/annulus')
                RenderState.PushBlendMode(BlendMode.Alpha)
                shader:start()
                shader:setFloat('innerRadius', innerR)
                shader:setFloat('outerRadius', outerR)
                shader:setFloat2('size', totalSize, totalSize)
                shader:setFloat4('color', 0.7, 0.5, 0.3, 0.25)
                Draw.Rect(ocx - outerR - pad, ocy - outerR - pad, totalSize, totalSize)
                shader:stop()
                RenderState.PopBlendMode()
            end

            -- Draw individual asteroid dots when zoomed in far enough
            -- Show all asteroid dots when zoomed in
            local beltCmp2 = entry.entity:get(CelestialComponents.AsteroidBelt)
            local screenMinDim = math.min(sx, sy)
            if beltCmp2 and beltScreenRadius > screenMinDim * 2 then
                local asteroids = beltCmp2:getAsteroidData()
                local dotOcx = cx + state.panX * state.zoom
                local dotOcy = cy + state.panY * state.zoom
                local sz = 2
                local dotColor = Color(0.8, 0.6, 0.3, 0.7)
                for i = 1, #asteroids do
                    local a = asteroids[i]
                    local ax = dotOcx + a.px * state.zoom
                    local ay = dotOcy + a.pz * state.zoom
                    if ax >= x - 2 and ax <= x + sx + 2 and ay >= y - 2 and ay <= y + sy + 2 then
                        DrawEx.SimpleRect(ax - 1, ay - 1, sz, sz, dotColor)
                    end
                end
            end

            -- Highlight selected asteroid
            if state.selected and state.selected.clickPos and state.selected.entity == entry.entity then
                local cp = state.selected.clickPos
                local dotOcx = cx + state.panX * state.zoom
                local dotOcy = cy + state.panY * state.zoom
                local ax = dotOcx + cp.x * state.zoom
                local ay = dotOcy + cp.z * state.zoom
                DrawEx.Rect(ax - 3, ay - 3, 6, 6, Color(1.0, 0.8, 0.3, 1.0))
            end
        end
    end

    -- Draw entities
    for _, entry in ipairs(state.entities) do
        -- Hide moons unless zoomed in close to their parent planet
        if entry.isMoon and entry.parentPos then
            local parentScreenDist = entry.orbitRadius * state.zoom
            if parentScreenDist < 20 then goto next_map_entity end
        end

        -- Belt entities: draw label at top of ring, not as a point
        if entry.isBelt and entry.orbitRadius and entry.orbitRadius > 0 then
            local beltScreenR = entry.orbitRadius * state.zoom
            if beltScreenR > 10 then
                local ocx = cx + state.panX * state.zoom
                local ocy = cy + state.panY * state.zoom
                local labelX = ocx
                local labelY = ocy - beltScreenR - 8
                local c = entry.color
                DrawEx.TextAlpha('Unageo-Medium', entry.label, 10,
                    labelX - 40, labelY - 6, 120, 14,
                    c[1], c[2], c[3], 0.7, 0.5, 0.5)
            end
            goto next_map_entity
        end

        local pos = entry.pos
        local screenX = cx + (pos.x + state.panX) * state.zoom
        local screenY = cy + (pos.z + state.panY) * state.zoom

        if screenX >= x - 50 and screenX <= x + sx + 50 and
           screenY >= y - 50 and screenY <= y + sy + 50 then

            local c = entry.color
            local ps = entry.size

            -- Draw point
            DrawEx.Rect(screenX - ps / 2, screenY - ps / 2, ps, ps,
                Color(c[1], c[2], c[3], 1.0))

            -- Draw selection ring
            if state.selected and state.selected.entity == entry.entity then
                DrawEx.Ring(screenX, screenY, ps + 6,
                    { r = 1.0, g = 0.3, b = 0.3, a = 1.0 }, true)
            end

            -- Draw label
            DrawEx.TextAlpha('Unageo-Medium', entry.label, 10,
                screenX + ps + 4, screenY - 6, 120, 14,
                c[1], c[2], c[3], 0.9, 0.0, 0.5)
        end
        ::next_map_entity::
    end

    -- Draw nav target marker (belt click position or autopilot destination)
    local navMarkerPos = (state.selected and state.selected.clickPos) or state.autopilotPos
    if navMarkerPos then
        local cp = navMarkerPos
        local cpx = cx + (cp.x + state.panX) * state.zoom
        local cpy = cy + (cp.z + state.panY) * state.zoom
        -- Crosshair marker
        local ms = 6
        DrawEx.Rect(cpx - ms, cpy - 0.5, ms * 2, 1, Color(1.0, 0.8, 0.3, 0.9))
        DrawEx.Rect(cpx - 0.5, cpy - ms, 1, ms * 2, Color(1.0, 0.8, 0.3, 0.9))
        DrawEx.TextAlpha('Unageo-Medium', "NAV TARGET", 9,
            cpx + ms + 2, cpy - 5, 100, 12, 1.0, 0.8, 0.3, 0.9, 0.0, 0.5)
    end

    -- Draw nav line from ship to selected destination or active autopilot target
    local navTarget = state.selected or (state.autopilotPos and { clickPos = state.autopilotPos })
    if navTarget and state.shipEntity then
        local shipRb = state.shipEntity:get(PhysicsComponents.RigidBody)
        if shipRb and shipRb:getRigidBody() then
            local shipPos = shipRb:getRigidBody():getPos()
            local shipSX = cx + (shipPos.x + state.panX) * state.zoom
            local shipSY = cy + (shipPos.z + state.panY) * state.zoom

            local destSX, destSY
            if navTarget.clickPos then
                local cp = navTarget.clickPos
                destSX = cx + (cp.x + state.panX) * state.zoom
                destSY = cy + (cp.z + state.panY) * state.zoom
            elseif navTarget.pos then
                local pos = navTarget.pos
                if navTarget.entity then
                    local rbCmp = navTarget.entity:get(PhysicsComponents.RigidBody)
                    if rbCmp and rbCmp:getRigidBody() then
                        pos = rbCmp:getRigidBody():getPos()
                    end
                end
                destSX = cx + (pos.x + state.panX) * state.zoom
                destSY = cy + (pos.z + state.panY) * state.zoom
            end

            if destSX then
                -- Fine dashed line
                local ldx = destSX - shipSX
                local ldy = destSY - shipSY
                local len = math.sqrt(ldx * ldx + ldy * ldy)
                if len > 5 then
                    local nx, ny = ldx / len, ldy / len
                    local dashLen = 4
                    local gapLen = 4
                    local t = 0
                    local lineColor = Color(1.0, 0.8, 0.3, 0.5)
                    while t < len do
                        local t2 = math.min(t + dashLen, len)
                        DrawEx.Line(
                            shipSX + nx * t, shipSY + ny * t,
                            shipSX + nx * t2, shipSY + ny * t2,
                            lineColor)
                        t = t + dashLen + gapLen
                    end
                end
            end
        end
    end

    -- Draw selected entity info panel
    if state.selected then
        self:drawSelectedInfo(state, x + sx - 250, y + 40, 240)
    end

    -- Draw zoom level
    DrawEx.TextAlpha('Unageo-Medium', string.format("Zoom: %.5f", state.zoom), 10,
        x + 10, y + sy - 25, 200, 14, 0.7, 0.7, 0.7, 0.7, 0.0, 0.5)

    RenderState.PopBlendMode()
end

--- Draw info panel for selected entity
---@param state table
---@param x number
---@param y number
---@param w number
function SystemMap:drawSelectedInfo(state, x, y, w)
    local entry = state.selected
    if not entry then return end

    local lineH = 18
    local ty = y

    DrawEx.Rect(x - 5, y - 5, w + 10, 140, Color(0.1, 0.1, 0.12, 0.9))

    local c = entry.color
    DrawEx.TextAlpha('Unageo-Medium', entry.label, 14, x, ty, w, lineH,
        c[1], c[2], c[3], 1.0, 0.0, 0.5)
    ty = ty + lineH + 4

    -- Position
    local pos = entry.pos
    DrawEx.TextAlpha('Unageo-Medium',
        string.format("Pos: (%.0f, %.0f, %.0f)", pos.x, pos.y, pos.z),
        10, x, ty, w, lineH, 0.8, 0.8, 0.8, 0.8, 0.0, 0.5)
    ty = ty + lineH

    -- Radius (real-world equivalent)
    local scale = entry.scale or 0
    DrawEx.TextAlpha('Unageo-Medium', "Radius: " .. formatDistance(scale), 10, x, ty, w, lineH,
        0.8, 0.8, 0.8, 0.8, 0.0, 0.5)
    ty = ty + lineH

    -- Distance from ship (real-world equivalent)
    if state.shipEntity then
        local shipRb = state.shipEntity:get(PhysicsComponents.RigidBody)
        if shipRb and shipRb:getRigidBody() then
            local shipPos = shipRb:getRigidBody():getPos()
            local dist = shipPos:distance(pos)
            DrawEx.TextAlpha('Unageo-Medium', "Distance: " .. formatDistance(dist), 10, x, ty, w, lineH,
                0.8, 0.8, 0.8, 0.8, 0.0, 0.5)
            ty = ty + lineH
        end
    end

    -- Orbit radius (real-world equivalent)
    if entry.orbitRadius and entry.orbitRadius > 0 then
        DrawEx.TextAlpha('Unageo-Medium', "Orbit: " .. formatDistance(entry.orbitRadius), 10, x, ty, w, lineH,
            0.8, 0.8, 0.8, 0.8, 0.0, 0.5)
    end
end

return SystemMap
