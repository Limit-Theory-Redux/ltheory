# Navigation maths (`nav`)

The `nav` crate holds the per-agent navigation maths: behaviours that say where an agent wants to go, avoidance that bends that course around spheres, and a controller that turns the result into normalised control axes. It has no engine types and no game vocabulary; the engine wraps it for Lua. Code: `engine/lib/nav/src/`.

## Model

A `World` holds agents and obstacles. Before a step, the caller sets each agent's `Body` (position, velocity, a frame of unit directions, bounding radius), its `Goal` and the obstacles. `World::step` then gives every agent six control axes (`Axis`: pitch, yaw, roll, thrust forward, right and up, each in `[-1, 1]`) and an arrival flag. Axes are one flat buffer, `AXIS_COUNT` values per agent, so the caller reads them without copying per agent.

Units are the caller's, and they only have to agree: lengths in one unit, time in seconds. The crate fixes no axis convention: the caller hands in the frame, and positive pitch turns the nose up and positive yaw turns it right, about the frame's right and up directions.

A step is stateless: the axes depend only on the bodies, goals and obstacles passed in. Each agent goes through three stages.

## Arrive

The wanted velocity points at the goal. Its speed is `min(max_speed, distance · k)`, with `k` the profile's linear drag. An agent coasting under drag `k` from speed `v` covers `v / k`, so this speed lets drag alone stop the agent on the goal. The rule ignores the agent's thrust limit; the controller never asks for more than full thrust, and an agent that can't brake in time overshoots and comes back.

An agent has arrived when its centre is within the goal's arrival radius. The flag is positional: the agent keeps steering to stop on the goal.

## Obstacle avoidance

Obstacles are spheres with a velocity, assumed constant. For each one, the agent's wanted velocity is taken relative to the obstacle, and the time when the agent first comes within the obstacle's *reach* is solved exactly (a quadratic). The reach is the obstacle's radius plus the agent's radius plus a clearance of `clearance_radii` agent radii. Obstacles never reached within `horizon_s` are ignored. When none is reached, the wanted velocity is kept.

Otherwise the course is bent around the threat. A *candidate course* passes one obstacle along the edge of the cone from the agent that just touches the reach sphere (half-angle `α`, `sin α = reach / distance`). The relative speed is kept, the obstacle's velocity is added back, and the speed is capped at the profile's maximum. The cone is aimed at `aim_factor` times the reach (a little above 1), so the new course isn't on the boundary the threat test uses, and the avoidance doesn't switch on and off every step.

Each threatening obstacle gives two candidates, one on the side the course already leans to and one on the other side. Every candidate is tested against all obstacles with the same threat test. An obstacle that blocks a candidate and has not been expanded yet adds its own two candidates, breadth first, up to `MAX_EXPANDED_THREATS` (4) obstacles per agent and step, so the cost stays a few candidates times the number of obstacles instead of growing with their square. The search stops at the first level that has a clear candidate. The agent takes:

- the clear candidate with the smallest turn from the wanted direction; or
- when none is clear, the candidate (the wanted course included) whose first contact comes latest.

While the wanted course leads into an obstacle, the chosen course's speed is also limited to `linear_drag` times the distance at which the agent's hull would touch that obstacle's surface. That is the speed from which drag alone stops the agent there, the same rule as in Arrive. A fast agent turns slowly, because its real velocity lags the wanted one, so without the limit it reaches the obstacle before its velocity has followed the detour. The direction is kept; only the speed falls.

Special cases:

- An agent already inside a reach slides out along that sphere toward its goal, with an outward component that grows with the depth, so it leaves the reach instead of circling in it.
- A course through the centre has no side. A fixed rule picks one from the direction to the centre, so two agents meeting head-on choose the same side, which makes them pass each other rather than mirror each other.

The method is adapted from the obstacle avoidance of Reynolds, "Steering Behaviors For Autonomous Characters" (Game Developers Conference, 1999): the agent predicts which obstacle it would hit and steers around it. Here the contact time with a moving sphere is solved exactly, and the new course is the tangent to the inflated sphere, instead of a probe cylinder ahead of the agent and a lateral force.

## Controller

The controller maps the wanted velocity and the direction of travel to axes. It is a proportional damper, with no model of the ship's response.

