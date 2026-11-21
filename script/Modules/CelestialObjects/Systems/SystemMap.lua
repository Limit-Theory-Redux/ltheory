local CoreComponents           = require("Modules.Core.Components")
local PhysicsComponents        = require("Modules.Physics.Components")
local SpatialComponents        = require("Modules.Spatial.Components")
local CelestialComponents      = require("Modules.CelestialObjects.Components")
local UniverseScaleConfig      = require("Config.Gen.UniverseScaleConfig")
local DrawEx                   = require("UI.DrawEx")
local ConstructsComponents     = require("Modules.Constructs.Components")
local Registry                 = require("Core.ECS.Registry")
local EntityPosition           = require("Shared.Helpers.EntityPosition")
local Pipelines                = require("Render.Pipelines")
local RouteOverlayRenderSystem = require("Modules.UI.Systems.RouteOverlayRenderSystem")
local GravityWellSystem        = require("Modules.Physics.Systems.GravityWellSystem")
local PlannerSnapshot          = require("Shared.Helpers.PlannerSnapshot")

-- Scratch color for the shapes below (the batcher copies it at the call).
local shapeColor = Color(0, 0, 0, 0)

-- Pipeline state of the asteroid dots: the dot mesh, alpha blended.
local dotState = { blend = BlendMode.Alpha }
local dotParamsType

local function formatDistance(gameUnits)
    return UniverseScaleConfig:formatDistance(gameUnits)
end

--- The map's top-down mapping for `RouteOverlayRenderSystem`: world x to screen x, world z to screen y.
---@param mapping table { state, cx, cy }
local function mapToScreen(mapping, pos)
    local state = mapping.state
    return mapping.cx + (pos.x + state.panX) * state.zoom, mapping.cy + (pos.z + state.panY) * state.zoom
end

-- The circles drawn under a body's point: its size, the distance the planner keeps a path outside of
-- (`PlannerSnapshot.safeRadius`) and its gravity well. All are alpha-blended annuli.
local BODY_DISC_ALPHA  = 0.3
local RING_WIDTH_PX    = 3
local MIN_CIRCLE_PX    = 2    -- a circle of a smaller radius is not drawn
local ANNULUS_FEATHER  = 2    -- the annulus shader's soft edge, in pixels
local ANNULUS_PAD_PX   = 4    -- the quad's margin around the circle, at least the feather
local MAX_QUAD_SCREENS = 4    -- a circle whose quad is larger than this many screens is not drawn
local MOON_HIDE_PX     = 20   -- a moon whose orbit is smaller on screen than this is hidden
local SAFE_RING_COLOR  = { r = 1.0, g = 0.35, b = 0.3, a = 0.55 }
local WELL_RING_COLOR  = { r = 0.3, g = 0.85, b = 1.0, a = 0.45 }
local LEGEND_LINE_PX   = 14
local LEGEND = {
    { text = "Disc: the body's size",                 color = { r = 0.8, g = 0.8, b = 0.8 } },
    { text = "Red ring: the path keeps outside it",   color = SAFE_RING_COLOR },
    { text = "Blue ring: gravity well",               color = WELL_RING_COLOR },
}

--- What the map draws, per kind marker. A belt and a ring share the belt drawing (annulus + dots).
--- `bodyKind` is the planner's kind of a body (`Config.game.autoPilot.safeRadiusFactor`): its circles are drawn.
local MAP_KINDS = {
    { marker = CelestialComponents.Star,         label = "Star",          color = { 1.0, 0.9, 0.3 }, size = 8, bodyKind = "star" },
    { marker = CelestialComponents.Planet,       label = "Planet",        color = { 0.3, 0.6, 1.0 }, size = 5, numbered = true, bodyKind = "planet" },
    { marker = CelestialComponents.Moon,         label = "Moon",          color = { 0.6, 0.6, 0.6 }, size = 3, isMoon = true, bodyKind = "moon" },
    { marker = ConstructsComponents.SpaceStation, label = "Station",      color = { 1.0, 0.5, 0.1 }, size = 4, bodyKind = "station" },
    { marker = CelestialComponents.AsteroidBelt, label = "Asteroid Belt", color = { 0.7, 0.5, 0.3 }, size = 3, isBelt = true },
    { marker = CelestialComponents.AsteroidRing, label = "Asteroid Ring", color = { 0.6, 0.5, 0.4 }, size = 3, isBelt = true },
    { marker = CelestialComponents.Asteroid,     label = "Asteroid",      color = { 0.7, 0.5, 0.3 }, size = 2 },
}

