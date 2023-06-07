--[[ TODO ----------------------------------------------------------------------
  - Generators should output ShipTypes, not just meshes (so that generating
    algorithm can select mount points, etc)
  - Now that entities are well-unified, these metatypes should be unified as
    well; we only need one 'prototype' class.
----------------------------------------------------------------------------]]
--

local Ship = require('GameObjects.Entities.Ship.Ship')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local ShipType = class(function(self, seed, generator, hull)
    local scale           = 4 -- TODO: determine scale based on hull type

    local rng             = RNG.Create(seed)
    self.seed             = seed
    self.mesh             = generator(seed, hull, Config.gen.shipRes):managed()
    self.bsp              = BSP.Create(self.mesh):managed()
    self.scale            = scale

    -- Get the maximum number of components of each type for the provided hull size
    self.countHull        = Config.gen.shipComponents[Enums.ShipComponents.Hull][hull]
    self.countComputer    = Config.gen.shipComponents[Enums.ShipComponents.Computer][hull]
    self.countSensor      = Config.gen.shipComponents[Enums.ShipComponents.Sensor][hull]
    self.countLifeSupport = Config.gen.shipComponents[Enums.ShipComponents.LifeSupport][hull]
    self.countCapacitor   = Config.gen.shipComponents[Enums.ShipComponents.Capacitor][hull]
    self.countThruster    = Config.gen.shipComponents[Enums.ShipComponents.Thruster][hull]
    self.countTurret      = Config.gen.shipComponents[Enums.ShipComponents.Turret][hull]
    self.countBay         = Config.gen.shipComponents[Enums.ShipComponents.Bay][hull]
    self.countInventory   = Config.gen.shipComponents[Enums.ShipComponents.Inventory][hull]
    self.countDrone       = Config.gen.shipComponents[Enums.ShipComponents.Drone][hull]
    self.countShield      = Config.gen.shipComponents[Enums.ShipComponents.Shield][hull]
    self.countArmor       = Config.gen.shipComponents[Enums.ShipComponents.Armor][hull]

    printf(
        "@@@ ShipType:(create) - ship = %s, hull = %d, scale = %s, countThruster = %d, countTurret = %d, countInventory = %d",
        self, hull, scale, self.countThruster, self.countTurret, self.countInventory)

    -- Add all the _positions_ for socketable components (the components are added later)
    self.sockets = {
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
    insert(self.sockets[SocketType.Hull], Vec3f(1, 1, 1))

    -- Computer sockets
    for i = 1, self.countComputer do
        insert(self.sockets[SocketType.Computer], Vec3f(1, 1, 1))
    end

    -- Sensor sockets
    for i = 1, self.countSensor do
        insert(self.sockets[SocketType.Sensor], Vec3f(1, 1, 1))
    end

    -- Life Support sockets
    for i = 1, self.countLifeSupport do
        insert(self.sockets[SocketType.LifeSupport], Vec3f(1, 1, 1))
    end

    -- Capacitor sockets
    for i = 1, self.countCapacitor do
        insert(self.sockets[SocketType.Capacitor], Vec3f(1, 1, 1))
    end

    -- Thruster sockets
    local p = nil
    for i = 1, self.countThruster do
        if hull == Enums.ShipHulls.VeryLarge then
            p = Gen.GenUtil.FindMountPoint(self.mesh, self.bsp, rng, Vec3f(-1, 0, 1), Vec3f(1, 1, 0), 1000)
        else
            p = Gen.GenUtil.FindMountPoint(self.mesh, self.bsp, rng, Vec3f(0, 0, -1), Vec3f(0, 0, -1), 1000)
        end
        if p then
            insert(self.sockets[SocketType.Thruster], p * Vec3f(1, 1, 1))
            insert(self.sockets[SocketType.Thruster], p * Vec3f(-1, 1, 1))
        end
    end

    -- Turret sockets
    for i = 1, self.countTurret do
        p = Gen.GenUtil.FindMountPoint(self.mesh, self.bsp, rng, Vec3f(0, 1, 0), Vec3f(0, 0, 1), 1000)
        if p then
            insert(self.sockets[SocketType.Turret], p * Vec3f(1, 1, 1))
            insert(self.sockets[SocketType.Turret], p * Vec3f(-1, 1, 1))
        end
    end

    -- Bay sockets
    for i = 1, self.countBay do
        p = Gen.GenUtil.FindMountPoint(self.mesh, self.bsp, rng, Vec3f(0, 1, 0), Vec3f(0, 0, 1), 1000)
        if p then
            insert(self.sockets[SocketType.Bay], Vec3f(0, 1, 1)) -- TODO: Replace with visible mount position for a Bay weapon
        end
    end

    -- Inventory sockets
    for i = 1, self.countInventory do
        insert(self.sockets[SocketType.Inventory], Vec3f(1, 1, 1))
    end

    -- Drone sockets
    for i = 1, self.countDrone do
        insert(self.sockets[SocketType.Drone], Vec3f(1, 1, 1)) -- TODO: Replace with visible mount position for a Drone rack
    end

    -- Shield sockets
    for i = 1, self.countShield do
        insert(self.sockets[SocketType.Shield], Vec3f(1, 1, 1))
    end

    -- Armor sockets
    for i = 1, self.countArmor do
        insert(self.sockets[SocketType.Armor], Vec3f(1, 1, 1))
    end

    rng:free()
end)

-- TODO: change how this works and create a generalized code structure for creating ships as this is b***shit
function ShipType:instantiate(hull)
    printf("@@@ ShipType:instantiate - hull = %s", hull)
    return Ship(self, hull)
end

return ShipType
