--[[
    Log required for "RequireAll", but shouldn't be in global space
]]
local Log = require('Core.Util.Log')

local requireAllCache = {}
function requireAll(path)
    -- NOTE : It may be more idiomatic to use package.searchers to handle this
    if requireAllCache[path] then return requireAllCache[path] end
    local pathWithSlashes = path:gsub('%.', '/')

    local dir
    local templates = package.path:split(';')
    for i = 1, #templates do
        local maybeDir = templates[i]
        if maybeDir ~= '' then
            maybeDir = maybeDir:gsub('[^/\\]*%?[^/\\]*$', '')
            maybeDir = maybeDir:gsub('%?', pathWithSlashes)
            maybeDir = maybeDir .. pathWithSlashes .. '/'
            if io.exists(maybeDir) and io.isdir(maybeDir) then
                dir = maybeDir
                break
            end
        end
    end
    if not dir then Log.Error('Failed to open directory <%s>', path) end

    local results = {}
    local files, dirs = io.listdirex(dir)
    for i = 1, #dirs do
        local dirName = dirs[i]
        results[dirName] = requireAll(path .. '.' .. dirName)
    end

    for i = 1, #files do
        local fileName = files[i]
        if fileName:sub(-4) == ".lua" then
            fileName = fileName:gsub('%..*$', '')
            if fileName:len() > 0 then
                results[fileName] = require(path .. '.' .. fileName)
            end
        end
    end

    requireAllCache[path] = results
    return results
end
