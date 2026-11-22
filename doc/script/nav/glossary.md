# Navigation glossary

The terms the navigation docs use, in alphabetical order. An entry is a short definition and a link to the section that defines the term: that section is the contract, and this page only helps you find it. Where one word has several meanings, each has its own entry.

Docs: [navigation guide](guide.md), [navigation orders](navigation.md), [system planner](system-planner.md), [navigation sandbox](navigation-sandbox.md), [navigation maths](../../engine/nav.md), [system maps](../system-map.md).

## A

**Active** — the status of an order that is the head of its queue and flies a plan or a behaviour. [Queue and status](navigation.md#queue-and-status).

**Agent** — a ship that the navigation world steers: it has a body, a profile, a behaviour and an id (a slot). An agent is a sphere to every other agent. [Model](../../engine/nav.md#model), [engine wrapper](../../engine/nav.md#engine-wrapper-navworld).

**Aim** — the part of the controller that points the ship's nose along the wanted acceleration or along the course. [Controller](../../engine/nav.md#controller).

**Aim factor** (`aim_factor`) — how much beyond the clearance the aim sphere reaches; a little above 1, so a new course is not on the boundary of the threat test. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance), [tuning](../../engine/nav.md#tuning).

**Aim sphere** — the sphere (obstacle radius plus the agent's radius plus the aim factor times the clearance) that a candidate course just touches. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Aligned entry** — the point a ship steers at to cross a goal well's entry sphere when the entry is the plan's last corner: where the line from the ship to the target's centre first meets the sphere. [Consumers](system-planner.md#consumers).

**Approach spec** (`ApproachSpec`) — data that says where a ship stops at an entity: a `StandOff` or a `Point`. [Approach specs](navigation.md#approach-specs).

**Arc** — the circle segment that rounds a corner of a plan, tangent to both legs and flown at a speed the ship can hold. A corner that cannot get one stays sharp. [Corners](system-planner.md#corners).

**Arrival radius** — the distance from a goal at which the agent counts as arrived (the planner uses it to cut the goal's time). The engine floors it at two of the agent's own radii. [Behaviours](../../engine/nav.md#behaviours), [`Out.T`](system-planner.md#input).

**Arrive** — the behaviour that flies to a point and stops there, at the goal's velocity. [Behaviours](../../engine/nav.md#behaviours).

**Autopilot** — the player's way to fly a ship by an order: `AutoPilotSystem` gives the ship an `Urgent` MoveTo, and a manual takeover cancels it. [The player's autopilot](navigation.md#the-players-autopilot).

**Avoidance** — the stage that bends an agent's wanted course around spheres. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Avoidance period** (`avoidancePeriodS`) — seconds between two runs of an agent's avoidance; 0 runs it at every step. A [rate class](#r) sets it. [Update rate](../../engine/nav.md#update-rate), [update-rate classes](navigation.md#update-rate-classes).

## B

**Behaviour** — what an agent does: Arrive, Seek, Pursue, Follow, Hold or Orbit. It gives a wanted velocity and a goal. [Behaviours](../../engine/nav.md#behaviours).

**Blocked** (`NavOrderBlocked`) — the event for a stall: the ship came no closer than `NavConfig.blockedProgressM` to what it flies to in `NavConfig.blockedWindowS`. [Events](navigation.md#events).

**Body** (planner) — an entry of the planner's `bodies` buffer: a star, planet, moon or station with its orbit, radius and safe radius. [Input](system-planner.md#input).

**Body** (engine) — an agent's state for a step: position, velocity, angular velocity, a frame of unit directions and a bounding radius. [Model](../../engine/nav.md#model).

**Braking curve** — the speed from which an agent can brake to rest within a distance, for the damped model with `BRAKING_THRUST_SHARE` of the thrust. Arrive, the moving points, Seek and avoidance share it. [Arrive](../../engine/nav.md#behaviours).

**Braking law** (travel drive) — the multiplier that lets the drive brake to the end of a run; the planner and the autopilot share `TravelDriveMath.brakingMultiplier`. [Drive](system-planner.md#drive).

**Braking margin** — the distance an agent covers in `BRAKING_REACTION_S` at its speed, which the closing-speed limit keeps short of a sphere's hull. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

## C

**Cancelled** — an ended order that was cancelled by `Nav.cancel`. One of the three ended statuses. [Queue and status](navigation.md#queue-and-status).

**Candidate course** — a course that passes one obstacle along the edge of the cone that touches its aim sphere; each threat gives two. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Carry** — see [frame velocity](#f).

**Claim** — a caller takes a ship's controls with `Nav.claim` and writes its input itself. A control owner. [Control ownership](navigation.md#control-ownership).

**Clearance** (engine) — the distance an agent takes to brake from the speed it closes on an obstacle at, clamped between `MIN_CLEARANCE_RADII` and `clearance_radii` agent radii. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Clearance ring** — the red ring on the 2D map: the safe radius of a body. [Circles on the 2D map](../system-map.md#circles-on-the-2d-map).

**Closing-speed limit** — the cap on the speed at which the chosen course closes on a sphere, from the braking curve to the hull less the braking margin. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Collision** (sandbox) — a ship's bounding sphere starts to overlap another body. [Metrics](navigation-sandbox.md#metrics).

**Control owner** (`Enums.ControlOwner`) — who writes a ship's control input: `Manual`, `Claim` or `Orders`; the strongest that wants it wins. [Control ownership](navigation.md#control-ownership).

**Controller** — the stage that turns the wanted velocity into six control axes: a velocity loop, aim, turn and roll. [Controller](../../engine/nav.md#controller).

**Corner** — a waypoint of a plan where the path turns. It is rounded into an [arc](#a), kept under the drive as a [drive turn](#d), or left sharp. [Corners](system-planner.md#corners).

**Cost profile** (`CostProfile`) — a ship's weights by zone tag: how much it minds each kind of cost zone. [Cost profiles](system-planner.md#cost-profiles).

**Cost zone** — a sphere that is dear to cross but can be crossed; the planner goes around it only when that is cheaper. [Cost zones](system-planner.md#cost-zones).

## D

**Deadlock** (sandbox) — a ship that has not come 10 m closer to its target than its best distance for 10 s of simulated time. `ProgressWatch` is the shared rule that orders call a stall. [Metrics](navigation-sandbox.md#metrics), [events](navigation.md#events).

**Depth limit** — how deep the planner recurses into the legs of a detour. [Algorithm](system-planner.md#algorithm).

**Detour** — a waypoint the planner adds to a leg that crosses a safe sphere, placed half way through the overlap on the side away from the centre. [Algorithm](system-planner.md#algorithm).

**Disc** — the body's circle on the 2D map: its bounding radius. [Circles on the 2D map](../system-map.md#circles-on-the-2d-map).

**Done** — an ended order that arrived. One of the three ended statuses. [Queue and status](navigation.md#queue-and-status).

**Drive leg** — a leg flown under the travel drive. `Out.Drive` is 1 at its end. [Drive](system-planner.md#drive).

**Drive run** — a chain of legs flown under one drive engagement: one charge at the start and one release before the end. [Run through a well's entry](system-planner.md#run-through-a-wells-entry).

**Drive turn** — a corner of a drive run that keeps the drive: no arc, and the velocity relaxes onto the next leg over the drag's time constant. [Corners under the drive](system-planner.md#corners-under-the-drive).

**Drop** — a replan that discards the old plan because the goal moved. [Replanning](navigation.md#replanning).

## E

**Ended** — the state after `Done`, `Failed` or `Cancelled`; the issuer releases the order. [Queue and status](navigation.md#queue-and-status).

**Entry radius** — `(1 − WELL_ENTRY_MARGIN)` of a well's radius: a goal inside it is planned with the well. [Frames](system-planner.md#frames).

**Entry waypoint** — the waypoint of a plan into a goal body's well, on the entry sphere, where the plan turns from the system frame to the body's frame. [Frames](system-planner.md#frames).

**Estimate** (`Nav.estimate`) — the time, distance and risk of a flight, computed without moving anything. [Estimates](navigation.md#estimates).

**ETA** — the order's estimated seconds to arrival, in `Nav.progress`. [Queue and status](navigation.md#queue-and-status).

**Event** — a message on the event bus, such as `NavOrderDone` or `NavZoneEntered`, delivered at `PostSim`. [Events](navigation.md#events).

## F

**Facing** — a goal for the ship's nose, held while an order flies; a ship with one flies without the travel drive. [Facing](navigation.md#facing).

**Failed** — an ended order that could not finish; `Nav.failure` gives `TargetLost` or `ShipLost`. [Queue and status](navigation.md#queue-and-status).

**Flown route** (`Nav.routeFlown`) — the path the ship has flown under the order's current plan. [Queue and status](navigation.md#queue-and-status).

**Follow** — an order and a behaviour that hold a slot in the leader's axes and match its velocity. [Orbit and Follow](navigation.md#orbit-and-follow), [Behaviours](../../engine/nav.md#behaviours).

**Frame** (an order's) — the entity whose body a MoveTo's point rides on, with the point's offset kept in world axes. [Kinds of order](navigation.md#kinds-of-order).

**Frame** (planner) — the body a waypoint is an offset from, or `SYSTEM_FRAME` for a point of the system. [Frames](system-planner.md#frames).

**Frame** (engine) — an agent's three unit directions (forward, right, up). [Model](../../engine/nav.md#model).

**Frame** (game loop) — one pass of the game's stages: orders update at `PreSim`, ships and physics step at `Sim`, events arrive at `PostSim`. [The frame](guide.md#the-frame).

**Frame velocity** (carry) — the velocity of the surroundings that carry an agent, such as a gravity well. Drag, top speed and braking work relative to it. [Model](../../engine/nav.md#model).

**Framed part** — the part of a plan into a well that is flown in the body's frame, from the entry waypoint to the goal. [Frames](system-planner.md#frames).

## G

**Goal** (engine) — a point with a velocity and an arrival radius, which a behaviour gives. [Behaviours](../../engine/nav.md#behaviours).

**Goal** (planner) — the point the plan ends at, in a frame; with a stand-off it is a centre. [Goal with a stand-off](system-planner.md#goal-with-a-stand-off).

**Goal stop point** — the point a stand-off distance from the goal's centre, on the line from a leg's start, where a leg that ends at the goal ends. [Goal with a stand-off](system-planner.md#goal-with-a-stand-off).

**Gravity well** — the zone of a body that carries ships with it. See [well](#w).

## H

**Hand-over** — the multiplier below which the drive is released; the planner releases it `HANDOVER_MULT · braking time · speed` before the end of a leg. [Drive](system-planner.md#drive).

**Head** — the first order of a ship's queue, the only one that runs. [Queue and status](navigation.md#queue-and-status).

**Held detour** — the detour an agent keeps flying between two avoidance runs. [Update rate](../../engine/nav.md#update-rate).

**Hold** — an order and a behaviour that stay at a point; the behaviour is Arrive at the body's own position. [Kinds of order](navigation.md#kinds-of-order), [Behaviours](../../engine/nav.md#behaviours).

## I

**Issuer** — whoever gives a ship an order (a script, an AI, a tool): it reads the order's status and events, and releases the order when it has ended. [Navigation orders](navigation.md), [releasing an order](guide.md#fly-somewhere-and-release-the-order-when-it-ends).

## J

**Judging** (rate) — choosing a ship's rate class from a rate zone, traffic or empty space. [Update-rate classes](navigation.md#update-rate-classes).

## K

**Knee** — the radius at which an arc's turn speed reaches `TURN_SPEED_KNEE` of its limit; a larger arc buys little speed. [Corners](system-planner.md#corners).

## L

**Lead** — the distance before a drive-turn corner at which the path leaves the first leg: `speed / drag`. [Corners under the drive](system-planner.md#corners-under-the-drive).

**Leg** — the straight (or framed) segment of a plan between two waypoints. [Algorithm](system-planner.md#algorithm).

**Lifetime** (of an order) — an order lives until its issuer releases it; the system never destroys one. [Lifetime](navigation.md#lifetime).

## M

**MoveTo** — the order that flies to an entity or a point and stops. [Kinds of order](navigation.md#kinds-of-order).

## N

**Nav** — the Lua module `Modules.Navigation` that issues and reads orders. [Using orders](navigation.md#using-orders).

**NavWorld** — the engine's Lua handle on a navigation world, created over one physics world. [Engine wrapper](../../engine/nav.md#engine-wrapper-navworld).

**Neighbours** (`Nav.neighbours`) — the entities within a radius of a ship, nearest first. [Neighbours](navigation.md#neighbours).

**Normal** (Orbit) — the axis an orbit circles about, anticlockwise. [Orbit and Follow](navigation.md#orbit-and-follow).

## O

**Obstacle** — a sphere with a velocity that avoidance bends the course around; every other agent is one too. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Orbit** — an order and a behaviour that circle a body at a radius, anticlockwise about a normal. [Orbit and Follow](navigation.md#orbit-and-follow), [Behaviours](../../engine/nav.md#behaviours).

**Order** — an entity that tells a ship where to go, linked to the ship by its id. [Navigation orders](navigation.md).

**Order ship** (sandbox) — a ship flown by `Nav.order`, not by the sandbox's own `NavWorld`. [Order ships](navigation-sandbox.md#order-ships).

**Outrun** (`GoalOutrun`) — the status when the goal's frame body can move as fast as the ship before the ship reaches its well. [Status](system-planner.md#status), [Frames](system-planner.md#frames).

## P

**Parallel navigation** — the rule for a moving point: the wanted velocity is the point's velocity plus a closing speed along the line of sight, so the bearing stays constant. [Behaviours](../../engine/nav.md#behaviours).

**Pass speed** (`pass_speed`) — the speed at which a Seek enters the goal's arrival radius. [Behaviours](../../engine/nav.md#behaviours).

**Path cost** (`PathCost`) — the cost per metre of crossing a zone, as authored; the profile weights it. [Cost profiles](system-planner.md#cost-profiles).

**Patrol** — the order that flies to each point in turn, once or in a loop. [Kinds of order](navigation.md#kinds-of-order).

**Plan** — the list of waypoints the planner returns for a flight, the goal last. [System planner](system-planner.md).

**Plan ship** (sandbox) — a ship with the `Plan` behaviour: it asks the planner for a path and passes its waypoints. [Scenarios](navigation-sandbox.md#scenarios).

**Planning** — the status of an order that waits for its plan. [Queue and status](navigation.md#queue-and-status).

**Planning speed** — the constant speed the planner flies a path at to time it. [Input](system-planner.md#input), [limits](system-planner.md#limits).

**Point** (approach) — an `ApproachSpec` that stops at an offset from the target's centre in the target's own axes. [Approach specs](navigation.md#approach-specs).

**Priority** (`Nav.Priority`) — `Routine`, `Urgent` or `Combat`: the queue order, and a higher one displaces the head. [Queue and status](navigation.md#queue-and-status).

**Profile** (engine) — what limits an agent's motion: drags, thrust and torque limits, and the speed cap. [Controller](../../engine/nav.md#controller).

**Progress** (`Nav.progress`) — `distanceLeft`, `eta`, `nextWaypoint` and, for a Patrol, `waypointIndex`. [Queue and status](navigation.md#queue-and-status).

**Pursue** — the behaviour that settles at a stand-off beyond a target's hull on the line of sight. [Behaviours](../../engine/nav.md#behaviours).

## Q

**Queue** (`OrderQueue`) — a ship's orders, ordered by priority; only the head runs. [Queue and status](navigation.md#queue-and-status).

**Queued** — the status of an order that waits behind the head. [Queue and status](navigation.md#queue-and-status).

## R

**Rate class** (`Enums.NavRateClass`) — `High`, `Normal` or `Low`: how often a ship's avoidance runs and its plan is checked. [Update-rate classes](navigation.md#update-rate-classes).

**Rate zone** — a zone entity with a `NavRateClassComponent` that gives the ships inside it a class. [Update-rate classes](navigation.md#update-rate-classes).

**Reach** — an obstacle's radius plus the agent's radius plus the clearance; the distance inside which the course is a threat. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Reciprocity** — agents of one world share an avoidance: against a steering agent each takes half. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Refresh** (agent) — `refresh(id)` makes an agent's avoidance due at the next step and drops its held detour. [Update rate](../../engine/nav.md#update-rate).

**Refresh** (order) — a replan that swaps the plan while the ship flies the old one. [Replanning](navigation.md#replanning).

**Release** (order) — `Nav.release` destroys an ended order. [Lifetime](navigation.md#lifetime).

**Release** (drive) — the end of a drive leg: the drive is released `HANDOVER_MULT · braking time · speed` before the end. [Drive](system-planner.md#drive).

**Replan period** (`replanPeriodS`) — seconds between the timer refreshes of a plan, by rate class. [Update-rate classes](navigation.md#update-rate-classes).

**Replanning** — making the plan again when what it was made under changes, or on a timer. [Replanning](navigation.md#replanning).

**Route** (`Nav.route`) — the path an order has left, as positions now. [Queue and status](navigation.md#queue-and-status), [the route](../system-map.md#the-route).

## S

**Safe radius** (`SafeRadius`) — the radius a path keeps out of; at least the hull radius. [Input](system-planner.md#input).

**Scenario** (sandbox) — a table that defines ships, obstacles and zones, and what a run checks. [Scenarios](navigation-sandbox.md#scenarios).

**Seek** — the behaviour that flies through a point at a pass speed and does not stop. [Behaviours](../../engine/nav.md#behaviours).

**Sensing** — the ball query on the physics world's index that finds bodies for the agents' avoidance. [Engine wrapper](../../engine/nav.md#engine-wrapper-navworld).

**Side memory** — the side on which an agent passed the first sphere at the last step; it is kept while the candidate on it is clear. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Slot** (Follow) — the offset in the leader's axes (right, up, forward) a follower holds. [Orbit and Follow](navigation.md#orbit-and-follow).

**Slot** (engine) — an agent's id; `removeAgent` frees it and the next agent takes the lowest free one. [Engine wrapper](../../engine/nav.md#engine-wrapper-navworld).

**Snapshot** (`PlannerSnapshot`) — the game's bodies and zones as plain numbers for a plan request. [Cost profiles](system-planner.md#cost-profiles), [consumers](system-planner.md#consumers).

**Speed cap** (`speed_cap`) — a lower top speed for an agent; the controller's limits are unchanged. [Behaviours](../../engine/nav.md#behaviours).

**Stall** — see [blocked](#b).

**Stand-off** — the distance a ship stops short of a target. For a `StandOff` approach it is in target radii or a gap in metres. [Approach specs](navigation.md#approach-specs), [goal with a stand-off](system-planner.md#goal-with-a-stand-off).

**StandOff** — an `ApproachSpec` that stops on a circle around the target. [Approach specs](navigation.md#approach-specs).

**Station** (sandbox) — a Pursue or Follow ship on its point, slow relative to the tracked body. A space station is a different thing: a body of the system that estimates and plans keep clear of. [Metrics](navigation-sandbox.md#metrics), [scenarios](navigation-sandbox.md#scenarios).

**Status** (order) — `Queued`, `Planning`, `Active`, `Suspended`, `Done`, `Failed` or `Cancelled`. [Queue and status](navigation.md#queue-and-status).

**Status** (planner) — the result of a plan: `Ok`, `DepthExceeded`, `NoDetour` and the others. [Status](system-planner.md#status).

**Stopping speed** — the inverse of the braking curve: the speed from which the agent stops within a distance. [Behaviours](../../engine/nav.md#behaviours).

**Suspended** — the status of an order that had begun and was displaced by a higher priority or by another control owner. [Queue and status](navigation.md#queue-and-status).

**System frame** (`SYSTEM_FRAME`) — the planner's frame for a point fixed in the system. [Frames](system-planner.md#frames).

## T

**Tag** — a `Core.Tag` on a zone entity from a tag group such as `ZoneKind`; a profile weights zones by tag. [Cost profiles](system-planner.md#cost-profiles).

**TargetLost** — a failure reason: the order's target entity (or leader, or facing target) was destroyed. [Queue and status](navigation.md#queue-and-status).

**Tail cap** — the extra cap `tail_gain · distance` on the wanted speed near a goal, which a loop of finite gain can follow. [Behaviours](../../engine/nav.md#behaviours).

**Through** (`Out.Through`) — 1 when the drive is not released at a waypoint: the legs on both sides are one run. [Input](system-planner.md#input).

**Tracked target** — the body a Pursue or Follow agent follows; it is left out of that agent's avoidance. [Obstacle avoidance](../../engine/nav.md#obstacle-avoidance).

**Traffic** — at least `NavConfig.trafficShips` other ships within `NavConfig.trafficRadiusM`; it gives the `High` rate class. [Update-rate classes](navigation.md#update-rate-classes).

**Travel drive** — the ship's drive that multiplies its speed in open space and is capped in zones. [Drive](system-planner.md#drive), [zones](../zones.md).

**Turn speed** — the speed at which an arc is passed, from the thrust share, drag and the radius (`nav::circle_speed`). [Corners](system-planner.md#corners).

## U

**Update-rate class** — see [rate class](#r).

## V

**Velocity loop** — the part of the controller that turns the wanted velocity into thrust: a gain on the error plus drag feed-forward. [Controller](../../engine/nav.md#controller).

## W

**Wanted velocity** — the velocity a behaviour asks for, before avoidance bends it. [Behaviours](../../engine/nav.md#behaviours).

**Waypoint** — a point of a plan, a body and an offset from its centre, with a time and a drive flag. [Frames](system-planner.md#frames), [input](system-planner.md#input).

**Well** — the gravity well of a body: the radius of its zone, inside which ships move with the body. The planner enters a goal's well at the entry waypoint. [Zones](../zones.md), [frames](system-planner.md#frames), [circles on the 2D map](../system-map.md#circles-on-the-2d-map).

## Z

**Zone** (cost) — see [cost zone](#c).

**Zone** (rate) — see [rate zone](#r).

**Zone cap** (`ZoneCap`) — the drive multiplier at the centre of a body's zone. [Input](system-planner.md#input), [drive](system-planner.md#drive).

**Zone signature** (`CostZones.signature`) — one number folded from every cost zone; an order compares it to the one its plan was made under. [Cost profiles](system-planner.md#cost-profiles).