--- Whether a moon is hidden at this zoom: its orbit is small on screen, so it sits on its planet.
---@param state table
---@param entry table
---@return boolean
local function isHiddenMoon(state, entry)
    return entry.isMoon and entry.parentPos ~= nil and entry.orbitRadius * state.zoom < MOON_HIDE_PX
end

--- Draws an annulus of `innerR` to `outerR` pixels around `(px, py)` in `shapeColor`, on the map area
--- `(x, y, sx, sy)`. Draws nothing, and returns true, when it is off the area. Returns false when its quad
--- would be larger than `MAX_QUAD_SCREENS` screens (a quad that large costs fill rate), and draws nothing.
local function drawAnnulus(px, py, innerR, outerR, x, y, sx, sy)
    if px + outerR < x or px - outerR > x + sx or py + outerR < y or py - outerR > y + sy then return true end
    local size = 2 * (outerR + ANNULUS_PAD_PX)
    if size >= math.max(sx, sy) * MAX_QUAD_SCREENS then return false end
    Imm.Shape(Shape.Annulus, px - size * 0.5, py - size * 0.5, size, size, shapeColor, innerR, outerR)
    return true
end

--- Whether the whole map area `(x, y, sx, sy)` lies inside the circle of radius `r` around `(px, py)`.
local function areaInsideCircle(px, py, r, x, y, sx, sy)
    local dx = math.max(math.abs(px - x), math.abs(px - (x + sx)))
    local dy = math.max(math.abs(py - y), math.abs(py - (y + sy)))
    return dx * dx + dy * dy < r * r
end

--- Draws a ring `RING_WIDTH_PX` wide at pixel radius `r` in `color` (`{ r, g, b, a }`), unless `r` is too small.
local function drawRing(px, py, r, color, x, y, sx, sy)
    if r < MIN_CIRCLE_PX then return end
    shapeColor.r, shapeColor.g, shapeColor.b, shapeColor.a = color.r, color.g, color.b, color.a
    drawAnnulus(px, py, r - RING_WIDTH_PX * 0.5, r + RING_WIDTH_PX * 0.5, x, y, sx, sy)
end

--- Draws the circles of a body of the planner (`entry.bodyKind`) under its point: a translucent disc
--- of its size, a ring at the distance the planner keeps a path outside of, and a ring at its gravity well.
local function drawBodyCircles(state, entry, cx, cy, x, y, sx, sy)
    local zoom = state.zoom
    local px = cx + (entry.pos.x + state.panX) * zoom
    local py = cy + (entry.pos.z + state.panY) * zoom

    local bodyR = entry.scale * zoom
    if bodyR >= MIN_CIRCLE_PX then
        local c = entry.color
        shapeColor.r, shapeColor.g, shapeColor.b, shapeColor.a = c[1], c[2], c[3], BODY_DISC_ALPHA
        -- An inner radius of minus the feather fills the centre (the shader feathers around the radius).
        local drawn = drawAnnulus(px, py, -ANNULUS_FEATHER, bodyR, x, y, sx, sy)
        -- A body that fills the map area is too large for a quad: the area is its disc.
        if not drawn and areaInsideCircle(px, py, bodyR, x, y, sx, sy) then
            DrawEx.SimpleRect(x, y, sx, sy, Color(c[1], c[2], c[3], BODY_DISC_ALPHA))
        end
    end
    drawRing(px, py, PlannerSnapshot.safeRadius(entry.scale, entry.bodyKind) * zoom, SAFE_RING_COLOR, x, y, sx, sy)
    if entry.wellRadius then
        drawRing(px, py, entry.wellRadius * zoom, WELL_RING_COLOR, x, y, sx, sy)
    end
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
        autopilotOverlay = nil,   -- the autopilot's target, route and progress for `RouteOverlayRenderSystem`, or nil
        dragging   = false,
        lastMouseX = 0,
        lastMouseY = 0,
    }
