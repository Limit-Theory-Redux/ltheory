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
          Config.audio.bSoundOn = data
        end
      elseif string.find(string.lower(line), "startinghorz") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.render.startingHorz = data
          Config.render.resXnew = data
        end
      elseif string.find(string.lower(line), "startingvert") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = tonumber(text)
          Config.render.startingVert = data
          Config.render.resYnew = data
        end
      elseif string.find(string.lower(line), "fullscreen") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          data = stringToBoolean[text]
          Config.render.fullscreen = data
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
      elseif string.find(string.lower(line), "cursor") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          if string.match(text, "simple") then
            Config.ui.cursor = Config.ui.cursorSimple
          elseif string.match(text, "smooth") then
            Config.ui.cursor = Config.ui.cursorSmooth
          end
        end
      elseif string.find(string.lower(line), "hudstyle") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.lower(string.sub(line, eIndex + 1))
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          if string.match(text, "tight") then
            Config.ui.hudDisplayed = Enums.HudModes.Tight
          elseif string.match(text, "balanced") then
            Config.ui.hudDisplayed = Enums.HudModes.Balanced
          elseif string.match(text, "wide") then
            Config.ui.hudDisplayed = Enums.HudModes.Wide
          end
        end
      elseif string.find(string.lower(line), "shipname") then
        eIndex = string.find(line, "=")
        if eIndex then
          text = string.sub(line, eIndex + 1)
          text = string.gsub(text, "^%s*(.-)%s*$", "%1")
          Config.game.humanPlayerShipName = text
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

  -- NOTE: Update this section as new cursors are added
  -- TODO: Functionalize this with config values
  local cursorType = "simple"
  if Config.ui.cursor == Config.ui.cursorSmooth then
    cursorType = "smooth"
  end
  local hudType = "tight"
  if Config.ui.hudDisplayed == Enums.HudModes.Balanced then
    hudType = "balanced"
  elseif Config.ui.hudDisplayed == Enums.HudModes.Wide then
    hudType = "wide"
  end

  -- Sets the input file for writing
  io.output(openedFile)

  -- Write individual values to user initialization file in standard order with groups
  -- NOTE: This is a naive early implementation -- not intended to be production-ready
  -- TODO: convert this into a table-driven process
  io.write("[Audio]", "\n")
  io.write(format("sound=%s",        Config.audio.bSoundOn), "\n")
  io.write("[Graphics]", "\n")
  io.write(format("startingHorz=%s", Config.render.resXnew), "\n")
  io.write(format("startingVert=%s", Config.render.resYnew), "\n")
  io.write(format("fullscreen=%s",   Config.render.fullscreen), "\n")
  io.write("[Generation]", "\n")
  io.write(format("nFields=%s",      Config.gen.nFields), "\n")
  io.write(format("nAsteroids=%s",   Config.gen.nAsteroids), "\n")
  io.write(format("nPlanets=%s",     Config.gen.nPlanets), "\n")
  io.write(format("nStations=%s",    Config.gen.nStations), "\n")
  io.write(format("nAIPlayers=%s",   Config.gen.nAIPlayers), "\n")
  io.write(format("nEconNPCs=%s",    Config.gen.nEconNPCs), "\n")
  io.write(format("nEscortNPCs=%s",  Config.gen.nEscortNPCs), "\n")
  io.write(format("uniqueShips=%s",  Config.gen.uniqueShips), "\n")
  io.write("[UI]", "\n")
  io.write(format("cursorStyle=%s",  cursorType), "\n")
  io.write(format("hudStyle=%s",     hudType), "\n")
  io.write("[Game]", "\n")
  io.write(format("shipname=%s",     Config.game.humanPlayerShipName), "\n")

  -- Closes the open file
  io.close(openedFile)
end

return InitFiles
