local MusicPlayer = class(function(self) end)

local MusicObject = require("Types.MusicObject")
local rng = RNG.FromTime()

function MusicPlayer:init()
    self.trackList = {}
    self.queue = {}
    self.currentlyPlaying = nil
    self.currentTrackNum = 0
    self.loadList = {}

    if GameState.audio.soundEnabled then
        self.volume = GameState.audio.musicVolume
    else
        self.volume = 0
    end

    self.lastVolume = self.volume

    self:findMusic()
    self:registerEvents()
end

function MusicPlayer:registerEvents()
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreRender), self, self.onPreRender)
end

function MusicPlayer:setVolume(volume, fadeMS)
    if volume == self.volume then
        return
    end

    self.lastVolume = GameState.audio.musicVolume
    GameState.audio.musicVolume = volume

    if not GameState.audio.soundEnabled then
        return
    end

    for _, soundObject in ipairs(self.trackList) do
        -- Log.Debug("MusicPlayer:setVolume: volume for '%s' set to %s", soundObject.name, volume)
        soundObject:setVolume(volume, fadeMS)
    end
end

function MusicPlayer:setGlobalVolume()
    local actualVolume = GameState.audio.musicVolume
    if not GameState.audio.soundEnabled then
        actualVolume = 0
    end

    for _, soundObject in ipairs(self.trackList) do
        -- Log.Debug("MusicPlayer:setVolume: volume for '%s' set to %s", soundObject.name, actualVolume)
        soundObject:setVolume(actualVolume, 0)
    end
end

function MusicPlayer:onPreRender(dt)
    if GameState.audio.musicVolume ~= self.volume then
        self.volume = GameState.audio.musicVolume
    end

    if self.currentlyPlaying and not self.currentlyPlaying:IsPlaying() then
        self.currentlyPlaying = nil
        self.currentTrackNum = 0
    elseif not self.currentlyPlaying and #self.queue > 0 then
        local trackNum = rng:getInt(1, #self.queue)

        if #self.queue > 1 then
            while trackNum == self.currentTrackNum do
                trackNum = rng:getInt(1, #self.queue)
            end
        end

        local track = self.queue[trackNum]
        self.currentlyPlaying = track -- randomly pick one of the queued tracks
        self.currentTrackNum = trackNum
        -- Log.Debug("*** MusicPlayer:OnUpdate: playing tracknum %d '%s' with volume %s", trackNum, track.name, self.volume)
        self.currentlyPlaying:play(self.volume, 2000)
    end
end

function MusicPlayer:playAmbient()
    -- Queue all tracks except Main Menu track
    MusicPlayer:clearQueue()

    for index, soundObject in ipairs(self.trackList) do
        if not string.match(soundObject.name, Config.audio.general.mainMenu) then
            -- ignore main menu
            -- replace this with music types later
            -- Log.Debug("MusicPlayer:playAmbient: queueTrack(false) for '%s'", soundObject.name)
            MusicPlayer:queueTrack(soundObject, false)
        end
    end

    -- Randomly select a track loaded to the queue and start playing it
    local trackNum = RNG.FromTime():getInt(1, #self.queue)
    MusicPlayer:startTrack(self.queue[trackNum])
end

function MusicPlayer:queueTrack(query, clearQueue)
    -- Add a track to the queue (possibly deleting all queued tracks first)
    -- Note: This just adds tracks to the queue; it doesn't start playing any of them
    local track = self:findTrack(query)

    if not track then
        Log.Warn("No track found for query")
        return
    end

    if clearQueue then
        MusicPlayer:clearQueue()
    end

    table.insert(self.queue, track)

    -- Log.Debug("Queuing Track: " .. track.name)
    return track
end

function MusicPlayer:clearQueue()
    if #self.queue > 0 then
        -- Log.Debug("MusicPlayer:clearQueue: clearing entire queue")
        self.queue = {}
        if self.currentlyPlaying then
            self.currentlyPlaying:stop()
            self.currentlyPlaying = nil
        end
    end
end

function MusicPlayer:clearQueueTrack(query)
    if #self.queue > 0 then
        if self.currentlyPlaying and self.currentlyPlaying == query then
            self.currentlyPlaying:stop()
            self.currentlyPlaying = nil
        end
        for i, track in ipairs(self.queue) do
            if track == query then
                -- Log.Debug("MusicPlayer:clearQueueTrack: clearing queued track '%s'", query.name)
                table.remove(self.queue, i)
                break
            end
        end
    end
end

function MusicPlayer:startTrack(query, fadeInMS)
    local track = self:findTrack(query)
    if self.currentlyPlaying ~= track then
        -- Log.Debug("MusicPlayer:startTrack: playing track '%s' with volume %s", track.name, self.volume)
        track:Rewind()
        track:play(self.volume, fadeInMS)
        self.currentlyPlaying = track
    end
end

function MusicPlayer:stopTrack(query)
    local track = self:findTrack(query)
    if track and self.currentlyPlaying == track then
        -- Log.Debug("MusicPlayer:stopTrack: stopping track '%s'", track.name)
        track:Pause()
        track:Rewind()
        self.currentlyPlaying = nil
    end
end

function MusicPlayer:findTrack(query)
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

function MusicPlayer:findMusic()
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
            self.loadList[fname] = path

            -- Generate Enums
            if not Enums.SoundtrackNames then Enums.SoundtrackNames = {} end
            table.insert(Enums.SoundtrackNames, fname)

            if not Enums.SoundtrackCount then Enums.SoundtrackCount = 0 end
            Enums.SoundtrackCount = Enums.SoundtrackCount + 1
        end
    end
end

function MusicPlayer:loadMusic()
    for fname, path in pairs(self.loadList) do
        local newMusicObject
        if string.find(path, Config.audio.general.mainMenu) then
            newMusicObject = MusicObject:create {
                name = fname,
                path = path,
                volume = self.volume,
                isLooping = true
            }
        else
            newMusicObject = MusicObject:create {
                name = fname,
                path = path,
                volume = self.volume,
                isLooping = false
            }
        end

        table.insert(self.trackList, newMusicObject)
    end

    Log.Info("Loaded Music: ")
    for index, soundObject in ipairs(self.trackList) do
        Log.Info("[" .. index .. "] " .. soundObject.name .. " (path: " ..
            tostring(soundObject.sound:getPath()) .. ")")
    end

    self.loadList = {}
end

return MusicPlayer
