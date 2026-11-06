# Navigation maths (`nav`)

The `nav` crate holds the per-agent navigation maths: behaviours that say where an agent wants to go, avoidance that bends that course around spheres, and a controller that turns the result into normalised control axes. It has no engine types and no game vocabulary; the engine wraps it for Lua. Code: `engine/lib/nav/src/`.

## Model

A `World` holds agents and obstacles. Before a step, the caller sets each agent's `Body` (position, velocity, angular velocity, a frame of unit directions, bounding radius), its `Profile` (what limits its motion: see Controller), its `Behaviour` and the obstacles. `World::step` then gives every agent six control axes (`Axis`: pitch, yaw, roll, thrust forward, right and up, each in `[-1, 1]`) and an arrival flag. Axes are one flat buffer, `AXIS_COUNT` values per agent, so the caller reads them without copying per agent.

Units are the caller's, and they only have to agree: lengths in one unit, time in seconds. The crate fixes no axis convention: the caller hands in the frame, and positive pitch turns the nose up and positive yaw turns it right, about the frame's right and up directions.

The axes depend on the bodies, profiles, goals and obstacles passed in, and on one memory per agent: the side it passes the sphere it is avoiding on (see Obstacle avoidance). Each agent goes through three stages.

## Behaviours

A `Behaviour` says what an agent does, and gives a wanted velocity plus a `Goal`: the point it wants to be at now. Avoidance and the arrival flag read that goal. Bodies inside a behaviour (a target, a leader, a centre) are snapshots; the caller sets the behaviour again before each step for a target that moves.

**Arrive.** The wanted velocity points at the goal. Its speed is `min(max_speed, distance · k)`, with `k` the profile's linear drag. An agent coasting under drag `k` from speed `v` covers `v / k`, so this speed lets drag alone stop the agent on the goal. The rule ignores the agent's thrust limit; the controller never asks for more than full thrust, and an agent that can't brake in time overshoots and comes back.

**Moving points.** Every behaviour that follows a point uses one rule: the wanted velocity is the point's velocity plus a closing speed along the line of sight, which keeps the bearing to the point constant (parallel navigation). The closing speed is `min(distance · k, c_max)`, where `c_max` is the largest speed that keeps the total within `max_speed`. This leads a crossing target without predicting its path, and an agent that is slower than its target flies the line of sight at top speed. For a point at rest it is exactly Arrive.

**Seek** flies at top speed toward a point and doesn't brake.

**Pursue** aims at the point `stand_off` beyond the target's hull and the agent's own along the line of sight, so it settles beside the target at the target's velocity.

**Follow** holds a slot, an offset in the leader's frame (`FrameOffset`: right, up, forward), moving with the leader.

**Hold** is Arrive at the body's own position (`Behaviour::hold`).

**Orbit** flies a circle of a given radius around a centre, in the plane normal to `normal`, anticlockwise about it. In the plane the direction is the guidance vector field `normalize(tangent − gain · e · radial)` with `e = (|p|² − r²) / r²` and `gain` `ORBIT_FIELD_GAIN`: tangent on the circle and turning inward from outside and outward from inside. Out of the plane a climb term `clamp(−height · k, ±s)` pulls the agent back to the plane. The circle speed `s` solves `s⁴ / r² + k² s² = (share · a)²`: the centripetal demand and the drag together use `ORBIT_THRUST_SHARE` of the forward thrust `a`, and `s` is capped at top speed, so a ship on the circle keeps thrust in reserve for avoidance. The centre's velocity is added, so a moving centre is followed. A zero normal or a non-positive radius makes the agent match the centre's velocity.

An agent has arrived when its centre is within the goal's arrival radius, held to at least `MIN_ARRIVAL_RADII` (2) of the agent's own radii (`Goal::effective_arrival_radius`). A smaller circle can't be honoured by a crowd: goals closer together than the hulls make the agents dodge each other around the goals without any of them reaching its circle, so none arrives. The flag is positional: the agent keeps steering to stop on the goal.

## Obstacle avoidance

