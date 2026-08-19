---@class ProceduralCatalog
local ProceduralCatalog = {
    _records = {},
    _order = {},
}

local function copyTable(value)
    if rawtype(value) ~= "table" then
        return value
    end
    local result = {}
    for key, entry in pairs(value) do
        result[key] = copyTable(entry)
    end
    return result
end

---@param identity table
---@param record table
---@return table
function ProceduralCatalog:register(identity, record)
    assert(type(identity) == "table" and type(identity.canonicalKey) == "string")
    assert(type(record) == "table", "procedural catalog record must be a table")
    local key = identity.canonicalKey
    assert(self._records[key] == nil,
        "duplicate procedural content key: " .. key)
    record.ref = identity
    record.canonicalKey = key
    self._records[key] = record
    table.insert(self._order, key)
    return record
end

---@param ref table|string
---@return table|nil
function ProceduralCatalog:resolve(ref)
    local key = type(ref) == "table" and ref.canonicalKey or ref
    if type(key) ~= "string" then
        return nil
    end
    return self._records[key]
end

---@param ref table|string
---@return boolean
function ProceduralCatalog:has(ref)
    return self:resolve(ref) ~= nil
end

---@return string[]
function ProceduralCatalog:getKeys()
    local keys = {}
    for index, key in ipairs(self._order) do
        keys[index] = key
    end
    return keys
end

---@param record table
---@return table
function ProceduralCatalog:snapshot(record)
    assert(record and record.ref and record.canonicalKey)
    return copyTable(record)
end

---@param snapshot table
---@return table
function ProceduralCatalog:restoreSnapshot(snapshot)
    assert(snapshot and snapshot.ref and snapshot.canonicalKey)
    local existing = self:resolve(snapshot.canonicalKey)
    if existing then
        return existing
    end
    return self:register(copyTable(snapshot.ref), copyTable(snapshot))
end

return ProceduralCatalog
