local MusicPlayer = class(function(self) end)

local MusicObject = require("Types.MusicObject")
local rng = RNG.FromTime()

function MusicPlayer:Init()
    self.trackList = {}
    self.queue = {}
    self.currentlyPlaying = nil
    self.currentTrackNum = 0

    if GameState.audio.soundEnabled then
        self.volume = GameState.audio.musicVolume
    else
        self.volume = 0
    end

    self.lastVolume = self.volume

    self:LoadMusic()
end

function MusicPlayer:SetVolume(volume, fadeMS)
    if volume == self.volume then
        return
    end

    self.lastVolume = GameState.audio.musicVolume
    GameState.audio.musicVolume = volume

    for _, soundObject in ipairs(self.trackList) do
        Log.Debug("MusicPlayer:SetVolume: volume for '%s' set to %s", soundObject.name, volume)
        soundObject:SetVolume(volume, fadeMS)
    end
end

function MusicPlayer:OnUpdate(dt)
    if GameState.audio.musicVolume ~= self.volume then
        self.volume = GameState.audio.musicVolume
    end

    if self.currentlyPlaying and not self.currentlyPlaying:IsPlaying() then
        self.currentlyPlaying = nil
        self.currentTrackNum = 0
    elseif not self.currentlyPlaying and #self.queue > 0 then
        local trackNum = rng:getInt(1, #self.queue)

        while trackNum == self.currentTrackNum do
            trackNum = rng:getInt(1, #self.queue)
        end

        local track = self.queue[trackNum]
        self.currentlyPlaying = track -- randomly pick one of the queued tracks
        self.currentTrackNum = trackNum
        Log.Debug("*** MusicPlayer:OnUpdate: playing tracknum %d '%s' with volume %s", trackNum, track.name, self.volume)
        self.currentlyPlaying:Play(self.volume, 2000)
    end
end

function MusicPlayer:PlayAmbient()
    -- Queue all tracks except Main Menu track
    MusicPlayer:ClearQueue()

    for index, soundObject in ipairs(self.trackList) do
        if not string.match(soundObject.name, Config.audio.general.mainMenu) then
            -- ignore main menu
            -- replace this with music types later
            Log.Debug("MusicPlayer:PlayAmbient: QueueTrack(false) for '%s'", soundObject.name)
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
        Log.Debug("No track found for query")
        return
    end

    if clearQueue then
        MusicPlayer:ClearQueue()
    end

    table.insert(self.queue, track)

    -- Log.Debug("Queuing Track: " .. track.name)
    return track
end

function MusicPlayer:ClearQueue()
    if #self.queue > 0 then
        --Log.Debug("MusicPlayer:ClearQueue: clearing entire queue")
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
                Log.Debug("MusicPlayer:ClearQueueTrack: clearing queued track '%s'", query.name)
                table.remove(self.queue, i)
                break
            end
        end
    end
end

function MusicPlayer:StartTrack(query, fadeInMS)
    local track = self:FindTrack(query)
    if self.currentlyPlaying ~= track then
        Log.Debug("MusicPlayer:StartTrack: playing track '%s' with volume %s", track.name, self.volume)
        track:Rewind()
        track:Play(self.volume, fadeInMS)
        self.currentlyPlaying = track
    end
end

function MusicPlayer:StopTrack(query)
    local track = self:FindTrack(query)
    if track and self.currentlyPlaying == track then
        Log.Debug("MusicPlayer:StopTrack: stopping track '%s'", track.name)
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
    Log.Warn("Couldn't find track")
    return nil
end

function MusicPlayer:LoadMusic()
    for _, fname in ipairs(io.listdirex(Config.paths.soundAmbiance)) do
        local path = Config.paths.soundAmbiance .. fname
        local fileUnsupported = false

        if #Config.audio.general.supportedFormats > 1 then
            for _, supportedFormat in ipairs(Config.audio.general.supportedFormats) do
                if string.find(path, supportedFormat) then
                    fileUnsupported = false
                    break
                else
                    fileUnsupported = true
                end
            end
        elseif not string.find(path, Config.audio.general.supportedFormats[1]) then
            fileUnsupported = true
        end

        if not fileUnsupported then
            local newMusicObject
            if string.find(path, Config.audio.general.mainMenu) then
                newMusicObject = MusicObject:Create {
                    name = fname,
                    path = path,
                    volume = self.volume,
                    isLooping = true
                }
            else
                newMusicObject = MusicObject:Create {
                    name = fname,
                    path = path,
                    volume = self.volume,
                    isLooping = false
                }
            end

            --Log.Debug("VOLUME: " .. self.volume)
            if newMusicObject then
                table.insert(self.trackList, newMusicObject)

                -- Generate Enums
                if not Enums.SoundtrackNames then Enums.SoundtrackNames = {} end
                table.insert(Enums.SoundtrackNames, newMusicObject.name)

                if not Enums.SoundtrackCount then Enums.SoundtrackCount = 0 end
                Enums.SoundtrackCount = Enums.SoundtrackCount + 1
            end
        end
    end

    Log.Info("Load Music: ")
    for index, soundObject in ipairs(self.trackList) do
        Log.Info("[" .. index .. "] " .. soundObject.name .. " (path: " ..
            tostring(soundObject.sound:getPath()) .. ")")
    end
end

return MusicPlayer
