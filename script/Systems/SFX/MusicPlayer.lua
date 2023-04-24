local MusicPlayer = class(function (self) end)

local MusicObject = require("Types.MusicObject")

function MusicPlayer:Init()
  self.trackList = {}
  self.queue = {}
  self.currentlyPlaying = nil

  if Config.audio.bSoundOn then
    self.volume = Config.audio.musicVolume
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
  Config.audio.musicVolume = volume

  for index, soundObject in ipairs(self.trackList) do
    Sound.SetVolume(soundObject.sound, volume)
  end
end

function MusicPlayer:OnUpdate(dt)
  if self.currentlyPlaying then
    if not self.currentlyPlaying:IsPlaying() then
      self.currentlyPlaying = nil
    end
  elseif not self.currentlyPlaying and #self.queue > 0 then
    self.currentlyPlaying = self.queue[1]
    self.currentlyPlaying:Play()
  end
end

function MusicPlayer:PlayAmbient()
  local queueClear = false
  for index, soundObject in ipairs(self.trackList) do
    if not string.match(soundObject.name, Config.audio.mainMenu) then
      -- ignore main menu
      -- replace this with music types later
      if not queueClear then
        MusicPlayer:QueueTrack(soundObject, true)
        queueClear = true
      else
        MusicPlayer:QueueTrack(soundObject, false)
      end
    end
  end
end

function MusicPlayer:QueueTrack(query, clearQueue)
  local track = self:FindTrack(query)

  if not track then
    printf("No track found for query")
    return
  end

  if clearQueue and #self.queue > 0 then
    self.queue = {}
    self.currentlyPlaying:Pause()
    self.currentlyPlaying:Rewind()
    self.currentlyPlaying = nil
  end

  table.insert(self.queue, track)

  if not self.currentlyPlaying then
    track:Rewind()
    track:SetVolume(self.volume)
    track:Play()
    self.currentlyPlaying = track
  end
  printf("Queuing Track: " .. track.name)
  return track
end

function MusicPlayer:StopTrack(query)
  local track = self:FindTrack(query)
  if track and self.currentlyPlaying == track then
    track:Pause()
    track:Rewind()
    self.currentlyPlaying = nil
  end
end

function MusicPlayer:FindTrack(query)
  for index, soundObject in pairs(self.trackList) do
    if type(query) == "string" then
      if string.find(soundObject.name, query) then
        return soundObject
      end
    elseif query == soundObject then
        return soundObject
    end
  end
  printf("Couldn´t find track")
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
        isLooping = true
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
