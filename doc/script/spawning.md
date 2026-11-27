# Spawning

The `Spawning` module (`script/Modules/Spawning`) puts a star system's economy into a scene.

## Planned with the system

A ruleset's `economy` section (`Config/Gen/Rulesets/EconomyDefaults.lua` for a ruleset without one)
holds rules, evaluated like the bodies' with the system's seed: how many factions and of which
kinds, stations per faction, haulers, miners and fighters per faction, ore fields, and whether
marauders come. `UniverseManager` has `EconomyPlanner` plan it when it makes the star system, from a
seed of its own drawn last (the bodies' draws stay as they were), and keeps the plan on the system
as an `EconomyLayoutComponent`.

The plan always trades: with four stations or more it has a power plant, a refinery, a foundry and
a colony before any more; with fewer, a power plant and outposts that live on its energy. Stations
are dealt round the factions and placed round the planets, clear of the rings.

## Built into the scene

`SpawnerSystem` is the one place that spawns economy content. A scene attaches it (`attach(world,
constructs, { weaponCombat, onSpawned })`) and clears it after `Nav.detach` (`clear`).

- `populate(planets, density)` builds the plan: factions with a leader each, stations (through
  `ConstructManager:createStation`, sized by a radius in metres), ore fields (`OreField`), and ships,
  their numbers scaled by `density`. The game calls it for every scenario with stations, with the
  New Game **Economy** setting (`Config.gen.economyDensities`: Off, Light, Normal, Busy).
- Direct calls for scenarios, testbeds and debugging: `spawnFaction`, `spawnStation`,
  `spawnOreField`, `spawnAgent({ kind, position, owner })`, `spawnAgentsAround`.
- A ship is of a kind (`Shared/Definitions/ShipKindDefs.lua`): a hauler, a miner (a mining laser) or a
  fighter (turrets, built through `ArmedShipBuilder` and run by the scene's weapons when it has
  them). It becomes an agent after the next physics step; its AI chooses its jobs by its equipment.
- **Fleets.** Every `fleetIntervalS` each faction buys the kind of ship the most valuable unfilled work
  on the job board calls for (trade: a hauler, mining: a miner, security: a fighter), when that work
  is worth `fleetDemand`, it has fewer than `maxFleetPerKind`, and its leader can pay `shipCost`.
  Marauders buy fighters out of their loot while they have prey. Lost ships are replaced the same way.
