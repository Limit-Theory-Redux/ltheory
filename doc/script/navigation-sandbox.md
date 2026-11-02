# Navigation sandbox

`Tests/NavigationSandbox` runs navigation scenarios with nobody at the window. It spawns each scenario's ships in an empty scene, steers them to their targets, logs what happens and ends with an exit code, so a script or CI can run it and judge the result.

Code: `script/States/App/Tests/NavigationSandbox.lua` and `script/States/App/Tests/NavigationSandbox/`.

## Running it

- `cargo run -p ltr -- NavigationSandbox` runs every scenario in turn, in the order of `SCENARIO_NAMES`.
- `cargo run -p ltr -- NavigationSandbox/<Name>` runs one scenario, for example `NavigationSandbox/HeadOn`.

Each scenario is a module of its own in `NavigationSandbox/`, and that module is an app: it returns `NavigationSandbox.ScenarioApp(scenario)`, a subclass of the sandbox that runs only that scenario. Selecting a scenario needs nothing but the app name.

## Scenarios

A scenario is a table:

| Field | Meaning |
|---|---|
| `name` | Name in the log lines and the HUD |
| `ships` | One entry per ship: `start` and `target`, `{ x, y, z }` in metres; `facing`, the direction the ship faces at the start (not along the y axis), toward its target by default |
| `timeLimit` | Seconds of simulated time for every ship to finish; 120 by default |

Every ship is a Solo fighter from `ShipGenerator` with the Solo flight profile, built from one fixed seed, so a run is repeatable up to the variable frame time.

| Scenario | What it checks |
|---|---|
| `SingleShip` | One ship flies to a point behind it and to one side: it turns, flies and stops |
| `HeadOn` | Two ships fly to each other's start along the same line |

## Scene

The scene is a fresh physics world per scenario, the ships, and an orbit camera that looks at the centre of all starts and targets from above. There is no star system. In place of a generated skybox, the sandbox pushes one small cube map of a uniform dim grey as both `envMap` and `irMap`, which the lighting shaders read; the irradiance of a uniform environment is the environment itself.

## Debug drawing

`Support/NavDebugDraw.lua` draws lines and spheres at world `Position`s. The render pipeline draws with the eye at the origin, so the helper subtracts the eye from each point in double precision (`Position:relativeTo`) before it reaches `Draw.Line3` and `Draw.Sphere`. A render function calls `begin()`, the draws, then `finish()`.

A sphere is never drawn smaller than a fixed angular radius as seen from the eye: its radius is `max(radius, distance · MIN_ANGULAR_RADIUS)`. Ships are tens of metres across and the scenarios span kilometres, so at the framing distance a ship is far below a pixel; the angular minimum keeps it visible at any distance without depending on the field of view or the window size.

The sandbox draws, in the additive pass, for each ship:

| Marker | Meaning |
|---|---|
| Filled sphere at the ship | Its state: blue flying, green arrived, red deadlocked |
| Wireframe sphere at the target | The arrival range |
| Line from the ship to its target | The straight course, while the ship is flying |

## Steering

`Support/SeekArrive.lua` is a temporary seek-and-arrive controller. It writes each ship's `ControlInputComponent`, like any other writer (see [flight control](flight.md)), until the navigation behaviours replace it.

- **Wanted velocity.** The ship wants to fly straight at the target at `min(maxSpeed, distance · linearDrag)`. Coasting under linear drag `k` from speed `v` covers `v / k`, so that speed lets drag alone stop the ship at the target.
- **Turning.** Pitch and yaw turn the nose toward the velocity error (wanted minus current velocity), at an input proportional to the heading error. When the ship is too fast toward the target, the nose stays on the target, and drag and reverse thrust slow it.
- **Thrust.** Forward thrust follows the velocity error along the nose, but only when the nose is within a set cosine of the course. Side and vertical thrust follow the error along those axes. Every axis reaches full input at a set fraction of `maxSpeed` of error.

A ship that has finished gets no input, and drag brings it to rest.

## Metrics

`Support/ScenarioMetrics.lua` updates every ship after each physics step and logs each event as it happens, prefixed `NavigationSandbox: <scenario>:`. Its thresholds are named constants in metres and seconds, converted to game units with `UniverseScaleConfig:toGameUnits`.

| Metric | Definition |
|---|---|
| Arrival | Within 100 m of the target and slower than 50 m/s |
| Deadlock | Not 10 m closer to the target than its best distance for 10 s of simulated time |
| Collision | A ship's bounding sphere starts to overlap another body. A `queryRadius` over the sphere finds the bodies, and each pair is counted once while it stays in contact |
| Frame time | Mean and largest frame time in ms, leaving out the first frame, which pays for spawning |

A ship that arrives or deadlocks is finished. The scenario ends when every ship is finished or the time limit runs out, with a summary line: arrivals, collisions, deadlocks, unfinished ships, simulated time and frame times.

Collisions are counted from bounding-sphere overlap instead of from the physics contacts, because the physics contact graph also holds pairs whose bounding boxes merely overlap.

## Pass and fail

A run fails when a scenario ends with a ship still flying at its time limit, or on a Lua error. The failure is a warning line, so the remaining scenarios still run, and the app quits with `Core.ExitCode.Failure` after the last one. Collisions and deadlocks are logged but don't fail a run: their thresholds come with the navigation behaviours.
