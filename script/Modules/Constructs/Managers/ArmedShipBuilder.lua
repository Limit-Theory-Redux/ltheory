---Builds an armed ship from a recipe: the hull (ConstructManager:createShip),
---the procedural content the recipe names (tracking module, generated
---weapons), then the armament (ShipArmamentManager: discover mounts, plan the
---loadout, install turrets, weapon control, capacitor, tracking, targeting).
---
---The result is a construction record that a WeaponBattery runs (see
---Modules/Constructs/Systems/WeaponBattery.lua). The ship itself is a normal
---ConstructManager handle (`construction.shipHandle`): whoever builds it owns
---it and destroys it (a scenario hands it to `game:addSceneConstruct`).
---
---Recipe fields (all optional unless noted):
---  seed               (required) hull, mount and content seed
---  ship               { shipType (required), config, stats } for createShip;
---                     config.generation picks the hull generator
---  mountDiscovery     { specifications, options } for HullMountDiscovery
---  mountPlacement     { clearanceMultiplier, minimumClearanceFraction }
---  proceduralWeapons  { { key, args }, ... } WeaponGenerator recipes; a
---                     loadout entry refers to one with `generatedKey = key`
---  tracking           TrackingModuleGenerator args (false: no module)
---  trackingComponent, control, capacitors, targeting
---                     component options, passed to installLoadout
---  loadout            explicit loadout per mount
---  loadoutPolicy      LoadoutGenerator policy (mirror pairs, quotas, rules)
---  defaultWeapon      weapon entry the battery falls back on (range, role)
---  requireLoadoutForAllMounts  default true
local ConstructManager = require("Modules.Constructs.Managers.ConstructManager")
local ShipArmamentManager = require("Modules.Constructs.Managers.ShipArmamentManager")
local TrackingModuleGenerator = require("Shared.Content.TrackingModuleGenerator")
local WeaponGenerator = require("Shared.Content.WeaponGenerator")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")

---@class ArmedShipConstruction
---@field seed integer
---@field streams table { master, content, target } RNG streams from the seed
---@field shipHandle table ConstructManager handle of the ship
---@field ship Entity
---@field shipBody RigidBody
---@field shipRadius number
---@field shipData ShipDataComponent
---@field discoveredMounts table[]
---@field discovery table
---@field mounts table[]
---@field mountIds string[]
---@field loadout table[]
---@field loadoutByMount table<string, table>
---@field turrets table[]
---@field turretsById table<string, table>
---@field generatedWeapons table<string, table>
---@field trackingModule table|nil
---@field control WeaponControlComponent
---@field capacitor WeaponCapacitorComponent|nil
---@field weaponTrackingComponent WeaponTrackingComponent|nil
---@field targeting TargetingComponent|nil
---@field defaultWeapon table
---@field defaultIdentity table|nil
---@field beamMountCount integer
---@field projectileMountCount integer

---@class ArmedShipBuilder
---@overload fun(): ArmedShipBuilder
local ArmedShipBuilder = Class("ArmedShipBuilder", function() end)

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
    local master = RNG.Create(seed)
    assert(master, "armed ship construction could not create its master RNG")
    local target = RNG.Create(master:get64())
    assert(target, "armed ship construction could not create its target RNG")
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
    for index, entry in ipairs(recipe.proceduralWeapons or {}) do
        assert(type(entry.key) == "string" and #entry.key > 0,
            "procedural weapon recipe " .. tostring(index) .. " requires a key")
        assert(not generatedWeapons[entry.key],
            "procedural weapon recipe duplicates key " .. entry.key)
        local generatorArgs = copyTable(entry.args)
        generatorArgs.universeSeed = generatorArgs.universeSeed or seed
        generatorArgs.contentSeed = generatorArgs.contentSeed or contentRng:get64()
        generatedWeapons[entry.key] = WeaponGenerator:generate(generatorArgs)
    end

    return {
        trackingModule = trackingModule,
        generatedWeapons = generatedWeapons,
    }
end

---The registered weapon a loadout entry names: a procedural weapon of the
---recipe (`generatedKey`) or a registry weapon (`weaponId` / `weaponRef`).
---@param entry table
---@param generatedWeapons table<string, table>|nil
---@return table weapon, table identity
function ArmedShipBuilder:resolveWeaponEntry(entry, generatedWeapons)
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

---Build the ship and its armament. Errors (bad recipe, a loadout that does not
---fit the hull) are raised after the half-built ship is destroyed again.
---@param recipe table see the file header
---@param options { constructManager: ConstructManager|nil, armamentManager: ShipArmamentManager|nil, shipConfig: table|nil }|nil
---shipConfig fields override recipe.ship.config (position, scale, rotation, isKinematic...)
---@return ArmedShipConstruction
function ArmedShipBuilder:build(recipe, options)
    assert(type(recipe) == "table", "armed ship construction requires a recipe")
    assert(recipe.seed ~= nil, "armed ship construction requires recipe.seed")
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
        shipHandle = constructManager:createShip({
            seed = seed,
            shipType = shipSpec.shipType,
            config = shipSpec.config,
            stats = shipSpec.stats,
        })

        local discoveryConfig = recipe.mountDiscovery or {}
        local discoveredMounts, discovery = armamentManager:discoverMounts(
            shipHandle,
            {
                seed = seed,
                specifications = discoveryConfig.specifications,
                options = copyTable(discoveryConfig.options or {}),
            })
        local loadoutPlan = armamentManager:planLoadout(discoveredMounts, {
            explicitLoadout = recipe.loadout,
            policy = recipe.loadoutPolicy,
            resolver = function(entry)
                return self:resolveWeaponEntry(entry, content.generatedWeapons)
            end,
            requireAll = recipe.requireLoadoutForAllMounts ~= false,
        })
        local defaultWeapon
        if recipe.defaultWeapon then
            defaultWeapon = self:resolveWeaponEntry(recipe.defaultWeapon, content.generatedWeapons)
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

return ArmedShipBuilder()
