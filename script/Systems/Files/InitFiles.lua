local InitFiles = {}

local function comment(commentTable)
    -- Config Desc
    for _, headerLine in ipairs(commentTable) do io.write(format("# %s\n", headerLine)) end
end

function InitFiles:readUserInits()
    -- Reads user initialization values from file
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

    -- Sets the input file for reading
    io.input(openedFile)

    -- Reads all lines from the file
    for line in openedFile.lines(openedFile) do
        lines[#lines + 1] = line
    end

    -- Closes the open file
    io.close(openedFile)

    -- Scan all lines and apply values to matched game values
    local stringToBoolean = { ["true"] = true, ["false"] = false }

    local function findCategory(line)
        if not string.match(line, "=") then
            return true
        end
        return false
    end

    local categories = {}

    for index, line in ipairs(lines) do
        -- Skip comments or empty lines
        if string.sub(line, 1, 1) == "#" or string.sub(line, 1, 1) == "" then
            goto skip
        end

        if findCategory(line) then
            local categoryName
            local subCategoryName
            categoryName = string.gsub(line, "%[", "")
            categoryName = string.gsub(categoryName, "%]", "")

            if string.match(categoryName, "%.") then
                subCategoryName = string.gsub(categoryName, ".*%.", "")
                categoryName = string.gsub(categoryName, "%..*", "")
            end

            local gameStateTable = GameState[categoryName]

            if subCategoryName then
                gameStateTable = GameState[categoryName][subCategoryName]
            end

            if gameStateTable then
                local categoryTable = {
                    name = categoryName,
                    gameState = gameStateTable,
                    index = index,
                    vars = {}
                }
                table.insert(categories, categoryTable)
            else
                Log.Warn("Could not find game state for config category: " .. categoryName)
            end
        end
        ::skip::
    end

    local function findValuesForCategory(categoryTable)
        local function checkIfCursorStyle(val)
            for cursorStyle = 1, Enums.CursorStyleCount do
                if string.match(string.lower(val), string.lower(Enums.CursorStyleNames[cursorStyle])) then
                    return cursorStyle
                end
            end
            return nil
        end

        local function checkIfHudStyle(val)
            for hudStyle = 1, Enums.HudStyleCount do
                if string.match(string.lower(val), string.lower(Enums.HudStyleNames[hudStyle])) then
                    return hudStyle
                end
            end
            return nil
        end

        local function checkIfCameraMode(val)
            for cameraMode = 1, Enums.CameraModeCount do
                if string.match(string.lower(val), string.lower(Enums.CameraModeNames[cameraMode])) then
                    return cameraMode
                end
            end
            return nil
        end

        local function checkIfSoundtrack(val)
            for soundtrackId = 1, Enums.SoundtrackCount do
                local soundtrackName = string.lower(string.gsub(Enums.SoundtrackNames[soundtrackId], "(%..-)$", ""))
                if string.match(string.lower(val), soundtrackName) then
                    return Enums.SoundtrackNames[soundtrackId]
                end
            end
            return nil
        end

        local function firstToLower(string)
            return (string:gsub("^%L", string.lower))
        end

        local function firstToUpper(string)
            return (string:gsub("^%l", string.upper))
        end

        local function setValue(var, val)
            local lower = firstToLower(var)
            local upper = firstToUpper(var)

            if categoryTable.gameState[lower] ~= nil then
                categoryTable.gameState[lower] = val
            elseif categoryTable.gameState[upper] ~= nil then
                categoryTable.gameState[upper] = val
            else
                Log.Warn("Can't find key in gamestate cat %s for var: %s with value %s", categoryTable.name, var,
                    val)
            end
        end

        local iterator = tonumber(categoryTable.index) + 1
        local vars = {}
        local currentLine = lines[iterator]

        while currentLine and not string.match(currentLine, "%[") do
            -- skip comments or empty lines
            if string.match(currentLine, "#") or string.sub(currentLine, 1, 1) == "" then
                iterator = iterator + 1
                currentLine = lines[iterator]
                goto skipLine
            end

            -- parse vars
            local eIndex = string.find(currentLine, "=")
            --Log.Debug("Line %s: %s", iterator, currentLine)
            --Log.Debug("Current eIndex: %s", eIndex)
            local var = string.sub(currentLine, 1, eIndex - 1)
            local val = string.sub(currentLine, eIndex + 1)
            val = string.gsub(val, "^%s*(.-)%s*$", "%1")

            if val == "true" or val == "false" then
                local bool = stringToBoolean[val]
                setValue(var, bool)
            elseif tonumber(val) then
                setValue(var, tonumber(val))
            elseif checkIfCursorStyle(val) then
                local style = checkIfCursorStyle(val)
                setValue(var, style)
                val = tostring(style)
            elseif checkIfHudStyle(val) then
                local style = checkIfHudStyle(val)
                setValue(var, style)
                val = tostring(style)
            elseif checkIfCameraMode(val) then
                local mode = checkIfCameraMode(val)
                setValue(var, mode)
                val = tostring(mode)
            elseif checkIfSoundtrack(val) then
                local soundtrack = checkIfSoundtrack(val)
                setValue(var, soundtrack)
                val = tostring(soundtrack)
            else
                setValue(var, val)
            end

            iterator = iterator + 1
            currentLine = lines[iterator]
            --Log.Debug("Setting var to gamestate: %s with value: %s", var, val)
            ::skipLine::
        end
        return vars
    end

    for _, categoryTable in ipairs(categories) do
        categoryTable.vars = findValuesForCategory(categoryTable)
        -- do whatever with vars if needed
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

    local cursorType = string.lower(Enums.CursorStyleNames[GameState.ui.cursorStyle])
    local hudType = string.lower(Enums.HudStyleNames[GameState.ui.hudStyle])
    local startupCameraMode = string.lower(Enums.CameraModeNames[GameState.player.currentCamera])
    local menuTheme = string.lower(GameState.audio.menuTheme):gsub("(%..-)$", "")

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

                        for _, name in ipairs(Enums.SoundtrackNames) do
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