end

--- Collect all entities with positions, one registry view per kind
---@param state table
---@param shipEntity Entity|nil
function SystemMap:collectEntities(state, shipEntity)
    -- Full collection only on first call or explicit invalidation
    if not state._entityListBuilt then
        state.entities = {}
        state.shipEntity = shipEntity
        state._planetIdx = 0
        state._wellRadii = {}
        for _, zone in ipairs(GravityWellSystem.zones) do state._wellRadii[zone.zoneEntity.id] = zone.radius end
        for _, kind in ipairs(MAP_KINDS) do
            self:_collectKind(state, kind)
        end

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
        state._entityListBuilt = true
    end

    -- Cheap per-frame position update (no tree walk)
    for _, entry in ipairs(state.entities) do
        local rbCmp = entry.entity:get(PhysicsComponents.RigidBody)
        if rbCmp and rbCmp:getRigidBody() then
            entry.pos = rbCmp:getRigidBody():getPos()
        elseif not entry.isBelt then
            local transform = entry.entity:get(PhysicsComponents.Transform)
            if transform then entry.pos = transform:getPos() end
        end
        -- Update parent position too (for rings/moons)
        if entry.parentPos and entry._parentEntity then
            local pRb = entry._parentEntity:get(PhysicsComponents.RigidBody)
            if pRb and pRb:getRigidBody() then
                entry.parentPos = pRb:getRigidBody():getPos()
            end
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
---@param kind table An entry of MAP_KINDS
function SystemMap:_collectKind(state, kind)
    for entity in Registry:view(kind.marker) do
        local transform = entity:get(PhysicsComponents.Transform)
        if transform then
            local pos = transform:getPos()
            local scale = transform:getScale()
            local rbCmp = entity:get(PhysicsComponents.RigidBody)
            if rbCmp and rbCmp:getRigidBody() then
                pos = rbCmp:getRigidBody():getPos()
                scale = rbCmp:getRigidBody():getBoundingRadius()
            end

            local label = kind.label
            if kind.numbered then
                state._planetIdx = state._planetIdx + 1
                label = label .. " " .. state._planetIdx
            end

            local orbitCmp = entity:get(SpatialComponents.Orbit)
            local orbitRadius = orbitCmp and orbitCmp:getOrbitRadius() or 0
            -- A ring has no Orbit component: its radius is in the belt data
            if orbitRadius == 0 and kind.isBelt then
                local beltCmp = entity:get(CelestialComponents.AsteroidBeltData)
                if beltCmp then orbitRadius = beltCmp:getOrbitRadius() end
            end

            local parentCmp = entity:get(CoreComponents.Parent)
            local parentEntity = parentCmp and parentCmp:getParent()
            local widthCmp = kind.isBelt and entity:get(SpatialComponents.Width)

            table.insert(state.entities, {
                entity        = entity,
                label         = label,
                pos           = pos,
                scale         = scale,
                bodyKind      = kind.bodyKind,
                wellRadius    = kind.bodyKind and state._wellRadii[entity.id] or nil,
                orbitRadius   = orbitRadius,
                color         = kind.color,
                size          = kind.size,
                isMoon        = kind.isMoon or false,
                isBelt        = kind.isBelt or false,
                beltWidth     = kind.isBelt and (widthCmp and widthCmp:getWidth() or 0) or nil,
                parentPos     = parentEntity and EntityPosition.Get(parentEntity) or nil,
                _parentEntity = parentEntity,
            })
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
        -- Update clickPos for ring asteroids (parent planet moves)
        if state.selected._clickLocalX and state.selected.parentPos then
            state.selected.clickPos = Position(
                state.selected.parentPos.x + state.selected._clickLocalX,
                0,
                state.selected.parentPos.z + state.selected._clickLocalZ)
        end

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
                local beltCmp = entry.entity:get(CelestialComponents.AsteroidBeltData)
                local beltScreenR = entry.orbitRadius * state.zoom
                -- Center on parent (planet for rings, star for belts)
                local ocx, ocy
                if entry.parentPos then
                    ocx = hx + (entry.parentPos.x + state.panX) * state.zoom
                    ocy = hy + (entry.parentPos.z + state.panY) * state.zoom
                else
                    ocx = hx + state.panX * state.zoom
                    ocy = hy + state.panY * state.zoom
                end

                -- Check individual asteroid dots
                local isRing = entry.parentPos ~= nil
                local dotClickThreshold = isRing and 20 or (math.min(Window:width(), Window:height()) * 2)
                if beltCmp and beltScreenR > dotClickThreshold then
                    local asteroids = beltCmp:getAsteroidData()
                    local mdxB = mp.x - ocx
                    local mdyB = mp.y - ocy
                    local mouseAngle = math.atan2(mdyB, mdxB)
                    local indices = beltCmp:getAsteroidsInAngleRange(mouseAngle - 0.1, mouseAngle + 0.1)
                    local bestAsteroidDist = 400
                    for _, i in ipairs(indices) do
                        local a = asteroids[i]
                        local ax = ocx + a.px * state.zoom
                        local ay = ocy + a.pz * state.zoom
                        local ad = (mp.x - ax)^2 + (mp.y - ay)^2
                        if ad < bestAsteroidDist and ad < bestDist then
                            bestDist = ad
                            bestEntry = entry
                            local ctrX = entry.parentPos and entry.parentPos.x or 0
                            local ctrZ = entry.parentPos and entry.parentPos.z or 0
                            bestClickPos = Position(ctrX + a.px, a.py, ctrZ + a.pz)
                            bestAsteroidDist = ad
                        end
                    end
                end

                -- Annulus ring click fallback
                local mdx = mp.x - ocx
                local mdy = mp.y - ocy
                local mouseDistFromCenter = math.sqrt(mdx * mdx + mdy * mdy)
                local beltScreenW = (entry.beltWidth or entry.orbitRadius * 0.1) * state.zoom
                local innerR = beltScreenR - beltScreenW * 0.5
                local outerR = beltScreenR + beltScreenW * 0.5
                local clickPad = 10
                if mouseDistFromCenter >= innerR - clickPad and mouseDistFromCenter <= outerR + clickPad then
                    local d = 10000  -- very low priority, dots always win
                    if d < bestDist then
                        bestDist = d
                        local angle = math.atan2(mdy, mdx)
                        local centerX = entry.parentPos and entry.parentPos.x or 0
                        local centerZ = entry.parentPos and entry.parentPos.z or 0
                        bestEntry = entry
                        bestClickPos = Position(
                            centerX + math.cos(angle) * entry.orbitRadius,
                            0,
                            centerZ + math.sin(angle) * entry.orbitRadius)
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
            bestEntry.clickPos = bestClickPos
            if bestClickPos and bestEntry.parentPos then
                bestEntry._clickLocalX = bestClickPos.x - bestEntry.parentPos.x
                bestEntry._clickLocalZ = bestClickPos.z - bestEntry.parentPos.z
            else
                bestEntry._clickLocalX = nil
                bestEntry._clickLocalZ = nil
            end
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

                local r = ringScreenRadius
                -- Skip if ring quad is off screen or too large
                local maxQuadSize = math.max(sx, sy) * 4
                if r * 2 < maxQuadSize
                   and ocx + r > x and ocx - r < x + sx
                   and ocy + r > y and ocy - r < y + sy then
                    local pad = 8
                    shapeColor.r, shapeColor.g, shapeColor.b, shapeColor.a = c.r, c.g, c.b, c.a
                    Imm.Shape(Shape.Ring, ocx - r - pad, ocy - r - pad, 2 * r + 2 * pad, 2 * r + 2 * pad,
                        shapeColor, r)
                end
            end
        end
    end

    -- Draw asteroid belt/ring as solid filled annulus
    for _, entry in ipairs(state.entities) do
        if entry.isBelt and entry.orbitRadius and entry.orbitRadius > 0 then
            local beltScreenRadius = entry.orbitRadius * state.zoom
            local beltWidth = (entry.beltWidth or entry.orbitRadius * 0.1) * state.zoom
            if beltScreenRadius > 2 then
                local ocx, ocy
                if entry.parentPos then
                    ocx = cx + (entry.parentPos.x + state.panX) * state.zoom
                    ocy = cy + (entry.parentPos.z + state.panY) * state.zoom
                else
                    ocx = cx + state.panX * state.zoom
                    ocy = cy + state.panY * state.zoom
                end

                local innerR = math.max(1, beltScreenRadius - beltWidth * 0.5)
                local outerR = beltScreenRadius + beltWidth * 0.5

                -- Skip if annulus quad is entirely off screen
                if ocx + outerR > x and ocx - outerR < x + sx
                   and ocy + outerR > y and ocy - outerR < y + sy then
                    -- Cap quad size to avoid GPU fillrate issues on huge rings
                    local maxQuadSize = math.max(sx, sy) * 4
                    if outerR * 2 < maxQuadSize then
                        local pad = 4
                        local totalSize = 2 * outerR + 2 * pad
                        shapeColor.r, shapeColor.g, shapeColor.b, shapeColor.a =
                            entry.color[1], entry.color[2], entry.color[3], 0.25
                        Imm.Shape(Shape.Annulus, ocx - outerR - pad, ocy - outerR - pad, totalSize, totalSize,
                            shapeColor, innerR, outerR)
                    end
                end
            end

            -- Draw individual asteroid dots — only when ring is visible on screen
            local beltCmp2 = entry.entity:get(CelestialComponents.AsteroidBeltData)
            -- Rings: show dots like moons (orbit > 20px). Belts: only at very high zoom.
            local isRing = entry.parentPos ~= nil
            local dotThreshold = isRing and 20 or (math.min(sx, sy) * 2)
            if beltCmp2 and beltScreenRadius > dotThreshold then
                local dotOcx, dotOcy
                if entry.parentPos then
                    dotOcx = cx + (entry.parentPos.x + state.panX) * state.zoom
                    dotOcy = cy + (entry.parentPos.z + state.panY) * state.zoom
                else
                    dotOcx = cx + state.panX * state.zoom
                    dotOcy = cy + state.panY * state.zoom
                end

                -- Skip if ring center is far off screen (no dots visible)
                local rOuter = beltScreenRadius + (entry.beltWidth or 0) * state.zoom * 0.5
                if dotOcx + rOuter < x or dotOcx - rOuter > x + sx
                    or dotOcy + rOuter < y or dotOcy - rOuter > y + sy then
                    goto skip_dots
                end

                -- Static dot mesh — built ONCE per belt/ring, never rebuilt
                local asteroids = beltCmp2:getAsteroidData()
                local cacheKey = tostring(entry.entity)
                if not state._dotCache then state._dotCache = {} end
                local cache = state._dotCache[cacheKey]

                if not cache then
                    -- Build mesh once with world-space positions (asteroid local x,z)
                    local dotMesh = Mesh.Create()
                    local dotCount = 0
                    for i = 1, #asteroids do
                        local a = asteroids[i]
                        -- All 4 verts at same world pos; UV encodes corner (0/1)
                        local vi = dotCount * 4
                        dotMesh:addVertex(a.px, a.pz, 0, 0, 0, 1, 0, 0)
                        dotMesh:addVertex(a.px, a.pz, 0, 0, 0, 1, 1, 0)
                        dotMesh:addVertex(a.px, a.pz, 0, 0, 0, 1, 1, 1)
                        dotMesh:addVertex(a.px, a.pz, 0, 0, 0, 1, 0, 1)
                        dotMesh:addQuad(vi, vi + 1, vi + 2, vi + 3)
                        dotCount = dotCount + 1
                    end
                    state._dotCache[cacheKey] = { mesh = dotMesh, count = dotCount }
                    cache = state._dotCache[cacheKey]
                end

                if cache.count > 0 then
                    local dotShader = Cache.Shader('mappoints', 'ui/mappoints')
                    local pass = Renderer:currentPass()
                    pass:setPipeline(Pipelines.get(dotShader, dotState))
                    dotParamsType = dotParamsType or dotShader:blockType('Params')
                    local p = pass:alloc(dotParamsType)
                    p.mapGeom.x, p.mapGeom.y, p.mapGeom.z, p.mapGeom.w = dotOcx, dotOcy, sx, sy
                    p.mapParams.x, p.mapParams.y = state.zoom, 3.0
                    p.dotColor.x, p.dotColor.y, p.dotColor.z, p.dotColor.w = 0.8, 0.6, 0.3, 0.7
                    pass:drawMesh(cache.mesh)
                end
            end
            ::skip_dots::

            -- Highlight selected asteroid
            if state.selected and state.selected.clickPos and state.selected.entity == entry.entity then
                local cp = state.selected.clickPos
                local ctrX = entry.parentPos and entry.parentPos.x or 0
                local ctrZ = entry.parentPos and entry.parentPos.z or 0
                local dotOcx = cx + (ctrX + state.panX) * state.zoom
                local dotOcy = cy + (ctrZ + state.panY) * state.zoom
                local ax = dotOcx + cp.x * state.zoom
                local ay = dotOcy + cp.z * state.zoom
                DrawEx.SimpleRect(ax - 3, ay - 3, 6, 6, Color(1.0, 0.8, 0.3, 1.0))
            end
        end
    end

    -- Body circles, under every point and label
    for _, entry in ipairs(state.entities) do
        if entry.bodyKind and not isHiddenMoon(state, entry) then
            drawBodyCircles(state, entry, cx, cy, x, y, sx, sy)
        end
    end

    -- Draw entities
    for _, entry in ipairs(state.entities) do
        -- Hide moons unless zoomed in close to their parent planet
        if isHiddenMoon(state, entry) then goto next_map_entity end

        -- Belt entities: draw label at top of ring, not as a point
        if entry.isBelt and entry.orbitRadius and entry.orbitRadius > 0 then
            local beltScreenR = entry.orbitRadius * state.zoom
            if beltScreenR > 10 then
                local ocx, ocy
                if entry.parentPos then
                    ocx = cx + (entry.parentPos.x + state.panX) * state.zoom
                    ocy = cy + (entry.parentPos.z + state.panY) * state.zoom
                else
                    ocx = cx + state.panX * state.zoom
                    ocy = cy + state.panY * state.zoom
                end
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

    -- The autopilot's target, next waypoint, route and distance/ETA, in the map's top-down mapping
    local overlay = state.autopilotOverlay
    if overlay then
        local mapping = state._mapping or {}
        state._mapping = mapping
        mapping.state, mapping.cx, mapping.cy = state, cx, cy
        RouteOverlayRenderSystem:draw(overlay, mapToScreen, mapping)
    end

    -- Selection marker (when no autopilot, just map selection)
    if not (overlay and overlay.targetPos) and state.selected and state.selected.clickPos then
        local cp = state.selected.clickPos
        local cpx = cx + (cp.x + state.panX) * state.zoom
        local cpy = cy + (cp.z + state.panY) * state.zoom
        local ms = 6
        DrawEx.SimpleRect(cpx - ms, cpy - 0.5, ms * 2, 1, Color(1.0, 0.8, 0.3, 0.9))
        DrawEx.SimpleRect(cpx - 0.5, cpy - ms, 1, ms * 2, Color(1.0, 0.8, 0.3, 0.9))
        DrawEx.TextAlpha('Unageo-Medium', "SELECTED", 9,
            cpx + ms + 2, cpy - 5, 80, 12, 1.0, 0.8, 0.3, 0.9, 0.0, 0.5)
    end

    -- Draw selected entity info panel
    if state.selected then
        self:drawSelectedInfo(state, x + sx - 250, y + 40, 240)
    end

    -- Legend of the body circles
    for i, line in ipairs(LEGEND) do
        local c = line.color
        DrawEx.TextAlpha('Unageo-Medium', line.text, 10,
            x + 10, y + sy - 25 - LEGEND_LINE_PX * (#LEGEND - i + 1), 240, 14, c.r, c.g, c.b, 0.9, 0.0, 0.5)
    end

    -- Draw zoom level
    DrawEx.TextAlpha('Unageo-Medium', string.format("Zoom: %.5f", state.zoom), 10,
        x + 10, y + sy - 25, 200, 14, 0.7, 0.7, 0.7, 0.7, 0.0, 0.5)
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
