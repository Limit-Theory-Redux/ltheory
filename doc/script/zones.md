# Zones and gravity wells

A zone is a generic region: `ZoneEntity` has a transform and a shape. A zone means something only through the components attached to it, and several features can use one zone at once.

## Gravity wells

`GravityWellSystem` (Physics module) turns the zones of stars, planets and moons into gravity wells.

- `collectZones()` goes through the star, planet and moon views and lists a well for each one that has a zone. A planet's well is at least 1.5 times the orbit radius of its outermost moon.
- A ship takes part when it has a `GravityWellComponent` and a rigid body. The component holds the ship's linear drag (1/s, the value its rigid body has), the well the ship is in (the smallest one that contains it), the zone type, the body's previous position and the velocity the ship is carried at. `update(dt, substeps)` processes every such ship before the physics step of `dt` seconds, and a ship in no well is in open space. `substeps` is `Physics:getSolverSubsteps()`.
- A ship in a well is carried with its body by a force, so that it drifts with an orbiting body and the physics step sees its velocity. The force is an environment force: it sits outside the thrust budget, and thrust and collisions come on top of it. The body's move over the step is the difference between the position `OrbitalSystem:predictPos` gives for this step ([Orbits](orbits.md)) and its position at the last update, since its rigid body reaches it only when the step runs. The ship's rigid body is not moved directly.
- The step runs `substeps` equal substeps. Each one adds an equal part of the force to the solver velocity and moves the position by the damped velocity of that substep, and the stored velocity is the final solver velocity damped over the whole step. For the carried part `V` of the velocity the force `F` is chosen so that the step moves the ship by exactly the body's move `dt·vNew`, and the carried velocity becomes `(V + g)/(1 + dt·k)` with `g = dt·F/m` (`k` is the linear drag, `m` the mass). The solver is linear, so the rest of the velocity moves the ship relative to the body as it would next to a body that stands still. The system works with the step as single precision, as `Physics:update` takes it, and applies the force in two parts so that its single-precision rounding does not build up as drift.
- Entering a well adds the body's velocity in one step, leaving removes it, and moving from one well to another bridges the difference: the velocity relative to the body stays continuous. A ship that has just entered takes the body's move over the coming step, measured from where its rigid body stands, not what the body moved before. A body move larger than 1000 game units in one update is a jump (a rebuilt or rebased scene) and keeps the carried velocity. After a change of the body's velocity the carried part settles within a few frames; the ship's own displacement is exact in every one.
- Contacts are not exact: the solver sees the carried part of the velocity, which differs from a kinematic body's own velocity by the drag taken in the step, so a ship that touches a body moving with its well does not reproduce a contact with a still body to the last digit.
- `ShipFlightSystem:getSpeed` is the speed relative to the carried velocity.
- `getMaxDriveSpeedAt(position)` is the travel-drive speed multiplier allowed at a position: `Config.game.travelDrive.maxSpeedByZone.openSpace` far from every body, falling with the square of the distance to the zone's minimum at the body's centre, starting at twice the well radius. Where influences overlap, the lowest value applies. The drive system, the autopilot and the HUD call it with a ship's position.

## Zone components for navigation

A zone that should affect routes carries optional components from the Navigation module. Navigation reads only the zones that carry them.

| Component | Contract |
|---|---|
| `SpeedLimit` | The highest speed allowed in the region, in game units per second (greater than zero). It is authored in m/s and converted with `UniverseScaleConfig:toGameUnits` before it is stored. |
| `PathCost` | The extra cost of each metre travelled in the region, dimensionless and at least zero, on top of the distance itself. Zero costs the same as open space. |
| `Core.Tag` | Tags such as an asteroid field or hostile space. |

The two navigation components reject a value outside their range when they are created or changed.
