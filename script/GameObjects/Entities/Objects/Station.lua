local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')
local Components = requireAll('GameObjects.Elements.Components')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local function damaged (self, event)
    local shipEntry = self:findInDamageList(event.source)
    if shipEntry ~= nil then
        shipEntry.damage = shipEntry.damage + event.amount
        print("Damage done: " .. shipEntry.damage)
    else
        shipEntry = {
            ship = event.source,
            damage = event.amount
        }
        table.insert(self.shipDamageList, shipEntry)
    end

    if shipEntry.damage > 100 then
        if not self:isDestroyed() and self:getOwner() ~= shipEntry.ship then
            -- Nobody enjoys getting shot
            self:modDisposition(shipEntry.ship, -0.2)

            -- Possibly make this station undockable to its attacker
            if self:hasDockable() and self:isDockable() then
                if self:isHostileTo(shipEntry.ship) and not self:isBanned(shipEntry.ship) then
                    self:distressCall(shipEntry.ship, 15000)
                    self:undockAndAttack(shipEntry.ship)
                    self:addBannedShip(shipEntry.ship)
                    printf("Station %s bans attacker %s", self:getName(), shipEntry.ship:getName())
                end
            end
        end
    end
end

local Station = subclass(Entity, function(self, seed, hull)
    local rng = RNG.Create(seed)
    local mesh = Gen.StationOld(seed):managed()
    local bsp = BSP.Create(mesh):managed()
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
    self.countHull        = Config.gen.stationComponents[Enums.StationComponents.Hull][hull]
    self.countComputer    = Config.gen.stationComponents[Enums.StationComponents.Computer][hull]
    self.countSensor      = Config.gen.stationComponents[Enums.StationComponents.Sensor][hull]
    self.countLifeSupport = Config.gen.stationComponents[Enums.StationComponents.LifeSupport][hull]
    self.countCapacitor   = Config.gen.stationComponents[Enums.StationComponents.Capacitor][hull]
    self.countTurret      = Config.gen.stationComponents[Enums.StationComponents.Turret][hull]
    self.countBay         = Config.gen.stationComponents[Enums.StationComponents.Bay][hull]
    self.countInventory   = Config.gen.stationComponents[Enums.StationComponents.Inventory][hull]
    self.countDrone       = Config.gen.stationComponents[Enums.StationComponents.Drone][hull]
    self.countShield      = Config.gen.stationComponents[Enums.StationComponents.Shield][hull]
    self.countArmor       = Config.gen.stationComponents[Enums.StationComponents.Armor][hull]

    self:addComponents()

    -- Add all the _positions_ for socketable components (the components are added later)
    self.positions = {
        [SocketType.Hull]        = {},
        [SocketType.Computer]    = {},
        [SocketType.Sensor]      = {},
        [SocketType.LifeSupport] = {},
        [SocketType.Capacitor]   = {},
        [SocketType.Thruster]    = {},
        [SocketType.Turret]      = {},
        [SocketType.Bay]         = {},
        [SocketType.Inventory]   = {},
        [SocketType.Drone]       = {},
        [SocketType.Shield]      = {},
        [SocketType.Armor]       = {},
    }

    -- Hull sockets
    insert(self.positions[SocketType.Hull], Vec3f(1, 1, 1))

    -- Computer sockets
    for i = 1, self.countComputer do
        insert(self.positions[SocketType.Computer], Vec3f(1, 1, 1))
    end

    -- Sensor sockets
    for i = 1, self.countSensor do
        insert(self.positions[SocketType.Sensor], Vec3f(1, 1, 1))
    end

    -- Life Support sockets
    for i = 1, self.countLifeSupport do
        insert(self.positions[SocketType.LifeSupport], Vec3f(1, 1, 1))
    end

    -- Capacitor sockets
    for i = 1, self.countCapacitor do
        insert(self.positions[SocketType.Capacitor], Vec3f(1, 1, 1))
    end

    -- Turret sockets
    -- TODO: Define mount point 'p' such that a turret points away from the station's center line
    for i = 1, self.countTurret do
        local normal = Vec3f(0, 1, 0)
        local facing = Vec3f(0, 0, -1)
        local dir = rng:choose({ 1, 2, 3, 4 })
        if dir == 1 then
            normal = Vec3f(0, 0, -1)
            facing = Vec3f(0, 0, -1)
        elseif dir == 2 then
            normal = Vec3f(0, -1, 0)
            facing = Vec3f(0, -1, 0)
        elseif dir == 3 then
            normal = Vec3f(-1, 0, 1)
            facing = Vec3f(-1, 0, 1)
        elseif dir == 4 then
            normal = Vec3f(1, 1, 0)
            facing = Vec3f(1, 1, 0)
        end
        local p = Gen.GenUtil.FindMountPoint(mesh, bsp, rng, normal, facing, 1000)
        if p then
            insert(self.positions[SocketType.Turret], p * Vec3f(1, 1, 1))
        else
            printf("No mount point found for turret %d being mounted on Station %s", i, self:getName())
        end
    end

    -- Bay sockets
    for i = 1, self.countBay do
        local p = Gen.GenUtil.FindMountPoint(mesh, bsp, rng, Vec3f(0, 1, 0), Vec3f(0, 0, 1), 1000)
        if p then
            insert(self.positions[SocketType.Bay], Vec3f(0, 1, 1)) -- TODO: Replace with visible mount position for a Bay weapon
        else
            printf("No mount point found for bay %d being mounted on Station %s", i, self:getName())
        end
    end

    -- Inventory sockets
    for i = 1, self.countInventory do
        insert(self.positions[SocketType.Inventory], Vec3f(1, 1, 1))
    end

    -- Drone sockets
    for i = 1, self.countDrone do
        insert(self.positions[SocketType.Drone], Vec3f(1, 1, 1)) -- TODO: Replace with visible mount position for a Drone rack
    end

    -- Shield sockets
    for i = 1, self.countShield do
        insert(self.positions[SocketType.Shield], Vec3f(1, 1, 1))
    end

    -- Armor sockets
    for i = 1, self.countArmor do
        insert(self.positions[SocketType.Armor], Vec3f(1, 1, 1))
    end

    -- Add all sockets to parent
    -- TODO : Suggestive that JS-style prototype objects + 'clone' would work
    --        better for ShipType etc.
    self:addSockets()

    for type, elems in pairs(self.positions) do
        for i, pos in ipairs(elems) do
            self:addSocket(type, pos, true)
        end
    end

    self:setDrag(10, 10) -- fix station in place
    self:setScale(Config.gen.scaleStation)

    self:setMass(Config.gen.stationHullMass[hull])

    self.explosionSize = 512 -- destroyed stations have visually larger explosions than ships
    self.shipDamageList = {}
    self.lastClearDamageTime = 0
    self.timer = 0
    self.stationPatrolJobs = 0
    self:register(Event.Update, Entity.updateStation)
    self:register(Event.Damaged, damaged)
end)

