-- Weapon Testbed: an armed capital ship fighting target drones, a few
-- kilometres from the player's ship in the default star system (it was the
-- separate Testbeds/WeaponSystem app state; that name now starts the game
-- straight into this scenario).
--
-- The capital is a layered capital hull with discovered mounts, a procedural
-- loadout and turrets (ArmedShipBuilder), run by the game's weapon combat
-- system like any armed ship (WeaponBattery): the weapon AI picks targets and
-- fire modes, the tracking aims, the weapon system fires beams and
-- projectiles, with impact and muzzle lights. Four drones of different sizes
-- orbit the capital; a destroyed drone comes back after a few seconds.
--
-- The camera starts orbiting the capital; C switches to flying your own ship.
-- Keys: F6 capital AI on/off; with the AI off, F7 cycles the fire mode (volley
-- / sequence) and F9 holds the trigger; R or B (Regenerate) brings all drones
-- back and starts the firing sequence over.
local Rulesets = require("Config.Gen.Rulesets")
local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents = require("Modules.Core.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local ArmedShipBuilder = require("Modules.Constructs.Managers.ArmedShipBuilder")
local WeaponBattery = require("Modules.Constructs.Systems.WeaponBattery")
local WeaponActions = require("Input.ActionBindings.WeaponTestbedActions")

local Weapon = Enums.Weapon
local ShipType = Enums.ShipType
local MountSurfaceBand = Weapon.MountSurfaceBand
local MountZone = Weapon.MountZone
local MountRole = Weapon.MountRole
local MountSizeClass = Weapon.MountSizeClass
local ProceduralKey = Weapon.ProceduralKey

-- The capital: hull, mount sockets, procedural weapons, capacitor banks and
-- loadout policy (an ArmedShipBuilder recipe).
local CAPITAL = {
    seed = 500,
    defaultWeapon = {
        generatedKey = ProceduralKey.BurstLaser,
    },
    ship = {
        shipType = ShipType.Capital,
        config = {
            generation = {
                id = Enums.ShipGeneration.LayeredCapital,
                mountDecks = {
                    sockets = {
                        {
                            pairId = Weapon.MountPairId.ForeOuter,
                            zone = MountZone.Fore,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.M,
                            mountRole = MountRole.Line,
                            z = 0.95,
                            xFactor = 0.78,
                        },
                        {
                            pairId = Weapon.MountPairId.ForeHeavy,
                            zone = MountZone.Fore,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.L,
                            mountRole = MountRole.Heavy,
                            z = 0.80,
                            xFactor = 0.40,
                        },
                        {
                            pairId = Weapon.MountPairId.ForeInner,
                            zone = MountZone.Fore,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = 0.68,
                            xFactor = 0.52,
                        },
                        {
                            pairId = Weapon.MountPairId.ForePdBottom,
                            zone = MountZone.Fore,
                            surfaceBand = MountSurfaceBand.Bottom,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = 0.68,
                            xFactor = 0.52,
                        },
                        {
                            pairId = Weapon.MountPairId.ForePdOuter,
                            zone = MountZone.Fore,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = 0.52,
                            xFactor = 0.28,
                        },
                        {
                            pairId = Weapon.MountPairId.MidPd,
                            zone = MountZone.Mid,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = 0.00,
                            xFactor = 0.78,
                        },
                        {
                            pairId = Weapon.MountPairId.MidPdBottom,
                            zone = MountZone.Mid,
                            surfaceBand = MountSurfaceBand.Bottom,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = 0.00,
                            xFactor = 0.78,
                        },
                        {
                            pairId = Weapon.MountPairId.Mid,
                            zone = MountZone.Mid,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.L,
                            mountRole = MountRole.Heavy,
                            z = -0.05,
                            xFactor = 0.52,
                        },
                        {
                            pairId = Weapon.MountPairId.MidOuter,
                            zone = MountZone.Mid,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.M,
                            mountRole = MountRole.Line,
                            z = 0.10,
                            xFactor = 0.88,
                        },
                        {
                            pairId = Weapon.MountPairId.FlakBattery,
                            zone = MountZone.Mid,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = 0.35,
                            xFactor = 0.60,
                        },
                        {
                            pairId = Weapon.MountPairId.RailgunBattery,
                            zone = MountZone.Fore,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.M,
                            mountRole = MountRole.Line,
                            z = 0.60,
                            xFactor = 0.62,
                        },
                        {
                            pairId = Weapon.MountPairId.AftInner,
                            zone = MountZone.Aft,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.L,
                            mountRole = MountRole.Missile,
                            z = -0.58,
                            xFactor = 0.52,
                        },
                        {
                            pairId = Weapon.MountPairId.MidHeavy,
                            zone = MountZone.Mid,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.M,
                            mountRole = MountRole.Missile,
                            z = -0.05,
                            xFactor = 0.88,
                        },
                        {
                            pairId = Weapon.MountPairId.AftPd,
                            zone = MountZone.Aft,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = -0.82,
                            xFactor = 0.78,
                        },
                        {
                            pairId = Weapon.MountPairId.AftPdBottom,
                            zone = MountZone.Aft,
                            surfaceBand = MountSurfaceBand.Bottom,
                            mountSizeClass = MountSizeClass.SM,
                            mountRole = MountRole.PointDefense,
                            z = -0.82,
                            xFactor = 0.78,
                        },
                        {
                            pairId = Weapon.MountPairId.AftOuter,
                            zone = MountZone.Aft,
                            surfaceBand = MountSurfaceBand.Top,
                            mountSizeClass = MountSizeClass.XL,
                            mountRole = MountRole.CapitalHeavy,
                            z = -1.05,
                            xFactor = 0.62,
                        },
                    },
                },
            },
            isKinematic = true,
        },
    },
    mountDiscovery = {
        options = {
            minNormalDot = 0.35,
            mirrorTolerance = 0.0001,
            requireStructuralSockets = true,
        },
    },
    mountPlacement = {
        clearanceMultiplier = 0.04,
        minimumClearanceFraction = 0.02,
    },
    proceduralWeapons = {
        {
            key = ProceduralKey.BurstLaser,
            args = {
                family = Weapon.Family.Laser,
                variant = Weapon.Variant.ShortBurst,
                baseWeaponId = Weapon.Type.Laser,
                burstCount = 3,
                -- Hold the beam visibly instead of a 0.06-0.11s flash.
                duration = 0.30,
            },
        },
        {
            key = ProceduralKey.BurstLaserGreen,
            args = {
                family = Weapon.Family.Laser,
                variant = Weapon.Variant.ShortBurst,
                baseWeaponId = Weapon.Type.LaserGreen,
                burstCount = 3,
                duration = 0.30,
            },
        },
        {
            key = ProceduralKey.BurstLaserBlue,
            args = {
                family = Weapon.Family.Laser,
                variant = Weapon.Variant.ShortBurst,
                baseWeaponId = Weapon.Type.LaserBlue,
                burstCount = 3,
                duration = 0.30,
            },
        },
        {
            key = ProceduralKey.BurstLaserViolet,
            args = {
                family = Weapon.Family.Laser,
                variant = Weapon.Variant.ShortBurst,
                baseWeaponId = Weapon.Type.LaserViolet,
                burstCount = 3,
                duration = 0.30,
            },
        },
        {
            key = ProceduralKey.FastPlasma,
            args = {
                family = Weapon.Family.Plasma,
                variant = Weapon.Variant.FastSmall,
                baseWeaponId = Weapon.Type.Plasma,
                mountSizeClass = MountSizeClass.L,
            },
        },
        {
            key = ProceduralKey.Missile,
            args = {
                family = Weapon.Family.Missile,
                variant = Weapon.Variant.Guided,
                baseWeaponId = Weapon.Type.Plasma,
            },
        },
        {
            key = ProceduralKey.RocketM,
            args = {
                family = Weapon.Family.Missile,
                variant = Weapon.Variant.Rocket,
                baseWeaponId = Weapon.Type.MissileRocket,
                mountSizeClass = MountSizeClass.M,
                launcherFamilyId = Weapon.LauncherFamily.Missile,
            },
        },
        {
            key = ProceduralKey.TorpedoL,
            args = {
                family = Weapon.Family.Missile,
                variant = Weapon.Variant.Torpedo,
                baseWeaponId = Weapon.Type.MissileTorpedo,
                mountSizeClass = MountSizeClass.L,
                launcherFamilyId = Weapon.LauncherFamily.Missile,
            },
        },
        {
            key = ProceduralKey.LongChargeXL,
            args = {
                family = Weapon.Family.Laser,
                variant = Weapon.Variant.LongCharge,
                baseWeaponId = Weapon.Type.XLLongChargeLaser,
                mountSizeClass = MountSizeClass.XL,
                fireMode = Weapon.FireMode.Volley,
                -- Violet profile: the XL pair carries the fourth laser color.
                laserProfileId = Weapon.LaserProfile.Violet,
                duration = 1.10,
            },
        },
        {
            key = ProceduralKey.FastBolt,
            args = {
                family = Weapon.Family.Laser,
                variant = Weapon.Variant.FastBolt,
                baseWeaponId = Weapon.Type.LaserBolt,
                burstCount = 3,
            },
        },
        {
            key = ProceduralKey.Flak,
            args = {
                family = Weapon.Family.Plasma,
                variant = Weapon.Variant.FastSmall,
                baseWeaponId = Weapon.Type.FlakCannon,
            },
        },
        {
            key = ProceduralKey.Railgun,
            args = {
                family = Weapon.Family.Plasma,
                variant = Weapon.Variant.FastSmall,
                baseWeaponId = Weapon.Type.Railgun,
                mountSizeClass = MountSizeClass.M,
            },
        },
    },
    tracking = {
        tier = 4,
    },
    trackingComponent = {},
    control = {
        active = true,
    },
    capacitors = {
        banks = {
            {
                id = "laser_small_medium",
                groupId = Weapon.CapacitorGroup.Laser,
                supportedSizeClasses = { MountSizeClass.SM, MountSizeClass.M },
                maxCharge = 30.0,
                chargeRate = 20.0,
            },
            {
                id = "laser_xl",
                groupId = Weapon.CapacitorGroup.Laser,
                supportedSizeClasses = { MountSizeClass.XL },
                maxCharge = 10.0,
                chargeRate = 2.5,
            },
            {
                id = "plasma_small_medium_large",
                groupId = Weapon.CapacitorGroup.Plasma,
                supportedSizeClasses = {
                    MountSizeClass.SM, MountSizeClass.M, MountSizeClass.L },
                maxCharge = 24.0,
                chargeRate = 6.0,
            },
        },
        policies = {
            [Weapon.FireMode.Volley] = { id = Weapon.CapacityPolicy.Burst },
            [Weapon.FireMode.Sequence] = { id = Weapon.CapacityPolicy.Sustain },
        },
    },
    targeting = {},
    loadoutPolicy = {
        mirrorPairs = true,
        allowAsymmetricOverrides = true,
        quotas = {
            [MountRole.PointDefense] = {
                minimumPairs = 4,
                generatedKey = ProceduralKey.FastBolt,
            },
        },
        rules = {
            -- Flak battery: dedicated anti-fighter screen (before generic PD).
            {
                when = {
                    mountRole = MountRole.PointDefense,
                    mountId = Weapon.MountId.FlakBatteryPort,
                },
                entry = { generatedKey = ProceduralKey.Flak },
            },
            {
                when = {
                    mountRole = MountRole.PointDefense,
                    mountId = Weapon.MountId.FlakBatteryStarboard,
                },
                entry = { generatedKey = ProceduralKey.Flak },
            },
            -- Railgun battery: Line-role medium-hull hunters.
            {
                when = {
                    mountRole = MountRole.Line,
                    mountId = Weapon.MountId.RailgunBatteryPort,
                },
                entry = { generatedKey = ProceduralKey.Railgun },
            },
            {
                when = {
                    mountRole = MountRole.Line,
                    mountId = Weapon.MountId.RailgunBatteryStarboard,
                },
                entry = { generatedKey = ProceduralKey.Railgun },
            },
            {
                when = { mountRole = MountRole.PointDefense },
                entry = { generatedKey = ProceduralKey.FastBolt },
            },
            {
                when = {
                    mountRole = MountRole.Missile,
                    mountSizeClass = MountSizeClass.M,
                },
                entry = { generatedKey = ProceduralKey.RocketM },
            },
            {
                when = {
                    mountRole = MountRole.Missile,
                    mountSizeClass = MountSizeClass.L,
                },
                entry = { generatedKey = ProceduralKey.TorpedoL },
            },
            {
                when = { mountRole = MountRole.CapitalHeavy },
                entry = { generatedKey = ProceduralKey.LongChargeXL },
            },
            {
                when = { mountRole = MountRole.Heavy },
                entry = { generatedKey = ProceduralKey.FastPlasma },
            },
            -- Line mounts: red burst lasers on the fore pair, one color
            -- variant per side on the outboard mid pair.
            {
                when = {
                    mountRole = MountRole.Line,
                    mountId = Weapon.MountId.ForeOuterPort,
                },
                entry = { generatedKey = ProceduralKey.BurstLaser },
            },
            {
                when = {
                    mountRole = MountRole.Line,
                    mountId = Weapon.MountId.ForeOuterStarboard,
                },
                entry = { generatedKey = ProceduralKey.BurstLaser },
            },
            {
                when = {
                    mountRole = MountRole.Line,
                    mountId = Weapon.MountId.MidOuterPort,
                },
                entry = { generatedKey = ProceduralKey.BurstLaserGreen },
            },
            {
                when = {
                    mountRole = MountRole.Line,
                    mountId = Weapon.MountId.MidOuterStarboard,
                },
                entry = { generatedKey = ProceduralKey.BurstLaserBlue },
            },
        },
    },
}

-- The capital's size: the largest hull scale, doubled (about 1 km).
local CAPITAL_SCALE_MULTIPLIER = 2.0

-- Where on a drone's hull the weapons aim, and how much the point wanders.
local TARGET_POINT = {
    motionAmplitude = 0.04,
    motionFrequency = 0.30,
    minFacingDot = 0.15,
}

-- The drones: hull sizes and defences for every weapon role, each on its own
-- orbit around the capital (radius in game units, speed in radians/s).
local DRONES = {
    {
        label = "Capital drone",
        shipType = ShipType.Capital,
        sizeClass = Enums.Target.SizeClass.Capital,
        maxHealth = 10000,
        maxShield = 800,
        shieldRegen = 40.0,
        scaleMultiplier = 1.0,
        orbitRadius = 3.2,
        orbitSpeed = 0.10,
        phase = 0.0,
    },
    {
        label = "Cruiser drone",
        shipType = ShipType.Capital,
        sizeClass = Enums.Target.SizeClass.Large,
        maxHealth = 2500,
        maxShield = 400,
        shieldRegen = 25.0,
        scaleMultiplier = 0.5,
        orbitRadius = 2.6,
        orbitSpeed = -0.16,
        phase = math.pi * 0.66,
    },
    {
        label = "Gunship drone",
        shipType = ShipType.Basic,
        hull = 3, -- Compact: legacy ShipBasic generator needs a hull index
        sizeClass = Enums.Target.SizeClass.Medium,
        maxHealth = 600,
        maxShield = 120,
        shieldRegen = 10.0,
        scaleMultiplier = 0.25,
        orbitRadius = 2.2,
        orbitSpeed = 0.24,
        phase = math.pi * 1.33,
    },
    {
        label = "Fighter drone",
        shipType = ShipType.Fighter,
        hull = 1, -- Solo: legacy ShipFighter generator needs a hull index
        sizeClass = Enums.Target.SizeClass.Small,
        maxHealth = 150,
        scaleMultiplier = 0.15,
        orbitRadius = 1.8,
        orbitSpeed = -0.34,
        phase = math.pi * 0.33,
    },
}
local DRONE_RESPAWN_DELAY = 4.0 -- seconds

-- Where the capital goes: this far from the player's ship (game units, 1 =
-- 10 km), with this much free space around it (the drone orbits and their
-- hulls); ahead of the ship when there is room there.
local ANCHOR_DISTANCES = { 5.0, 6.5, 8.0, 12.0 }
local ANCHOR_CLEARANCE = 4.5
local ANCHOR_DIRECTIONS = 96
local ROCK_RADIUS_FACTOR = 1.5 -- a belt rock's radius, in units of its scale
local PLANET_RADIUS_FACTOR = 1.2 -- with its atmosphere

