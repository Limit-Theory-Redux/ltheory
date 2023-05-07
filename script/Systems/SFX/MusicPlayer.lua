local MusicPlayer = class(function (self) end)

local MusicObject = require("Types.MusicObject")

function MusicPlayer:Init()
  self.trackList = {}
  self.queue = {}
  self.currentlyPlaying = nil

  if GameState.audio.soundEnabled then
    self.volume = GameState.audio.musicVolume
  else
    self.volume = 0
  end
  self:LoadMusic()
end
-- add block queing

function MusicPlayer:SetVolume(volume)
  if volume == self.volume then
    return
  end

  self.volume = volume
  GameState.audio.musicVolume = volume

  for _, soundObject in ipairs(self.trackList) do
printf("MusicPlayer:SetVolume: volume for '%s' set to %s", soundObject.name, self.volume)
    Sound.SetVolume(soundObject.sound, volume)
  end
end

function MusicPlayer:OnUpdate(dt)
  local rng = RNG.FromTime()
  if self.currentlyPlaying then
    if not self.currentlyPlaying:IsPlaying() then
      self.currentlyPlaying = nil
    end
  elseif not self.currentlyPlaying and #self.queue > 0 then
    local trackNum = rng:getInt(1, #self.queue)
    local track = self.queue[trackNum]
    self.currentlyPlaying = track -- randomly pick one of the queued tracks
printf("*** MusicPlayer:OnUpdate: playing tracknum %d '%s' with volume %s", trackNum, track.name, self.volume)
    self.currentlyPlaying:Play()
    track:SetVolume(self.volume)
  end
end

function MusicPlayer:PlayAmbient()
  -- Queue all tracks except Main Menu track
  MusicPlayer:ClearQueue()

  for index, soundObject in ipairs(self.trackList) do
    if not string.match(soundObject.name, Config.audio.mainMenu) then
      -- ignore main menu
      -- replace this with music types later
printf("MusicPlayer:PlayAmbient: QueueTrack(false) for '%s'", soundObject.name)
      MusicPlayer:QueueTrack(soundObject, false)
    end
  end

  -- Randomly select a track loaded to the queue and start playing it
  local trackNum = RNG.FromTime():getInt(1, #self.queue)
  MusicPlayer:StartTrack(self.queue[trackNum])
end

function MusicPlayer:QueueTrack(query, clearQueue)
  -- Add a track to the queue (possibly deleting all queued tracks first)
  -- Note: This just adds tracks to the queue; it doesn't start playing any of them
  local track = self:FindTrack(query)

  if not track then
    printf("No track found for query")
    return
  end

  if clearQueue then
    MusicPlayer:ClearQueue()
  end

  table.insert(self.queue, track)

--  printf("Queuing Track: " .. track.name)
  return track
end

function MusicPlayer:ClearQueue()
  if #self.queue > 0 then
--printf("MusicPlayer:ClearQueue: clearing entire queue")
    self.queue = {}
    if self.currentlyPlaying then
      self.currentlyPlaying:Pause()
      self.currentlyPlaying:Rewind()
      self.currentlyPlaying = nil
    end
  end
end

function MusicPlayer:ClearQueueTrack(query)
  if #self.queue > 0 then
    if self.currentlyPlaying and self.currentlyPlaying == query then
      self.currentlyPlaying:Pause()
      self.currentlyPlaying:Rewind()
      self.currentlyPlaying = nil
    end
    for i, track in ipairs(self.queue) do
      if track == query then
printf("MusicPlayer:ClearQueueTrack: clearing queued track '%s'", query.name)
        table.remove(self.queue, i)
        break
      end
    end
  end
end

function MusicPlayer:StartTrack(query)
  local track = self:FindTrack(query)
  if self.currentlyPlaying ~= track then
printf("MusicPlayer:StartTrack: playing track '%s' with volume %s", track.name, self.volume)
    track:Rewind()
    track:Play()
    track:SetVolume(self.volume)
    self.currentlyPlaying = track
  end
end

function MusicPlayer:StopTrack(query)
  local track = self:FindTrack(query)
  if track and self.currentlyPlaying == track then
printf("MusicPlayer:StopTrack: stopping track '%s'", track.name)
    track:Pause()
    track:Rewind()
    self.currentlyPlaying = nil
  end
end

function MusicPlayer:FindTrack(query)
  for _, soundObject in pairs(self.trackList) do
    if type(query) == "string" then
      if string.find(soundObject.name, query) then
        return soundObject
      end
    elseif query == soundObject then
        return soundObject
    end
  end
  printf("Couldn't find track")
  return nil
end

function MusicPlayer:LoadMusic()
  for _, fname in ipairs(io.listdirex(Config.paths.soundAmbiance)) do
    local path = Config.paths.soundAmbiance .. fname
    local fileUnsupported = false

    if #Config.audio.supportedFormats > 1 then
      for supportedFormat in ipairs(Config.audio.supportedFormats) do
        if not string.find(path, supportedFormat) then
          fileUnsupported = true
        end
      end
    elseif not string.find(path, Config.audio.supportedFormats[1]) then
      fileUnsupported = true
    end

    if not fileUnsupported then
      local newSoundObject = MusicObject:Create{
        name = fname,
        path = path,
        volume = self.volume,
        isLooping = true -- temporary
      }

      printf("VOLUME: " .. self.volume)
      if newSoundObject then
        table.insert(self.trackList, newSoundObject)
      end
    end
  end

  printf("Load Music: ")
  for index, soundObject in ipairs(self.trackList) do
    printf("[" .. index .. "] " .. soundObject.name .. " (path: ".. tostring(Sound.GetPath(soundObject.sound)) .. ")")
  end
end

return MusicPlayer
