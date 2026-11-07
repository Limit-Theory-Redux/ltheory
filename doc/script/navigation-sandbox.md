# Navigation sandbox

`Tests/NavigationSandbox` runs navigation scenarios with nobody at the window. It spawns each scenario's ships in an empty scene, steers them through a `NavWorld` (see [navigation maths](../engine/nav.md)), logs what happens and ends with an exit code, so a script or CI can run it and judge the result.

Code: `script/States/App/Tests/NavigationSandbox.lua` and `script/States/App/Tests/NavigationSandbox/`.

## Running it

- `cargo run -p ltr -- NavigationSandbox` runs every scenario in turn, in the order of `SCENARIO_NAMES`.
- `cargo run -p ltr -- NavigationSandbox/<Name>` runs one scenario, for example `NavigationSandbox/HeadOn`.

`Slice` is not in `SCENARIO_NAMES`: it has 100 ships and takes much longer, so it is run on its own, `NavigationSandbox/Slice`.

Each scenario is a module of its own in `NavigationSandbox/`, and that module is an app: it returns `NavigationSandbox.ScenarioApp(scenario)`, a subclass of the sandbox that runs only that scenario. Selecting a scenario needs nothing but the app name.

## Scenarios

A scenario is a table:

| Field | Meaning |
|---|---|
| `name` | Name in the log lines and the HUD |
| `ships` | One entry per ship: `start`, and either `target` (the ship stops there) or `route` (waypoints flown in a loop, the first one first), each `{ x, y, z }` in metres; `facing`, the direction the ship faces at the start (not along the y axis), toward its first target by default; `hull`, an `Enums.ShipHulls` value that sets the ship's size, Solo by default |
| `ships[i].behaviour` | Optional, instead of `target` and `route`: `{ kind = … }` with `Seek` (`through`: points passed once, then the ship arrives at `target`; each pass is judged against `passRadiusM`), `Hold` (`holdAtSpeed`: the fraction of top speed at which the ship is switched from flying to `target` to holding its position), `Pursue` (`obstacle`: the circling obstacle's index, `standOffM`), `Follow` (`ship`: the leader's index, `offsetM = { right, up, forward }` in the leader's axes) or `Orbit` (`obstacle`, `radiusM`, `normal`). Distances are in metres |
| `obstacles` | Optional spheres: `radius` in metres, and either `position` `{ x, y, z }` for a static one or `circle = { centre, radius, periodS }` for one that moves on a circle in the xz plane, starting on the +x side and moving toward +z |
| `timeLimit` | Seconds of simulated time for every ship to finish; 120 by default. Route ships never finish, so a scenario with them runs to its limit |
| `avoidance` | Optional `{ horizonS, clearanceRadii, aimFactor }` passed to `NavWorld:setAvoidanceTuning`; the engine defaults when absent |

Every ship is a fighter from `ShipGenerator` with the Solo flight profile, built from one fixed seed, so a run is repeatable up to the variable frame time. `hull` changes only the size (the mesh scale and so the bounding radius), not the flight profile. An obstacle is a kinematic unit-sphere body scaled to its radius, owned by an entity, so a physics query reports it by entity id. A circling obstacle gets a kinematic target before each physics step, the position it will have at the end of the step, so physics and the `NavWorld` see its velocity.

| Scenario | What it checks |
|---|---|
| `SingleShip` | One ship flies to a point behind it and to one side: it turns, flies and stops |
| `HeadOn` | Two ships fly to each other's start along the same line |
| `CrossingStreams` | Two streams of four ships, abreast and 800 m apart, cross at right angles at the origin at the same time |
| `CircleSwap` | Eight ships on a 5 km circle each fly to the opposite point, so all meet at the centre; hulls alternate Solo and Medium |
| `ObstacleField` | Three ships abreast, 600 m apart, fly 20 km along x through six static spheres: four on the courses, two beside them |
| `SeekThrough` | One ship is sent through three points with Seek and then stops at a target with Arrive: each pass must be at half of top speed or more |
| `HoldAfterCruise` | One ship is switched to Hold at half of top speed: it brakes, comes back to the point it was switched at and stops |
| `PursueRock` | A rock circles on a 4 km circle in 120 s; two ships pursue it to 200 m from its hull, one starting ahead of it and one behind |
| `FollowLeader` | A leader flies 15 km and stops; three followers start in slots, 1.5 km to its left, 1.5 km to its right and 1.5 km above and 1.5 km behind it, and hold them. A follower doesn't avoid its leader, so none may touch it |
| `OrbitRock` | A rock circles on a 3 km circle in 200 s; two ships go once round it, at 1.5 km and 2.5 km, one starting in the plane of its circle and one 4 km above |
| `Slice` | 100 ships loop through four waypoints each among 60 asteroids (100–800 m) in a 20 km × 2 km × 20 km slab, and a 3 km planet that circles the middle on a 6 km circle in 60 s. A seeded generator keeps every waypoint and start at least 1 km from the surface of every asteroid and from the planet's whole path, so a waypoint never lies inside a ship's avoidance reach (the scenario's aim factor of 1.4 widens the reach, so a nearer waypoint can't be approached); it runs for 60 s with its own avoidance tuning. It is the measurement for the navigation budget |

## Scene

The scene is a fresh physics world per scenario, the ships and obstacles, and an orbit camera that looks at the centre of all starts, targets and obstacles from above.

The physics step is at most 0.05 s. The first frame loads shaders and assets and can take seconds, and a step that long would carry ships at full speed through kilometres, past each other and any obstacle, so a long frame advances the simulation by the cap only. There is no star system. In place of a generated skybox, the sandbox pushes one small cube map of a uniform dim grey as both `envMap` and `irMap`, which the lighting shaders read; the irradiance of a uniform environment is the environment itself.

## Debug drawing

`Support/NavDebugDraw.lua` draws lines and spheres at world `Position`s. The render pipeline draws with the eye at the origin, so the helper subtracts the eye from each point in double precision (`Position:relativeTo`) before it reaches `Imm.Line3` and the sphere draw. Lines are `Imm.Line3`. A filled sphere is a unit `Mesh.BoxSphere` drawn into the open scene pass with the additive pipeline, the `wvp` vertex shader and the `flat_color_draw` fragment shader (the colour is `drawUser[0]` of the `DrawBlock`). A wireframe sphere is three `Imm.Line3` great circles. A render function calls `begin()`, the draws, then `finish()`.

A sphere is never drawn smaller than a fixed angular radius as seen from the eye: its radius is `max(radius, distance · MIN_ANGULAR_RADIUS)`. Ships are tens of metres across and the scenarios span kilometres, so at the framing distance a ship is far below a pixel; the angular minimum keeps it visible at any distance without depending on the field of view or the window size.

The sandbox draws, in the additive pass, for each ship:

| Marker | Meaning |
|---|---|
| Filled sphere at the ship | Its state: blue flying, green arrived, red deadlocked |
| Wireframe sphere at the target | The arrival range; for Pursue, Follow and Orbit, at the station point the judging uses |
| Line from the ship to its target | The straight course, while the ship is flying |
| Grey filled sphere at an obstacle | Its radius |

Filled spheres are drawn at a small fraction of their colour: the additive pass is exposed for dim content, and a full-strength fill saturates to white.

## Steering

The sandbox creates a `NavWorld` over the scenario's physics world. Each ship is an agent (its flight profile's drags, thrusts and torques; the mass and inertia come from its body), sensing is turned on with `setSensing` so each ship finds the obstacles (kinematic bodies in the physics world) with a ball query, and each ship's behaviour is set once every agent is registered: `setArrive` to its target with the arrival range as radius, or the behaviour its spec asks for. The maths is in [navigation maths](../engine/nav.md); the sandbox only runs the frame:

