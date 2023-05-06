local InitFiles = {}

function InitFiles:readUserInits ()
  -- Reads user initialization values from file
  -- TODO: Encase io.xxx functions in local wrappers for security/safety
  local filename = Config.userInitFilename
  local filepath = Config.paths.files
  local openedFile = io.open(filepath .. filename, "r")

  if openedFile then
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
    -- NOTE: This is a naive early implementation -- not intended to be production-ready
    local stringToBoolean = { ["true"] = true, ["false"] = false }

    local function findCategory(line)
      if not string.match(line, "=") then
        return true
      end
      return false
    end

    local categories = {}

    for index, line in ipairs(lines) do
      if findCategory(line) then
        local categoryName
        categoryName = string.gsub(line, "%[", "")
        categoryName = string.gsub(categoryName, "%]", "")
        local gameStateTable = GameState[categoryName]

        if gameStateTable then
          local categoryTable = {
            name = categoryName,
            gameState = gameStateTable,
            index = index,
            vars = {}
          }
          table.insert(categories, categoryTable)
        else
          error("Could not find game state for config category: " .. categoryName)
        end
      end
    end

    local function findValuesForCategory(categoryTable)
      local function checkIfCursorStyle(val)
        for cursorStyle = 1, Enums.CursorStyleCount do
          if string.match(string.lower(val), string.lower(Enums.CursorStyleNames[cursorStyle])) then
            return true, cursorStyle
          end
        end
        return false
      end

      local function checkIfHudStyle(val)
        for hudStyle = 1, Enums.HudStyleCount do
          if string.match(string.lower(val), string.lower(Enums.HudStyleNames[hudStyle])) then
            return true, hudStyle
          end
        end
        return false
      end

      local function firstToLower(string)
        return (string:gsub("^%L", string.lower))
      end

      local function setValue(var, val)
        if categoryTable.gameState[var] ~= nil then
          categoryTable.gameState[var] = val
        else
          error(format("Can't find key in gamestate cat %s for var: %s with value %s", categoryTable.name, var, val))
        end
      end

      local iterator = tonumber(categoryTable.index) + 1
      local vars = {}
      local currentLine = lines[iterator]

      --printf("[%s]", categoryTable.name)
      while currentLine and not string.match(currentLine, "%[") do
        -- parse vars
        local eIndex = string.find(currentLine, "=")
        --printf("Current line: %s", currentLine)
        --printf("Current eIndex: %s", eIndex)
        local var = firstToLower(string.sub(currentLine, 1, eIndex - 1))
        local val = string.sub(currentLine, eIndex + 1)
        val = string.gsub(val, "^%s*(.-)%s*$", "%1")

        if val == "true" or val == "false" then
          local bool = stringToBoolean[val]
          setValue(var, bool)
        elseif tonumber(val) then
          setValue(var, tonumber(val))
        elseif checkIfCursorStyle(val) then
          local _, style = checkIfCursorStyle(val)
          setValue(var, style)
          val = tostring(style)
        elseif checkIfHudStyle(val) then
          local _, style = checkIfHudStyle(val)
          setValue(var, style)
          val = tostring(style)
        else
          setValue(var, val)
        end
        iterator = iterator + 1
        currentLine = lines[iterator]
        --printf("Setting var to gamestate: %s with value: %s", var, val)
      end
      return vars
    end

    for _, categoryTable in ipairs(categories) do
      categoryTable.vars = findValuesForCategory(categoryTable)
      -- do whatever with vars if needed
    end
    print("Finished loading config.ini")
  end
end

function InitFiles:writeUserInits ()
  -- Writes user initialization values to file
  -- TODO: Encase io.xxx functions in local wrappers for security/safety
  local filename = Config.userInitFilename
  local filepath = Config.paths.files
  local openedFile = io.open(filepath .. filename, "w")

  local cursorType = string.lower(Enums.CursorStyleNames[GameState.ui.cursorStyle])
  local hudType = string.lower(Enums.HudStyleNames[GameState.ui.hudStyle])

  -- Sets the input file for writing
  io.output(openedFile)

  -- Clean up GameState table
  local noFunctions = {}

  for l_Index, l_Value in pairs(GameState) do
    if type(l_Value) == "table" then
      noFunctions[l_Index] = l_Value
    end
  end

  local function pairsByKeys (t, f)
    local a = {}
    for n in pairs(t) do table.insert(a, n) end
    table.sort(a, f)
    local i = 0      -- iterator variable
    local iter = function ()   -- iterator function
      i = i + 1
      if a[i] == nil then return nil
      else return a[i], t[a[i]]
      end
    end
    return iter
  end

  for l_Category, l_CategoryTable in pairsByKeys(noFunctions) do
    -- this is dirty for now, but it´s the only category without anything we need to save
    if l_Category ~= "world" then
      io.write(format("[%s]", tostring(l_Category)), "\n")
    end

    for l_Variable, l_Value in pairsByKeys(l_CategoryTable) do
      local pass = true
      -- excluded
      if string.match(l_Variable, "current")
      or string.match(l_Variable, "weaponGroup")
      or string.match(l_Variable, "autonavTimestamp")
      or string.match(l_Variable, "mapSystemZoom") then
        do
          pass = false
        end
      end
      -- don´t allow any other than string, boolean and numbers also ignore "current" variables
      if pass and type(l_Value) == "string"
      or pass and type(l_Value) == "boolean"
      or pass and type(l_Value) == "number" then
        do
          if l_Variable == "cursorStyle" then
            l_Value = cursorType
          elseif l_Variable == "hudStyle" then
            l_Value = hudType
          end
          --printf("writing %s: %s", l_Variable, l_Value)
          io.write(format("%s=%s", tostring(l_Variable), tostring(l_Value)), "\n")
        end
      end
    end
  end
  -- Closes the open file
  io.close(openedFile)
end

return InitFiles
