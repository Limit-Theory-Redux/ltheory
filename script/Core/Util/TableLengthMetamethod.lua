local function countKeys(tbl)
    local count = 0
    for _ in pairs(tbl) do
        count = count + 1
    end
    return count
end

function SetLengthMetamethod(tbl)
    local mt = getmetatable(tbl) or {}
    mt.__len = function()
        return countKeys(tbl)
    end
    setmetatable(tbl, mt)
end
