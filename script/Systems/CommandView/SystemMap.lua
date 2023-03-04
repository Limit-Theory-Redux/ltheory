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

  -- If an object is target locked in flight view (via HUD), give it focus in the System Map
  local playerShip = Config.game.currentShip
  local playerTarget = nil
  if playerShip ~= nil then
    if self.focus == playerShip then
      playerShip:setTarget(nil)
    end
    playerTarget = playerShip:getTarget()
  end
  if playerTarget ~= nil then
--printf("Targeting a %s", Config:getObjectInfo("object_types", playerTarget:getType()))
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
      Draw.PointSize(3.0)

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
        UI.DrawEx.Ring(x, y, Config.game.mapSystemZoom * e:getScale() * 10, { r = 0.1, g = 0.5, b = 1.0, a = 1.0 })
      end

      if e:hasYield() then
--printf("Yield: %s", e:getName())
        UI.DrawEx.Ring(x, y, Config.game.mapSystemZoom * e:getScale(), { r = 1.0, g = 0.5, b = 0.1, a = 0.5 })
      end

      if self.focus == e then
--printf("Focus: %s", e:getName())
        UI.DrawEx.Ring(x, y, 8, { r = 1.0, g = 0.0, b = 0.3, a = 1.0 })
      end

      -- Select the nearest object
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
    -- If focused-on object in the System Map is a ship (not the player's) or a station, make it the current target
    if Config.game.currentShip ~= nil and Config.game.currentShip ~= self.focus then
--      if self.focus:getType() == Config:getObjectTypeByName("object_types", "Ship")    or
--         self.focus:getType() == Config:getObjectTypeByName("object_types", "Station") then
        Config.game.currentShip:setTarget(self.focus)
--      else
        -- TODO: extend targetable objects in space (via HUD) to include asteroids and planets (or planetary colonies)
        --       For now, a non-Ship or non-Station was focused on, so clear the target for the HUD
--        Config.game.currentShip:setTarget(nil)
--      end
    end
  end

  do -- Debug Info
    local dbg = DebugContext(16, 16)
    dbg:text("--- System ---")
    dbg:indent()
    self.system:send(Event.Debug(dbg))
    dbg:undent()

    if self.focus then
      local objtype    = Config:getObjectInfo("object_types", self.focus:getType())
      local objsubtype = Config:getObjectSubInfo("object_types", self.focus:getType(), self.focus:getSubType())
      local owner = self.focus:getOwner()
      local objval = 0
      local objemit = ""
      local boomtext = ""
      if self.focus:isDestroyed() then boomtext = " (destroyed)" end
      dbg:text("")
      dbg:text("--- %s %s %s%s ---", objsubtype, objtype, self.focus:getName(), boomtext)
      dbg:indent()
      if owner ~= nil then
        dbg:text("Owner: %s", owner:getName())
      else
        dbg:text("Owner: [None]")
      end
      objval = self.focus:getRadius()
      if string.match(objtype, "Planet") then
        objval = objval * 9 -- planets needs to be a certain radius for the game currently, so fake their reported radius for printing
      end
      objemit = "Radius: %d m"
      if objval > 120000000000 then
        objval = objval / 149598000000
        objemit = "Radius: %0.1f AU"
      elseif objval > 600000000 then
        objval = objval / 695700000
        objemit = "Radius: %0.1f Sr"
      elseif objval > 1000 then
        objval = objval / 1000
        objemit = "Radius: %0.1f km"
      end
      dbg:text(objemit, objval)
      objval = self.focus:getMass()
      objemit = "Mass: %0.0f kg"
      if objval > 1.0e28 then
        objval = objval / 1.99e30
        objemit = "Mass: %0.2f Sm"
      elseif objval > 5e23 then
        objval = objval / 5.97e24
        objemit = "Mass: %0.2f Em"
      elseif objval > 1000000 then
        objval = objval / 1000000
        objemit = "Mass: %0.1f mt"
      elseif objval > 907.18474 then
        objval = objval / 907.18474
        objemit = "Mass: %0.1f kt"
      end
      dbg:text(objemit, objval)
      if Config.game.currentShip then
        local posMe = Config.game.currentShip:getPos()
        local posIt = self.focus:getPos()
        objval = posMe:distance(posIt)
        -- TODO: Add check here to see if our ship is docked to the target, and set displayed range to 0 if so
        objemit = "Range: %0.0f m"
        if objval > 149598000 then
          objval = objval / 149598000
          objemit = "Range: %0.2f AU"
        elseif objval > 1000 then
          objval = objval / 1000
          objemit = "Range: %0.1f km"
        end
        dbg:text(objemit, objval)
      end
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
end

function SystemMap.Create (system)
  local self = setmetatable(UI.Window('System Map', false), SystemMap)
  self:setStretch(1, 1)
  self.system = system

  if Config.game.currentShip ~= nil then
    if Config.game.mapSystemPos == nil then
      -- Initialize system map starting position only if not already initialized
      Config.game.mapSystemPos = Config.game.currentShip:getPos()
    end
  else
    Config.game.mapSystemPos = Vec3f(0, 0, 0)
  end
  if Config.game.mapSystemZoom == nil then
    -- Initialize system map zoom level only if not already initialized
    Config.game.mapSystemZoom = 0.0001
  end
  return self
end

return SystemMap
