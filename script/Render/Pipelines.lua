--- Pipelines for code that draws into a render pass without a `Material`.
---
--- A pipeline is a shader plus fixed-function state. Draws inside the scene
--- passes (skybox, effects, belts, ...) take their state from one of the
--- `Pipelines.*` state tables below and fetch the pipeline with
--- `Pipelines.get(shader, state)`. The result is cached per shader resource
--- (a hot reload swaps it) and per state table, so the call is two table
--- lookups; the Rust side hashes the full description once.
---
--- State tables are compared by identity: define one per distinct state at
--- module level instead of building a table per call.

---@class PipelineState
---@field blend BlendMode|nil      default Disabled
---@field cull CullFace|nil        default None
---@field depthTest boolean|nil    default false
---@field depthWrite boolean|nil   default true
---@field depthFunc CompareFn|nil  default LessEqual
---@field topology Topology|nil    default Triangles
---@field vertex VertexLayout|nil  default Mesh

local Pipelines = {}

-- The scene states are built on first use: this module is loaded by
-- `requireAll` before `Config` exists, and the opaque pass reads
-- `Config.render.renderState.cullFace`.
local function buildSceneStates()
    local opaqueCull = Config.render.renderState.cullFace and CullFace.Back or CullFace.None

    --- State of the scene passes, in the order they run. Materials draw with
    --- the table of their blend mode; code with its own shaders picks a pass
    --- table or one of the variants.
    ---@type PipelineState
    local opaque = { blend = BlendMode.Disabled, cull = opaqueCull, depthTest = true, depthWrite = true }
    ---@type PipelineState
    local additive = { blend = BlendMode.Additive, cull = CullFace.None, depthTest = true, depthWrite = false }
    ---@type PipelineState
    local alpha = { blend = BlendMode.Alpha, cull = CullFace.None, depthTest = true, depthWrite = false }
    return {
        Opaque = opaque,
        Additive = additive,
        Alpha = alpha,
        --- Opaque pass, but no culling and no depth writes (skyboxes). The box is
        --- drawn through the immediate batcher (`Imm.Box3`).
        ---@type PipelineState
        OpaqueBackdrop = {
            blend = BlendMode.Disabled, cull = CullFace.None, depthTest = true, depthWrite = false,
            vertex = VertexLayout.Imm3D,
        },
        --- The scene state of a blend mode.
        ---@type table<BlendMode, PipelineState>
        Scene = {
            [BlendMode.Disabled] = opaque,
            [BlendMode.Additive] = additive,
            [BlendMode.Alpha] = alpha,
        },
    }
end

setmetatable(Pipelines, {
    __index = function(t, key)
        local states = buildSceneStates()
        for k, v in pairs(states) do rawset(t, k, v) end
        return rawget(t, key)
    end,
})

-- shader resource id -> state table -> PipelineId
local cache = {}

--- The `PipelineId` of `shader` drawing with `state`.
---@param shader Shader
---@param state PipelineState
---@return integer
function Pipelines.get(shader, state)
    local resource = tonumber(shader:resourceId())
    local byState = cache[resource]
    if not byState then
        byState = setmetatable({}, { __mode = 'k' })
        cache[resource] = byState
    end
    local pipeline = byState[state]
    if pipeline then return pipeline end

    local desc = PipelineDesc.Create(shader)
    desc:blend(state.blend or BlendMode.Disabled)
    desc:cull(state.cull or CullFace.None)
    desc:depth(state.depthTest or false, state.depthWrite ~= false, state.depthFunc or CompareFn.LessEqual)
    if state.topology then desc:topology(state.topology) end
    if state.vertex then desc:vertex(state.vertex) end
    pipeline = Pipeline.Get(desc)
    byState[state] = pipeline
    return pipeline
end

return Pipelines
