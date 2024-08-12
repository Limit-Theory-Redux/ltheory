local Converter = {}

function Converter.ToValuePtr(value, type)
    local valuePtr = nil
    if value then
        valuePtr = ffi.new(type .. "[1]") -- convert to pointer since we use rust option
        valuePtr[0] = value
    end
    return valuePtr
end

function Converter.TableToString(tbl, indent)
    if type(tbl) ~= "table" then
        return tostring(tbl)
    end
    if not indent then indent = 0 end
    local toprint = "{\r\n"
    indent = indent + 2
    for k, v in pairs(tbl) do
        toprint = toprint .. string.rep(" ", indent)
        if (type(k) == "number") then
            toprint = toprint .. "[" .. k .. "] = "
        elseif (type(k) == "string") then
            toprint = toprint .. k .. " = "
        end
        if (type(v) == "number") then
            toprint = toprint .. v .. "/number,\r\n"
        elseif (type(v) == "string") then
            toprint = toprint .. v .. "/string,\r\n"
        elseif (type(v) == "table") then
            toprint = toprint .. Converter.TableToString(v, indent + 2) .. ",\r\n"
        elseif (type(v) == "boolean") then
            toprint = toprint .. tostring(v) .. "/boolean,\r\n"
        else
            toprint = toprint .. "\"" .. tostring(v) .. "\",\r\n"
        end
    end
    toprint = toprint .. string.rep(" ", indent - 2) .. "}"
    return toprint
end

return Converter
