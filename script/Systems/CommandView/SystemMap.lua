local DebugContext = require('Systems.CommandView.DebugContext')
local Bindings = require('States.ApplicationBindings')
local Player = require('GameObjects.Entities.Player')

local SystemMap = {}
SystemMap.__index  = SystemMap
setmetatable(SystemMap, UI.Container)

local kPanSpeed = 500
local kZoomSpeed = 0.1

SystemMap.scrollable = true
SystemMap.focusable  = true
SystemMap:setPadUniform(0)

function SystemMap:onDraw (state)
  Draw.Color(0.1, 0.11, 0.12, 1)
  local x, y, sx, sy = self:getRectGlobal()
  Draw.Rect(x, y, sx, sy)

  Draw.Color(0, 1, 0, 1)
  local hx, hy = sx / 2, sy / 2
  local dx, dy = Config.game.mapSystemPos.x + hx, Config.game.mapSystemPos.y + hy

  local c = {
    r = 0.1,
    g = 0.5,
    b = 1.0,
    a = 0.1,
  }

  local best = nil
  local bestDist = math.huge
  local mp = Input.GetMousePosition()

  local playerTarget = Config.game.currentShip:getTarget()
  if playerTarget ~= nil then
    self.focus = playerTarget
  end

  BlendMode.PushAlpha()
  Draw.SmoothPoints(true)
--printf("------------------------------")
  for _, e in self.system:iterChildren() do
    -- Check to make sure this isn't a ship that has exploded
    if e.body ~= nil then
--printf("Drawing %s '%s'", Config.objectInfo[1]["elems"][e:getType()][2], e:getName())
      local p = e:getPos()
      local x = p.x - dx
      local y = p.z - dy
      x = self.x + x * Config.game.mapSystemZoom + hx
      y = self.y + y * Config.game.mapSystemZoom + hy
      Draw.PointSize(2.0)

      if e:hasActions() then
--printf("Action: %s", e:getName())
        if Config.game.currentShip == e then
          Draw.Color(0.9, 0.5, 1.0, 1.0) -- player ship
        else
          local entAction = e:getCurrentAction()
          if entAction ~= nil then
--printf("Action is '%s', target is '%s'", entAction:getName(), entAction.target:getName())
            if string.find(entAction:getName(), "Attack") and entAction.target == Config.game.currentShip then
              Draw.Color(1.0, 0.3, 0.3, 1.0) -- other ship, hostile (has a current action of "Attack player's ship")
            else
              Draw.Color(0.2, 0.6, 1.0, 1.0) -- other ship, non-hostile (TODO: divide into friendly [green] and neutral [blue])
            end
          else
            Draw.Color(1.0, 1.0, 1.0, 1.0) -- some other object that suddenly has no actions
          end
        end
      else
        Draw.Color(0.4, 0.4, 0.4, 1.0) -- planet, asteroid, station
      end
      Draw.Point(x, y)

      if e:hasFlows() and not e:isDestroyed() then
--printf("Flow: %s", e:getName())
        UI.DrawEx.Ring(x, y, Config.game.mapSystemZoom * e:getScale(), { r = 0.1, g = 0.5, b = 1.0, a = 1.0 })
      end

      if e:hasYield() then
--printf("Yield: %s", e:getName())
        UI.DrawEx.Ring(x, y, Config.game.mapSystemZoom * e:getScale(), { r = 1.0, g = 0.5, b = 0.1, a = 0.5 })
      end

      if self.focus == e then
--printf("Focus: %s", e:getName())
        UI.DrawEx.Ring(x, y, 8, { r = 1.0, g = 0.0, b = 0.3, a = 1.0 })
      end

      local d = Vec2f(x, y):distanceSquared(mp)
      if d < bestDist then
        bestDist = d
        best = e
      end
