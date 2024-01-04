function Iterator(t)
    local i = 0
    local n = #t
    return function()
        i = i + 1
        if i <= n then return t[i] end
    end
end

--[[
    example of usage:

    function Entity:iterAssets()
        return Iterator(self:getAssets())
    end
]]
