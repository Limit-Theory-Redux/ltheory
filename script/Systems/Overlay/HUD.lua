local Bindings = require('States.ApplicationBindings')
local CameraBindings = require('Systems.Controls.Bindings.CameraBindings')
local ShipBindings = require('Systems.Controls.Bindings.ShipBindings')
local Disposition = require('GameObjects.Components.NPC.Dispositions')
local Entity = require('GameObjects.Entity')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local HUD = {}
HUD.__index = HUD
setmetatable(HUD, UI.Panel)

HUD.name = 'HUD'
HUD.focusable = true
HUD:setPadUniform(8)

local dockingAllowed = true

function HUD:onEnable ()
  -- TODO : Wtf does this do? Who wrote this?? WHY.
  local pCamera = self.gameView.camera
  local camera = self.gameView.camera

  -- Lock camera back to player ship when HUD is enabled!
  -- (e.g., changing from "Dock" control -> "Ship" control in MasterControl.lua)
  self.gameView:setOrbit(false)

  camera:warp()
  camera:lerpFrom(pCamera.pos, pCamera.rot)
end

function HUD:controlThrust (e)
  if not e:hasThrustController() then return end
  local c = e:getThrustController()
  c:setThrust(
    ShipBindings.ThrustZ:get(),
    ShipBindings.ThrustX:get(),
    0,
    ShipBindings.Yaw:get(),
   -ShipBindings.Pitch:get(),
    ShipBindings.Roll:get(),
    ShipBindings.Boost:get())
  self.aimX = c.yaw
  self.aimY = c.pitch
end

function HUD:controlTurrets (e)
  -- TODO: Should this really be here in HUD.lua?
  local targetPos, targetVel
  local target = e:getTarget()

  if target and target:getOwnerDisposition(self.player) <= 0.0 then
    targetPos = target:getPos()
    targetVel = target:getVelocity()
  end

  local firing   = ShipBindings.Fire:get() > 0 and 1 or 0
  local camera   = self.gameView.camera
  local ndc      = Vec3f(self.aimX, self.aimY)
  local fallback = camera:mouseToRay(1):getPoint(e.socketRangeMin)

  -- Compute a firing solution separately for each turret to support
  -- different projectile velocities & ranges
  for turret in e:iterSocketsByType(SocketType.Turret) do
    if Config.game.autoTarget and targetPos then
      turret:aimAtTarget(target, fallback)
    else
      turret:aimAt(fallback)
    end
    turret.firing = firing
  end
end

function HUD:controlTargetLock (e)
  if ShipBindings.LockTarget:get() > 0.5 then e:setTarget(self.target) end
  if ShipBindings.ClearTarget:get() > 0.5 then e:setTarget(nil) end
end

function HUD:drawTargets (a)
  if not Config.ui.showTrackers then return end
  local camera = self.gameView.camera

  local cTarget = Color(0.5, 1.0, 0.1, 1.0 * a)
  local cLock =   Color(1.0, 0.5, 0.1, 1.0 * a)

  local player = self.player
  local playerShip = player:getControlling()
  local playerTarget = playerShip:getTarget()

  local closest = nil
  local minDist = 128
  local center = Vec2f(self.sx / 2, self.sy / 2)

  for i = 1, #self.targets.tracked do
    local target = self.targets.tracked[i]
    if target and target ~= playerShip then
      if target:getTrackable() then
        local pos = target:getPos()
        local ndc = camera:worldToNDC(pos)
        local ndcMax = max(abs(ndc.x), abs(ndc.y))

--        local disp = target:getOwnerDisposition(player) -- might need to switch back to this version
        local disp = Config.game.dispoNeutral -- disposition to neutral by default
        if target:hasAttackable() and target:getAttackable() then disp = target:getDisposition(playerShip) end