Obstacles are spheres with a velocity, assumed constant. Every other agent is one too: a sphere at its position with its bounding radius and its current velocity, whatever its goal (an agent that has arrived or has no goal is a resting sphere). An agent never avoids itself. For each obstacle, the agent's wanted velocity is taken relative to the obstacle, and the time when the agent first comes within the obstacle's *reach* is solved exactly (a quadratic). The reach is the obstacle's radius plus the agent's radius plus a *clearance*. The clearance is the distance drag alone takes the agent to stop from the speed it currently closes on that obstacle at (`closing speed / linear_drag`, from the agent's body velocity, not the wanted one), clamped between `MIN_CLEARANCE_RADII` (0.5) and `clearance_radii` agent radii. A slow agent therefore fits through gaps that a fast one has to avoid. The clearance is computed from the state at the start of the step, never from the course being tested, so the threat test doesn't change with the course it judges. It is also capped so that the goal's arrival circle (the effective arrival radius) lies outside the reach, down to 0 (the hull itself): a goal next to a sphere, such as the berth beside a ship that has arrived, can still be approached. Obstacles never reached within `horizon_s` are ignored. When none is reached, the wanted velocity is kept.

Each course is tested against every sphere, so a step computes what the tests need of a sphere (offset, distance, reach, aim radius, clearance) once per agent and sphere, and the tests read it. A sphere that no course within the agent's speed bound (its top speed, or its wanted or current speed when larger) can come within reach of inside the horizon is skipped by the course search. Such a sphere can't be a threat, so skipping it never changes the result. The closing-speed limit still looks at every sphere with the limited course itself, which can be faster than the bound.

Otherwise the course is bent around the threat. A *candidate course* passes one obstacle along the edge of the cone from the agent that just touches the aim sphere (half-angle `α`, `sin α = aim radius / distance`). The aim radius is the obstacle's radius plus the agent's radius plus `aim_factor` times the clearance (not times the whole reach, which would put the aim sphere far past the surface of a big rock). The relative speed is kept, the obstacle's velocity is added back, and the speed is capped at the profile's maximum. `aim_factor` is a little above 1, so the new course isn't on the boundary the threat test uses, and the avoidance doesn't switch on and off every step.

Each threatening obstacle gives two candidates, one on the side the course already leans to and one on the other side. Every candidate is tested against all obstacles with the same threat test. An obstacle that blocks a candidate and has not been expanded yet adds its own two candidates, breadth first, up to `MAX_EXPANDED_THREATS` (4) obstacles per agent and step, so the cost stays a few candidates times the number of obstacles instead of growing with their square. The search stops at the first level that has a clear candidate. When the agent has a side memory (see below), it stops only at a clear candidate on the kept side, and otherwise searches on while obstacles remain to expand. The agent takes:

- the clear candidate that passes the first sphere on the side the agent passed it on at the previous step, if there is one (the *side memory*; see below); otherwise
- the clear candidate with the smallest turn from the wanted direction; or
- when none is clear, the candidate (the wanted course included) whose first contact comes latest.

The side memory is the sphere's index and the direction, across the line to its centre, that the course leaned to. While the same sphere is the first one reached, the candidate on that side is taken whenever it is clear, even if the other side turns less. Without it, the side flips as the agent's position and velocity shift, and an agent whose nose follows the course never builds speed. The memory is replaced at every step that has a threat and dropped when the course has no side (it runs at the centre). A candidate on the kept side that a farther obstacle blocks is expanded around that obstacle, passing it on the side the course leans to, before a clear candidate on the other side is accepted: a kept side that a nearby obstacle blocks by a few metres would otherwise flip the side, and an agent that flips between two obstacles slides sideways without getting through. The kept side is given up only when no candidate on it is clear within the expanded obstacles.

The speed the chosen course *closes* on a sphere at is also limited, for every sphere that the wanted course, the chosen course or the agent's own velocity leads into. The limit is `linear_drag` times the distance to the sphere's hull (its surface plus the agent's radius) less a *braking margin*, the speed from which drag alone stops the agent short of the hull, the same rule as in Arrive. The margin is the distance the agent covers in `BRAKING_REACTION_S` (0.3 s) at its current speed, and at least the smallest clearance. A fast agent turns slowly, because its real velocity lags the wanted one, and a course that flips near a sphere needs room to turn: without the margin the agent reaches the sphere before its nose and velocity have followed. Only the component toward the sphere is limited; the speed along it is kept, so an agent passing a sphere isn't slowed for it. (Capping the whole speed instead makes agents crawl and dither beside every rock.)

Special cases:

- An agent already inside a reach slides out along that sphere toward its goal, with an outward component that grows with the depth, so it leaves the reach instead of circling in it.
- A course through the centre has no side. A fixed rule picks one from the direction to the centre, so two agents meeting head-on choose the same side, which makes them pass each other rather than mirror each other.

Seek, pursuit and offset pursuit (Follow) follow Reynolds' steering behaviours (see below), with the lead taken from constant bearing instead of a predicted position. The Orbit field is from García de Marina and Hattenberger, "Guidance Vector Fields for Robot Navigation along Orbits" (IMAV 2017, arXiv 1709.01110); its out-of-plane term is ours.

The method is adapted from the obstacle avoidance of Reynolds, "Steering Behaviors For Autonomous Characters" (Game Developers Conference, 1999): the agent predicts which obstacle it would hit and steers around it. Here the contact time with a moving sphere is solved exactly, and the new course is the tangent to the inflated sphere, instead of a probe cylinder ahead of the agent and a lateral force.

## Controller

The controller maps the wanted velocity and the direction of travel to axes with a model of the ship: a damped double integrator, `dv/dt = a − k·v`, with thrust limited per axis. The agent's `Profile` gives the model's constants: the linear and angular drag, the acceleration at full input along the ship's forward, right and up axes, and the angular acceleration at full input about its right (pitch), up (yaw) and forward (roll) axes. A limit of 0 means no authority on that axis.

- **Velocity loop.** The wanted acceleration is `VELOCITY_GAIN · (wanted velocity − velocity)` (10 per second) plus `k · wanted velocity`, the thrust that holds the wanted velocity against drag. It is split along the ship's axes, and each part is divided by that axis's limit. When an axis exceeds its limit, all three inputs are scaled down together, so the ship still accelerates in the wanted direction. Forward and reverse thrust follow the sign of the projection, so there is no angular cut-off.
- **Aim.** The nose points along the wanted acceleration while that speeds the agent up toward its course and exceeds what the side thrusters give, so the main engine does the work. Otherwise it points along the course, and strafing and braking do the rest.
- **Turn.** Pitch and yaw are a PD law on the heading error and the measured angular velocity, in units of the axis's angular acceleration. The stiffness gives the full limit at `TURN_SATURATION_ANGLE` (0.5 rad), capped at a natural frequency of `MAX_TURN_FREQUENCY` (10 rad/s). The damping makes the loop critically damped with the angular drag counted, and is 0 when drag alone damps more. A reversed course turns about the right axis.
- **Roll.** A rate damper: the roll axis opposes the measured roll rate, at full input from `ROLL_RATE_SPAN` (1 rad/s). No behaviour supplies an up direction, so roll isn't steered to one.

## Tuning

`AvoidanceTuning` holds `horizon_s` (10 s), `clearance_radii` (2) and `aim_factor` (1.05). `clearance_radii` is the upper clamp of the speed-dependent clearance; the lower clamp (`MIN_CLEARANCE_RADII`) and `BRAKING_REACTION_S` are constants of `avoidance.rs`, and the controller's gains are constants in `controller.rs`. A longer horizon starts a detour earlier and costs more steering; a larger clearance keeps more distance from fast approaches. A larger `aim_factor` passes obstacles wider, but a goal that lies between the reach and the aim radius of a sphere can't be approached: the tangent misses it, and the agent circles outside. The goal clamp on the clearance (see Obstacle avoidance) covers goals that lie inside the reach, not those between the reach and the aim radius. A longer braking reaction trades contacts for deadlocks: agents stop in front of more gaps. In a dense scene no value of it gives neither.

## Limits

- Obstacles are spheres only, with constant velocity; the agent's own thrust limits aren't part of the avoidance. A course is judged on the wanted velocity, but a fast agent's real velocity lags it. The clearance and the braking margin cover this only as far as drag can stop the agent.
- The nose follows the aim direction at the turn gear's rate, and the wanted velocity can change faster than that. A course that flips by 100–170° within a few hundred metres of a large sphere at cruise speeds of kilometres per second can still graze it, so a dense scene with a large moving sphere sees occasional contacts. The side memory reduces such flips but doesn't remove them.
- Other agents are spheres that keep their velocity, and an agent doesn't know that they steer too. Agents whose goals are closer than their hulls plus clearances (berths a hull apart) can't all hold them: the clearance clamp lets them approach, and the arrival radius floor lets them arrive at the edge of the crowd instead of dodging in front of the berths. A crowd that arrives together still dodges for longer than a lone agent.
- The closing-speed limit makes an agent stop in front of a gap that its clearance makes impassable, instead of squeezing through it: it waits there until the goal or the obstacles move.
- The two candidates of an obstacle lie in the plane of the relative velocity and the direction to the centre. A pocket that only opens out of that plane, or more than `MAX_EXPANDED_THREATS` obstacles in a row, can be missed; the agent then takes the latest contact.
- An orbit that passes inside an obstacle's reach is pushed off its circle by avoidance, and goes back to it once clear. Ships on one orbit aren't spaced out along it.
- Goals closer to an obstacle than `aim_factor` times its reach are hard to approach (see Tuning).
- The cost of a step grows with agents × obstacles (agents are obstacles too); there is no spatial index. The reachability skip rejects spheres only beyond what a course covers within the horizon (88 km for a 8.8 km/s ship and the default horizon), so it saves little at the scale of one system's local space.

## Engine wrapper (`NavWorld`)

`NavWorld` (`engine/lib/phx/src/nav/`) is the Lua handle on a `World`. It is created for one `Physics` (`NavWorld.Create(physics)`) and reads its bodies from that physics world, so Lua pushes no body state.

- **Registration.** `addAgent(body, linearDrag, angularDrag, thrustForward, thrustRight, thrustUp, torquePitch, torqueYaw, torqueRoll)` returns the agent's id (a `u32`: 0, then consecutive). The drags are per second; the thrusts and torques are the forces and torques the ship applies along and about its own axes at full input. `NavWorld` reads the body's mass and principal inertia from physics at every step and turns them into the accelerations of the `Profile`, so a change of mass reaches the controller; the top speed is the forward acceleration over the linear drag. `addObstacle(body)` adds a sphere every agent steers around (the agents are spheres to each other already). A body must be a root body added to that physics world. There is no removal: a registered body has to stay in the physics world, and `step` panics, naming the agent or obstacle, if one has left it.
- **Goals and tuning.** `setGoal(id, target, arrivalRadius)` takes a `Position` in world coordinates (the arrival radius is floored at two of the agent's radii, see Arrive); `clearGoal(id)` leaves the agent without input. `setAvoidanceTuning(horizonS, clearanceRadii, aimFactor)` replaces the tuning (defaults as in `AvoidanceTuning`).
- **Step.** `step()` reads each registered body's position, velocity, angular velocity, mass, inertia, rotation and the bounding radius of its collider, turns the rotation into a frame (forward is -z, right +x and up +y in the body's axes, as `Quat` does), and runs `World::step`. All values are in game units and seconds.
- **Results.** `copyAxes(buffer, length)` fills a `double[?]` with `AxisCount()` values per agent, in agent order and in the order of the `NavAxis` enum (`Pitch`, `Yaw`, `Roll`, `ThrustForward`, `ThrustRight`, `ThrustUp`); `copyArrived(buffer, length)` fills a `bool[?]` with one flag per agent. A buffer of the wrong length panics.

## Tests and benchmarks

Unit tests sit with the modules. `tests/flight.rs` flies agents through a model of a ship (drag, per-axis thrust limits, angular acceleration from torque with angular drag) past static and moving spheres, and checks they arrive and keep clear. It checks that behaviour, avoidance and controller work together, not the real ship's response.

`NavWorld`'s tests (in `phx`) use real rigid bodies in a `Physics`: they check that a step follows the body in physics, that obstacle bodies change the axes, and that bad registrations, ids, buffers and removed bodies panic.

`cargo bench -p nav --bench step` measures one step for 100 agents and 60 obstacles in two scenes: a sparse one spread over a large cube, where the avoidance rarely triggers, and a dense one where every agent flies through a field of obstacles on its course, so the candidate search runs at almost every step.
