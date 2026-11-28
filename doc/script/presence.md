# Presence: data until seen

A star system with one or two thousand ships can't keep every one as a physics construct with
turrets, a flight controller and a navigation order. Most of them are far from the camera, where
nothing needs their hull, their collision shape or their turrets' aim. Presence keeps such entities
as data and builds their construct only where they are seen.

## The pieces

- `PresenceComponent` (`Modules/Presence`): the entity's materializer `kind`, whether it is
  `materialized`, and its `drive`: which body is the truth while it is materialized.
- `VirtualBodyComponent`: where it is and how it moves without a physics body (position, velocity,
  rotation, drag).
- `PresenceSystem`: every frame it moves the virtual bodies; every `checkS` it materializes the
  entities within `materializeRange` of the active camera and dematerializes those beyond
  `dematerializeRange`. The gap between the two keeps an entity at the edge from going in and out.
  A run without a camera (the headless economy sim) keeps everything virtual.
- A **materializer** per kind, registered with `PresenceSystem:register(kind, { materialize,
  dematerialize })`: it builds the construct where the virtual body is and returns its body, and
  takes it down again. The entity itself stays.

The two drives:

| Drive | While materialized | Used by |
|---|---|---|
| `virtual` | The virtual body moves on; the construct (kinematic) is placed on it | Wrecks |
| `physical` | The construct flies; its body is copied into the virtual one, so it dematerializes where it is | Ships (planned) |

## Wrecks

A ship whose hull is destroyed is taken out of the scene by the spawner a moment later (its death
explosion is taken from its body first): its jobs end, its weapons and construct go, its crew lose
their post (`CrewSystem`: `ShipLost`, then free for another). It leaves a wreck (`Modules/Salvage`):
a `WreckComponent` with the kind and hull design it was, the scrap its hull yields (`ShipKindDefs`
`scrap`), the cargo it carried and its integrity (`wreckIntegrity`). Its virtual body drifts on
from the ship's last motion, slowed by `WreckSystem.drag`.

Seen, a wreck is materialized as a hulk: the ship's hull without weapons or crew, kinematic, out of
the weapons' targeting. Shots that hit it wear the wreck's integrity, which stays with the wreck
when the hulk is taken down. A wreck is gone when it breaks up (no integrity left) or is salvaged
empty.

## Planned: salvage

A salvage job on the job board for ships with a salvage tool: fly to a wreck, take its cargo and
scrap into the hold, sell the scrap to a station that recycles it (a recipe turning scrap into
iron). Salvage works on the wreck's data, so it runs the same whether the wreck is a hulk or not.

## Planned: virtual ships

Ships get the same treatment, with a `physical` drive. A virtual ship has no hull, turrets or
flight controller; what it does is resolved over time instead of flown:

- **Flight.** A navigation order of a virtual ship moves its virtual body along the route at the
  speed its estimate gives (`NavEstimate`), without steering.
- **Docking and trade.** Arriving at a station docks at once; the economy's stages (buy, sell, mine)
  run on time as they do now, since they don't need the hull.
- **Combat.** Fights between virtual ships are resolved by rates (damage a second by armament),
  not by shots; a fight that comes into view materializes and is flown.
- **Materializing a ship** builds its hull, turrets and crew posts where the virtual body is, with
  its velocity, and gives its navigation order back to the flight systems.

This is the attention level that legacy had and never filled in (`economy-legacy.md`), and the way to
1–2k ships: only what is seen pays for physics, aim and rendering.