--        local c = target:getDispositionColor(disp) -- this version is preserved for future changes (esp. faction)
        local c = Disposition.GetColor(disp)

        c.a = a * c.a
        if ndcMax <= 1.0 and ndc.z > 0 then
          do
            -- Get tracker box extents based on object size, and adjust inward slightly
            local bx1, by1, bsx, bsy = camera:entityToScreenRect(target)
            bx1 = bx1 + 20
            by1 = by1 + 20
            local bx2, by2 = bx1 + bsx, by1 + bsy
            bx2 = bx2 - 40
            by2 = by2 - 40

            -- Draw rounded box corners
            --local a = a * (1.0 - exp(-0.5 * max(0.0, max(bsx, bsy) - 2.0)))
            if target:hasAttackable() and target:getAttackable() then
              UI.DrawEx.Wedge(bx2, by1, 4, 4, 0.125, 0.2, c)
              UI.DrawEx.Wedge(bx1, by1, 4, 4, 0.375, 0.2, c)
              UI.DrawEx.Wedge(bx1, by2, 4, 4, 0.625, 0.2, c)
              UI.DrawEx.Wedge(bx2, by2, 4, 4, 0.875, 0.2, c)
            end
            if playerTarget == target then
              UI.DrawEx.Wedge(bx2, by1, 12, 12, 0.125, 0.3, cLock)
              UI.DrawEx.Wedge(bx1, by1, 12, 12, 0.375, 0.3, cLock)
              UI.DrawEx.Wedge(bx1, by2, 12, 12, 0.625, 0.3, cLock)
              UI.DrawEx.Wedge(bx2, by2, 12, 12, 0.875, 0.3, cLock)
            elseif self.target == target then
              UI.DrawEx.Wedge(bx2, by1, 8, 8, 0.125, 0.2, cTarget)
              UI.DrawEx.Wedge(bx1, by1, 8, 8, 0.375, 0.2, cTarget)
              UI.DrawEx.Wedge(bx1, by2, 8, 8, 0.625, 0.2, cTarget)
              UI.DrawEx.Wedge(bx2, by2, 8, 8, 0.875, 0.2, cTarget)
            end

            -- Draw target name
            if playerTarget == target then
              local targetName = target:getName()
              if target:getType() == Config:getObjectTypeByName("object_types", "Planet") then
                targetName = "Planet " .. target:getName()
              elseif target:getType() == Config:getObjectTypeByName("object_types", "Asteroid") then
                targetName = "Asteroid " .. target:getName()
              elseif target:getType() == Config:getObjectTypeByName("object_types", "Station") then
                targetName = "Station " .. target:getName()
              elseif target:getType() == Config:getObjectTypeByName("object_types", "Jumpgate") then
                targetName = "Jumpgate " .. target:getName()
              end
              local tcr = 1
              local tcg = 1
              local tcb = 1
              if target:isDestroyed() then
                tcr = 0
                tcg = 0
                tcb = 0
              end
              UI.DrawEx.TextAdditive(
                'NovaRound',
                targetName,
                10,
                (bx1 + bx2) / 2 - targetName:len() / 2, by1 - 30, targetName:len(), 20,
                tcr, tcg, tcb, a,
                0.5, 0.5
              )
            end

            -- Draw target health bar
            if playerTarget == target and target:hasHealth() then
              local targetHealthPct = target:getHealthPercent()
              if targetHealthPct > 0.0 then
                local targetHealthCI = math.min(50, math.floor((targetHealthPct / 2.0) + 0.5) + 1)
                UI.DrawEx.RectOutline(bx1 + 2, by2 - 3, (bx2 - bx1) - 6, 8, Config.ui.color.borderBright)
                UI.DrawEx.Rect(bx1 + 3, by2 - 1, (bx2 - bx1) - 8, 4, Config.ui.color.healthColor[targetHealthCI])
              end
            end
          end

          local ss = camera:ndcToScreen(ndc)
          local dist = ss:distance(center)
          if disp < 0.5 and dist < minDist then
            closest = target
            minDist = dist
          end
        else
          ndc.x = ndc.x / ((1 + 16/camera.sx) * ndcMax)
          ndc.y = ndc.y / ((1 + 16/camera.sy) * ndcMax)
          local x = ( ndc.x + 1)/2 * camera.sx
          local y = (-ndc.y + 1)/2 * camera.sy
          if disp < 0.0 then
            c.a = c.a * 0.5
            UI.DrawEx.Point(x, y, 64, c)
          end
        end
      end
    end
  end

  self.target = closest
end

