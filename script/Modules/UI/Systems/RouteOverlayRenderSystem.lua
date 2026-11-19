local DrawEx              = require("UI.DrawEx")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")
local ScreenProjection    = require("Shared.Helpers.ScreenProjection")

--- RouteOverlayRenderSystem — draws an order's route (target, next waypoint, remaining path,
--- distance and ETA) through any world-to-screen mapping: the maps and the flight view share it.
---@class RouteOverlayRenderSystem
local RouteOverlayRenderSystem = {}

-- A route leg shorter than this on screen is not drawn.
local MIN_LEG_PIXELS = 5
-- A point projected farther than this from the origin of the screen (a point just in front of the eye)
-- is dropped, so a line never gets a coordinate that breaks the batcher.
local MAX_COORD_PIXELS = 1e5
local TARGET_CROSS_HALF = 6
local NEXT_CROSS_HALF = 4
local NEXT_DIAMOND_HALF = 7
local LABEL_GAP_PX = 2

local TARGET_COLOR = Color(1.0, 0.8, 0.3, 0.9)
local TARGET_LINE_COLOR = Color(1.0, 0.8, 0.3, 0.2)
local NEXT_COLOR = Color(0.3, 0.8, 1.0, 0.7)
local ROUTE_COLOR = Color(1.0, 1.0, 1.0, 0.6)

--- The pixel of `pos` through `toScreen(ctx, pos)`, or nil when it is hidden or absurdly far.
local function project(toScreen, ctx, pos)
    local px, py = toScreen(ctx, pos)
    if not px or math.abs(px) > MAX_COORD_PIXELS or math.abs(py) > MAX_COORD_PIXELS then return nil end
    return px, py
end

local function drawCross(x, y, half, color)
    DrawEx.SimpleRect(x - half, y - 0.5, half * 2, 1, color)
    DrawEx.SimpleRect(x - 0.5, y - half, 1, half * 2, color)
end

--- Draws the route polyline from the ship through the route's points, and returns the first leg drawn
--- (`{ x1, y1, x2, y2 }`) or nil.
local function drawPolyline(overlay, toScreen, ctx, shipX, shipY)
    local fromX, fromY = shipX, shipY
    local firstLeg
    for _, point in ipairs(overlay.route) do
        local toX, toY = project(toScreen, ctx, point)
        if not toX then
            fromX, fromY = nil, nil
        elseif not fromX then
            fromX, fromY = toX, toY
        elseif (toX - fromX) ^ 2 + (toY - fromY) ^ 2 > MIN_LEG_PIXELS ^ 2 then
            DrawEx.Line(fromX, fromY, toX, toY, ROUTE_COLOR)
            firstLeg = firstLeg or { shipX, shipY, toX, toY }
            fromX, fromY = toX, toY
        end
    end
    return firstLeg
end

--- Draws the overlay.
--- `overlay` is `{ shipPos, targetPos, route, progress, planning }` (`Position`s, the route from `Nav.route`,
--- the order's `OrderProgressComponent`); any of them may be nil. `toScreen(ctx, pos)` returns the pixel of a
--- position, or nil when it can't be shown.
---@param overlay table
---@param toScreen fun(ctx: any, pos: Position): number|nil, number|nil
---@param ctx any
function RouteOverlayRenderSystem:draw(overlay, toScreen, ctx)
    local route = overlay.route

    local targetX, targetY
    if overlay.targetPos then
        targetX, targetY = project(toScreen, ctx, overlay.targetPos)
        if targetX then
            drawCross(targetX, targetY, TARGET_CROSS_HALF, TARGET_COLOR)
            DrawEx.TextAlpha('Unageo-Medium', "TARGET", 9,
                targetX + TARGET_CROSS_HALF + LABEL_GAP_PX, targetY - 5, 80, 12, 1.0, 0.8, 0.3, 0.9, 0.0, 0.5)
        end
    end

    -- The next waypoint, when it is not the goal itself
    if route and #route >= 2 then
        local nextX, nextY = project(toScreen, ctx, route[1])
        if nextX then
            drawCross(nextX, nextY, NEXT_CROSS_HALF, NEXT_COLOR)
            DrawEx.TextAlpha('Unageo-Medium', "NEXT", 8,
                nextX + NEXT_CROSS_HALF + LABEL_GAP_PX, nextY - 5, 80, 10, 0.3, 0.8, 1.0, 0.7, 0.0, 0.5)
        end
    end

    if not (route and #route > 0 and overlay.shipPos) then return end
    local shipX, shipY = project(toScreen, ctx, overlay.shipPos)
    if not shipX then return end

    -- Dim line: ship to the actual target
    if targetX and (targetX - shipX) ^ 2 + (targetY - shipY) ^ 2 > MIN_LEG_PIXELS ^ 2 then
        DrawEx.Line(shipX, shipY, targetX, targetY, TARGET_LINE_COLOR)
    end

    local firstLeg = drawPolyline(overlay, toScreen, ctx, shipX, shipY)

    -- Distance and ETA left to the goal, as the order reports them, at the first leg's midpoint
    local progress = overlay.progress
    if firstLeg and progress then
        local timeStr = overlay.planning and "PLANNING"
            or ("ETA " .. UniverseScaleConfig:formatDuration(progress.eta))
        DrawEx.TextAlpha('Unageo-Medium',
            UniverseScaleConfig:formatDistance(progress.distanceLeft) .. " | " .. timeStr, 10,
            (firstLeg[1] + firstLeg[3]) * 0.5 - 60, (firstLeg[2] + firstLeg[4]) * 0.5 - 14, 200, 12,
            1.0, 1.0, 1.0, 0.8, 0.5, 0.5)
    end
end

--- Draws a diamond and "NEXT <distance>" at `pos` in the flight view, through the projector. Draws nothing
--- when the point is behind the eye or outside the viewport (no edge arrow).
---@param projector ScreenProjector
---@param pos Position the next waypoint
---@param shipPos Position the distance is measured from
function RouteOverlayRenderSystem:drawNextMarker(projector, pos, shipPos)
    local px, py = ScreenProjection.toScreen(projector, pos)
    if not px or not ScreenProjection.inViewport(projector, px, py) then return end

    local h = NEXT_DIAMOND_HALF
    DrawEx.SimpleTri(px, py - h, px + h, py, px, py + h, NEXT_COLOR)
    DrawEx.SimpleTri(px, py - h, px, py + h, px - h, py, NEXT_COLOR)
    DrawEx.TextAlpha('Unageo-Medium', "NEXT " .. UniverseScaleConfig:formatDistance(pos:distance(shipPos)), 10,
        px + h + LABEL_GAP_PX, py - 6, 160, 12, 0.3, 0.8, 1.0, 0.9, 0.0, 0.5)
end

return RouteOverlayRenderSystem
