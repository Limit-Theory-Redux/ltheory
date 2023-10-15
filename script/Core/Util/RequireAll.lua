--[[
    Log required for "RequireAll", but shouldn't be in global space
]]
local Log = require('Core.Util.Log')

local requireAllCache = {}

-- Load Lua modules from the path recursively
function requireAll(path)
    -- NOTE : It may be more idiomatic to use package.searchers to handle this
    if requireAllCache[path] then return requireAllCache[path] end

    return collectAllLuaFiles(path,
        function(fileName, filePath) return require(filePath) end,
        function(p, r) requireAllCache[p] = r end)
end

-- Loads generated Lua modules from the path recursively. Calls `declareType` function of each module loader.
-- Returns:
-- @results - path to subfolder or module loader table
-- @genFiles - Lua file path to loader table. Used fo later calling of `defineType` functions
-- @opaques - array of opaque type names
-- @structs - array of struct/transparent type names
function requireAllGenerated(path)
    -- NOTE : It may be more idiomatic to use package.searchers to handle this
    if requireAllCache[path] then return requireAllCache[path] end

    local genFiles = {}
    local opaques = {}
    local structs = {}

    local results = collectAllLuaFiles(path,
        function(fileName, filePath)
            local loader = require(filePath)

            -- Load type declarations
            local typeId, typeName = loader.declareType()

            if typeId == 1 then
                table.insert(opaques, typeName)
            elseif typeId == 2 then
                table.insert(structs, typeName)
            end

            genFiles[fileName] = loader

            return loader
        end,
        function(p, r) requireAllCache[p] = r end)

    return results, genFiles, opaques, structs
end

-- Recursive walk through folders and Lua files.
-- Parameters:
-- @path - starting path
-- @processFile(fileName, filePath) - function to apply to each found Lua file
-- @processResults(path, results) - function to apply to each folder results
-- Returns:
-- @results - folder's hierarchical loaders
function collectAllLuaFiles(path, processFile, processResults)
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
        results[dirName] = collectAllLuaFiles(path .. '.' .. dirName, processFile, processResults)
    end

    for i = 1, #files do
        local fileName = files[i]
        if fileName:sub(-4) == ".lua" then
            fileName = fileName:gsub('%..*$', '')
            if fileName:len() > 0 then
                results[fileName] = processFile(fileName, path .. '.' .. fileName)
            end
        end
    end

    processResults(path, results)
    return results
end
