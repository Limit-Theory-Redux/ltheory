local InitFiles = {}

function InitFiles:readUserInits ()
  -- Reads user initialization values from file
  -- TODO: Encase io.xxx functions in local wrappers for security/safety
  local filename = Config.userInitFilename
  local filepath = Config.paths.files
  local file = io.open(filepath .. filename, "r")
  if file then
    local lines = {}

    -- Sets the input file for reading
    io.input(file)

    -- Reads all lines from the file
    for line in io.lines(filename) do
      lines[#lines + 1] = line
    end

    -- Closes the open file
    io.close(file)

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
          Config.gen.nFields = data
        end
      elseif string.find(string.lower(line), "nasteroids") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.gen.nAsteroids = data
        end
      elseif string.find(string.lower(line), "nplanets") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.gen.nPlanets = data
        end
      elseif string.find(string.lower(line), "nstations") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.gen.nStations = data
        end
      elseif string.find(string.lower(line), "naiplayers") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.gen.nAIPlayers = data
        end
      elseif string.find(string.lower(line), "neconnpcs") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.gen.nEconNPCs = data
        end
      elseif string.find(string.lower(line), "nescortnpcs") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.gen.nEscortNPCs = data
        end
      elseif string.find(string.lower(line), "uniqueships") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = stringToBoolean[text]
          Config.gen.uniqueShips = data
        end
      end
    end
  end
end

function InitFiles:writeUserInits (window)
  -- Writes user initialization values to file
  -- TODO: Encase io.xxx functions in local wrappers for security/safety
  local filename = Config.userInitFilename
  local filepath = Config.paths.files
  local file = io.open(filepath .. filename, "w")

  -- Sets the input file for writing
  io.output(file)

  -- Write individual values to user initialization file in standard order with groups
  -- NOTE: This is a naive early implementation -- not intended to be production-ready
  -- TODO: convert this into a table-driven process
  local size = window:getSize()
  io.write("[Audio]", "\n")
  io.write(format("sound=%s",        GameState.audio.enabled), "\n")
  io.write("[Graphics]", "\n")
  io.write(format("startingHorz=%s", GameState.render.resX), "\n")
  io.write(format("startingVert=%s", GameState.render.resY), "\n")
  io.write(format("fullscreen=%s",   GameState.render.fullscreen), "\n")
  io.write("[Generation]", "\n")
  io.write(format("nFields=%s",      Config.gen.nFields), "\n")
  io.write(format("nAsteroids=%s",   Config.gen.nAsteroids), "\n")
  io.write(format("nPlanets=%s",     Config.gen.nPlanets), "\n")
  io.write(format("nStations=%s",    Config.gen.nStations), "\n")
  io.write(format("nAIPlayers=%s",   Config.gen.nAIPlayers), "\n")
  io.write(format("nEconNPCs=%s",    Config.gen.nEconNPCs), "\n")
  io.write(format("nEscortNPCs=%s",  Config.gen.nEscortNPCs), "\n")
  io.write(format("uniqueShips=%s",  Config.gen.uniqueShips), "\n")

  -- Closes the open file
  io.close(file)
end

return InitFiles
