---@class HullEngineDiscovery
---@overload fun(): HullEngineDiscovery
local HullEngineDiscovery = Class("HullEngineDiscovery", function() end)

local DEFAULT_AFT = Vec3f(0, 0, 1)

---Collect surface-triangle candidates whose normal faces roughly the
---aft direction (the stern of the hull), in mesh-local coordinates.
---Same triangle scan technique as HullMountDiscovery - works on any
---procedurally generated hull, no assumptions about ship shape.
---@param mesh Mesh
---@param aft Vec3f
---@param minDot number
---@return table[] candidates {position=Vec3f, surfaceNormal=Vec3f}
local function collectAftCandidates(mesh, aft, minDot)
    local aftLength = math.sqrt(aft.x * aft.x + aft.y * aft.y + aft.z * aft.z)
    assert(aftLength > 0.000001, "hull engine discovery requires a non-zero aft direction")

    local vertices = {}
    for index = 0, mesh:getVertexCount() - 1 do
        local vertex = mesh:getVertex(index)
        vertices[index] = {
            x = vertex.px,
            y = vertex.py,
            z = vertex.pz,
        }
    end

    local candidates = {}
    local indexCount = mesh:getIndexCount()
    mesh:lockIndexData(function(indices)
        for index = 0, indexCount - 3, 3 do
            local p0 = vertices[tonumber(indices[index])]
            local p1 = vertices[tonumber(indices[index + 1])]
            local p2 = vertices[tonumber(indices[index + 2])]
            if p0 and p1 and p2 then
                local e1x = p1.x - p0.x
                local e1y = p1.y - p0.y
                local e1z = p1.z - p0.z
                local e2x = p2.x - p0.x
                local e2y = p2.y - p0.y
                local e2z = p2.z - p0.z
                local nx = e1y * e2z - e1z * e2y
                local ny = e1z * e2x - e1x * e2z
                local nz = e1x * e2y - e1y * e2x
                local normalLength = math.sqrt(nx * nx + ny * ny + nz * nz)
                if normalLength > 0.000001 then
                    local dot = (nx * aft.x + ny * aft.y + nz * aft.z)
                        / (normalLength * aftLength)
                    if dot >= minDot then
                        table.insert(candidates, {
                            position = Vec3f(
                                (p0.x + p1.x + p2.x) / 3.0,
                                (p0.y + p1.y + p2.y) / 3.0,
                                (p0.z + p1.z + p2.z) / 3.0),
                            surfaceNormal = Vec3f(
                                nx / normalLength,
                                ny / normalLength,
                                nz / normalLength),
                        })
                    end
                end
            end
        end
    end)
    return candidates
end