--    else
--      -- Non-object entities (e.g., zones)
--printf("Found %s '%s'", Config.objectInfo[1]["elems"][e:getType()][2], e:getName())
--      local p = e:getPos()
--      local x = p.x - dx
--      local y = p.z - dy
--      x = self.x + x * Config.game.mapSystemZoom + hx
--      y = self.y + y * Config.game.mapSystemZoom + hy
--      Draw.PointSize(2.0)
--      Draw.Color(1.0, 1.0, 1.0, 1)
--      Draw.Point(x, y)
--      --UI.DrawEx.Ring(x, y, Config.game.mapSystemZoom * e:getScale(), { r = 0.8, g = 0.3, b = 0.8, a = 0.7 })
    end
  end
  Draw.Color(1, 1, 1, 1)
  Draw.SmoothPoints(false)
  BlendMode.Pop()

  if Input.GetDown(Button.Mouse.Left) then
    self.focus = best
    -- If focused-on object in the System Map is a ship or a station, make it the current target
    if Config.game.currentShip ~= self.focus and Config.game.currentShip:getTarget() ~= self.focus then
      if self.focus:getType() == Config:getObjectTypeByName("object_types", "Ship")    or
         self.focus:getType() == Config:getObjectTypeByName("object_types", "Station") then
        Config.game.currentShip:setTarget(self.focus)
      end
    end
  end

  do -- Debug Info
    local dbg = DebugContext(16, 16)
    dbg:text("--- System ---")
    dbg:indent()
    self.system:send(Event.Debug(dbg))
    dbg:undent()

    if self.focus then
      local boomtext = ""
      if self.focus:isDestroyed() then boomtext = " (destroyed)" end
      dbg:text('')
      dbg:text("--- %s %s%s ---", Config:getObjectInfo("object_types", self.focus:getType()), self.focus:getName(), boomtext)
      dbg:indent()
      self.focus:send(Event.Debug(dbg))
      dbg:undent()
    end
  end
end

function SystemMap:onInput (state)
  Config.game.mapSystemZoom = Config.game.mapSystemZoom * exp(kZoomSpeed * Input.GetMouseScroll().y)
  Config.game.mapSystemPos.x = Config.game.mapSystemPos.x + (kPanSpeed * state.dt / Config.game.mapSystemZoom) * (
    Input.GetValue(Button.Keyboard.D) - Input.GetValue(Button.Keyboard.A))
  Config.game.mapSystemPos.y = Config.game.mapSystemPos.y + (kPanSpeed * state.dt / Config.game.mapSystemZoom) * (
    Input.GetValue(Button.Keyboard.S) - Input.GetValue(Button.Keyboard.W))
  Config.game.mapSystemZoom = Config.game.mapSystemZoom * exp(10.0 * kZoomSpeed * state.dt * (
    Input.GetValue(Button.Keyboard.P) - Input.GetValue(Button.Keyboard.O)))

  if Input.GetPressed(Bindings.MoveTo) then
    if not Config.game.currentShip:isDestroyed() and not Config.game.shipDocked and self.focus ~= nil and self.focus ~= Config.game.currentShip then
      if Config.game.currentShip:getCurrentAction() == nil or not string.find(Config.game.currentShip:getCurrentAction():getName(),"MoveTo") then
        -- Move undestroyed, undocked player ship to area of selected target
        local autodistance = Config.game.autonavRanges[self.focus:getType()]
        Config.game.autonavTimestamp = Config.getCurrentTimestamp()
        Config.game.playerMoving = true -- must be set to true before pushing the MoveTo action
        Config.game.currentShip:pushAction(Actions.MoveTo(self.focus, autodistance))
printf("-> %s at time %s, range = %s (moving = %s)",
  Config.game.currentShip:getCurrentAction():getName(), Config.game.autonavTimestamp, autodistance, Config.game.playerMoving)
      end
    end
  end
end

function SystemMap.Create (system)
  local self = setmetatable(UI.Window('System Map', false), SystemMap)
  self:setStretch(1, 1)
  self.system = system
  if Config.game.currentShip ~= nil then
    if Config.game.mapSystemPos == nil then
      -- Initialize system map starting position only if not already initialized
      Config.game.mapSystemPos = Config.game.currentShip:getPos()
      -- Adjust map centering with magic numbers to center on current position of player's ship
      Config.game.mapSystemPos.x = Config.game.mapSystemPos.x - 803
      Config.game.mapSystemPos.y = Config.game.mapSystemPos.y - 449
    end
  else
    Config.game.mapSystemPos = Vec2f(0, 0)
  end
  if Config.game.mapSystemZoom == nil then
    -- Initialize system map zoom level only if not already initialized
    Config.game.mapSystemZoom = 0.01
  end
  return self
end

return SystemMap