-- The orbit camera's start around the capital (game units, radians): inside
-- the drone orbits, so the beams and shots fly out past the camera
local CAMERA_DISTANCE = 1.3
local CAMERA_YAW = 2.45 -- off the capital's bow quarter
local CAMERA_PITCH = 0.35

-- Fire mode names for the overlay
local FIRE_MODE_NAMES = {
    [Weapon.FireMode.Volley] = "volley",
    [Weapon.FireMode.Sequence] = "sequence",
}

local function rigidBodyOf(entity)
    local bodyComponent = entity and entity:get(PhysicsComponents.RigidBody)
    return bodyComponent and bodyComponent:getRigidBody()
end

--- Spheres to keep the capital and its drones away from: belt rocks,
--- planets and moons, stations, the player's ship. Only those that could be
--- near a candidate spot.
---@return table[] { x, y, z, r }
local function collectObstacles(game, center, reach)
    local obstacles = {}
    local function consider(x, y, z, r)
        local dx, dy, dz = x - center.x, y - center.y, z - center.z
        local limit = reach + r
        if dx * dx + dy * dy + dz * dz <= limit * limit then
            obstacles[#obstacles + 1] = { x = x, y = y, z = z, r = r }
        end
    end

    for _, belt in ipairs(game.beltEntities or {}) do
        local beltComponent = belt:get(CelestialComponents.AsteroidBelt)
        -- Rock positions are relative to the belt's render origin: its parent
        -- body when it has one (as in LimitTheoryRedux:defaultShipSpawn)
        local origin = belt
        local parentComponent = belt:get(CoreComponents.Parent)
        local parent = parentComponent and parentComponent:getParent()
        if parent and parent:get(PhysicsComponents.Transform) then
            origin = parent
        end
        local transform = origin:get(PhysicsComponents.Transform)
        local o = transform and transform:getPos() or Position(0, 0, 0)
        for _, rock in ipairs(beltComponent and beltComponent:getAsteroidData() or {}) do
            consider(o.x + rock.px, o.y + rock.py, o.z + rock.pz, rock.scale * ROCK_RADIUS_FACTOR)
        end
    end

    for _, item in ipairs(game.labeledEntities or {}) do
        local transform = item.entity:get(PhysicsComponents.Transform)
        if transform then
            local p = transform:getPos()
            consider(p.x, p.y, p.z, transform:getScale() * PLANET_RADIUS_FACTOR)
        end
    end

    for _, station in ipairs(game.stations or {}) do
        local bodyComponent = station:get(PhysicsComponents.RigidBody)
        local body = bodyComponent and bodyComponent:getRigidBody()
        if body then
            local p = body:getPos()
            consider(p.x, p.y, p.z, bodyComponent:getRadius())
        end
    end
    return obstacles
end

--- Free space around a point: the distance to the nearest obstacle's surface.
local function clearanceAt(x, y, z, obstacles)
    local clearance = math.huge
    for i = 1, #obstacles do
        local o = obstacles[i]
        local dx, dy, dz = x - o.x, y - o.y, z - o.z
        local d = math.sqrt(dx * dx + dy * dy + dz * dz) - o.r
        if d < clearance then clearance = d end
    end
    return clearance
end

--- Where the capital goes: a free spot a few kilometres from the player's
--- ship, ahead of it if possible (the ship starts next to the belt rocks).
---@return Position
local function findAnchor(game)
    local shipBodyComponent = game.playerShip:get(PhysicsComponents.RigidBody)
    local ship = shipBodyComponent:getPos()
    local forward = shipBodyComponent:getForward()
    local farthest = ANCHOR_DISTANCES[#ANCHOR_DISTANCES]
    local obstacles = collectObstacles(game, ship, farthest + ANCHOR_CLEARANCE)

    local best, bestClearance = nil, -math.huge
    for _, distance in ipairs(ANCHOR_DISTANCES) do
        local chosen, chosenScore = nil, -math.huge
        -- Directions evenly spread over a sphere (Fibonacci lattice)
        for i = 0, ANCHOR_DIRECTIONS - 1 do
            local y = 1 - (i + 0.5) * 2 / ANCHOR_DIRECTIONS
            local ring = math.sqrt(1 - y * y)
            local angle = i * math.pi * (3 - math.sqrt(5))
            local dx, dy, dz = math.cos(angle) * ring, y, math.sin(angle) * ring
            local px, py, pz = ship.x + dx * distance, ship.y + dy * distance, ship.z + dz * distance
            local clearance = clearanceAt(px, py, pz, obstacles)
            if clearance >= ANCHOR_CLEARANCE then
                -- Ahead of the ship, and level with it rather than far above or below
                local score = dx * forward.x + dy * forward.y + dz * forward.z - 0.5 * math.abs(dy)
                if score > chosenScore then
                    chosen, chosenScore = Position(px, py, pz), score
                end
            end
            if clearance > bestClearance then
                best, bestClearance = Position(px, py, pz), clearance
            end
        end
        if chosen then
            return chosen
        end
    end
    Log.Warn("Weapon Testbed: no free spot near the player's ship, using the clearest one (%.2f free)",
        bestClearance)
    return best
end

--- A drone on its orbit around the capital, `time` seconds after it started.
local function dronePosition(data, drone, time)
    local angle = drone.phase + drone.config.orbitSpeed * time
    local anchor = data.anchor
    return Position(
        anchor.x + math.cos(angle) * drone.config.orbitRadius,
        anchor.y,
        anchor.z + math.sin(angle) * drone.config.orbitRadius)
end

--- Build drone `slot` (DRONES[slot]) and put it on its orbit.
local function spawnDrone(game, slot)
    local data = game.scenarioData
    local config = DRONES[slot]
    -- Each start of a slot begins a bit further along its orbit
    local drone = {
        slot = slot,
        config = config,
        phase = config.phase + slot * 0.7 + (data.spawnCount[slot] or 0) * 0.9,
        time = 0,
    }
    data.spawnCount[slot] = (data.spawnCount[slot] or 0) + 1
    drone.handle = game.constructManager:createTarget({
        seed = data.droneSeeds[slot],
        shipType = config.shipType,
        hull = config.hull,
        position = dronePosition(data, drone, 0),
        scale = data.capitalScale * config.scaleMultiplier,
        isKinematic = true,
        collidable = true,
        maxHealth = config.maxHealth,
        maxShield = config.maxShield or 0,
        shieldRegen = config.shieldRegen or 0,
        sizeClass = config.sizeClass,
    })
    drone.entity = drone.handle.root
    drone.body = drone.handle.rigidBody
    drone.health = drone.entity:get(CoreComponents.Health)
    drone.defense = drone.entity:get(ConstructComponents.Defense)
    game:addSceneConstruct(drone.handle, config.label)
    table.insert(data.drones, drone)
end

local function removeDrone(game, index)
    local data = game.scenarioData
    local drone = table.remove(data.drones, index)
    game:removeSceneConstruct(drone.handle)
end

--- All drones back at full health on their starting orbits.
local function resetDrones(game)
    local data = game.scenarioData
    for index = #data.drones, 1, -1 do
        removeDrone(game, index)
    end
    data.respawns = {}
    data.spawnCount = {}
    for slot = 1, #DRONES do
        spawnDrone(game, slot)
    end
    if data.battery then
        data.battery:resetSequence()
    end
end

--- "Fighter drone 150/150 +0" style status of a drone slot.
local function droneStatus(data, slot)
    for _, drone in ipairs(data.drones) do
        if drone.slot == slot and drone.entity:isValid() then
            local shield = drone.defense and drone.defense:getCurrentShield() or 0
            return string.format("%s %.0f%s", drone.config.label:match("^(%S+)"),
                drone.health:getCurrentHealth(),
                shield >= 1 and string.format(" +%.0f", shield) or "")
        end
    end
    for _, respawn in ipairs(data.respawns) do
        if respawn.slot == slot then
            return string.format("%s back in %.0fs", DRONES[slot].label:match("^(%S+)"), math.max(0, respawn.time))
        end
    end
    return DRONES[slot].label:match("^(%S+)") .. " -"
end

--- The drone nearest a point (what an impact hit).
local function nearestDroneLabel(data, position)
    local best, bestDistance = nil, math.huge
    for _, drone in ipairs(data.drones) do
        if drone.entity:isValid() then
            local p = drone.body:getPos()
            local dx, dy, dz = p.x - position.x, p.y - position.y, p.z - position.z
            local d = dx * dx + dy * dy + dz * dz
            if d < bestDistance then
                best, bestDistance = drone.config.label, d
            end
        end
    end
    return best or "?"
end

--- The first few entries of a list, and how many more.
local function shortList(list, count)
    if #list == 0 then return "-" end
    local shown = {}
    for i = 1, math.min(count, #list) do shown[i] = list[i] end
    local text = table.concat(shown, ", ")
    if #list > count then
        text = text .. string.format(" (+%d)", #list - count)
    end
    return text
end

return {
    name = "Weapon Testbed",
    description = "An armed capital ship fighting target drones.",
    ruleset = Rulesets.SinglePlanetWithBelt,

    setup = function(game)
        local data = game.scenarioData
        data.anchor = findAnchor(game)
        data.capitalScale = Config.game.shipHulls.scale[6] * CAPITAL_SCALE_MULTIPLIER

        local construction = ArmedShipBuilder:build(CAPITAL, {
            constructManager = game.constructManager,
            shipConfig = {
                position = data.anchor,
                scale = data.capitalScale,
                isKinematic = true,
            },
        })
        construction.shipBody:setCollidable(true)
        game:addSceneConstruct(construction.shipHandle, "Armed capital")
        data.capital = construction

        data.battery = game.weaponCombat:addBattery(WeaponBattery(construction, {
            name = "Testbed capital",
            aiActive = true,
            targetPoint = TARGET_POINT,
        }))

        -- One seed per drone slot, from the capital's seed: the same drones
        -- every time
        data.droneSeeds = {}
        for slot = 1, #DRONES do
            data.droneSeeds[slot] = construction.streams.target:get64()
        end
        data.drones = {}
        data.respawns = {}
        data.spawnCount = {}
        data.destroyed = 0
        resetDrones(game)

        -- The camera starts close enough to see the turrets, looking down
        -- across the drone orbits; scrolling zooms out over the whole fight
        local radius = construction.shipRadius
        local distances = game:orbitDistances(radius)
        distances.distance = CAMERA_DISTANCE
        game:setOrbitFocus(construction.ship, distances)
        game.player:getActiveController():setAngles(CAMERA_YAW, CAMERA_PITCH)
        Log.Info("Weapon Testbed: capital (radius %.3f, %d turrets: %d beam, %d projectile) and %d drones",
            radius, #construction.turrets, construction.beamMountCount,
            construction.projectileMountCount, #data.drones)
    end,

    -- Drones along their orbits; destroyed ones come back after a while
    update = function(game, dt)
        local data = game.scenarioData
        if not data.drones then return end

        for index = #data.drones, 1, -1 do
            local drone = data.drones[index]
            if not drone.entity:isValid() or drone.health:isDestroyed() then
                Log.Info("Weapon Testbed: %s destroyed", drone.config.label)
                data.destroyed = data.destroyed + 1
                table.insert(data.respawns, { slot = drone.slot, time = DRONE_RESPAWN_DELAY })
                removeDrone(game, index)
            else
                drone.time = drone.time + dt
                drone.body:setPos(dronePosition(data, drone, drone.time))
            end
        end

        for index = #data.respawns, 1, -1 do
            local respawn = data.respawns[index]
            respawn.time = respawn.time - dt
            if respawn.time <= 0 then
                table.remove(data.respawns, index)
                spawnDrone(game, respawn.slot)
            end
        end
    end,

    onInput = function(game, dt)
        local battery = game.scenarioData.battery
        if not battery then return end
        for _, binding in pairs(WeaponActions) do
            binding:update(dt)
        end

        if WeaponActions.AI:isPressed() then
            battery:setAIActive(not battery:isAIActive())
        end
        if WeaponActions.CycleMode:isPressed() and not battery:isAIActive() then
            local mode = battery.control:getMode() == Weapon.FireMode.Volley
                and Weapon.FireMode.Sequence or Weapon.FireMode.Volley
            battery:setFireMode(mode)
        end
        battery:setTriggerHeld(WeaponActions.Fire:isDown())
        if WeaponActions.Reset:isPressed() then
            resetDrones(game)
        end
    end,

    -- Regenerate (B): the drones again, the capital stays
    regenerate = function(game)
        resetDrones(game)
    end,

    controls = {
        { keys = "F6", action = "Capital's AI on / off" },
        { keys = "F7", action = "Fire mode: volley / sequence (AI off)" },
        { keys = "F9 (held)", action = "Fire the capital's weapons (AI off)" },
        { keys = "R or B", action = "Reset the drones" },
    },

    overlayLines = function(game)
        local data = game.scenarioData
        local battery = data.battery
        if not battery then return end

        local control = battery.control
        local capacitors = battery:getCapacitorSummary()
        local impact = battery.lastImpact or battery.lastBeamImpact
        local drones = {}
        for slot = 1, #DRONES do
            drones[slot] = droneStatus(data, slot)
        end
        return {
            "WEAPON TESTBED",
            string.format("Capital AI: %s   Fire mode: %s%s",
                battery:isAIActive() and "on" or "off",
                FIRE_MODE_NAMES[control:getMode()] or tostring(control:getMode()),
                control:isTriggerHeld() and "   FIRING" or ""),
            "Drones: " .. table.concat(drones, "   "),
            string.format("Destroyed: %d   Projectiles: %d   Beams: %d",
                data.destroyed, #battery.projectiles, #battery.beams),
            string.format("Capacitors: %.0f/%.0f   laser %.0f/%.0f   plasma %.0f/%.0f",
                capacitors.charge, capacitors.maxCharge,
                capacitors.laser and capacitors.laser.charge or 0,
                capacitors.laser and capacitors.laser.maxCharge or 0,
                capacitors.plasma and capacitors.plasma.charge or 0,
                capacitors.plasma and capacitors.plasma.maxCharge or 0),
            "Last fire order: " .. shortList(battery.lastShotOrder or {}, 4),
            "Last impact: " .. (impact
                and string.format("%s on %s", impact.mountId or "?", nearestDroneLabel(data, impact.position))
                or "-"),
            "F6 AI on/off   F7 fire mode   F9 hold trigger (AI off)   R reset drones   C fly your ship",
        }
    end,

    teardown = function(game)
        local data = game.scenarioData
        -- The capital and the drones go with the scene; the battery with it
        if data.battery then
            game.weaponCombat:removeBattery(data.battery)
        end
        data.battery = nil
        data.drones = nil
    end,
}
