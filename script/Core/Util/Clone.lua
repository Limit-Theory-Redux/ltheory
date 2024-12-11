function ShallowClone(obj)
    local copy = {}
    for key, value in pairs(obj) do
        copy[key] = value
    end

    -- Preserve the metatable
    setmetatable(copy, getmetatable(obj))

    return copy
end

function DeepClone(obj, seen)
    if type(obj) ~= "table" then
        return obj
    end

    if seen and seen[obj] then
        return seen[obj] -- Handle circular references
    end

    local copy = {}
    seen = seen or {}
    seen[obj] = copy

    for key, value in pairs(obj) do
        copy[DeepClone(key, seen)] = DeepClone(value, seen)
    end

    setmetatable(copy, getmetatable(obj)) -- Preserve metatable
    return copy
end
