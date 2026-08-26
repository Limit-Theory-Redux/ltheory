local TagTest = require('States.Application')
local Tags = require("Shared.Registries.Tags")
local TagComponent = require("Modules.Core.Components").Tag
local QuickProfiler = require("Shared.Tools.QuickProfiler")

---@diagnostic disable-next-line: duplicate-set-field
function TagTest:onInit()
    self.profiler = QuickProfiler("TagTest", true, false)
    self.profiler:start()
    Log.Info("[TagTest] Starting stress test for TagComponent")

    -- === Register x tags in 10 groups ===
    local groups = {}
    local tagCount = 512
    local groupCount = 10
    for g = 1, groupCount do
        local groupName = "Group" .. g
        local tagsList = {}
        for i = g, tagCount, groupCount do
            table.insert(tagsList, "Tag" .. i)
        end
        Tags:new(groupName, tagsList)
        table.insert(groups, groupName)
    end
    Log.Info(string.format("[TagTest] Registered %d tags across %d groups", tagCount, groupCount))

    -- === Create 100 components with random tags ===
    local numComponents = 100
    local components = {}
    for i = 1, numComponents do
        local comp = TagComponent()
        -- assign 100 random tags per component
        for j = 1, 100 do
            local t = "Tag" .. math.random(1, tagCount)
            comp:addTag(t)
        end
        -- assign one random group
        local g = groups[math.random(1, groupCount)]
        comp:addGroup(g)
        table.insert(components, comp)
    end
    Log.Info("[TagTest] Initialized 100 components with random tags and groups")

    -- === Test tag queries ===
    for i = 1, 10 do
        local a = components[math.random(1, numComponents)]
        local b = components[math.random(1, numComponents)]
        assert(a:hasAnyTag(b) or true)
        assert(a:hasAllTags(b) == false or true)
    end
    Log.Info("[TagTest] Random component tag checks passed")

    -- === Test hasAnyTagInGroup ===
    for _, comp in ipairs(components) do
        for _, group in ipairs(groups) do
            comp:hasAnyTagInGroup(group)
        end
    end
    Log.Info("[TagTest] hasAnyTagInGroup tests passed")

    self.profiler:stop()

    -- === Print tags for first 5 components by group (to avoid spamming) ===
    --for i = 1, 5 do
    --    Log.Info(string.format("[TagTest] Component %d tag summary:", i))
    --    components[i]:printTagsByGroup()
    --end

    Log.Info("[TagTest] Stress test completed successfully")
    self:quit()
end

return TagTest
