-- MemoryReporter — Lua-side memory analysis for the stats dashboard.
--
-- Walks the ECS registry once per sample interval and pushes a per-category
-- object census (name, count, bytes) into the Rust MemoryReport store,
-- served live at /memory.json by the stats server. Sampling the ACTUAL
-- object counts (entities by class, rigid bodies, meshes, textures,
-- asteroid data arrays) shows what is growing over time — the leak
-- signature — instead of one opaque RSS number.
--
-- The Lua VM's own heap is reported via collectgarbage('count'); the rest
-- are counted objects with per-object byte estimates.

local Registry = require('Core.ECS.Registry')
local Entity   = require('Core.ECS.Entity')

local MemoryReporter = {}

local SAMPLE_INTERVAL = 1.0  -- seconds between censuses

-- Rough per-object byte estimates (Lua tables + component payloads).
local BYTES = {
    entity       = 128,   -- Entity + component index entry
    rigidBody    = 256,   -- rapier RigidBody + collider (Rust-side)
    transform    = 64,
    render       = 96,    -- RenderComponent + mesh list
    mesh         = 4096,  -- GPU mesh + CPU vertex data (avg, varies)
    asteroidData = 64,    -- per-asteroid Lua struct {px,py,pz,scale,rotSeed}
    unknown      = 64,
}

local lastSample = 0
--- Count entities by class name (from tostring) and component totals.
---@return table<string, number> byClass
---@return number rigidBodies, renders, transforms, meshes
local function censusEntities()
    local byClass = {}
    local rigidBodies = 0
    local renders = 0
    local transforms = 0
    local meshes = 0

    -- Registry.entities: id -> component-index table (every live entity).
    -- Direct iteration is the only complete entity set; component views
    -- require at least one component type and would miss empty entities.
    local entities = Registry.entities or {}
    for id, compIndex in pairs(entities) do
        local name = tostring(Entity(id)):match("^(%w+)") or "Entity"
        byClass[name] = (byClass[name] or 0) + 1

        for compType in pairs(compIndex) do
            local t = tostring(compType):lower()
            if t:find("rigidbody") then rigidBodies = rigidBodies + 1
            elseif t:find("render") then
                renders = renders + 1
            elseif t:find("transform") then transforms = transforms + 1 end
        end
    end

    -- Mesh count: sum mesh lists on render components
    local RenderComp = require('Modules.Rendering.Components').Render
    for _, comp in Registry:view(RenderComp) do
        local list = comp:getMeshes()
        if list then meshes = meshes + #list end
    end

    return byClass, rigidBodies, renders, transforms, meshes
end

--- Report the current census into the Rust store.
function MemoryReporter.sample()
    -- Debug: log every sample so failures are visible in the game log
    -- (the pcall swallow hid a broken binding).
    local mrGlobal = _G.MemoryReport
    local mrGen = Core and Core.FFI and Core.FFI.Gen and Core.FFI.Gen.MemoryReport
    Log.Info('MemoryReporter: sampling, _G.MemoryReport=%s Core.FFI.Gen.MemoryReport=%s',
        tostring(mrGlobal), tostring(mrGen))

    local byClass, rigidBodies, renders, transforms, meshes = censusEntities()

    -- MemoryReport is the ffi_gen-generated binding (injected into the
    -- global namespace at Init, like Profiler). Static functions bind
    -- WITHOUT self - dot-call, not colon-call.
    MemoryReport.BeginFrame()

    -- Lua VM heap (KB -> bytes)
    local luaKb = collectgarbage('count')
    MemoryReport.Add('Lua heap (collectgarbage)', 1, math.floor(luaKb * 1024))

    -- Entity classes
    local totalEntities = 0
    for name, count in pairs(byClass) do
        totalEntities = totalEntities + count
        MemoryReport.Add('entities/' .. name, count, count * BYTES.entity)
    end
    MemoryReport.Add('entities total', totalEntities, totalEntities * BYTES.entity)

    -- Component-level counts
    MemoryReport.Add('components/RigidBody', rigidBodies, rigidBodies * BYTES.rigidBody)
    MemoryReport.Add('components/Render', renders, renders * BYTES.render)
    MemoryReport.Add('components/Transform', transforms, transforms * BYTES.transform)
    MemoryReport.Add('meshes', meshes, meshes * BYTES.mesh)
end

--- Per-frame hook: sample on a fixed interval (cheap otherwise).
--- Uses wall-clock time (Engine:getTime) - sim dt can be 0 (paused/
--- frame-locked states) and must not stall the census.
---@param dt number|table (ignored; kept for call-site compatibility)
function MemoryReporter.update(dt)
    local now = Engine:getTime()
    if now - lastSample < SAMPLE_INTERVAL then return end
    lastSample = now
    local ok, err = pcall(MemoryReporter.sample)
    if not ok then
        Log.Warn('MemoryReporter: sample failed: %s', tostring(err))
    end
end

return MemoryReporter
