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

Obstacles are spheres with a velocity, assumed constant. For each one, the agent's wanted velocity is taken relative to the obstacle, and the time when the agent first comes within the obstacle's *reach* is solved exactly (a quadratic). The reach is the obstacle's radius plus the agent's radius plus a clearance of `clearance_radii` agent radii. Obstacles never reached within `horizon_s` are ignored. Of the others, the one reached first decides.

The course is then bent to pass that obstacle on the side the course already leans to, along the edge of the cone from the agent that just touches the reach sphere (half-angle `α`, `sin α = reach / distance`). The relative speed is kept, the obstacle's velocity is added back, and the speed is capped at the profile's maximum. The cone is aimed at `aim_factor` times the reach (a little above 1), so the new course isn't on the boundary the threat test uses, and the avoidance doesn't switch on and off every step.

- An agent already inside the reach moves straight away from the obstacle's centre.
- A course through the centre has no side. A fixed rule picks one from the direction to the centre, so two agents meeting head-on choose the same side, which makes them pass each other rather than mirror each other.
- Only the nearest threat is handled each step. Two obstacles close together can make the agent steer around one into the other; the agent corrects at the next step, but the margin is smaller.

The method is adapted from the obstacle avoidance of Reynolds, "Steering Behaviors For Autonomous Characters" (Game Developers Conference, 1999): the agent predicts which obstacle it would hit and steers around it. Here the contact time with a moving sphere is solved exactly, and the new course is the tangent to the inflated sphere, instead of a probe cylinder ahead of the agent and a lateral force.

## Controller

The controller maps the wanted velocity and the direction of travel to axes. It is a proportional damper, with no model of the ship's response.

- **Course.** When the agent is too slow along the direction of travel, the nose points along the velocity change it needs, so forward thrust adds to it. Otherwise the nose stays on the direction of travel, and drag and reverse thrust slow the agent down.
- **Turn.** Pitch and yaw are proportional to the angle between the nose and the course (`2` per radian, so full input from half a radian), about the axis that brings the nose onto the course. A reversed course turns about the right axis.
- **Thrust.** Each thrust axis is the velocity error along the frame's axis, divided by a tenth of the profile's top speed, clamped. Forward thrust is applied only while the nose is within about 45° of the course (cosine above 0.7); strafing and vertical thrust always are, which also cancels sideways drift.
- Roll is left at 0.

## Tuning

`AvoidanceTuning` holds `horizon_s` (10 s), `clearance_radii` (2) and `aim_factor` (1.05). The controller's gains are constants in `controller.rs`. A longer horizon starts a detour earlier and costs more steering; a larger clearance keeps more distance and makes narrow gaps impassable.

## Limits

- Obstacles are spheres only, with constant velocity; the agent's own thrust limits aren't part of the avoidance, so a fast agent close to a large obstacle can't always turn in time. The horizon and clearance must cover its turning distance.
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

`cargo bench -p nav --bench step` measures one step for 100 agents and 60 obstacles.
