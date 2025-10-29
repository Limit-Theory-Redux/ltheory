local Component = require("Core.ECS.Component")
local Tags = require("Shared.Registries.Tags")

---@class TagComponent: Component
---@overload fun(self: TagComponent, ...: string): TagComponent subclass internal
---@overload fun(...: string): TagComponent subclass external
local TagComponent = Subclass("TagComponent", Component, function(self, ...)
    self:setComponentName("Tag")
    self:init(...)
end)

-- Shared cache across all components
TagComponent._tagCache = {}

-- Compute word index and 64-bit mask safely
local function getBitPos(tagId)
    local word = math.floor((tagId - 1) / 64)
    local bitPos = (tagId - 1) % 64
    local mask = Bit.Bitmask64(bitPos)
    return word, mask
end

--- Initialize component
---@param ... string
function TagComponent:init(...)
    local nWords = math.ceil(Tags._count / 64)
    self.tags = ffi.new("uint64_t[?]", nWords)
    self.tagWords = nWords
    for i = 0, nWords - 1 do self.tags[i] = 0ULL end

    -- Add initial tags
    for i = 1, select("#", ...) do
        self:addTag(select(i, ...))
    end
end

--- Add a tag
---@param tag string|integer
function TagComponent:addTag(tag)
    local id
    if type(tag) == "string" then
        id = TagComponent._tagCache[tag] or Tags:getId(tag)
        if not id then return end
        TagComponent._tagCache[tag] = id
    else
        id = tag
    end

    local word, mask = getBitPos(id)
    if word >= self.tagWords then return end
    self.tags[word] = Bit.Or64(self.tags[word], mask)
end

--- Remove a tag
---@param tag string|integer
function TagComponent:removeTag(tag)
    local id
    if type(tag) == "string" then
        id = TagComponent._tagCache[tag] or Tags:getId(tag)
        if not id then return end
        TagComponent._tagCache[tag] = id
    else
        id = tag
    end

    local word, mask = getBitPos(id)
    if word >= self.tagWords then return end
    self.tags[word] = Bit.And64(self.tags[word], Bit.Not64(mask))
end

--- Add all tags from a group
---@param groupName string
function TagComponent:addGroup(groupName)
    local groupMask = Tags:getGroupMask(groupName)
    if not groupMask then return end
    for i = 0, self.tagWords - 1 do
        local wordMask = groupMask[i] or 0ULL
        self.tags[i] = Bit.Or64(self.tags[i], wordMask)
    end
end

--- Check if a tag exists
---@param tag string|integer
---@return boolean
function TagComponent:hasTag(tag)
    local id
    if type(tag) == "string" then
        id = TagComponent._tagCache[tag] or Tags:getId(tag)
        if not id then return false end
        TagComponent._tagCache[tag] = id
    else
        id = tag
    end

    local word, mask = getBitPos(id)
    if word >= self.tagWords then return false end
    return Bit.Has64(self.tags[word], mask)
end

--- Check if any tag exists in a group
---@param groupName string
---@return boolean
function TagComponent:hasAnyTagInGroup(groupName)
    local mask = Tags:getGroupMask(groupName)
    if not mask then return false end
    for i = 0, self.tagWords - 1 do
        local wordMask = mask[i] or 0ULL
        if Bit.HasAny64(self.tags[i], wordMask) then
            return true
        end
    end
    return false
end

--- Sparse-aware: check any tag in another component
---@param other TagComponent
---@return boolean
function TagComponent:hasAnyTag(other)
    local len = math.min(self.tagWords, other.tagWords)
    for i = 0, len - 1 do
        if Bit.Has64(Bit.And64(self.tags[i], other.tags[i]), other.tags[i]) then
            return true
        end
    end
    return false
end

--- Sparse-aware: check all tags in another component
---@param other TagComponent
---@return boolean
function TagComponent:hasAllTags(other)
    local len = math.min(self.tagWords, other.tagWords)
    for i = 0, len - 1 do
        if Bit.And64(self.tags[i], other.tags[i]) ~= other.tags[i] then
            return false
        end
    end
    return true
end

--- Check if all tags in a group are present
---@param groupName string
---@return boolean
function TagComponent:hasAllTagsInGroup(groupName)
    local mask = Tags:getGroupMask(groupName)
    if not mask then return false end
    for i = 0, self.tagWords - 1 do
        local wordMask = mask[i] or 0ULL
        if Bit.And64(self.tags[i], wordMask) ~= wordMask then
            return false
        end
    end
    return true
end

--- Get all active tag names
---@return string[]
function TagComponent:getTags()
    local result = {}
    for wordIndex = 0, self.tagWords - 1 do
        local word = self.tags[wordIndex]
        if word ~= 0ULL then
            for bitPos = 0, 63 do
                local mask = Bit.Bitmask64(bitPos)
                if Bit.Has64(word, mask) then
                    local id = wordIndex * 64 + bitPos + 1
                    local def = Tags:getDefinition(id)
                    if def then table.insert(result, def.name) end
                end
            end
        end
    end
    return result
end

--- Print tags grouped
function TagComponent:printTagsByGroup()
    for groupName, mask in pairs(Tags._groupMasks) do
        local active = {}
        for i = 0, self.tagWords - 1 do
            local word = self.tags[i]
            local wordMask = mask[i] or 0ULL
            local intersect = Bit.And64(word, wordMask)
            if intersect ~= 0ULL then
                for bitPos = 0, 63 do
                    local bitMask = Bit.Bitmask64(bitPos)
                    if Bit.HasAny64(intersect, bitMask) then
                        local id = i * 64 + bitPos + 1
                        local def = Tags:getDefinition(id)
                        if def then table.insert(active, def.name) end
                    end
                end
            end
        end
        if #active > 0 then
            table.sort(active)
            Log.Info(string.format("[%s] %s", groupName, table.concat(active, ", ")))
        end
    end
end

return TagComponent
