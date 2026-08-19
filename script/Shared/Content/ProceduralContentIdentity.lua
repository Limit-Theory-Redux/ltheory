---@class ProceduralContentIdentity
local ProceduralContentIdentity = {}

local function requireNonEmptyString(value, label)
    assert(type(value) == "string" and #value > 0, label .. " must be a non-empty string")
    return value
end

---@param args table
---@return ProceduralContentIdentity
function ProceduralContentIdentity.new(args)
    assert(type(args) == "table")
    local domain = requireNonEmptyString(args.domain, "content identity domain")
    local generatorId = requireNonEmptyString(args.generatorId, "content identity generatorId")
    local generatorVersion = args.generatorVersion
    assert(type(generatorVersion) == "number" and generatorVersion >= 1
        and generatorVersion % 1 == 0,
        "content identity generatorVersion must be a positive integer")
    assert(args.universeSeed ~= nil, "content identity requires universeSeed")
    assert(args.contentSeed ~= nil, "content identity requires contentSeed")
    local ordinal = args.ordinal or 0
    assert(type(ordinal) == "number" and ordinal >= 0 and ordinal % 1 == 0,
        "content identity ordinal must be a non-negative integer")

    local canonicalKey = args.canonicalKey or table.concat({
        domain,
        generatorId,
        tostring(generatorVersion),
        tostring(args.universeSeed),
        tostring(args.contentSeed),
        tostring(ordinal),
    }, ":")
    requireNonEmptyString(canonicalKey, "content identity canonicalKey")

    return {
        domain = domain,
        generatorId = generatorId,
        generatorVersion = generatorVersion,
        universeSeed = args.universeSeed,
        contentSeed = args.contentSeed,
        ordinal = ordinal,
        canonicalKey = canonicalKey,
    }
end

---@param ref ProceduralContentIdentity
---@return ProceduralContentIdentity
function ProceduralContentIdentity.copy(ref)
    assert(type(ref) == "table" and ref.canonicalKey)
    return ProceduralContentIdentity.new(ref)
end

return ProceduralContentIdentity
