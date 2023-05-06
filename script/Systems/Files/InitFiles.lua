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
    -- TODO: convert this into a table-driven process
    local eIndex = 0
    local text = nil
    local data = nil
    local stringToBoolean = { ["true"] = true, ["false"] = false }
    for _, line in pairs(lines) do
      if string.find(string.lower(line), "sound") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1)) -- get all text after the equals sign and lowercase it
          text = string.gsub(text, "^%s*(.-)%s*$", "%1") -- in a non-Lua language, this would be "trim whitespace"
          data = stringToBoolean[text]
          GameState.audio.enabled = data
        end
      elseif string.find(string.lower(line), "defaultResX") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.render.resX = data
        end
      elseif string.find(string.lower(line), "defaultResY") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.render.resY = data
        end
      elseif string.find(string.lower(line), "fullscreen") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = stringToBoolean[text]
          GameState.render.fullscreen = data
        end
      elseif string.find(string.lower(line), "nfields") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.gen.nFields = data
        end
      elseif string.find(string.lower(line), "nasteroids") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.gen.nAsteroids = data
        end
      elseif string.find(string.lower(line), "nplanets") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.gen.nPlanets = data
        end
      elseif string.find(string.lower(line), "nstations") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.gen.nStations = data
        end
      elseif string.find(string.lower(line), "naiplayers") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.gen.nAIPlayers = data
        end
      elseif string.find(string.lower(line), "neconnpcs") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.gen.nEconNPCs = data
        end
      elseif string.find(string.lower(line), "nescortnpcs") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          GameState.gen.nEscortNPCs = data
        end
      elseif string.find(string.lower(line), "uniqueships") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = stringToBoolean[text]
          GameState.gen.uniqueShips = data
        end
      elseif string.find(string.lower(line), "cursor") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          for cursorStyle = 1, Enums.CursorStyleCount do
            if string.match(text, string.lower(Enums.CursorStyleNames[cursorStyle])) then
              GameState.ui.cursorStyle = cursorStyle
              break
            end
          end
        end
      elseif string.find(string.lower(line), "hudstyle") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          for hudStyle = 1, Enums.HudStyleCount do
            if string.match(text, string.lower(Enums.HudStyleNames[hudStyle])) then
              GameState.ui.hudStyle = hudStyle
              break
            end
          end
        end
      elseif string.find(string.lower(line), "shipname") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.sub(line, eIndex + 1)
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          GameState.player.humanPlayerShipName = text
        end
      end
    end
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
      if string.match(l_Variable, "current") then
        pass = false
      end
      -- don´t allow any other than string, boolean and numbers also ignore "current" variables
      if pass and type(l_Value) == "string"
      or pass and type(l_Value) == "boolean"
      or pass and type(l_Value) == "number" then
        do
          printf("writing %s: %s", l_Variable, l_Value)
          io.write(format("%s=%s", tostring(l_Variable), tostring(l_Value)), "\n")
        end
      end
    end
  end

  --io.write("[Audio]", "\n")
  --io.write(format("sound=%s",        GameState.audio.enabled), "\n")
  --io.write("[Graphics]", "\n")
  --io.write(format("startingHorz=%s", GameState.render.resX), "\n")
  --io.write(format("startingVert=%s", GameState.render.resY), "\n")
  --io.write(format("fullscreen=%s",   GameState.render.fullscreen), "\n")
  --io.write("[Generation]", "\n")
  --io.write(format("nFields=%s",      GameState.gen.nFields), "\n")
  --io.write(format("nAsteroids=%s",   GameState.gen.nAsteroids), "\n")
  --io.write(format("nPlanets=%s",     GameState.gen.nPlanets), "\n")
  --io.write(format("nStations=%s",    GameState.gen.nStations), "\n")
  --io.write(format("nAIPlayers=%s",   GameState.gen.nAIPlayers), "\n")
  --io.write(format("nEconNPCs=%s",    GameState.gen.nEconNPCs), "\n")
  --io.write(format("nEscortNPCs=%s",  GameState.gen.nEscortNPCs), "\n")
  --io.write(format("uniqueShips=%s",  GameState.gen.uniqueShips), "\n")
  --io.write("[UI]", "\n")
  --io.write(format("cursorStyle=%s",  cursorType), "\n")
  --io.write(format("hudStyle=%s",     hudType), "\n")
  --io.write("[Game]", "\n")
  --io.write(format("shipname=%s",     GameState.player.humanPlayerShipName), "\n")

  -- Closes the open file
  io.close(openedFile)
end

return InitFiles
