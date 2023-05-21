local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local Station = subclass(Entity, function (self, seed, hull)
  local mesh = Gen.StationOld(seed):managed()
  self:addRigidBody(true, mesh)
  self:addVisibleMesh(mesh, Material.Metal())

  self:addActions()
  self:addAttackable(true)
  self:addChildren()
  self:addDispositions()
  self:addExplodable()
  self:addMinable(false)
  self:addTrackable(true)

  self:addDockable()
  self:addFlows()

  -- TEMP: give each station the maximum number of every applicable component
  self.countHull      = Config.gen.stationComponents[Enums.StationComponents.Hull     ][hull]
  self.countComputer  = Config.gen.stationComponents[Enums.StationComponents.Computer ][hull]
  self.countSensor    = Config.gen.stationComponents[Enums.StationComponents.Sensor   ][hull]
  self.countLife      = Config.gen.stationComponents[Enums.StationComponents.Life     ][hull]
  self.countCapacitor = Config.gen.stationComponents[Enums.StationComponents.Capacitor][hull]
  self.countTurret    = Config.gen.stationComponents[Enums.StationComponents.Turret   ][hull]
  self.countBay       = Config.gen.stationComponents[Enums.StationComponents.Bay      ][hull]
  self.countCargo     = Config.gen.stationComponents[Enums.StationComponents.Cargo    ][hull]
  self.countDrone     = Config.gen.stationComponents[Enums.StationComponents.Drone    ][hull]
  self.countShield    = Config.gen.stationComponents[Enums.StationComponents.Shield   ][hull]
  self.countArmor     = Config.gen.stationComponents[Enums.StationComponents.Armor    ][hull]

  self:addHealth   (Config.gen.compHullStats.health          * self.countHull)
  self:addCapacitor(Config.gen.compCapacitorStats.chargeMax  * self.countCapacitor,
                    Config.gen.compCapacitorStats.chargeRate * self.countCapacitor)
  self:addInventory(Config.gen.compCargoStats.cargoUnits     * self.countCargo)
  self:addShield   (Config.gen.compShieldStats.strengthMax   * self.countShield,
                    Config.gen.compShieldStats.reviveRate    * self.countShield)
  self:addArmor    (Config.gen.compArmorStats.strength       * self.countArmor)

  self:addSockets()
  self.sockets = {
    [SocketType.Hull]      = {},
    [SocketType.Computer]  = {},
    [SocketType.Sensor]    = {},
    [SocketType.Life]      = {},
    [SocketType.Capacitor] = {},
    [SocketType.Turret]    = {},
    [SocketType.Bay]       = {},
    [SocketType.Cargo]     = {},
    [SocketType.Drone]     = {},
    [SocketType.Shield]    = {},
    [SocketType.Armor]     = {},
  }

  local rng = RNG.Create(seed)
  local bsp = BSP.Create(mesh):managed()

  -- TODO: Define mount point 'p' such that a turret points away from the station's center line
  for i = 1, self.countTurret do
    local p = nil
    local normal = Vec3f( 0,  1,  0)
    local facing = Vec3f( 0,  0, -1)
    local dir = rng:choose({1, 2, 3, 4})
    if dir == 1 then
      normal = Vec3f( 0,  0, -1)
      facing = Vec3f( 0,  0, -1)
    elseif dir == 2 then
      normal = Vec3f( 0, -1,  0)
      facing = Vec3f( 0, -1,  0)
    elseif dir == 3 then
      normal = Vec3f(-1,  0,  1)
      facing = Vec3f(-1,  0,  1)
    elseif dir == 4 then
      normal = Vec3f( 1,  1,  0)
      facing = Vec3f( 1,  1,  0)
    end
    p = Gen.GenUtil.FindMountPoint(mesh, bsp, rng, normal, facing, 1000)
    if p then
      insert(self.sockets[SocketType.Turret], p * Vec3f(1, 1, 1))
    else
      printf("No mount point found for turret %d being mounted on Station %s", i, self:getName())
    end
  end
  for type, elems in pairs(self.sockets) do
    for i, pos in ipairs(elems) do
      self:addSocket(type, pos, true)
--printf("Added socket %d of type %s at pos %s", i, type, pos)
    end
  end

  self:setDrag(10, 10) -- fix station in place
  self:setScale(Config.gen.scaleStation)

  self:setMass(Config.gen.stationHullMass[hull])

  self.explosionSize = 512 -- destroyed stations have visually larger explosions than ships
end)

function Station:attackedBy (target)
  -- This station has been attacked, probably by a band of ragtag rebel scum who pose no threat
  -- TODO: Allow a number of "grace" hits that decay over time
  if not self:isDestroyed() then
--printf("Station %s (health at %3.2f%%) attacked by %s", self:getName(), self:getHealthPercent(), target:getName())
    -- Stations currently have no turrets, so pushing an Attack() action generates an error

    -- Nobody enjoys getting shot
    self:modDisposition(target, -0.2)

    -- Possibly make this station undockable to its attacker
    if self:hasDockable() and self:isDockable() then
      if self:isHostileTo(target) and not self:isBanned(target) then
        self:addBannedShip(target)
printf("Station %s bans attacker %s", self:getName(), target:getName())

        -- If this station is not currently attacking its attacker,
        --    add an action to Attack its attacker
        if self:hasActions() then
          local actionName = format("Attack %s", target:getName()) -- must match namegen in Attack.lua
          local attackAction = self:findAction(actionName)
          if attackAction then
            if attackAction ~= self:getCurrentAction(actionName) then
              -- If the action to attack the attacker exists in this entity's Actions queue but isn't the current
              --     action, delete the old Attack action and push a new instance to the top of the Actions queue
              self:deleteAction(actionName)
              self:pushAction(Actions.Attack(target))
            end
          else
            self:pushAction(Actions.Attack(target))
          end
        else
          self:pushAction(Actions.Attack(target))
        end
      end
    end
  end
end

return Station