- **Course.** When the agent is too slow along the direction of travel, the nose points along the velocity change it needs, so forward thrust adds to it. Otherwise the nose stays on the direction of travel, and drag and reverse thrust slow the agent down.
- **Turn.** Pitch and yaw are proportional to the angle between the nose and the course (`2` per radian, so full input from half a radian), about the axis that brings the nose onto the course. A reversed course turns about the right axis.
- **Thrust.** Each thrust axis is the velocity error along the frame's axis, divided by a tenth of the profile's top speed, clamped. Forward thrust is applied only while the nose is within about 45° of the course (cosine above 0.7); strafing and vertical thrust always are, which also cancels sideways drift.
- Roll is left at 0.

## Tuning

`AvoidanceTuning` holds `horizon_s` (10 s), `clearance_radii` (2) and `aim_factor` (1.05). The controller's gains are constants in `controller.rs`. A longer horizon starts a detour earlier and costs more steering; a larger clearance keeps more distance and makes narrow gaps impassable. A larger `aim_factor` passes obstacles wider, but a goal that lies between the reach and `aim_factor` times the reach can't be approached: the tangent to the inflated sphere misses it, and the agent circles outside.

## Limits

- Obstacles are spheres only, with constant velocity; the agent's own thrust limits aren't part of the avoidance. A course is judged on the wanted velocity, but a fast agent's real velocity lags it. The speed limit near a threat covers this only as far as drag can stop the agent; the horizon and clearance must cover its turning distance.
- The speed limit makes an agent stop in front of a gap that its clearance makes impassable, instead of squeezing through it: it waits there until the goal or the obstacles move.
- The two candidates of an obstacle lie in the plane of the relative velocity and the direction to the centre. A pocket that only opens out of that plane, or more than `MAX_EXPANDED_THREATS` obstacles in a row, can be missed; the agent then takes the latest contact.
- Goals closer to an obstacle than `aim_factor` times its reach are hard to approach (see Tuning).
- Avoidance between agents isn't implemented.
- The cost of a step grows with agents × obstacles; there is no spatial index.

## Engine wrapper (`NavWorld`)

`NavWorld` (`engine/lib/phx/src/nav/`) is the Lua handle on a `World`. It is created for one `Physics` (`NavWorld.Create(physics)`) and reads its bodies from that physics world, so Lua pushes no body state.

- **Registration.** `addAgent(body, maxSpeed, linearDrag)` returns the agent's id (a `u32`: 0, then consecutive) and `addObstacle(body)` adds a sphere every agent steers around. A body must be a root body added to that physics world. There is no removal: a registered body has to stay in the physics world, and `step` panics, naming the agent or obstacle, if one has left it.
- **Goals and tuning.** `setGoal(id, target, arrivalRadius)` takes a `Position` in world coordinates; `clearGoal(id)` leaves the agent without input. `setAvoidanceTuning(horizonS, clearanceRadii, aimFactor)` replaces the tuning (defaults as in `AvoidanceTuning`).
- **Step.** `step()` reads each registered body's position, velocity, rotation and the bounding radius of its collider, turns the rotation into a frame (forward is -z, right +x and up +y in the body's axes, as `Quat` does), and runs `World::step`. All values are in game units and seconds.
- **Results.** `copyAxes(buffer, length)` fills a `double[?]` with `AxisCount()` values per agent, in agent order and in the order of the `NavAxis` enum (`Pitch`, `Yaw`, `Roll`, `ThrustForward`, `ThrustRight`, `ThrustUp`); `copyArrived(buffer, length)` fills a `bool[?]` with one flag per agent. A buffer of the wrong length panics.

## Tests and benchmarks

Unit tests sit with the modules. `tests/flight.rs` flies agents through a point-mass model of a ship (drag, thrust along its axes, turn rate proportional to input) past static and moving spheres, and checks they arrive and keep clear. It checks that behaviour, avoidance and controller work together, not the real ship's response.

`NavWorld`'s tests (in `phx`) use real rigid bodies in a `Physics`: they check that a step follows the body in physics, that obstacle bodies change the axes, and that bad registrations, ids, buffers and removed bodies panic.

`cargo bench -p nav --bench step` measures one step for 100 agents and 60 obstacles in two scenes: a sparse one spread over a large cube, where the avoidance rarely triggers, and a dense one where every agent flies through a field of obstacles on its course, so the candidate search runs at almost every step.
