local Action = require('GameObjects.Action')

local Undock = subclass(Action, function (self) end)

function Undock:clone ()
  return Undock()
end

function Undock:getName ()
  return 'Undock'
end

function Undock:onUpdateActive (e, dt)
  if e:getParent():hasDockable() then
    e:getParent():removeDocked(e)

    Config.game.shipDocked = false

    local typename = Config:getObjectInfo("object_types", Config.game.currentStation:getType())
    printf("Undocked from %s '%s'", typename, Config.game.currentStation:getName())
  end
  e:popAction()
end

return Undock