function HUD:drawLock (a)
  local playerShip = self.player:getControlling()
  local target = playerShip:getTarget()

  if not target or target:isDestroyed() then return end

  local camera = self.gameView.camera
  local center = Vec2f(self.sx / 2, self.sy / 2)

  do -- Direction indicator
    local r = 96
    local pos = target:getPos()
    local ndc = camera:worldToNDC(pos)
    local ndcMax = max(abs(ndc.x), abs(ndc.y))
    if ndcMax > 1 or ndc.z <= 0 then ndc:idivs(ndcMax) end
    local ss = camera:ndcToScreen(ndc)
    local dir = ss - center
    local dist = dir:length()
    if dist > 1 then
      dir:inormalize()
      ss = center + dir:scale(r)
      local a = a * (1.0 - exp(-max(0.0, dist / (r + 16) - 1.0)))
      UI.DrawEx.Arrow(ss, dir:scale(6), Color(1.0, 0.5, 0.1, a))
    end
  end

  -- Predictive impact point
  -- Takes into account player's movement, target's movement,
  --   and the speed of the currently selected weapon/projectile
  -- TEMP: change reference to Config.game.pulseRange from App.lua when multiple weapon types are available
  local range = playerShip:getPos():distance(target:getPos())
  if target:hasAttackable() and target:getAttackable() and range < Config.game.pulseRange then
    if playerShip.socketSpeedMax > 0 then
      local tHit, pHit = Math.Impact(
        playerShip:getPos(),
        target:getPos(),
        playerShip:getVelocity(),
        target:getVelocity(),
        playerShip.socketSpeedMax)

      if tHit then
        local ndc = camera:worldToNDC(pHit)
        local ndcMax = max(abs(ndc.x), abs(ndc.y))
        if ndcMax <= 1 and ndc.z > 0 then
          local ss = camera:ndcToScreen(ndc)
          UI.DrawEx.Ring(ss.x, ss.y, 10, Color(1.0, 0.3, 0.3, a))
        end
      end
    end
  end
end

function HUD:drawReticle (a)
  local cx, cy = self.sx / 2, self.sy / 2
  do -- Reticle
    do -- Central Crosshair
      local c = Color(0.1, 0.5, 1.0, a)
      local phase = 0.125
      local r1 = 24
      local r2 = 28
      local n = 3
      for i = 0, n - 1 do
        local angle = -(Math.Pi2 + (i / n) * Math.Tau)
        local dx, dy = cos(angle), sin(angle)
        UI.DrawEx.Line(cx + r1 * dx, cy + r1 * dy, cx + r2 * dx, cy + r2 * dy, c)
      end
    end

    if false then -- Aim (not terribly useful now, but preserved in case we can improve it)
      local c = Color(0.1, 0.5, 1.0, a)
      local yaw, pitch = ShipBindings.Yaw:get(), ShipBindings.Pitch:get()
      local x = cx + 0.5 * self.sx * self.aimX
      local y = cy - 0.5 * self.sy * self.aimY
      UI.DrawEx.Ring(x, y, 16, c)
    end
  end
end

function HUD:drawPlayerHealth (a)
  local cx, cy = self.sx / 2, self.sy / 2
  local x, y, sx, sy = self:getRectGlobal()
  local playerShip = self.player:getControlling()
  local playerRadius = playerShip:getRadius()
  local playerHealthPct = playerShip:getHealthPercent()
  local playerHealthText = format("Health: %3.2f%%", playerHealthPct)
  local playerHealthCI = math.min(50, math.floor((playerHealthPct / 2.0) + 0.5) + 1)

  -- Draw text of player ship name
  UI.DrawEx.TextAdditive(
    'NovaRound',
    playerShip:getName(),
    10,
    112, sy - 240, 100, 12,
    1, 1, 1, a,
    0.5, 0.5
  )

  -- Draw hologram of player ship
--local yaw, pitch = ShipBindings.Yaw:get(), ShipBindings.Pitch:get()
--printf("x = %d, y = %d, sx = %d, sy = %d", x, y, sx, sy)
--printf("radius = %3.2f, yaw = %3.2f, pitch = %3.2f", radius, yaw, pitch)
--printf("radius = %3.2f, radius / 1.7 = %3.2f", radius, radius / 1.7)
  local hc = Color(1, 1, 1, 1)
  hc.r = Config.ui.color.healthColor[playerHealthCI].r
  hc.g = Config.ui.color.healthColor[playerHealthCI].g
  hc.b = Config.ui.color.healthColor[playerHealthCI].b
  hc.a = 1.0
  UI.DrawEx.Hologram(playerShip.mesh, 20, sy - 260, 260, 260, hc, playerRadius / 1.7, -1.5, 0.0)

  -- Draw text of player ship health
  UI.DrawEx.TextAdditive(
    'NovaRound',
    playerHealthText,
    10,
    112, sy - 34, 100, 12,
    1, 1, 1, a,
    0.5, 0.5
  )

  UI.DrawEx.RectOutline(cx - 22, cy + 18, 44, 8, Config.ui.color.borderBright)
  UI.DrawEx.Rect(cx - 20, cy + 20, 40, 4, Config.ui.color.healthColor[playerHealthCI])

