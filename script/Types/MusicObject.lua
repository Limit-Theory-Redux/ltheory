local MusicObject = {}
MusicObject.__index = MusicObject

function MusicObject:Create(arg)
  if not arg.name or not arg.path or not arg.volume then print("error") return end
  printf("create new sound: " .. arg.name)

  local object = {}
  object.name = arg.name
  object.path = arg.path
  object.sound = Sound.Load(arg.path, arg.isLooping, false)
  object.volume = arg.volume
  setmetatable(object, MusicObject)
  return object
end

function MusicObject:Play(volume)
  local vol = volume or self.volume
  Sound.SetVolume(self.sound, vol)
  Sound.Play(self.sound)
end

function MusicObject:Pause()
  Sound.Pause(self.sound)
end

function MusicObject:Rewind()
  Sound.Rewind(self.sound)
end

function MusicObject:SetVolume(volume)
  Sound.SetVolume(self.sound, volume)
end

function MusicObject:IsPlaying()
  return Sound.IsPlaying(self.sound)
end

return MusicObject