---@private
---Discover engine nozzle connection points on a procedural hull: find
---the aft-facing stern surfaces, then pick one connection point per
---requested engine (default a port/starboard pair), rear-most and
---symmetric about the hull plane where possible.
---
---Mesh-local output: each engine is {position, direction} where
---direction is the outward jet axis (the hull-local stern axis, always).
---@param mesh Mesh Generated hull mesh in local coordinates
---@param options table|nil {aft=Vec3f, minNormalDot=number, count=integer,
--- symmetry=boolean, rearFraction=number, engineX=number, engineY=number}
---@return table[] engines {position=Vec3f, direction=Vec3f}
function HullEngineDiscovery:discover(mesh, options)
    assert(mesh, "hull engine discovery requires a generated mesh")

    options = options or {}
    -- Ship-gen convention: hull-local forward is -Z, so the stern/engines
    -- face +Z. Callers with a different hull convention can pass their own.
    local aft = options.aft or DEFAULT_AFT
    local minNormalDot = options.minNormalDot or 0.30
    local count = options.count or 2
    local symmetry = options.symmetry ~= false
    local rearFraction = options.rearFraction or 0.35
    local engineX = options.engineX  -- nil = derive from hull
    local engineY = options.engineY  -- nil = hull centerline height

    local candidates = collectAftCandidates(mesh, aft, minNormalDot)
    if #candidates == 0 then
        return {}
    end

    -- Hull extents (mesh-local) for zone filtering and targets.
    local minX, maxX = math.huge, -math.huge
    local minY, maxY = math.huge, -math.huge
    local minZ, maxZ = math.huge, -math.huge
    for _, candidate in ipairs(candidates) do
        local p = candidate.position
        minX, maxX = math.min(minX, p.x), math.max(maxX, p.x)
        minY, maxY = math.min(minY, p.y), math.max(maxY, p.y)
        minZ, maxZ = math.min(minZ, p.z), math.max(maxZ, p.z)
    end
    local width = math.max(maxX - minX, 0.0001)
    local height = math.max(maxY - minY, 0.0001)
    local rearLimit = minZ + (maxZ - minZ) * rearFraction

    -- Engine row targets: the stern, off-center for the pair, at the
    -- centerline height (typical engine row placement).
    local targetX = engineX or width * 0.35
    local targetY = engineY or (minY + maxY) * 0.5 + height * 0.05

    -- Pick one rear-most candidate per engine, alternating sides of the
    -- hull plane for a symmetric pair.
    local picked = {}
    local used = {}
    for engineIndex = 1, count do
        local sideSign = (engineIndex % 2 == 1) and -1 or 1
        local best, bestScore = nil, math.huge
        for index, candidate in ipairs(candidates) do
            if not used[index] then
                local p = candidate.position
                local onSide = sideSign < 0 and p.x < 0 or sideSign > 0 and p.x > 0
                if onSide and p.z <= rearLimit then
                    local score = math.abs(p.z - minZ) * 2.0    -- rear-most wins
                        + math.abs(math.abs(p.x) - targetX) * 0.8
                        + math.abs(p.y - targetY) * 0.6
                    if score < bestScore then
                        best, bestScore = candidate, index
                    end
                end
            end
        end
        if not best then
            -- No candidate on this side: fall back to any remaining
            -- stern candidate (asymmetric hulls).
            for index, candidate in ipairs(candidates) do
                if not used[index] and candidate.position.z <= rearLimit then
                    if not best or candidate.position.z < best.position.z then
                        best, bestScore = candidate, index
                    end
                end
            end
        end
        if best then
            used[bestScore] = true
            table.insert(picked, {
                position = best.position,
                surfaceNormal = best.surfaceNormal,
            })
        end
    end

    -- Mirror each pair across the hull plane (x -> -x), FORCE applying
    -- the exact mirror of the left engine: same y/z (never stacked
    -- vertically), flipped x (one left, one right). Conditional mirrors
    -- leave asymmetric stern faces unmatched, which put both engines on
    -- one side.
    if symmetry and #picked >= 2 then
        for pair = 1, math.floor(#picked / 2) do
            local left = picked[(pair * 2) - 1]
            local right = picked[pair * 2]
            right.position = Vec3f(-left.position.x, left.position.y, left.position.z)
            right.surfaceNormal = Vec3f(-left.surfaceNormal.x, left.surfaceNormal.y,
                                        left.surfaceNormal.z)
        end
    end

    local engines = {}
    local nAft = Vec3f(aft.x, aft.y, aft.z)
    local nAftLen = math.sqrt(nAft.x * nAft.x + nAft.y * nAft.y + nAft.z * nAft.z)
    for _, engine in ipairs(picked) do
        table.insert(engines, {
            position = Vec3f(engine.position.x, engine.position.y, engine.position.z),
            -- Jets always stream along the stern AXIS, not the picked
            -- face's surface normal (a top-tilted face would blast the
            -- exhaust upward instead of backward).
            direction = Vec3f(nAft.x / nAftLen, nAft.y / nAftLen, nAft.z / nAftLen),
        })
    end
    return engines
end

return HullEngineDiscovery