end

function HUD:drawTargetHealth (a)
  local playerShip = self.player:getControlling()
  local target = playerShip:getTarget()
  if target and target:hasHealth() then
    local cx, cy = self.sx / 2, self.sy / 2
    local x, y, sx, sy = self:getRectGlobal()
    local targetName = target:getName()
    local targetHealthPct = target:getHealthPercent()
    if targetHealthPct > 0.0 then
      local targetHealthText = format("Health: %3.2f%%", targetHealthPct)
      local targetHealthCI = math.min(50, math.floor((targetHealthPct / 2.0) + 0.5) + 1)
      local targetRadius = target:getRadius()
      local targetRadiusAdj = targetRadius

      if target:getType() == Config:getObjectTypeByName("object_types", "Ship")    then
        targetRadiusAdj = 5.9
      end
      if target:getType() == Config:getObjectTypeByName("object_types", "Station") then
        targetRadiusAdj = 26
        targetName = "Station " .. target:getName()
      end

      -- Draw text of target name
      UI.DrawEx.TextAdditive(
        'NovaRound',
        targetName,
        10,
        sx - 208, sy - 240, 100, 12,
        1, 1, 1, a,
        0.5, 0.5
      )

      -- Draw hologram of target entity
      local hc = Color(1, 1, 1, 1)
      hc.r = Config.ui.color.healthColor[targetHealthCI].r
      hc.g = Config.ui.color.healthColor[targetHealthCI].g
      hc.b = Config.ui.color.healthColor[targetHealthCI].b
      hc.a = 1.0
      UI.DrawEx.Hologram(target.mesh, sx - 300, sy - 260, 260, 260, hc, targetRadiusAdj, -1.5, 0.0)

      -- Draw text of target health
      UI.DrawEx.TextAdditive(
        'NovaRound',
        targetHealthText,
        10,
        sx - 208, sy - 34, 100, 12,
        1, 1, 1, a,
        0.5, 0.5
      )
    end
  end
end

function HUD:drawDockPrompt (a)
  local x, y, sx, sy = self:getRectGlobal()
  local dockText = nil

  if dockingAllowed then
    dockText = "Press F to Dock" -- TODO: connect Docking input to bindings
  else
    dockText = "Docking is refused at this Station"
  end

  UI.DrawEx.TextAdditive(
    'NovaMono',
    dockText,
    16,
    x, y, sx, sy,
    1, 1, 1, self.dockPromptAlpha * a,
    0.5, 0.96
  )
end

function HUD:onInput (state)
  local camera = self.gameView.camera
  camera:push()
  camera:modRadius(exp(-0.1 * CameraBindings.Zoom:get()))
  -- camera:modYaw(0.005 * CameraBindings.Yaw:get())
  -- camera:modPitch(0.005 * CameraBindings.Pitch:get())

  local e = self.player:getControlling()
  if not e:isDestroyed() then
    self:controlThrust(e)
    self:controlTurrets(e)
    self:controlTargetLock(e)
  end

  camera:pop()

  if self.dockable then
--printf("%s %s is dockable = %s", Config:getObjectInfo("object_types", self.dockable:getType()), self.dockable:getName(), self.dockable:isDockable())
    if self.dockable:isDockable() then
      if not Config.game.gamePaused and ShipBindings.Dock:get() > 0 then
        -- TODO: migrate this action outside the HUD
        e:pushAction(Actions.DockAt(self.dockable))
        self.dockable = nil
      end
    end
  end
end

