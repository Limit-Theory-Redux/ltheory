# Zones and gravity wells

A zone is a generic region: `ZoneEntity` has a transform and a shape. A zone means something only through the components attached to it, and several features can use one zone at once.

## Gravity wells

`GravityWellSystem` (Physics module) turns the zones of stars, planets and moons into gravity wells.

- `collectZones(root)` walks the entity hierarchy from `root` and lists a well for each star, planet and moon that has a zone. A planet's well is at least 1.5 times the orbit radius of its outermost moon.
- A ship takes part when it has a `GravityWellComponent` and a rigid body. The component holds the well the ship is in (the smallest one that contains it), the zone type and the body's previous position. `update()` processes every such ship, and a ship in no well is in open space.
- A ship in a well moves by the distance its body moved since the last update, so that it drifts with an orbiting body. A ship that has just entered a well is not moved. A body move larger than 1000 game units in one update is clamped. This teleports the ship with `setPos`; it matters only while orbital motion is on.
- `getMaxDriveSpeedAt(position)` is the travel-drive speed multiplier allowed at a position: `Config.game.travelDrive.maxSpeedByZone.openSpace` far from every body, falling with the square of the distance to the zone's minimum at the body's centre, starting at twice the well radius. Where influences overlap, the lowest value applies. The drive system, the autopilot and the HUD call it with a ship's position.

## Zone components for navigation

A zone that should affect routes carries optional components from the Navigation module. Navigation reads only the zones that carry them.

| Component | Contract |
|---|---|
| `SpeedLimit` | The highest speed allowed in the region, in game units per second (greater than zero). It is authored in m/s and converted with `UniverseScaleConfig:toGameUnits` before it is stored. |
| `PathCost` | The extra cost of each metre travelled in the region, dimensionless and at least zero, on top of the distance itself. Zero costs the same as open space. |
| `Core.Tag` | Tags such as an asteroid field or hostile space. |

The two navigation components reject a value outside their range when they are created or changed.
