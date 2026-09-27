---@class MusicPlayer
---@overload fun(): MusicPlayer
local MusicPlayer = Class("MusicPlayer", function(self) end)

local MusicObject = require("Types.MusicObject")
local rng = RNG.FromTime()

---Initialize the player: read the current audio state and enumerate the
---track directory. Idempotent (state init + view open may both call it).
function MusicPlayer:init()
    self.trackList = {}
    self.queue = {}
    self.currentlyPlaying = nil
    self.currentTrackNum = 0
    self.loadList = {}
    self.volume = GameState.audio.soundEnabled and GameState.audio.musicVolume or 0
    self.lastVolume = self.volume

    self:findMusic()
    self:registerEvents()
end

function MusicPlayer:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
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
        soundObject:setVolume(volume, fadeMS)
    end
end

function MusicPlayer:setGlobalVolume()
    local actualVolume = GameState.audio.musicVolume
    if not GameState.audio.soundEnabled then
        actualVolume = 0
    end

    for _, soundObject in ipairs(self.trackList) do
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
        self.currentlyPlaying = track
        self.currentTrackNum = trackNum
        self.currentlyPlaying:play(self.volume, 2000)
    end
end

---Queue every track except the main-menu theme and start playing one
---randomly (the in-game soundtrack).
function MusicPlayer:playAmbient()
    self:clearQueue()

    for _, soundObject in ipairs(self.trackList) do
        if not string.match(soundObject.name, Config.audio.general.mainMenu) then
            self:queueTrack(soundObject, false)
        end
    end

    if #self.queue > 0 then
        local trackNum = RNG.FromTime():getInt(1, #self.queue)
        self:startTrack(self.queue[trackNum])
    else
        Log.Warn("MusicPlayer:playAmbient: no tracks queued")
    end
end

function MusicPlayer:queueTrack(query, clearQueue)
    local track = self:findTrack(query)

    if not track then
        return nil
    end

    if clearQueue then
        self:clearQueue()
    end

    table.insert(self.queue, track)
    return track
end

function MusicPlayer:clearQueue()
    if #self.queue > 0 then
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
                table.remove(self.queue, i)
                break
            end
        end
    end
end

function MusicPlayer:startTrack(query, fadeInMS)
    local track = self:findTrack(query)
    if track and self.currentlyPlaying ~= track then
        track:Rewind()
        track:play(self.volume, fadeInMS)
        self.currentlyPlaying = track
    end
end

function MusicPlayer:stopTrack(query)
    local track = self:findTrack(query)
    if track and self.currentlyPlaying == track then
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
    return nil
end

---@private
---Scan the music directory for supported formats (fills loadList).
function MusicPlayer:findMusic()
    for _, fname in ipairs(io.listdirex(Config.paths.soundAmbiance)) do
        local path = Config.paths.soundAmbiance .. fname
        local fileUnsupported = false

        for _, supportedFormat in ipairs(Config.audio.general.supportedFormats) do
            if string.find(path, supportedFormat) then
                fileUnsupported = false
                break
            else
                fileUnsupported = true
            end
        end

        if not fileUnsupported then
            self.loadList[fname] = path

            -- Track registry for the settings menu (song list).
            if not Enums.SoundtrackNames then Enums.SoundtrackNames = {} end
            table.insert(Enums.SoundtrackNames, fname)
            if not Enums.SoundtrackCount then Enums.SoundtrackCount = 0 end
            Enums.SoundtrackCount = Enums.SoundtrackCount + 1
        end
    end
end

---Load the discovered tracks as MusicObjects (streamed by the audio
---manager). The main-menu track is loopable, the rest play once.
function MusicPlayer:loadMusic()
    for fname, path in pairs(self.loadList) do
        local newMusicObject = MusicObject:create {
            name = fname,
            path = path,
            volume = self.volume,
            isLooping = string.find(path, Config.audio.general.mainMenu) ~= nil,
        }
        table.insert(self.trackList, newMusicObject)
    end

    self.loadList = {}
end

return MusicPlayer
