local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')
local Components = requireAll('GameObjects.Elements.Components')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local function wasDamaged(self, event)
    local shipEntry = self:findInDamageList(event.source)
    if shipEntry ~= nil then
        shipEntry.damage = shipEntry.damage + event.amount
        --print("Damage done: " .. shipEntry.damage)
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

            if self:hasActions() then
                local actionName = format("Attack %s", shipEntry.ship:getName()) -- must match namegen in Attack.lua
                local attackAction = self:findAction(actionName)
                if attackAction then
                    if attackAction ~= self:getCurrentAction(actionName) then
                        -- If the action to attack the attacker exists in this entity's Actions queue but isn't the current
                        -- action, delete the old Attack action and push a new instance to the top of the Actions queue
                        self:deleteAction(actionName)
                        self:pushAction(Actions.Attack(shipEntry.ship))
                    end
                else
                    self:pushAction(Actions.Attack(shipEntry.ship))
                end
            else
                self:pushAction(Actions.Attack(shipEntry.ship))
            end

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
    local mesh = Gen.StationOld(seed)
    local bsp = BSP.Create(mesh):managed()
    self:addRigidBody(true, mesh, Enums.ColliderType.ConvexDecomposition)
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
    self.countArmor     = Config.gen.stationComponents[Enums.StationComponents.Armor][hull]
    self.countBay       = Config.gen.stationComponents[Enums.StationComponents.Bay][hull]
    self.countCapacitor = Config.gen.stationComponents[Enums.StationComponents.Capacitor][hull]
    self.countCloak     = 0
    self.countCommo     = Config.gen.stationComponents[Enums.StationComponents.Commo][hull]
    self.countComputer  = Config.gen.stationComponents[Enums.StationComponents.Computer][hull]
    self.countDrone     = Config.gen.stationComponents[Enums.StationComponents.Drone][hull]
    self.countHull      = Config.gen.stationComponents[Enums.StationComponents.Hull][hull]
    self.countInventory = Config.gen.stationComponents[Enums.StationComponents.Inventory][hull]
    self.countSensor    = Config.gen.stationComponents[Enums.StationComponents.Sensor][hull]
    self.countShield    = Config.gen.stationComponents[Enums.StationComponents.Shield][hull]
    self.countThruster  = 0
    self.countTurret    = Config.gen.stationComponents[Enums.StationComponents.Turret][hull]

    self:addComponents()

    -- Add all the _positions_ for socketable components (the components are added later)
    self.positions = {
        [SocketType.Armor]     = {},
        [SocketType.Bay]       = {},
        [SocketType.Capacitor] = {},
        [SocketType.Cloak]     = {}, -- not used
        [SocketType.Commo]     = {},
        [SocketType.Computer]  = {},
        [SocketType.Drone]     = {},
        [SocketType.Hull]      = {},
        [SocketType.Inventory] = {},
        [SocketType.Sensor]    = {},
        [SocketType.Shield]    = {},
        [SocketType.Thruster]  = {}, -- not used
        [SocketType.Turret]    = {},
    }

    local p = nil

    -- Armor sockets
    for i = 1, self.countArmor do
        insert(self.positions[SocketType.Armor], Vec3f(1, 1, 1))
    end

    -- Bay sockets
    for i = 1, self.countBay do
        p = Gen.GenUtil.FindMountPoint(mesh, bsp, rng, Vec3f(0, 1, 0), Vec3f(0, 0, 1), 1000)
        if p then
            insert(self.positions[SocketType.Bay], Vec3f(0, 1, 1)) -- TODO: Replace with visible mount position for a Bay weapon
        else
            Log.Debug("No mount point found for bay %d being mounted on Station %s", i, self:getName())
        end
    end

    -- Capacitor sockets
    for i = 1, self.countCapacitor do
        insert(self.positions[SocketType.Capacitor], Vec3f(1, 1, 1))
    end

    -- Communicator sockets
    for i = 1, self.countCommo do
        insert(self.positions[SocketType.Commo], Vec3f(1, 1, 1))
    end

    -- Computer sockets
    for i = 1, self.countComputer do
        insert(self.positions[SocketType.Computer], Vec3f(1, 1, 1))
    end

    -- Drone sockets
    for i = 1, self.countDrone do
        insert(self.positions[SocketType.Drone], Vec3f(1, 1, 1)) -- TODO: Replace with visible mount position for a Drone rack
    end

    -- Hull sockets
    insert(self.positions[SocketType.Hull], Vec3f(1, 1, 1))

    -- Inventory sockets
    for i = 1, self.countInventory do
        insert(self.positions[SocketType.Inventory], Vec3f(1, 1, 1))
    end

    -- Sensor sockets
    for i = 1, self.countSensor do
        insert(self.positions[SocketType.Sensor], Vec3f(1, 1, 1))
    end

    -- Shield sockets
    for i = 1, self.countShield do
        insert(self.positions[SocketType.Shield], Vec3f(1, 1, 1))
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
        p = Gen.GenUtil.FindMountPoint(mesh, bsp, rng, normal, facing, 1000)
        if p then
            insert(self.positions[SocketType.Turret], p * Vec3f(1, 1, 1))
        else
            Log.Debug("No mount point found for turret %d being mounted on Station %s", i, self:getName())
        end
    end

    -- Add all sockets to parent
    -- TODO : Suggestive that JS-style prototype objects + 'clone' would work
    -- better for ShipType etc.
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
    self:register(OldEvent.Update, Entity.updateStation)
    self:register(OldEvent.Damaged, wasDamaged)
end)

function Station:undockAndAttack(target)
    for key, ship in pairs(self:getDocked(self)) do
        self:removeDocked(ship)
        ship:pushAction(Actions.Attack(target))
        printf("Ship %s defends Station %s", ship:getName(), self:getName())
    end
end

function Station:findInDamageList(entity)
    for _, shipEntry in ipairs(self.shipDamageList) do
        if shipEntry.ship == entity then
            return shipEntry
        end
    end
end

function Station:distressCall(target, range)
    local owner = self:getOwner()
    for asset in owner:iterAssets() do
        if asset:getType() == Config:getObjectTypeByName("object_types", "Ship") and self:getDistance(asset) < range then
            local currentAction = asset:getCurrentAction()

            if currentAction and not string.find(currentAction:getName(), "Attack") then
                asset:pushAction(Actions.Attack(target))
                print(asset:getName() .. " answering distress call of " .. self:getName())
            end
        end
    end
end

function Entity:updateStation(state)
    if self.timer > self.lastClearDamageTime + 30 then
        self.shipDamageList = {}
        self.lastClearDamageTime = self.timer
    end

    self.timer = self.timer + state.dt
end

return Station
