local InitFiles = {}

local function comment(commentTable)
    -- Config Desc
    for _, headerLine in ipairs(commentTable) do io.write(format("# %s\n", headerLine)) end
end

-- Settings whose user.ini value is a name from an option list (or its number).
local optionLists = {
    cursorStyle   = function() return Enums.CursorStyleNames end,
    hudStyle      = function() return Enums.HudStyleNames end,
    startupCamera = function() return Enums.CameraModeNames end,
}

---A file name in lower case, without its extension.
---@param name string
---@return string
local function withoutExtension(name)
    return (string.lower(name):gsub("(%..-)$", ""))
end

---The value a user.ini line gives a setting, checked against what the setting holds.
---@param var string the setting's name
---@param text string the value as written in user.ini
---@param current any the setting's current value, which gives the kind of value it takes
---@return any|nil value nil if the text doesn't fit
---@return string|nil problem why it doesn't fit
local function parseSettingValue(var, text, current)
    local optionList = optionLists[var]
    local names = optionList and optionList()
    if names then
        local index = tonumber(text)
        if index and names[index] then
            return index
        end
        for nameIndex, name in ipairs(names) do
            if string.lower(name) == string.lower(text) then
                return nameIndex
            end
        end
        return nil, format("expected one of <%s>", string.lower(table.concat(names, ", ")))
    end

    if var == "menuTheme" then
        for _, name in ipairs(Enums.SoundtrackNames or {}) do
            if withoutExtension(name) == withoutExtension(text) then
                return name
            end
        end
        return nil, "no soundtrack has that name"
    end

    local kind = type(current)
    if kind == "boolean" then
        if text == "true" then return true end
        if text == "false" then return false end
        return nil, "expected true or false"
    elseif kind == "number" then
        local number = tonumber(text)
        if number and number == number then -- not NaN
            return number
        end
        return nil, "expected a number"
    elseif kind == "string" then
        return text
    end
    return nil, format("can't be set from user.ini (it holds a %s)", kind)
end

