---@class TagDefinition
---@field id integer
---@field name string
---@field group string

---@class Tags
---@field _tagToId table<string, integer>
---@field _idToTag table<integer, TagDefinition>
---@field _count integer
---@field _groupMasks table<string, uint64_t[?]>
local Tags = {
    _tagToId = {},   -- [tagName] = id
    _idToTag = {},   -- [id] = TagDefinition
    _count = 0,      -- total tags
    _groupMasks = {} -- [groupName] = ffi.new("uint64_t[?]")
}

--- Register a group of tags
---@param groupName string
---@param def { tags: string[] }
function Tags:new(groupName, def)
    if type(groupName) ~= "string" or groupName:match("^%s*$") then
        Log.Warn("[Tags] Invalid group name")
        return
    end

    groupName = groupName:gsub(" ", "")
    local tags = def.tags
    if type(tags) ~= "table" or #tags == 0 then
        Log.Warn("[Tags] Empty or invalid tag list for group:", groupName)
        return
    end

    for i = 1, #tags do
        local name = tags[i]
        if type(name) ~= "string" then
            Log.Warn("[Tags] Tag name is not a string:", tostring(name))
            goto continue
        end
        name = name:gsub(" ", "")
        if name == "" then
            Log.Warn("[Tags] Ignoring empty tag name in group:", groupName)
            goto continue
        end

        if not self._tagToId[name] then
            self._count = self._count + 1
            local def = { id = self._count, name = name, group = groupName }
            self._tagToId[name] = self._count
            self._idToTag[self._count] = def
        else
            Log.Warn("[Tags] Duplicate tag name ignored:", name)
        end
        ::continue::
    end

    -- Rebuild group mask
    local nWords = math.ceil(self._count / 64)
    local mask = ffi.new("uint64_t[?]", nWords)
    for i = 0, nWords - 1 do mask[i] = 0ULL end

    for id, def in pairs(self._idToTag) do
        if def.group == groupName then
            local word = math.floor((id - 1) / 64)
            local bitPos = (id - 1) % 64
            mask[word] = mask[word] + ffi.cast("uint64_t", 2 ^ bitPos)
        end
    end
    self._groupMasks[groupName] = mask
end

--- Get tag ID by name
---@param name string
---@return integer|nil
function Tags:getId(name)
    return self._tagToId[name]
end

--- Get tag definition by ID
---@param id integer
---@return TagDefinition|nil
function Tags:getDefinition(id)
    return self._idToTag[id]
end

--- Get the precomputed group mask (ffi array)
---@param groupName string
---@return uint64_t[?]|nil
function Tags:getGroupMask(groupName)
    return self._groupMasks[groupName]
end

return Tags
