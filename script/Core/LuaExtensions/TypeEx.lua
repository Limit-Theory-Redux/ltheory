rawtype = type

type = function(value)
    local mt = getmetatable(value)
    if mt and mt.__type then
        return mt.__type
    else
        return rawtype(value)
    end
end
