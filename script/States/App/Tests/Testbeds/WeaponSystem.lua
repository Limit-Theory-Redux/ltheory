Namespace.LoadInline("Legacy")
Namespace.LoadInline("Legacy.Systems")

local Application = require("States.Application")

local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local PhysicsComponents = require("Modules.Physics.Components")
local RenderingComponents = require("Modules.Rendering.Components")
local CoreComponents = require("Modules.Core.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ConstructEntities = require("Modules.Constructs.Entities")
local ConstructManager = require("Modules.Constructs.Managers.ConstructManager")
local ShipArmamentManager = require("Modules.Constructs.Managers.ShipArmamentManager")

-- Testbed configuration (formerly WeaponSystemScenario.lua). Testbed-only
-- recipe data: seeds, sockets, procedural weapon recipes, capacitor banks,
-- loadout policy, target/motion/presentation config. Shared definitions and
-- procedural generation remain owned by reusable generators/registries.
local Weapon = Enums.Weapon
local ShipType = Enums.ShipType
local MountSurfaceBand = Weapon.MountSurfaceBand
local MountZone = Weapon.MountZone
local MountRole = Weapon.MountRole
local MountSizeClass = Weapon.MountSizeClass
local ProceduralKey = Weapon.ProceduralKey

local TESTBED_CONFIG = {
    seed = 500,
    defaultWeapon = {
        generatedKey = ProceduralKey.BurstLaser,
    },
    ship = {
        shipType = ShipType.Capital,
        scaleMultiplier = 2.0,
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
        allowFallback = false,
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
                when = {
                    mountRole = MountRole.Missile,
                    mountSizeClass = MountSizeClass.M,
                },
                entry = { generatedKey = ProceduralKey.RocketM },
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
    -- Multiple targets with different hull sizes and orbit parameters so
    -- targeting/selection/interception can be tuned against mixed contacts.
    targets = {
        {
            label = "capital",
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
            label = "cruiser",
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
            label = "gunship",
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
            label = "fighter",
            shipType = ShipType.Fighter,
            hull = 1, -- Solo: legacy ShipFighter generator needs a hull index
            sizeClass = Enums.Target.SizeClass.Small,
            maxHealth = 150,
            scaleMultiplier = 0.15,
            orbitRadius = 1.8,
            orbitSpeed = -0.34,
            phase = math.pi * 0.33,
        },
    },
    targetRespawnDelay = 4.0,
    deferredLighting = true,
    pointLightDiagnostics = false,
    aiActive = true,
    targetMotion = {
        enabled = true,
        mode = "orbit",
        startMode = 1,
        center = { x = 0, y = 0, z = 0 },
        orbitModes = {
            {
                name = "horizontal-clockwise",
                plane = "xz",
                angularSpeed = 0.32,
                direction = 1,
                phase = 0,
            },
            {
                name = "horizontal-counterclockwise",
                plane = "xz",
                angularSpeed = 0.32,
                direction = -1,
                phase = 0,
            },
            {
                name = "vertical-clockwise",
                plane = "xy",
                angularSpeed = 0.32,
                direction = 1,
                phase = 0,
            },
            {
                name = "tilted-counterclockwise",
                plane = "tilted",
                tilt = math.rad(35),
                angularSpeed = 0.32,
                direction = -1,
                phase = 0,
            },
            {
                name = "fore-aft-vertical",
                plane = "yz",
                angularSpeed = 0.32,
                direction = 1,
                phase = 0,
            },
        },
    },
    targetPoint = {
        motionAmplitude = 0.04,
        motionFrequency = 0.30,
        minFacingDot = 0.15,
    },
}

local CameraEntity = require("Modules.Cameras.Entities").Camera
local SkyboxEntity = require("Modules.CelestialObjects.Entities.SkyboxEntity")
local CameraDataComponent = require("Modules.Cameras.Components.CameraDataComponent")
local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local OrbitCameraController = require("Modules.Cameras.Managers.CameraControllers.OrbitCameraController")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local LightManager = require("Modules.Rendering.Managers.LightManager")
local CameraSystem = require("Modules.Cameras.Systems.CameraSystem")
local Generator = require("Legacy.Systems.Gen.Generator")
local Starfield = require("Legacy.Systems.Gen.Starfield")
local Pulse = require("Legacy.GameObjects.Entities.Effects.Pulse")
local WeaponSystem = require("Modules.Constructs.Systems.WeaponSystem")
local AIWeaponSystem = require("Modules.Constructs.Systems.AIWeaponSystem")
local ProjectileSystem = require("Modules.Constructs.Systems.ProjectileSystem")
local BeamSystem = require("Modules.Constructs.Systems.BeamSystem")
local WeaponTrackingSystem = require("Modules.Constructs.Systems.WeaponTrackingSystem")
local TrackingModuleGenerator = require("Shared.Content.TrackingModuleGenerator")
local WeaponGenerator = require("Shared.Content.WeaponGenerator")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")
local WeaponActions = require("Input.ActionBindings.WeaponTestbedActions")
local DrawEx = require("UI.DrawEx")

---@class WeaponSystemTestbed: Application
local WeaponSystemTestbed = Subclass("WeaponSystemTestbed", Application)
WeaponSystemTestbed.TestbedConfig = TESTBED_CONFIG


local function removeRigidBody(world, entity)
    if not entity or not entity:isValid() then
        return
    end
    local rbComponent = entity:get(PhysicsComponents.RigidBody)
    if rbComponent and rbComponent:getRigidBody() then
        world:removeRigidBody(rbComponent:getRigidBody())
    end
end

local function copyTable(source)
    if rawtype(source) ~= "table" then
        return source
    end
    local result = {}
    for key, value in pairs(source) do
        result[key] = copyTable(value)
    end
    return result
end

local function createSeedStreams(seed)
    assert(seed ~= nil, "testbed construction requires a seed")
    local master = RNG.Create(seed)
    assert(master, "testbed construction could not create its master RNG")
    local target = RNG.Create(master:get64())
    assert(target, "testbed construction could not create its target RNG")
    return {
        master = master,
        content = master,
        target = target,
    }
end

local function generateContent(recipe, seed, streams)
    local contentRng = streams.content
    local trackingModule
    if recipe.tracking ~= false then
        local trackingArgs = copyTable(recipe.tracking or {})
        trackingArgs.universeSeed = trackingArgs.universeSeed or seed
        trackingArgs.contentSeed = trackingArgs.contentSeed or contentRng:get64()
        trackingModule = TrackingModuleGenerator:generate(trackingArgs)
    end

    local generatedWeapons = {}
    local generatedKeys = {}
    for index, entry in ipairs(recipe.proceduralWeapons or {}) do
        assert(type(entry.key) == "string" and #entry.key > 0,
            "procedural weapon recipe " .. tostring(index) .. " requires a key")
        assert(not generatedKeys[entry.key],
            "procedural weapon recipe duplicates key " .. entry.key)
        generatedKeys[entry.key] = true
        local generatorArgs = copyTable(entry.args)
        generatorArgs.universeSeed = generatorArgs.universeSeed or seed
        generatorArgs.contentSeed = generatorArgs.contentSeed or contentRng:get64()
        generatedWeapons[entry.key] = WeaponGenerator:generate(generatorArgs)
    end

    return {
        streams = streams,
        trackingModule = trackingModule,
        generatedWeapons = generatedWeapons,
    }
end

local function resolveWeaponEntry(entry, generatedWeapons)
    assert(type(entry) == "table", "weapon identity must be a table")
    local weaponId = entry.weaponId
    local weaponRef = entry.weaponRef
    if entry.generatedKey then
        local generated = generatedWeapons and generatedWeapons[entry.generatedKey]
        assert(generated,
            "loadout references an unknown procedural weapon key: " .. tostring(entry.generatedKey))
        weaponId = nil
        weaponRef = generated.weaponRef
    end
    local weapon = WeaponRegistry:resolveIdentity(weaponId, weaponRef)
    assert(weapon, "loadout has no registered weapon identity")
    return weapon, {
        weaponId = weaponId,
        weaponRef = weaponRef,
    }
end

local function buildProductionConstruction(recipe, options)
    assert(type(recipe) == "table", "testbed construction requires a recipe")
    assert(recipe.seed ~= nil, "testbed construction requires recipe.seed")
    options = options or {}
    local seed = recipe.seed
    local constructManager = options.constructManager or ConstructManager()
    local armamentManager = options.armamentManager or ShipArmamentManager()
    local streams = createSeedStreams(seed)
    local content = generateContent(recipe, seed, streams)
    local shipHandle

    local succeeded, result = xpcall(function()
        local shipSpec = copyTable(recipe.ship or {})
        shipSpec.config = shipSpec.config or {}
        for key, value in pairs(options.shipConfig or {}) do
            shipSpec.config[key] = value
        end
        local shipType = shipSpec.shipType
        assert(shipType == Enums.ShipType.Fighter
            or shipType == Enums.ShipType.Capital
            or shipType == Enums.ShipType.Basic,
            "testbed ship recipe requires a known Enums.ShipType value")
        shipHandle = constructManager:createShip({
            seed = seed,
            shipType = shipType,
            config = shipSpec.config,
            stats = shipSpec.stats,
        })

        local discoveryConfig = recipe.mountDiscovery or {}
        local discoveryOptions = copyTable(discoveryConfig.options or {})
        local discoveredMounts, discovery = armamentManager:discoverMounts(
            shipHandle,
            {
                seed = seed,
                specifications = discoveryConfig.specifications,
                options = discoveryOptions,
            })
        local loadoutPlan = armamentManager:planLoadout(discoveredMounts, {
            explicitLoadout = recipe.loadout,
            policy = recipe.loadoutPolicy,
            resolver = function(entry)
                return resolveWeaponEntry(entry, content.generatedWeapons)
            end,
            requireAll = recipe.requireLoadoutForAllMounts ~= false,
        })
        local defaultEntry = recipe.defaultWeapon
        local defaultWeapon
        if defaultEntry then
            defaultWeapon = resolveWeaponEntry(defaultEntry, content.generatedWeapons)
        end
        local armament = armamentManager:installLoadout(
            shipHandle,
            discoveredMounts,
            loadoutPlan,
            {
                trackingModule = content.trackingModule,
                mountPlacement = recipe.mountPlacement,
                control = recipe.control,
                capacitors = recipe.capacitors,
                trackingComponent = recipe.trackingComponent,
                targeting = recipe.targeting,
                defaultWeapon = defaultWeapon,
            })
        return {
            seed = seed,
            streams = streams,
            shipHandle = shipHandle,
            ship = shipHandle.root,
            shipBody = shipHandle.rigidBody,
            shipRadius = shipHandle.bodyComponent:getRadius(),
            shipData = shipHandle.shipData,
            discoveredMounts = discoveredMounts,
            discovery = discovery,
            mounts = armament.mounts,
            mountIds = armament.mountIds,
            loadout = armament.loadout,
            loadoutByMount = armament.loadoutByMount,
            turrets = armament.turrets,
            turretsById = armament.turretsById,
            generatedWeapons = content.generatedWeapons,
            trackingModule = content.trackingModule,
            control = armament.control,
            capacitor = armament.capacitor,
            weaponTrackingComponent = armament.weaponTrackingComponent,
            targeting = armament.targeting,
            defaultWeapon = armament.defaultWeapon,
            defaultIdentity = armament.defaultIdentity,
            beamMountCount = armament.beamMountCount,
            projectileMountCount = armament.projectileMountCount,
        }
    end, debug.traceback)
    if not succeeded then
        if shipHandle then
            constructManager:destroy(shipHandle)
        end
        error(result, 0)
    end
    return result
end

function WeaponSystemTestbed:getActiveTargetMotion()
    local configured = self.targetMotion or {}
    local modes = configured.orbitModes
    local selected = modes and modes[self.targetMotionModeIndex or 1] or nil
    local motion = {}
    for key, value in pairs(configured) do
        if key ~= "orbitModes" then
            motion[key] = value
        end
    end
    for key, value in pairs(selected or {}) do
        motion[key] = value
    end
    motion.phase = (motion.phase or 0) + (self.targetMotionPhaseOffset or 0)
    return motion
end

function WeaponSystemTestbed:getTargetMotionLabel()
    local motion = self:getActiveTargetMotion()
    if motion.mode == "orbit" then
        return string.format(
            "orbit %s plane=%s dir=%+d phase=%.2f",
            motion.name or "unnamed",
            motion.plane or "xz",
            motion.direction or 1,
            motion.phase or 0)
    end
    return string.format(
        "%s axis=%s",
        motion.mode or "linear",
        motion.axis or "y")
end

function WeaponSystemTestbed:resetTargetMotionPosition()
    if not self.targetBody then
        return
    end
    local sample = WeaponSystem:sampleTargetMotion(
        self.targetMotionBasePosition,
        self.targetMotionTime or 0,
        self:getActiveTargetMotion())
    self.targetBody:setPos(Position(
        sample.position.x,
        sample.position.y,
        sample.position.z))
    self.targetMotionVelocity = sample.velocity
    self.targetVelocity = sample.velocity
    if self.targetCandidates and self.targetCandidates[1] then
        self.targetCandidates[1].position = sample.position
        self.targetCandidates[1].velocity = sample.velocity
    end
end

function WeaponSystemTestbed:setTargetOrbitMode(index)
    local modes = self.targetMotion and self.targetMotion.orbitModes or {}
    if #modes == 0 then
        return
    end
    self.targetMotionModeIndex = ((index - 1) % #modes) + 1
    self.targetMotionPhaseOffset = 0
    self.targetMotionTime = 0
    self:resetTargetMotionPosition()
    Log.Info("WeaponSystem target orbit: " .. self:getTargetMotionLabel())
end

function WeaponSystemTestbed:advanceTargetOrbitPhase()
    if not self.targetMotion or not self.targetMotion.orbitModes then
        return
    end
    self.targetMotionPhaseOffset = (self.targetMotionPhaseOffset or 0) + math.pi / 2
    self.targetMotionTime = 0
    self:resetTargetMotionPosition()
    Log.Info("WeaponSystem target orbit phase: " .. self:getTargetMotionLabel())
end

function WeaponSystemTestbed:spawnTargets()
    if #self.targets > 0 then
        return
    end

    assert(self.targetSeedRng, "testbed target generation requires a deterministic seed stream")
    local configs = self.testbedConfig.targets or {}
    for index, config in ipairs(configs) do
        local targetSeed = self.targetSeedRng:get64()
        -- Distribute contacts around the ship; each follows its own orbit.
        local orbitRadius = config.orbitRadius or 6.0
        local phase = (config.phase or 0) + index * 0.7
        local basePosition = Position(
            math.cos(phase) * orbitRadius,
            0,
            math.sin(phase) * orbitRadius)
        local scale = self.capitalScale * (config.scaleMultiplier or 1.0)
        local handle = self.constructManager:createTarget({
            seed = targetSeed,
            shipType = config.shipType or Enums.ShipType.Capital,
            hull = config.hull,
            position = basePosition,
            scale = scale,
            isKinematic = true,
            collidable = true,
            maxHealth = config.maxHealth or 500,
            maxShield = config.maxShield or 0,
            shieldRegen = config.shieldRegen or 0,
            sizeClass = config.sizeClass or Enums.Target.SizeClass.Small,
        })
        local surface = WeaponSystem:buildTargetSurface(
            handle.shipData:getGeneratedMesh())
        table.insert(self.targets, {
            handle = handle,
            entity = handle.root,
            body = handle.rigidBody,
            health = handle.root:get(CoreComponents.Health),
            defense = handle.root:get(ConstructComponents.Defense),
            sizeClass = config.sizeClass or Enums.Target.SizeClass.Small,
            label = config.label or ("target" .. index),
            surface = surface,
            targetPointSeed = handle.targetPointSeed,
            orbitRadius = orbitRadius,
            orbitSpeed = config.orbitSpeed or 0.15,
            phase = phase,
            motionTime = 0.0,
        })
        Log.Info(string.format(
            "WeaponSystem contact '%s' spawned (%d triangles)",
            config.label or ("target" .. index),
            #surface))
    end
    self:rebuildTargetCandidates()
end

-- Legacy single-target entry point kept for HUD/beam code paths.
function WeaponSystemTestbed:spawnTarget()
    self:spawnTargets()
end

function WeaponSystemTestbed:destroyTargets()
    self:clearBeams()
    for _, target in ipairs(self.targets) do
        self.constructManager:destroy(target.handle)
    end
    self.targets = {}
    self:rebuildTargetCandidates()
    self.weaponTargetEntity = nil
    self.weaponTargetBody = nil
    if self.targeting then
        self.targeting:setTarget(nil)
    end
end

-- Legacy single-target entry point kept for HUD/beam code paths.
function WeaponSystemTestbed:destroyTarget()
    if not self.target then
        return
    end

    self:clearBeams()
    local target = self.target
    if self.targetHandle then
        self.constructManager:destroy(self.targetHandle)
        self.targetHandle = nil
    elseif target:isValid() then
        Registry:destroyEntity(target, Registry.DESTROY_MODE.KEEP_CHILDREN)
    end

    self.target = nil
    self.targetBody = nil
    self.weaponTargetEntity = nil
    self.weaponTargetBody = nil
    self.targetHealth = nil
    self.targetRadius = nil
    self.targetSurface = nil
    self.targetPointSeed = nil
    self.targetPointLogTime = nil
    self.targetMotionBasePosition = nil
    self.targetMotionVelocity = nil
    self.targetVelocity = nil
    self.targetCandidates = {}
    self.targetRespawnRemaining = self.targetRespawnDelay
    if self.targeting then
        self.targeting:setTarget(nil)
    end
end

function WeaponSystemTestbed:onTargetDestroyed(destroyedEntity)
    -- A contact died; remove it and schedule its respawn. BeamSystem passes
    -- the destroyed entity; ProjectileSystem calls without arguments.
    local removedLabel
    for index, target in ipairs(self.targets) do
        local matches = destroyedEntity ~= nil
            and target.entity == destroyedEntity
        if not matches then
            matches = not target.entity:isValid()
                or target.health:isDestroyed()
        end
        if matches then
            removedLabel = target.label
            Log.Info("WeaponSystem contact destroyed: " .. target.label)
            -- Drop the dead contact's motion track so long sessions don't
            -- accumulate stale track tables (unbounded heap growth).
            if self.contactTrackKeys and self.contactTracks then
                local key = self.contactTrackKeys[target.body]
                if key then
                    self.contactTracks[key] = nil
                    self.contactTrackKeys[target.body] = nil
                end
            end
            self.constructManager:destroy(target.handle)
            table.remove(self.targets, index)
            self.respawnQueue[#self.respawnQueue + 1] =
                (self.testbedConfig.targetRespawnDelay or 4.0)
            break
        end
    end
    self:rebuildTargetCandidates()
end

function WeaponSystemTestbed:rebuildTargetCandidates()
    self.targetCandidates = {}
    for _, target in ipairs(self.targets) do
        if target.entity:isValid() and not target.health:isDestroyed() then
            local maxHealth = target.health:getMaxHealth()
            local currentHealth = target.health:getCurrentHealth()
            self.targetCandidates[#self.targetCandidates + 1] = {
                id = target.entity.id,
                entity = target.entity,
                body = target.body,
                position = target.body:getPos(),
                velocity = Vec3f(),
                enabled = true,
                sizeClass = target.sizeClass,
                label = target.label,
                -- Feeds X4-style kill-ease scoring: weakened contacts are
                -- attractive finishes.
                healthFraction = maxHealth > 0
                    and (currentHealth / maxHealth)
                    or 1.0,
            }
        end
    end
end

function WeaponSystemTestbed:updateTargetLifecycle(dt)
    -- Advance each contact along its own orbit and refresh candidates.
    for index, target in ipairs(self.targets) do
        if target.entity:isValid() and not target.health:isDestroyed() then
            target.motionTime = target.motionTime + dt
            local angle = target.phase + target.orbitSpeed * target.motionTime
            local position = Position(
                math.cos(angle) * target.orbitRadius,
                0,
                math.sin(angle) * target.orbitRadius)
            target.body:setPos(position)
            local velocity = Vec3f(
                -math.sin(angle) * target.orbitSpeed * target.orbitRadius,
                0,
                math.cos(angle) * target.orbitSpeed * target.orbitRadius)
            target.velocity = velocity
        else
            self:onTargetDestroyed(index)
            break
        end
    end

    self:rebuildTargetCandidates()

    -- Keep candidate positions/velocities/health in sync with the sim.
    for _, candidate in ipairs(self.targetCandidates) do
        for _, target in ipairs(self.targets) do
            if candidate.id == target.entity.id then
                candidate.position = target.body:getPos()
                candidate.velocity = target.velocity or Vec3f()
                local maxHealth = target.health:getMaxHealth()
                local currentHealth = target.health:getCurrentHealth()
                candidate.healthFraction = maxHealth > 0
                    and (currentHealth / maxHealth)
                    or 1.0
            end
        end
    end

    -- Respawn destroyed contacts after their delay.
    for index = #self.respawnQueue, 1, -1 do
        self.respawnQueue[index] = self.respawnQueue[index] - dt
        if self.respawnQueue[index] <= 0 then
            table.remove(self.respawnQueue, index)
            self:spawnDestroyedContact()
        end
    end
end

function WeaponSystemTestbed:spawnDestroyedContact()
    assert(self.targetSeedRng, "testbed target generation requires a deterministic seed stream")
    local configs = self.testbedConfig.targets or {}
    if #configs == 0 then
        return
    end
    -- Rotate through the configured contacts so variety is preserved.
    self.contactCursor = ((self.contactCursor or 0) % #configs) + 1
    local config = configs[self.contactCursor]
    local targetSeed = self.targetSeedRng:get64()
    local orbitRadius = config.orbitRadius or 6.0
    local phase = (config.phase or 0) + (self.contactCursor * 0.7)
        + (self.targetGeneration or 0) * 0.9
    local basePosition = Position(
        math.cos(phase) * orbitRadius,
        0,
        math.sin(phase) * orbitRadius)
    local scale = self.capitalScale * (config.scaleMultiplier or 1.0)
    local handle = self.constructManager:createTarget({
        seed = targetSeed,
        shipType = config.shipType or Enums.ShipType.Capital,
        hull = config.hull,
        position = basePosition,
        scale = scale,
        isKinematic = true,
        collidable = true,
        maxHealth = config.maxHealth or 500,
        maxShield = config.maxShield or 0,
        shieldRegen = config.shieldRegen or 0,
        sizeClass = config.sizeClass or Enums.Target.SizeClass.Small,
    })
    self.targetGeneration = (self.targetGeneration or 0) + 1
    -- Respawned contacts carry the same targeting metadata as initial ones;
    -- omitting surface/seed silently degrades per-mount aiming after the
    -- first destruction wave.
    local surface = WeaponSystem:buildTargetSurface(
        handle.shipData:getGeneratedMesh())
    table.insert(self.targets, {
        handle = handle,
        entity = handle.root,
        body = handle.rigidBody,
        health = handle.root:get(CoreComponents.Health),
        sizeClass = config.sizeClass or Enums.Target.SizeClass.Small,
        label = config.label or ("contact" .. self.contactCursor),
        surface = surface,
        targetPointSeed = self.targetSeedRng and self.targetSeedRng:getUniform() or nil,
        orbitRadius = orbitRadius,
        orbitSpeed = config.orbitSpeed or 0.15,
        phase = phase,
        motionTime = 0.0,
    })
    Log.Info("WeaponSystem contact respawned: " .. (config.label or "?"))
end

function WeaponSystemTestbed:onInit()
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    self.testbedConfig = TESTBED_CONFIG
    self.seed = self.testbedConfig.seed or 1
    self.targetSeedRng = nil
    self.world = Physics.Create()
    self.constructManager = ConstructManager(Registry, self.world)
    self.armamentManager = ShipArmamentManager()
    self.projectiles = {}
    self.beams = {}
    self.turrets = {}
    self.turretsById = {}
    self.mountIds = {}
    self.mountCount = 0
    self.lastShotOrder = {}
    self.lastImpact = nil
    self.deferredLightingEnabled = self.testbedConfig.deferredLighting ~= false
    self.previousDeferredLightingEnabled = RenderCoreSystem.settings.deferredLighting
    RenderCoreSystem:setDeferredLightingEnabled(self.deferredLightingEnabled)
    --LightManager:setDiagnosticsEnabled(true) -- TEMP: verify light positions
    self.lastEffectLightCount = -1
    Log.Info(string.format(
        "WeaponSystem deferred lighting: %s",
        self.deferredLightingEnabled and "enabled" or "disabled"))
    local targetConfig = self.testbedConfig.target or {}
    self.targetRespawnDelay = self.testbedConfig.targetRespawnDelay
        or targetConfig.respawnDelay or 3.0
    self.targetMaxHealthConfig = targetConfig.maxHealth or 300
    self.targetSizeClass = targetConfig.sizeClass or Enums.Target.SizeClass.Small
    self.targetMotion = self.testbedConfig.targetMotion or { enabled = false }
    self.targetMotionModeIndex = self.targetMotion.startMode or 1
    self.targetMotionPhaseOffset = 0
    self.targetPointOptions = self.testbedConfig.targetPoint or {
        motionAmplitude = 0.08,
        motionFrequency = 0.60,
    }
    self.targetGeneration = 0
    self.targetRespawnRemaining = 0
    self.targets = {}
    self.respawnQueue = {}
    self.contactCursor = 0

    self.skybox = SkyboxEntity(self.seed, function(entity, blendMode)
        local placeholder = entity:get(CoreComponents.Empty)
        if not placeholder then
            placeholder = entity:add(CoreComponents.Empty)
        end

        if not placeholder.envMap then
            require("Legacy.Systems.Gen.Nebula.Nebula1")
            local nebulaRNG = RNG.Create(entity:get(CoreComponents.Seed):getSeed() + 0xC0104FULL)
            local starAngle = nebulaRNG:getDir2()
            placeholder.starDir = Vec3f(starAngle.x, 0, starAngle.y)
            placeholder.envMap = Generator.Get("Nebula", nebulaRNG)(
                nebulaRNG,
                Config.gen.nebulaRes,
                placeholder.starDir)
            placeholder.irMap = placeholder.envMap:genIRMap(256)
            placeholder.stars = Starfield(nebulaRNG, Config.gen.nStars(nebulaRNG))
            CameraManager:setStarDir(placeholder.starDir)
            ShaderVar.PushTexCube("envMap", placeholder.envMap)
            ShaderVar.PushTexCube("irMap", placeholder.irMap)
        end

        if blendMode == BlendMode.Disabled then
            RenderState.PushDepthWritable(false)
            local shader = Cache.Shader("farplane", "skybox")
            RenderState.PushCullFace(CullFace.None)
            shader:start()
            Draw.Box3(Box3f(-1, -1, -1, 1, 1, 1))
            shader:stop()
            RenderState.PopCullFace()
            RenderState.PopDepthWritable()
        elseif blendMode == BlendMode.Additive then
            local shader = Cache.Shader("farplane", "starbg")
            shader:start()
            shader:setFloat("brightnessScale", 3)
            shader:setTexCube("irMap", placeholder.irMap)
            shader:setTexCube("envMap", placeholder.envMap)
            placeholder.stars:draw()
            shader:stop()
        end
    end)

    self.pulseRenderEntity = Entity.Create(
        "WeaponPulseEffects",
        RenderingComponents.Render(function(_, blendMode)
            if #self.projectiles > 0 then
                Pulse.Render(self.projectiles, {
                    mode = blendMode,
                    eye = CameraManager:getEye(),
                })
            end
        end))

    -- CameraSystem owns controller input/pre-render updates through its event subscriptions.
    local _ = CameraSystem
    local camera = CameraEntity()
    CameraManager:registerCamera("WeaponOrbit", camera)
    self.cameraController = OrbitCameraController(camera, {
        distance = 0.35,
        minDistance = 0.02,
        maxDistance = 1.2,
        initialYaw = 0.0,
        initialPitch = 0.22,
        zoomSpeed = 0.35,
    })
    camera:get(CameraDataComponent):setController(self.cameraController)
    CameraManager:setActiveCamera("WeaponOrbit")

    local shipConfig = self.testbedConfig.ship or {}
    local capitalScale = Config.game.shipHulls.scale[6]
        * (shipConfig.scaleMultiplier or 1.0)
    self.capitalScale = capitalScale

    local construction = buildProductionConstruction(self.testbedConfig, {
        constructManager = self.constructManager,
        armamentManager = self.armamentManager,
        shipConfig = {
            position = Position(0, 0, 0),
            scale = capitalScale,
            isKinematic = shipConfig.config
                and shipConfig.config.isKinematic ~= false,
        },
    })
    if construction.discovery.fallbackUsed then
        Log.Warn(string.format(
            "WeaponSystem mount discovery fallback for seed %d: %s",
            self.seed,
            construction.discovery.strictError))
    end

    self.capitalHandle = construction.shipHandle
    self.capital = construction.ship
    self.capitalBody = construction.shipBody
    self.capitalBody:setCollidable(true)
    self.capitalBodyInWorld = true

    self.weapon = construction.defaultWeapon
    assert(self.weapon, "testbed construction produced no default weapon")
    self.generatedWeapons = construction.generatedWeapons
    self.trackingModule = construction.trackingModule
    self.targetSeedRng = construction.streams.target
    self.control = construction.control
    self.aiWeaponCombatRole = construction.defaultWeapon
        and construction.defaultWeapon.combatRole
    self.control:setActive(self.testbedConfig.aiActive == true)
    self.capacitor = construction.capacitor
    self.weaponCapacitor = self.capacitor
    self.weaponTrackingComponent = construction.weaponTrackingComponent
    self.targeting = construction.targeting
    self.targeting:setAutoAcquireEnabled(self.control:isActive())
    self.mounts = construction.mounts
    self.mountIds = construction.mountIds
    self.mountCount = #self.mountIds
    self.turrets = construction.turrets
    self.turretsById = construction.turretsById
    -- Per-mount weapon combat role for role-group target distribution.
    self.weaponRoleByMount = {}
    for _, loadout in ipairs(construction.loadout) do
        if loadout.mountId and loadout.weapon then
            self.weaponRoleByMount[loadout.mountId] =
                loadout.weapon.combatRole
        end
    end

    local laserCapacitorBank = self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Laser)[1]
    local plasmaCapacitorBank = self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Plasma)[1]
    assert(laserCapacitorBank and plasmaCapacitorBank,
        "testbed requires separate laser and plasma capacitor banks")
    local capacitorTotal = 0
    local capacitorMax = 0
    for _, bank in ipairs(self.capacitor:getBanks()) do
        capacitorTotal = capacitorTotal + bank.charge
        capacitorMax = capacitorMax + bank.maxCharge
    end
    Log.Info(string.format(
        "WeaponSystem testbed capacitor: %.2f/%.2f charge, laser %.2f/%.2f, plasma %.2f/%.2f, %d banks",
        capacitorTotal,
        capacitorMax,
        laserCapacitorBank.charge,
        laserCapacitorBank.maxCharge,
        plasmaCapacitorBank.charge,
        plasmaCapacitorBank.maxCharge,
        #self.capacitor:getBanks()))

    local capitalRadius = construction.shipRadius
    capitalRadius = math.max(capitalRadius, 0.02)
    -- Starboard is +X in the testbed convention. Keep Z exactly zero so the
    -- target is a side-on LOS case rather than another diagonal presentation.
    -- Distance floor scales with hull size; cap at 80% of the default
    -- weapon's range so mounts can still reach.
    local targetDistance = math.max(
        capitalRadius * 2.5,
        math.min(capitalRadius * 12.0, self.weapon.range * 0.80))
    self.targetOrbitAnchorPosition = Position(targetDistance, 0, 0)
    self.targetPosition = Position(
        self.targetOrbitAnchorPosition.x,
        self.targetOrbitAnchorPosition.y,
        self.targetOrbitAnchorPosition.z)
    local initialMotion = self:getActiveTargetMotion()
    if initialMotion.mode == "orbit" then
        local initialSample = WeaponSystem:sampleTargetMotion(
            self.targetOrbitAnchorPosition,
            0,
            initialMotion)
        self.targetPosition = Position(
            initialSample.position.x,
            initialSample.position.y,
            initialSample.position.z)
    end
    self:spawnTargets()
    Log.Info("WeaponSystem testbed target motion: " .. self:getTargetMotionLabel())
    self.capitalRadius = capitalRadius
    self.targetDistance = targetDistance
    self.losObstacles = {
        {
            body = self.capitalBody,
            kind = "hull",
            radius = capitalRadius,
            position = self.capitalBody:getPos(),
        },
    }

    assert(construction.beamMountCount > 0 and construction.projectileMountCount > 0,
        "testbed loadout must contain both beam and projectile registry effects")
    Log.Info(string.format(
        "WeaponSystem testbed loadout: %d beam mounts, %d projectile mounts, modern ECS effects",
        construction.beamMountCount,
        construction.projectileMountCount))
    local laserProfileNames = {}
    local laserProfileIds = {}
    for _, loadout in ipairs(construction.loadout) do
        local profile = WeaponRegistry:getLaserProfile(loadout.weapon)
        if profile then
            laserProfileIds[profile.id] = true
        end
    end
    for _, profileId in ipairs(LaserProfileRegistry:getIds()) do
        if laserProfileIds[profileId] then
            table.insert(laserProfileNames, LaserProfileRegistry:get(profileId).name)
        end
    end
    Log.Info("WeaponSystem testbed laser profiles: " .. table.concat(laserProfileNames, ", "))
    local variantKeys = {}
    for _, loadout in ipairs(construction.loadout) do
        if loadout.generatedKey then
            variantKeys[loadout.generatedKey] = true
        end
    end
    for _, requiredKey in ipairs({
        Enums.Weapon.ProceduralKey.BurstLaser,
        Enums.Weapon.ProceduralKey.BurstLaserGreen,
        Enums.Weapon.ProceduralKey.BurstLaserBlue,
        Enums.Weapon.ProceduralKey.FastBolt,
        Enums.Weapon.ProceduralKey.RocketM,
        Enums.Weapon.ProceduralKey.TorpedoL,
        Enums.Weapon.ProceduralKey.LongChargeXL,
        Enums.Weapon.ProceduralKey.Flak,
        Enums.Weapon.ProceduralKey.Railgun,
    }) do
        assert(variantKeys[requiredKey],
            "live WeaponSystem loadout is missing generated variant: " .. tostring(requiredKey))
    end
    local installedVariantKeys = {}
    for key in pairs(variantKeys) do
        table.insert(installedVariantKeys, key)
    end
    table.sort(installedVariantKeys)
    Log.Info("WeaponSystem testbed variants: " .. table.concat(installedVariantKeys, ", "))

    self.cameraController:setTarget(self.capital)
    Log.Info(string.format(
        "WeaponSystem testbed: capital radius %.4f, target distance %.4f, %d turrets ready",
        capitalRadius,
        targetDistance,
        #self.turrets))
end

function WeaponSystemTestbed:removeBeam(index)
    local beam = table.remove(self.beams, index)
    if beam and beam.entity:isValid() then
        Registry:destroyEntity(beam.entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
end

function WeaponSystemTestbed:clearBeams()
    for index = #self.beams, 1, -1 do
        self:removeBeam(index)
    end
end

---@param state table|nil
---@return string[]
function WeaponSystemTestbed:collectNonePointDiagnostics(state)
    state = state or self
    local noneMounts = {}
    for mountId, reason in pairs(state.sightReasonByMount or {}) do
        if reason == "none" then
            local pointState = state.targetPointByMount and state.targetPointByMount[mountId]
            local pointPosition = pointState and pointState.position
            table.insert(noneMounts, string.format(
                "%s@(%.3f,%.3f,%.3f)",
                mountId,
                pointPosition and pointPosition.x or 0,
                pointPosition and pointPosition.y or 0,
                pointPosition and pointPosition.z or 0))
        end
    end
    table.sort(noneMounts)
    return noneMounts
end

---@param mount table
---@param solution table
---@param weapon table
---@param shotSerial integer
function WeaponSystemTestbed:spawnBeam(mount, solution, weapon, shotSerial)
    assert(weapon.effect.kind == Enums.Weapon.Effect.Beam,
        "projectile weapons cannot spawn beams")
    local presentation = WeaponRegistry:getPresentation(weapon)
    local beamTargetPoint = solution.targetPoint or solution.position
    local entity = ConstructEntities.Beam(shotSerial, {}, {
        source = mount.entity,
        target = (self.mountTargetByMount
                and self.mountTargetEntities
                and self.mountTargetByMount[mount.mountId]
                and self.mountTargetEntities[self.mountTargetByMount[mount.mountId]]
                and self.mountTargetEntities[self.mountTargetByMount[mount.mountId]].entity)
            or self.weaponTargetEntity or self.target,
        effect = weapon.effect,
        visual = presentation,
        damagePerSecond = WeaponRegistry:getDamagePerSecond(weapon),
        duration = weapon.effect.duration or weapon.cooldown,
        targetPoint = beamTargetPoint,
        targetPointLocal = solution.targetPointLocal,
        aimAngles = solution.aimAngles,
        swayPhase = solution.swayPhase,
        swayTime = solution.swayTime,
        swayBasis = solution.swayBasis,
    })
    table.insert(self.beams, {
        entity = entity,
        component = entity:get(ConstructComponents.Beam),
        sourceBody = mount.body,
        targetBody = self.targetBody,
        targetPoint = beamTargetPoint,
        targetPointLocal = solution.targetPointLocal,
        mountId = mount.mountId,
        shotSerial = shotSerial,
        lightColor = presentation.lightColor,
        lightRadius = presentation.lightRadius,
        lightIntensity = presentation.lightIntensity,
    })
end

---@param mount table
---@param solution table
---@param weapon table
---@param shotSerial integer
function WeaponSystemTestbed:spawnProjectile(mount, solution, weapon, shotSerial)
    local sourcePosition = mount.body:getPos()
    local sourceVelocity = mount.body:getVelocity()
    local dx = solution.position.x - sourcePosition.x
    local dy = solution.position.y - sourcePosition.y
    local dz = solution.position.z - sourcePosition.z
    local length = math.sqrt(dx * dx + dy * dy + dz * dz)
    if length < 0.000001 then
        return
    end

    local direction = Vec3f(dx / length, dy / length, dz / length)
    local projectile = weapon.effect
    assert(projectile.kind == Enums.Weapon.Effect.Projectile, "beam weapons cannot spawn projectiles")
    local visual = projectile.visual
    local scale = projectile.speed / length
    local velocity = Vec3f(
        dx * scale + sourceVelocity.x,
        dy * scale + sourceVelocity.y,
        dz * scale + sourceVelocity.z)
    local pulse = Pulse:new()
    pulse.pos = sourcePosition
    pulse.vel = velocity
    pulse.dir = velocity:normalize()
    pulse.lifeMax = projectile.lifetime
    pulse.life = pulse.lifeMax
    pulse.dist = 0

    local entity = ConstructEntities.Projectile(shotSerial, {}, {
        source = mount.entity,
        effect = projectile,
        position = sourcePosition,
        velocity = velocity,
        damage = weapon.damage,
        lifetime = projectile.lifetime,
        scale = projectile.scale,
        guidance = projectile.guidance,
        targetBody = (self.mountTargetByMount
                and self.mountTargetEntities
                and self.mountTargetByMount[mount.mountId]
                and self.mountTargetEntities[self.mountTargetByMount[mount.mountId]]
                and self.mountTargetEntities[self.mountTargetByMount[mount.mountId]].body)
            or self.weaponTargetBody or self.targetBody,
        targetEntity = (self.mountTargetByMount
                and self.mountTargetEntities
                and self.mountTargetByMount[mount.mountId]
                and self.mountTargetEntities[self.mountTargetByMount[mount.mountId]]
                and self.mountTargetEntities[self.mountTargetByMount[mount.mountId]].entity)
            or self.weaponTargetEntity or self.target,
        bodyMesh = nil,
    })
    local body = entity:get(PhysicsComponents.RigidBody):getRigidBody()
    table.insert(self.projectiles, {
        entity = entity,
        body = body,
        component = entity:get(ConstructComponents.Projectile),
        mountId = mount.mountId,
        shotSerial = shotSerial,
        effect = pulse,
        pulseDistance = 0,
        lightColor = visual.lightColor,
        lightRadius = visual.lightRadius,
        lightIntensity = visual.lightIntensity,
        pColorR = visual.bodyColor.r,
        pColorG = visual.bodyColor.g,
        pColorB = visual.bodyColor.b,
        pulseHeadSize = visual.headSize,
        pulseTailWidth = visual.tailWidth,
        pulseTailLength = visual.tailLength,
        shaderKey = projectile.shaderKey or (projectile.archetype == "missile" and "missile"),
        inWorld = false,
    })
end

function WeaponSystemTestbed:removeProjectile(index)
    local projectile = table.remove(self.projectiles, index)
    if not projectile then
        return
    end
    if projectile.inWorld then
        removeRigidBody(self.world, projectile.entity)
    end
    Registry:destroyEntity(projectile.entity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
end

---@param data EventData
function WeaponSystemTestbed:onPreSim(data)
    local dt = data:deltaTime()
    WeaponActions.Fire:update(dt)
    WeaponActions.Volley:update(dt)
    WeaponActions.Sequence:update(dt)
    WeaponActions.AI:update(dt)
    WeaponActions.Reset:update(dt)
    WeaponActions.Orbit:update(dt)
    WeaponActions.OrbitPhase:update(dt)

    if WeaponActions.Orbit:isPressed() then
        self:setTargetOrbitMode((self.targetMotionModeIndex or 1) + 1)
    elseif WeaponActions.OrbitPhase:isPressed() then
        self:advanceTargetOrbitPhase()
    end

    -- Engage EVERY contact simultaneously: assign mounts round-robin
    -- across live candidates. The AI focus target remains for the HUD;
    -- actual per-mount engagement uses mountTargetByMount.
    if self.control:isActive() and #self.targetCandidates > 0 then
        self.mountTargetByMount = {}
        self.mountTargetEntities = {}
        -- body -> candidate record, consumed by AIWeaponSystem when
        -- resolving per-mount assignments.
        self.contactRecords = {}
        self.contactTrackKeys = {}
        for index, candidate in ipairs(self.targetCandidates) do
            self.contactTrackKeys[candidate.body] = tostring(candidate.id)
            self.mountTargetEntities[candidate.body] = candidate
            self.contactRecords[candidate.body] = candidate
        end
        local mountIds = {}
        for _, turret in ipairs(self.turrets) do
            table.insert(mountIds, turret.mountId)
        end
        table.sort(mountIds)
        -- Every mount runs chooseBestTarget over all live contacts; a
        -- claimed-count penalty spreads fire so contacts aren't stacked.
        -- self.focusTargetEntity (settable by external systems) overrides.
        local assigned = {}
        local claimedCount = {}
        for _, candidate in ipairs(self.targetCandidates) do
            claimedCount[candidate.body] = 0
        end
        for _, mountId in ipairs(mountIds) do
            local mount = self.turretsById[mountId]
            if mount and mount.body then
                local role = self.weaponRoleByMount[mountId]
                local chosen = WeaponSystem:chooseBestTarget(
                    mount,
                    self.targetCandidates,
                    mount.body:getPos(),
                    self.weapon.range * 0.9,
                    role,
                    self.focusTargetEntity)
                if not chosen then
                    -- Nothing in range: track nearest so the turret stays useful.
                    chosen = WeaponSystem:selectNearestTarget(
                        mount.body:getPos(),
                        self.targetCandidates,
                        math.huge)
                end
                if chosen then
                    assigned[mountId] = chosen
                    claimedCount[chosen.body] = (claimedCount[chosen.body] or 0) + 1
                end
            end
        end
        -- Publish per-mount engagement: this is what tracking/spawn read.
        self.mountTargetByMount = {}
        for mountId, contact in pairs(assigned) do
            self.mountTargetByMount[mountId] = contact.body
        end

        -- Resolve per-mount surfaces + point seeds from the contact records.
        self.mountSurfaces = {}
        self.mountTargetPointSeeds = {}
        for mountId, contactBody in pairs(self.mountTargetByMount) do
            local candidate = self.mountTargetEntities[contactBody]
            if candidate then
                for _, target in ipairs(self.targets) do
                    if target.entity.id == candidate.id then
                        self.mountSurfaces[mountId] = target.surface
                        self.mountTargetPointSeeds[mountId] =
                            target.targetPointSeed
                        break
                    end
                end
            end
        end
    else
        self.mountTargetByMount = nil
        self.contactTrackKeys = nil
    end

    if WeaponActions.AI:isPressed() then
        local active = not self.control:isActive()
        self.control:setActive(active)
        self.targeting:setAutoAcquireEnabled(active)
        if active then
            self.control:setSequenceIndex(1)
        else
            -- AI off: drop the weapon target so turrets stop slewing and
            -- no stale solution can fire (manual LMB fire still works).
            self.weaponTargetEntity = nil
            self.weaponTargetBody = nil
            self.control:setTriggerHeld(false)
        end
    end

    if not self.control:isActive() and WeaponActions.Volley:isPressed() then
        self.control:setMode(Enums.Weapon.FireMode.Volley)
        self.control:setSequenceIndex(1)
    elseif not self.control:isActive() and WeaponActions.Sequence:isPressed() then
        self.control:setMode(Enums.Weapon.FireMode.Sequence)
        self.control:setSequenceIndex(1)
    end

    if WeaponActions.Reset:isPressed() then
        if self.target then
            self.targetHealth:setCurrentHealth(self.targetMaxHealth)
        else
            self.targetRespawnRemaining = 0
        end
        self.control:setSequenceIndex(1)
        self.lastImpact = nil
        self.targetMotionTime = 0
        self.targetMotionPhaseOffset = 0
        self:resetTargetMotionPosition()
    end

    if not self.control:isActive() and WeaponActions.Fire:isReleased() then
        self.control:setSequenceIndex(1)
    end
    if not self.control:isActive() then
        self.control:setTriggerHeld(WeaponActions.Fire:isDown())
    else
        self.control:setTriggerHeld(false)
    end
end

function WeaponSystemTestbed:syncTurretTransforms()
    local capitalRigidBodyComponent = self.capital:get(PhysicsComponents.RigidBody)
    local parentRotation = self.capitalBody:getRot()
    for _, turret in ipairs(self.turrets) do
        turret.body:setPos(capitalRigidBodyComponent:toWorldScaled(
            turret.bodyLocalPosition or turret.localPosition))
        turret.body:setRot(parentRotation * turret.localRotation)
    end
end

---@param data EventData
function WeaponSystemTestbed:onSim(data)
    local dt = data:deltaTime()
    self.simTime = (self.simTime or 0) + dt
    self:updateTargetLifecycle(dt)
    self:syncTurretTransforms()
    self.world:update(dt)
    AIWeaponSystem:update(self, dt)
    WeaponTrackingSystem:update(self, dt)
    WeaponSystem:update(self, dt)
    -- Diagnostics follow the AI's current weapon target (any contact).
    if not self.lastTargetPoint and self.weaponTargetBody then
        self.lastTargetPoint = { position = self.weaponTargetBody:getPos(), triangleIndex = -1 }
    end
    -- Health diagnostics at 1 Hz, not per-tick: per-frame formatting and
    -- logging can dominate the testbed's frame budget.
    self.healthLogTime = (self.healthLogTime or 0) + dt
    if self.healthLogTime >= 1.0 then
        self.healthLogTime = 0
        local healthSummary = {}
        for _, target in ipairs(self.targets) do
            if target.entity:isValid() and not target.health:isDestroyed() then
                healthSummary[#healthSummary + 1] = string.format(
                    "%s=%.0f/%.0f",
                    target.label,
                    target.health:getCurrentHealth(),
                    target.health:getMaxHealth())
            end
        end
        if #healthSummary > 0 then
            Log.Info("WeaponSystem contact health: " .. table.concat(healthSummary, ", "))
        end
    end
    if self.lastTargetPoint then
        local pointTime = self.targetPointTime or 0
        if not self.targetPointLogTime or pointTime - self.targetPointLogTime >= 0.5 then
            local point = self.lastTargetPoint.position
            Log.Info(string.format(
                "WeaponSystem target point: surface triangle %d at (%.3f, %.3f, %.3f)",
                self.lastTargetPoint.triangleIndex,
                point.x,
                point.y,
                point.z))
            local sightMounts = {}
            for _, mount in ipairs(self.turrets or {}) do
                if self.sightByMount and self.sightByMount[mount.mountId] then
                    table.insert(sightMounts, mount.mountId)
                end
            end
            local reasons = self.sightReasons or {}
            local noneMounts = self:collectNonePointDiagnostics()
            Log.Info(string.format(
                "WeaponSystem sight: %d/%d ready %d/%d target %d hull %d none %d other %d mounts [%s] nonePoints [%s]",
                self.sightCount or 0,
                self.mountCount or 0,
                self.readyCount or 0,
                self.mountCount or 0,
                reasons.target or 0,
                reasons.hull or 0,
                reasons.none or 0,
                reasons.other or 0,
                table.concat(sightMounts, ","),
                table.concat(noneMounts, ",")))
            self.targetPointLogTime = pointTime
        end
    end
end

---@param data EventData
function WeaponSystemTestbed:onPostSim(data)
    ProjectileSystem:update(self, data:deltaTime())
    BeamSystem:update(self, data:deltaTime())
    for index = #(self.lightEffects or {}), 1, -1 do
        local effect = self.lightEffects[index]
        if not effect or not effect:isValid() then
            table.remove(self.lightEffects, index)
        end
    end
end

---@param data EventData
function WeaponSystemTestbed:onRender(data)
    RenderCoreSystem:render(data)
    self.effectLights = LightManager:getPointLights()
    if #self.effectLights ~= self.lastEffectLightCount then
        Log.Info(string.format(
            "WeaponSystem deferred lighting: %d weapon point lights",
            #self.effectLights))
        self.lastEffectLightCount = #self.effectLights
    end

    self:immediateUI(function()
        local mode = self.control:getMode()
        local health = self.targetHealth and self.targetHealth:getCurrentHealth() or 0
        local targetState = self.target
            and string.format("%.0f/%.0f", health, self.targetMaxHealth)
            or string.format("respawn %.1fs", self.targetRespawnRemaining or 0)
        local lastShot = #self.lastShotOrder > 0 and table.concat(self.lastShotOrder, ", ") or "-"
        local mountCount = self.mountCount or #self.turrets
        local impactState = self.lastImpact or self.lastBeamImpact
        local targetMotionPosition = self.targetBody and self.targetBody:getPos()
        local targetMotionVelocity = self.targetMotionVelocity or Vec3f()
        local targetMotionLabel = self:getTargetMotionLabel()
        local targetPointState = self.lastTargetPoint
        local targetPointPosition = targetPointState and targetPointState.position
        local capacitorCharge = 0
        local capacitorMax = 0
        local laserCapacitorBank = self.capacitor
            and self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Laser)[1]
        local plasmaCapacitorBank = self.capacitor
            and self.capacitor:getBanks(Enums.Weapon.CapacitorGroup.Plasma)[1]
        for _, bank in ipairs(self.capacitor and self.capacitor:getBanks() or {}) do
            capacitorCharge = capacitorCharge + bank.charge
            capacitorMax = capacitorMax + bank.maxCharge
        end
        local impact = impactState and string.format(
            "hit %s at (%.3f, %.3f, %.3f)",
            impactState.mountId or "?",
            impactState.position.x,
            impactState.position.y,
            impactState.position.z) or "-"
        local sightMounts = {}
        for _, turret in ipairs(self.turrets) do
            if self.sightByMount and self.sightByMount[turret.mountId] then
                table.insert(sightMounts, turret.mountId)
            end
        end
        local lines = {
            "WeaponSystem Testbed",
            "LMB: fire   1: volley   2: sequence   A: toggle AI   O: next orbit   P: phase   R: reset target",
            string.format(
                "Mode: %s   AI: %s   Target HP: %s   Projectiles: %d   Beams: %d",
                mode,
                self.control:isActive() and "active" or "off",
                targetState,
                #self.projectiles,
                #self.beams),
            string.format(
                "Deferred lighting: %s   Weapon point lights: %d",
                self.deferredLightingEnabled and "enabled" or "disabled",
                self.lastEffectLightCount >= 0 and self.lastEffectLightCount or 0),
            string.format(
                "Target motion: %s pos (%.3f, %.3f, %.3f) vel (%.3f, %.3f, %.3f)",
                targetMotionLabel,
                targetMotionPosition and targetMotionPosition.x or 0,
                targetMotionPosition and targetMotionPosition.y or 0,
                targetMotionPosition and targetMotionPosition.z or 0,
                targetMotionVelocity.x,
                targetMotionVelocity.y,
                targetMotionVelocity.z),
            string.format(
                "Target point: %s (%.3f, %.3f, %.3f) triangle %s",
                targetPointPosition and "surface" or "center",
                targetPointPosition and targetPointPosition.x or (self.targetPosition and self.targetPosition.x or 0),
                targetPointPosition and targetPointPosition.y or (self.targetPosition and self.targetPosition.y or 0),
                targetPointPosition and targetPointPosition.z or (self.targetPosition and self.targetPosition.z or 0),
                targetPointState and tostring(targetPointState.triangleIndex) or "-"),
            string.format(
                "Capacitor: %.2f/%.2f   Laser %.2f/%.2f   Plasma %.2f/%.2f",
                capacitorCharge,
                capacitorMax,
                laserCapacitorBank and laserCapacitorBank.charge or 0,
                laserCapacitorBank and laserCapacitorBank.maxCharge or 0,
                plasmaCapacitorBank and plasmaCapacitorBank.charge or 0,
                plasmaCapacitorBank and plasmaCapacitorBank.maxCharge or 0),
            string.format(
                "Capacity: %s   Inter-shot gap: %.3f",
                mode == Enums.Weapon.FireMode.Volley and "burst" or "sustain",
                self.control.interShotGap or 0),
            string.format(
                "Capital radius: %.2f   Orbit radius: %.2f   Sight: %d/%d   Ready: %d/%d",
                self.capitalRadius or 0,
                self.targetDistance or 0,
                self.sightCount or 0,
                mountCount,
                self.readyCount or 0,
                mountCount),
            string.format(
                "LOS ray results: target %d  hull %d  none %d  other %d  source %d",
                self.sightReasons and self.sightReasons.target or 0,
                self.sightReasons and self.sightReasons.hull or 0,
                self.sightReasons and self.sightReasons.none or 0,
                self.sightReasons and self.sightReasons.other or 0,
                self.sightReasons and self.sightReasons.source or 0),
            string.format(
                "Ray hit position: %s",
                self.sightHitPosition
                and string.format(
                    "%.2f, %.2f, %.2f",
                    self.sightHitPosition.x,
                    self.sightHitPosition.y,
                    self.sightHitPosition.z)
                or "-"),
            "Sight mounts: " .. (#sightMounts > 0 and table.concat(sightMounts, ", ") or "-"),
            "Last fire order: " .. lastShot,
            "Last impact: " .. impact,
        }
        local y = 32
        for _, line in ipairs(lines) do
            DrawEx.TextAdditive("Unageo-Medium", line, 11, 32, y, 40, 20, 0.9, 0.9, 0.9, 0.95, 0.0, 0.5)
            y = y + 22
        end
    end)
end

function WeaponSystemTestbed:onExit()
    LightManager:clearPointLights()
    LightManager:setDiagnosticsEnabled(false)
    for _, effect in ipairs(self.lightEffects or {}) do
        if effect and effect:isValid() then
            Registry:destroyEntity(effect, Registry.DESTROY_MODE.DESTROY_CHILDREN)
        end
    end
    self.lightEffects = {}
    self:clearBeams()
    for index = #self.projectiles, 1, -1 do
        self:removeProjectile(index)
    end
    if self.targetHandle then
        self.constructManager:destroy(self.targetHandle)
        self.targetHandle = nil
    end
    if self.capitalHandle then
        self.constructManager:destroy(self.capitalHandle)
        self.capitalHandle = nil
    end
    if self.pulseRenderEntity and self.pulseRenderEntity:isValid() then
        Registry:destroyEntity(self.pulseRenderEntity, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    if self.skybox and self.skybox:isValid() then
        Registry:destroyEntity(self.skybox, Registry.DESTROY_MODE.DESTROY_CHILDREN)
    end
    if self.previousDeferredLightingEnabled ~= nil then
        RenderCoreSystem:setDeferredLightingEnabled(self.previousDeferredLightingEnabled)
        self.previousDeferredLightingEnabled = nil
    end
    CameraManager:unregisterCamera("WeaponOrbit")
end

return WeaponSystemTestbed
