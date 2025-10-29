function table.clear(t)
    for i = #t, 1, -1 do t[i] = nil end
end

function table.tostring(t, recurse, visited)
    if recurse == nil then recurse = false end
    visited = visited or {}

    if visited[t] then return '< already visited >' end
    visited[t] = true
    local mt = getmetatable(t)
    if mt and mt.__tostring then return mt.__tostring(t) end
    local vals = {}
    for k, v in pairs(t) do
        local combined
        if recurse and type(v) == 'table' then
            combined = format('%s = %s', k, table.tostring(v, recurse, visited))
        else
            combined = format('%s = %s', k, v)
        end
        combined = combined:split('\n')
        for i = 1, #combined do
            combined[i] = '  ' .. combined[i]
        end
        vals[#vals + 1] = table.concat(combined, '\n')
    end
    if #vals < 1 then
        return format('table @ %p', t)
    else
        return format('table @ %p :\n%s\n', t, table.concat(vals, '\n'))
    end
end

function table.equal(o1, o2, ignore_mt)
    if o1 == o2 then return true end
    local o1Type = type(o1)
    local o2Type = type(o2)
    if o1Type ~= o2Type then
        -- Log.Warn(tostring(o1Type) .. " ~= " .. tostring(o2Type))
        return false
    end
    if o1Type ~= 'table' then
        -- Log.Warn(tostring(o1Type) .. " ~= 'table'")
        return false
    end

    if not ignore_mt then
        local mt1 = getmetatable(o1)
        if mt1 and mt1.__eq then
            --compare using built in method
            return o1 == o2
        end
    end

    local keySet = {}

    for key1, value1 in pairs(o1) do
        local value2 = o2[key1]
        if value2 == nil then
            -- Log.Warn("Missing key: " .. tostring(key1))
            return false
        end
        if table.equal(value1, value2, ignore_mt) == false then
            -- Log.Warn(tostring(value1) .. " ~= " .. tostring(value2))
            return false
        end
        keySet[key1] = true
    end

    for key2, _ in pairs(o2) do
        if not keySet[key2] then
            -- Log.Warn("Tables have different key: " .. tostring(key2))
            return false
        end
    end
    return true
end

function table.removeValue(t, value)
    for i, v in ipairs(t) do
        if v == value then
            table.remove(t, i)
            return
        end
    end
    for i, v in ipairs(t) do
        print(tostring(i) .. ": " .. tostring(v))
    end
    Log.Error("Cannot find table value " .. tostring(value))
end

function table.copy(t)
    local copy = {}
    for k, v in pairs(t) do
        copy[k] = v
    end
    return copy
end

function table.toSet(list)
    local set = {}
    for _, item in ipairs(list) do
        set[item] = true
    end
    return set
end

function table.contains(tbl, element)
    for _, value in pairs(tbl) do
        if value == element then
            return true
        end
    end
    return false
end
