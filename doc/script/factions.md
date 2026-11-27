# Factions

The `Factions` module (`script/Modules/Factions`): organisations of players that own ships and
stations, their relations, the threat at their stations, and the fighting their ships do.

## Membership and relations

- A **faction** is an entity (`FactionEntity`): its name and kind (`Enums.FactionType`, named like
  "Vexa Corporation" or "Kingdom of Vexa"), its colour, its weapons team (`faction:<id>`), its
  relations and its treasury (a `PlayerBankAccount`).
- A **player** belongs to a faction through its `FactionMembershipComponent`. A ship or a station
  belongs to its owner's faction (its `OwnershipComponent`); the spawner makes each faction a leader
  player that owns what the faction spawns and pays for it.
- **Relations** are per pair of factions, the same both ways: a disposition from -1 to 1 and a
  baseline it drifts back to (`FactionSystem.driftPerSecond`). Two factions are hostile at or below
  `Config.game.dispoHostileThreshold`. Marauders' baseline with everyone is -1.
- `FactionManager` creates factions and members and answers `factionOf`, `relation`, `isHostile`,
  `teamOf`; `enlist` gives a ship or station its faction's weapons team.

## What changes relations

- **Hits.** The beam and projectile code report each hit (`Shared/Helpers/DamageEvents`), resolving a
  turret to its ship. A faction whose member is hit by another faction's weapons turns against it
  (`hitPenalty`, at most once per `hitCooldownS` per pair).
- **Raids** turn the victim's faction against the raider's (`CombatAgentSystem.raidPenalty`).

The weapons engage by the factions' hostility: `FactionSystem` gives `WeaponCombatSystem` the rule
`FactionManager:teamsHostile`, so allied and neutral factions' ships don't fire at each other, while a
team that is no faction's is an enemy of every other team as before.

## Threat and fighting jobs

- `ThreatSystem` keeps a threat per station: it rises when a ship of the station's faction is hit
  within `radius` of it or raided there, and halves every `halfLifeS`.
- The job board posts a **security job** at each station with a threat of `minThreat` or more, paid by
  the station by its threat (at most `securityShare`, a quarter, of what the station earns over the
  patrol, `AccountManager:earningRate`, split over its patrols: a station that earns little can
  afford little protection), for as many ships as the threat calls for. Marauders' armed ships see
  **raid jobs** while there is unarmed hostile traffic.
- No ship has a fixed role: an armed ship's AI weighs these jobs against trade and mining (see
  [economy](economy.md#agents)). While it is on one, `CombatAgentSystem` flies it:
  - **Patrol**: round its station; a ship of a hostile faction within `guardRange` is followed close,
    and the weapons do the rest.
  - **Raid**: the nearest unarmed agent of a hostile faction out of port, a laden one first, is
    followed; staying within `raidRange` for `raidS` takes `raidShare` of its cargo, fences part of
    its value into the raider's account, raises the threat at the victim's stations and sends the
    raider off for `pirateCooldownS`. A hostile patrol close by drives a raider off.
- The ships patrolling one station fly its route as a **wing** ([Formations](formations.md)): the
  ship whose captain is most senior (patrol experience counting double) leads and flies the route of
  16 points; later guards take the first slot behind a less senior one. An engagement is a Combat
  order, which takes a wingman out of its slot; it takes its slot again when the fight ends. A
  wingman whose leader leaves leads in its place.

So raids make threat, threat makes paid security work, and the factions buy fighters for it.

`Testbeds/Factions` shows it: a trading guild with haulers and fighters, marauders near its lane.