- **PreSim.** `NavWorld:step()`, then the glue: `copyAxes` and `copyArrived` into two preallocated buffers, a route ship that has arrived gets its next waypoint (`setArrive`), a Seek ship that has passed a point gets the next one and then `setArrive`, a Hold ship is switched with `setHold`, a finished ship's behaviour is cleared once (`clearBehaviour`; not for ships that track a moving point, which keep it), and each ship's six axes are written into its `ControlInputComponent`, like any other writer (see [flight control](flight.md)).
- **Sim.** `ShipFlightSystem` turns the control inputs into forces, the circling obstacles get their kinematic targets, and the physics world steps.

A ship that has finished has no behaviour, so it gets zero axes and drag brings it to rest. The `NavWorld` is dropped before a scenario's bodies are removed, because the registered agents' bodies must stay in physics while it exists.

## Metrics

`Support/ScenarioMetrics.lua` updates every ship after each physics step and logs each event as it happens, prefixed `NavigationSandbox: <scenario>:`. Its thresholds are named constants in metres and seconds, converted to game units with `UniverseScaleConfig:toGameUnits`.

| Metric | Definition |
|---|---|
| Arrival | Within 100 m of the target and slower than 50 m/s. A route ship never arrives |
| Waypoint | A route ship's goal is reached when the `NavWorld` reports it within 100 m (positional, no speed condition); the ship then gets the next one |
| Seek pass | A Seek ship passes a point when the `NavWorld` reports it within `passRadiusM`; the pass is too slow below half of its top speed |
| Station | A Pursue or Follow ship is on station when it is within 100 m of the point the sandbox computes from the definition (the stand-off point on the line of sight, or the slot in the leader's axes) and slower than 50 m/s relative to the tracked body; it has arrived when it has held that for 5 s. A Follow ship is judged only once the leader has stopped. The largest distance from the point after the ship first reached it is logged, not judged |
| Orbit | An Orbit ship has arrived when it has turned once about the centre, measured about the normal, while staying within 10 % of the radius (at least 100 m) of the circle; leaving the circle after the turn has begun restarts it. The tolerance is wider than the arrival range because the wanted velocity is the circle's tangent, and a ship at kilometres per second runs a little wide of it |
| Hold | After the switch the ship must leave the arrival range, then arrive back at the point it was switched at |
| Deadlock | Not 10 m closer to the target than its best distance for 10 s of simulated time; reaching a waypoint restarts the count. Not checked for a ship that tracks a moving point once it has been on station, nor for a Hold ship after the switch. The log line gives the distance, the speed, and the two nearest obstacles, each with its distance from the ship's centre to its surface and that distance as a multiple of the ship's reach (surface plus ship radius plus its largest clearance), then the gap between their two surfaces |
| Collision | A ship's bounding sphere starts to overlap another body. A `queryRadius` over the sphere finds the bodies, and each pair is counted once while it stays in contact. A body that is not a sandbox ship is an obstacle, so the count is split into ship–obstacle and ship–ship. The log names a body by its label (`ship 3`, `obstacle 2`) and gives the ship's speed; for an obstacle it also gives the speed the ship closes on it at (relative to the obstacle, which can move) |
| Frame time | Mean and largest frame time in ms, leaving out the first frame, which pays for spawning |
| Navigation time | Per frame, in ms with a monotonic clock, mean and largest, leaving out the first frame: the `NavWorld` step, the glue after it (copies, goals, control inputs) and the actuation, and their sum. Physics is not included |

A ship that arrives or deadlocks is finished. The scenario ends when every ship is finished or the time limit runs out, with two summary lines: arrivals, waypoints and Seek points reached, collisions by kind, deadlocks, unfinished ships, simulated time and frame times; and the navigation times.

Collisions are counted from bounding-sphere overlap instead of from the physics contacts, because the physics contact graph also holds pairs whose bounding boxes merely overlap.

## Pass and fail

A scenario fails on a ship–obstacle or a ship–ship collision, on a deadlock, on a Seek point passed too slowly, or when a ship that was to arrive (reach its station, finish its turn) is still flying at its time limit (route ships fly on by design). A run fails on a failed scenario or on a Lua error. The failure is a warning line, so the remaining scenarios still run, and the app quits with `Core.ExitCode.Failure` after the last one.
Collisions are found after each physics step, so a ship that crosses a small obstacle between two checks is missed: the count is a lower bound.

`Slice` (100 ships at cruise speed among rocks and a moving planet) is the hardest scenario and does not pass the rule yet: a few ship–planet contacts and deadlocks in narrow gaps remain (see Limits in [navigation maths](../engine/nav.md)). The failure is real, so it stays out of the CI smoke runs, and its debug run takes most of the app run limit.