---Reads user.ini into GameState. Nothing in the file stops the game: a line that
---can't be read, an unknown section or setting, or a value of the wrong kind is
---logged as a warning and skipped, and that setting keeps its current value.
function InitFiles:readUserInits()
    -- TODO: Encase io.xxx functions in local wrappers for security/safety
    local filename = Config.userInitFilename
    local filetype = Config.userInitFiletype
    local filepath = Config.paths.files
    local configPath = filepath .. filename .. filetype
    local openedFile = io.open(configPath, "r")

    if not openedFile then
        Log.Warn("Cannot open config file: %s", configPath)
        return
    end

    local lines = {}
    for line in openedFile:lines() do
        lines[#lines + 1] = line
    end
    openedFile:close()

    local section     -- the GameState table of the current [section]; nil if unknown
    local sectionName -- its name as written, for warnings

    local function firstToLower(name)
        return (name:gsub("^%u", string.lower))
    end

    local function firstToUpper(name)
        return (name:gsub("^%l", string.upper))
    end

    ---A [section] line: find its GameState table ("ui.trackerBracketingRenderDistances"
    ---is GameState.ui.trackerBracketingRenderDistances).
    local function readSection(name)
        sectionName = name
        local found = GameState
        for part in string.gmatch(name, "[^%.]+") do
            found = type(found) == "table" and found[part] or nil
        end
        if type(found) == "table" then
            section = found
        else
            section = nil
            Log.Warn("user.ini: no settings section [%s]; its lines are skipped", name)
        end
    end

    ---A name=value line: set the setting if the value fits it.
    local function readSetting(lineNumber, var, text)
        if not section then
            return -- the unknown section was reported once
        end
        local key = var
        if section[key] == nil then key = firstToLower(var) end
        if section[key] == nil then key = firstToUpper(var) end
        if section[key] == nil then
            Log.Warn("user.ini line %d: no setting %s in [%s]; skipped", lineNumber, var, sectionName)
            return
        end

        local value, problem = parseSettingValue(key, text, section[key])
        if value == nil then
            Log.Warn("user.ini line %d: %s=%s ignored (%s); keeping %s", lineNumber, var, text, problem,
                tostring(section[key]))
            return
        end
        section[key] = value
    end

    local function readLine(lineNumber, line)
        line = string.match(line, "^%s*(.-)%s*$")
        if line == "" or string.sub(line, 1, 1) == "#" then
            return
        end
        local name = string.match(line, "^%[(.+)%]$")
        if name then
            readSection(name)
            return
        end
        local var, text = string.match(line, "^([^=]-)%s*=%s*(.*)$")
        if var and var ~= "" then
            readSetting(lineNumber, var, text)
            return
        end
        Log.Warn("user.ini line %d: not a [section] or name=value line; skipped: %s", lineNumber, line)
    end

    for lineNumber, line in ipairs(lines) do
        local ok, err = pcall(readLine, lineNumber, line)
        if not ok then
            Log.Warn("user.ini line %d skipped, it could not be read (%s): %s", lineNumber, tostring(err), line)
        end
    end

    Log.Info("Loaded configuration from: %s", configPath)

    if GameState.debug.printConfig then
        Log.Info("---------- Configuration File ----------")
        for _, line in pairs(lines) do if not string.match(line, "#") then Log.Info(line) end end
        Log.Info("----------------------------------------")
    end
end

function InitFiles:writeUserInits()
    -- Writes user initialization values to file
    -- TODO: Encase io.xxx functions in local wrappers for security/safety
    local filename = Config.userInitFilename
    local filetype = Config.userInitFiletype
    local filepath = Config.paths.files
    local configPath = filepath .. filename .. filetype
    local backupConfigPath = filepath .. filename .. "_backup" .. filetype

    -- If a config file exists, open it so we can take a backup.
    local openedFileReadable = io.open(configPath, "r")
    if openedFileReadable then
        -- Create a safety backup
        local oldContent = openedFileReadable:read("*a")
        openedFileReadable:close()

        local backupFile, backupFileErr = io.open(backupConfigPath, "w")
        if not backupFile then
            Log.Warn("Cannot open backup configuration file for writing: %s", backupFileErr)
            return
        end

        backupFile:write(oldContent) -- write the content to the backup file
        backupFile:close()           -- close the backup file
    end

    -- Open the configuration file for writing
    local openedFileWritable, openedFileWritableErr = io.open(configPath, "w")
    if not openedFileWritable then
        Log.Warn("Cannot open configuration file for writing: %s", openedFileWritableErr)
        return
    end

    ---An option's name as user.ini writes it (lower case), or its number if it has
    ---none (the file is already open for writing: an error here would leave it empty).
    local function optionName(names, index)
        local name = names and names[index]
        if name then
            return string.lower(name)
        end
        Log.Warn("user.ini: no option name for %s; writing it as a number", tostring(index))
        return tostring(index)
    end

    local cursorType = optionName(Enums.CursorStyleNames, GameState.ui.cursorStyle)
    local hudType = optionName(Enums.HudStyleNames, GameState.ui.hudStyle)
    local startupCameraMode = optionName(Enums.CameraModeNames, GameState.player.currentCamera)
    local menuTheme = withoutExtension(tostring(GameState.audio.menuTheme))

    -- Sets the input file for writing
    io.output(openedFileWritable)

    -- Clean up GameState table
    local noFunctions = {}

    for l_Index, l_Value in pairs(GameState) do
        if type(l_Value) == "table" then
            noFunctions[l_Index] = l_Value
        end
    end

    local function pairsByKeys(t, f)
        local a = {}
        for n in pairs(t) do table.insert(a, n) end
        table.sort(a, f)
        local i = 0             -- iterator variable
        local iter = function() -- iterator function
            i = i + 1
            if a[i] == nil then
                return nil
            else
                return a[i], t[a[i]]
            end
        end
        return iter
    end

    local function writeSubCat(cat, var, val)
        io.write("\n") -- empty line before every sub-category
        io.write(format("[%s]", tostring(cat) .. "." .. tostring(var)), "\n")
        for l_SubCat, l_SubTable in pairsByKeys(val) do
            io.write(format("%s=%s", tostring(l_SubCat), tostring(l_SubTable)), "\n")
        end
    end

    local function writeOptions(optionTitle, optionTable, optionDesc)
        local optionString = string.lower(table.concat(optionTable, ", "))

        comment {
            optionTitle .. "Options: <" .. optionString .. ">",
            optionDesc
        }
    end

    comment {
        "Hello World! This is the Limit Theory Redux Configuration File",
        "Support the LTR project by discussing, contributing or silent participation:",
        "GitHub: " .. Config.orgInfo.repository,
        "Discord: " .. Config.orgInfo.discord,
        "Wiki: " .. Config.orgInfo.wiki,
        "Blog: " .. Config.orgInfo.blog,
        "Reddit: " .. Config.orgInfo.reddit
    }

    for l_Category, l_CategoryTable in pairsByKeys(noFunctions) do
        -- this is dirty for now, but its the only category without anything we need to save
        if l_Category ~= "world" then
            io.write("\n") -- empty line before every category
            io.write(format("[%s]", tostring(l_Category)), "\n")
        end

        local cacheSubCat
        local cacheSubCatVar
        local cacheSubCatVal

        for l_Variable, l_Value in pairsByKeys(l_CategoryTable) do
            local pass = true
            -- excluded
            if string.match(l_Variable, "current")
                or string.match(l_Variable, "lastCamera")
                or string.match(l_Variable, "weaponGroup")
                or string.match(l_Variable, "autonavActive")
                or string.match(l_Variable, "mapSystemZoom")
                or string.match(l_Variable, "uiCanvas") then
                do
                    pass = false
                end
            end
            -- dont allow any other than string, boolean and numbers also ignore "current" variables
            if pass and type(l_Value) == "string"
                or pass and type(l_Value) == "boolean"
                or pass and type(l_Value) == "number" then
                do
                    if l_Variable == "cursorStyle" then
                        l_Value = cursorType
                        writeOptions("cursorStyle", Enums.CursorStyleNames, "The game`s currently used cursor style.")
                    elseif l_Variable == "hudStyle" then
                        l_Value = hudType
                        writeOptions("hudStyle", Enums.HudStyleNames, "The game`s currently used hud style.")
                    elseif l_Variable == "startupCamera" then
                        l_Value = startupCameraMode
                        writeOptions("startupCamera", Enums.CameraModeNames, "The camera mode the game starts up with.")
                    elseif l_Variable == "menuTheme" then
                        l_Value = menuTheme
                        local cleanSoundtrackNames = {}

                        for _, name in ipairs(Enums.SoundtrackNames or {}) do
                            local cleanName = name:gsub("(%..-)$", "")
                            table.insert(cleanSoundtrackNames, cleanName)
                        end
                        writeOptions("mainMenuTheme", cleanSoundtrackNames, "The soundtrack used in the main menu.")
                    end
                    --Log.Debug("writing %s: %s", l_Variable, l_Value)
                    io.write(format("%s=%s", tostring(l_Variable), tostring(l_Value)), "\n")
                end
            elseif pass and type(l_Value) == "table" and not string.match(l_Variable, "humanPlayer") then
                cacheSubCat = l_Category
                cacheSubCatVar = l_Variable
                cacheSubCatVal = l_Value
            end
        end

        if cacheSubCat and cacheSubCatVar and cacheSubCatVal then
            writeSubCat(cacheSubCat, cacheSubCatVar, cacheSubCatVal)
        end
    end

    if openedFileWritable ~= nil then
        -- Closes the open file
        Log.Debug("Saved configuration at: %s", configPath)
        io.close(openedFileWritable)
    end
end

return InitFiles