function HUD:onUpdate (state)
  if not Config.game.gamePaused then
    if Input.GetPressed(Bindings.ToggleHUD) then
      Config.ui.HUDdisplayed = not Config.ui.HUDdisplayed
    end

    self.targets:update()
    self.dockables:update()

    self.dockable = HUD:getDockable(self)

    local f = 1.0 - exp(-state.dt * 8.0)
    local alphaT = 0
    if self.dockable then
      if self.dockable:isDockable() then
        dockingAllowed = true
        alphaT = 1
      else
        dockingAllowed = false
        if not self.dockable:isDestroyed() and self.dockable:isHostileTo(self.player:getControlling()) then
          alphaT = 1
        else
          alphaT = 0
        end
      end
    end
    self.dockPromptAlpha = Math.Lerp(self.dockPromptAlpha, alphaT, f)
  end
end

function HUD:getDockable (self)
  local dockableObj = nil

  local pPos    = self.player:getControlling():getPos()
  local pRad    = self.player:getControlling():getRadius()
  self.dockable = nil
  for i = 1, #self.dockables.tracked do
    local dockable = self.dockables.tracked[i]

    local dPos = dockable:getPos()
    local dRad = dockable:getRadius()
    local dist = pPos:distance(dPos) - pRad - dRad
    if dist < Config.game.dockRange then
      -- return the Entity instance of the first dockable object found (might not be closest if several are within range)
      dockableObj = dockable
      break
    end
  end

  return dockableObj
end

function HUD:onDraw (focus, active)
  local playerShip = self.player:getControlling()
  if playerShip:isAlive() then
    if Config.ui.HUDdisplayed then
      Profiler.Begin('HUD.DrawTargets')      self:drawTargets     (self.enabled) Profiler.End()
      Profiler.Begin('HUD.DrawLock')         self:drawLock        (self.enabled) Profiler.End()
      Profiler.Begin('HUD.DrawPlayerHealth') self:drawPlayerHealth(self.enabled) Profiler.End()
      Profiler.Begin('HUD.DrawTargetHealth') self:drawTargetHealth(self.enabled) Profiler.End()
    end

    Profiler.Begin('HUD.DrawReticle') self:drawReticle   (self.enabled) Profiler.End()
    Profiler.Begin('HUD.DrawPrompt')  self:drawDockPrompt(self.enabled) Profiler.End()
  end
end

function HUD:onDrawIcon (iconButton, focus, active)
  local borderColor = iconButton == active
                      and Config.ui.color.controlActive
                      or iconButton == focus
                         and Config.ui.color.controlFocused
                         or Config.ui.color.control
  local contentColor = self:isEnabled()
                       and Config.ui.color.controlFocused
                       or Config.ui.color.control

  local x, y, sx, sy = iconButton:getRectGlobal()
  UI.DrawEx.RectOutline(x, y, sx, sy, borderColor)

  local cx = x + sx/2
  local w1y, w1sx, w1sy = 10, 10, 8
  local w2y, w2sx, w2sy =  0,  5, 4
  local ty, by = y + 8, y + sy - 12
  UI.DrawEx.Line(cx,     ty,       cx,        by,              contentColor)
  UI.DrawEx.Line(cx + 2, ty + w1y, cx + w1sx, ty + w1y + w1sy, contentColor)
  UI.DrawEx.Line(cx - 2, ty + w1y, cx - w1sx, ty + w1y + w1sy, contentColor)
  UI.DrawEx.Line(cx + 2, by,       cx + w2sx, by + w2y + w2sy, contentColor)
  UI.DrawEx.Line(cx - 2, by,       cx - w2sx, by + w2y + w2sy, contentColor)
end

function HUD.Create (gameView, player)
  local self = setmetatable({
    gameView        = gameView,
    player          = player,
    icon            = UI.Icon(),

    target          = nil,
    targets         = Systems.CommandView.TrackingList(player, Entity.isTrackable),
--    targets         = Systems.CommandView.TrackingList(player, Entity.isAlive),

    -- TODO Probably want a reusable prompt thing
    dockPromptAlpha = 0,
    dockable        = nil,
    dockables       = Systems.CommandView.TrackingList(player, Entity.hasDockable),
    aimX            = 0,
    aimY            = 0,
    impacts         = 0,

    children  = List(),
  }, HUD)

  self.icon:setOnDraw(function (ib, focus, active)
    self:onDrawIcon(ib, focus, active)
  end)
  self.targets:update()
  self.dockables:update()
  return self
end

return HUD