function Station:attackedBy(target)
    -- This station has been attacked, probably by a band of ragtag rebel scum who pose no threat
    -- TODO: Allow a number of "grace" hits that decay over time
    if not self:isDestroyed() then
        --printf("Station %s (health at %3.2f%%) attacked by %s", self:getName(), self:mgrHullGetHullPercent(), target:getName())
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

function Station:undockAndAttack(target)
    for key, ship in pairs(self:getDocked(self)) do
        self:removeDocked(ship)
        ship:pushAction(Actions.Attack(target))
    end
end

function Entity:updateStation(state)
    if self.timer > self.lastClearDamageTime + 30 then
        self.shipDamageList = {}
        self.lastClearDamageTime = self.timer
    end
    self.timer = self.timer + state.dt
end

function Station:undockAndAttack(target)
  --for key, ship in pairs(self:getDocked(self)) do
  --  self:removeDocked(ship)
  --  ship:pushAction(Actions.Attack(target))
  --end
end

function Station:attackedBy (target)
  -- This station has been attacked, probably by a band of ragtag rebel scum who pose no threat
  -- TODO: Allow a number of "grace" hits that decay over time
  -- TODO: If and when stations are armed, modify this method to let the station shoot back
end

function Entity:updateStation(state)

  if self.timer > self.lastClearDamageTime + 30 then
    self.shipDamageList = {}
    self.lastClearDamageTime = self.timer
  end

  self.timer = self.timer + state.dt
end

return Station
