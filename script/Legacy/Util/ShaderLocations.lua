-- Uniform-location lookup for the Legacy effect objects, which still set their
-- uniforms through `shader:iSet*` (the old per-draw API, removed in S6 of the
-- render API v2 plan). Replaces the former global `ShaderVarCache`: returns a
-- table of `name -> location`, `-1` for names the shader does not have.

---@param shader Shader
---@param names string[]
---@return table<string, integer>
local function ShaderLocations(shader, names)
    local locations = {}
    for i = 1, #names do
        local name = names[i]
        if shader:hasVariable(name) then
            locations[name] = shader:getVariable(name)
        else
            locations[name] = -1
        end
    end
    return locations
end

return ShaderLocations
