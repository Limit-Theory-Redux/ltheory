use std::collections::{BTreeMap, HashSet};

use ::nav::{
    AXIS_COUNT, AgentId, AvoidanceTuning, Behaviour, Body, Frame, FrameOffset, Goal, NO_SPEED_CAP,
    Obstacle, ObstacleKey, Profile, Speed, sensing_range,
};

use crate::math::*;
use crate::physics::*;
use crate::rf::Rf;

/// What an agent's ship can do, as forces and torques along and about its own axes (forward, right,
/// up; pitch, yaw, roll), with its drags per second.
#[derive(Clone, Copy)]
struct FlightLimits {
    /// The top speed the agent may be planned to fly at, relative to its frame; kept here because
    /// `profile_of` rebuilds the profile at each step.
    speed_cap: Speed,
    linear_drag: f64,
    angular_drag: f64,
    thrust: [f64; 3],
    torque: [f64; 3],
}

/// A behaviour that is about another body: the behaviour as last built, and the body whose state
/// it is rebuilt from at each step.
struct Tracking {
    handles: BodyHandles,
    behaviour: Behaviour,
    /// Where an Arrive goal lies from the body's centre, in world axes; unused by the other
    /// behaviours.
    offset: DVec3,
}

/// An agent: its id in `nav`, the body it is steered from and the limits of its ship.
struct Agent {
    id: AgentId,
    handles: BodyHandles,
    limits: FlightLimits,
    /// The velocity the agent's surroundings carry it at, as last given to `nav`.
    frame_velocity: DVec3,
    /// `Some` while the agent's behaviour names a target body.
    tracking: Option<Tracking>,
}

/// Writes the current state of the target body into the body that `behaviour` names. An Arrive
/// goal is the point `offset` from the body, moving with it; Seek has no body.
fn refresh_target(behaviour: &mut Behaviour, body: Body, offset: DVec3) {
    match behaviour {
        Behaviour::Pursue { target, .. } => *target = body,
        Behaviour::Follow { leader, .. } => *leader = body,
        Behaviour::Orbit { centre, .. } => *centre = body,
        Behaviour::Arrive(goal) => {
            *goal = Goal {
                target: body.position + offset,
                velocity: body.velocity,
                ..*goal
            };
        }
        Behaviour::Seek { .. } => {}
    }
}

/// How far from its centre an agent senses obstacles: the range `nav` gives its body and profile,
/// with drag, and so the distance it covers while braking, following its velocity relative to
/// its frame.
fn range_of(
    body: &Body,
    profile: &Profile,
    frame_velocity: DVec3,
    tuning: &AvoidanceTuning,
) -> f64 {
    let relative = Body {
        velocity: body.velocity - frame_velocity,
        ..*body
    };
    sensing_range(&relative, profile, tuning)
}

/// `value / divisor`, or 0 when the divisor is 0: a body without mass or inertia gets no
/// acceleration from a force on it.
fn per_unit(value: f64, divisor: f64) -> f64 {
    if divisor > 0.0 { value / divisor } else { 0.0 }
}

/// The accelerations `limits` give a body with this mass and inertia. The inertia is the
/// diagonal of the inverse tensor about the body axes (x right, y up, z backward); roll is about
/// z and pitch about x, so each torque pairs with the diagonal entry of its axis.
fn profile_of(limits: &FlightLimits, state: &BodyState) -> Profile {
    let inverse_mass = per_unit(1.0, state.mass);
    let [thrust_forward, thrust_right, thrust_up] = limits.thrust.map(|force| force * inverse_mass);
    Profile {
        speed_cap: limits.speed_cap,
        linear_drag: limits.linear_drag,
        angular_drag: limits.angular_drag,
        accel_forward: thrust_forward,
        accel_right: thrust_right,
        accel_up: thrust_up,
        turn_pitch: limits.torque[0] * state.inverse_inertia.x,
        turn_yaw: limits.torque[1] * state.inverse_inertia.y,
        turn_roll: limits.torque[2] * state.inverse_inertia.z,
    }
}

/// Lua handle on the navigation state of one simulated space.
///
/// The only wrapper over the `nav` crate; the algorithms live there. It is bound to one
/// `Physics`: agents are bodies of that physics world, registered once, and each
/// [`step`](NavWorld::step) reads their state from it. Agents (and the bodies that behaviours
/// track) must stay in the physics world for as long as the `NavWorld` steps. Obstacles are
/// found by sensing, never registered.
///
/// An agent's id is its slot: [`remove_agent`](NavWorld::remove_agent) frees it and
/// [`add_agent`](NavWorld::add_agent) reuses the lowest free one, and the buffers the `copy_*`
/// methods fill have one entry per slot.
pub struct NavWorld {
    physics: Rf<PhysicsWorld>,
    world: ::nav::World,
    /// The agents by slot, which is `world`'s order too; `None` for a free slot.
    agents: Vec<Option<Agent>>,
    /// The collision mask of the bodies agents sense; [`SENSING_OFF`] senses none.
    sensing: CollisionMask,
    /// The keys of the bodies sensing leaves out: the agents' (they are spheres to each other
    /// already).
    excluded: HashSet<BodyKey>,
    /// The bodies the agents sensed in the current step, by key, reused between steps.
    sensed: BTreeMap<BodyKey, SensedBody>,
    /// The balls queried in the current step, reused between steps.
    queried: Vec<Ball>,
    /// The obstacles of the current step, reused between steps.
    obstacle_spheres: Vec<Obstacle>,
}

/// The most a merged sensing query's radius may exceed the larger of the radii it replaces.
const MAX_QUERY_GROWTH: f64 = 1.5;

/// A sphere of space in game units.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ball {
    centre: DVec3,
    radius: f64,
}

impl Ball {
    fn new(centre: DVec3, radius: f64) -> Self {
        Self { centre, radius }
    }

    /// The smallest ball that contains both.
    fn enclosing(self, other: Ball) -> Ball {
        let offset = other.centre - self.centre;
        let distance = offset.length();
        if distance + other.radius <= self.radius {
            return self;
        }
        if distance + self.radius <= other.radius {
            return other;
        }
        let radius = (distance + self.radius + other.radius) * 0.5;
        Ball::new(
            self.centre + offset * ((radius - self.radius) / distance),
            radius,
        )
    }
}

/// Adds `ball` to the sensing queries, merged into the first one it can share without the merged
/// radius exceeding [`MAX_QUERY_GROWTH`] times the larger radius. Bodies found by the merged ball
/// include everything either ball would find, and the obstacle cull in `nav` drops the ones an
/// agent can't reach, so agents close together cost one query instead of one each.
fn add_to_queries(queries: &mut Vec<Ball>, ball: Ball) {
    for query in queries.iter_mut() {
        let merged = query.enclosing(ball);
        if merged.radius <= MAX_QUERY_GROWTH * query.radius.max(ball.radius) {
            *query = merged;
            return;
        }
    }
    queries.push(ball);
}

/// The sensing mask that senses no body.
const SENSING_OFF: CollisionMask = 0;

impl NavWorld {
    fn handles_of(&self, body: &RigidBody, what: &str) -> BodyHandles {
        body.handles_in(&self.physics).unwrap_or_else(|| {
            panic!("NavWorld {what}: the body is not a root body of its physics world")
        })
    }

    fn agent_mut(&mut self, id: u32) -> &mut Agent {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.agents.get_mut(index))
            .and_then(Option::as_mut)
            .unwrap_or_else(|| panic!("NavWorld has no agent {id}"))
    }

    /// Gives the agent a behaviour that names no body.
    fn set_fixed(&mut self, id: u32, behaviour: Behaviour) {
        let agent = self.agent_mut(id);
        agent.tracking = None;
        let agent = agent.id;
        self.world.set_behaviour(agent, Some(behaviour));
        self.world.set_tracked(agent, None);
    }

    /// The key `nav` knows `handles`' body by: the agent's id if the body is an agent's, else the
    /// key sensing gives it.
    fn obstacle_key_of(&self, handles: &BodyHandles) -> ObstacleKey {
        let key = handles.key();
        self.agents
            .iter()
            .flatten()
            .find(|agent| agent.handles.key() == key)
            .map_or(ObstacleKey::Body(key), |agent| ObstacleKey::Agent(agent.id))
    }

    /// Gives the agent a behaviour built by `build` from the state of `target`, which each step
    /// reads again (see [`refresh_target`] for `offset`). With `leave_out`, the agent doesn't
    /// avoid `target`.
    fn set_tracking(
        &mut self,
        id: u32,
        target: &RigidBody,
        leave_out: bool,
        offset: DVec3,
        build: impl FnOnce(Body) -> Behaviour,
    ) {
        // The agent is looked up first, so an unknown id is reported before a bad target.
        self.agent_mut(id);
        let handles = self.handles_of(target, "target");
        let state = Self::state_of(&self.physics.as_ref(), &handles, "target");
        let body = nav_body(&state);
        let mut behaviour = build(body);
        refresh_target(&mut behaviour, body, offset);
        let tracked = leave_out.then(|| self.obstacle_key_of(&handles));
        let agent = self.agent_mut(id);
        agent.tracking = Some(Tracking {
            handles,
            behaviour,
            offset,
        });
        let agent = agent.id;
        self.world.set_behaviour(agent, Some(behaviour));
        self.world.set_tracked(agent, tracked);
    }

    /// Gives `nav` each agent's body, profile and tracked target as physics holds them now, and
    /// gathers the bodies each one senses into `sensed`. An agent whose avoidance isn't due in a
    /// step of `dt` seconds senses nothing: it doesn't use the obstacles.
    fn read_agents(&mut self, physics: &PhysicsWorld, dt: f64) {
        for (index, agent) in self.agents.iter_mut().enumerate() {
            let Some(agent) = agent else {
                continue;
            };
            let state = Self::state_of(physics, &agent.handles, &format!("agent {index}"));
            let body = nav_body(&state);
            // Read each step, so a change of mass reaches the controller.
            let profile = profile_of(&agent.limits, &state);
            self.world.set_body(agent.id, body);
            self.world.set_profile(agent.id, profile);
            // A target is a snapshot in `nav`, so it is read again for every step.
            if let Some(tracking) = &mut agent.tracking {
                let target = Self::state_of(
                    physics,
                    &tracking.handles,
                    &format!("agent {index}'s target"),
                );
                refresh_target(&mut tracking.behaviour, nav_body(&target), tracking.offset);
                self.world.set_behaviour(agent.id, Some(tracking.behaviour));
            }
            if self.sensing != SENSING_OFF && self.world.avoidance_due(agent.id, dt) {
                let range = range_of(&body, &profile, agent.frame_velocity, self.world.tuning());
                add_to_queries(&mut self.queried, Ball::new(state.position, range));
            }
        }
        for ball in &self.queried {
            physics.sense_ball(
                ball.centre,
                ball.radius,
                self.sensing,
                &self.excluded,
                &mut self.sensed,
            );
        }
    }

    /// Fills `obstacle_spheres` with the sensed bodies in key order, so the scene is the same for
    /// the same bodies.
    fn collect_obstacles(&mut self) {
        self.obstacle_spheres.clear();
        self.obstacle_spheres
            .extend(self.sensed.iter().map(|(&key, body)| Obstacle {
                key: ObstacleKey::Body(key),
                position: body.position,
                velocity: body.velocity,
                radius: body.radius,
            }));
    }

    fn state_of(world: &PhysicsWorld, handles: &BodyHandles, what: &str) -> BodyState {
        world
            .body_state(handles)
            .unwrap_or_else(|| panic!("NavWorld {what}: its body has left the physics world"))
    }
}

/// The body of an agent as `nav` steers it. The frame follows the engine's convention: forward
/// is -z, right is +x and up is +y in the body's own axes.
fn nav_body(state: &BodyState) -> Body {
    let rotation = Quat::from_na(&state.rotation);
    Body {
        position: state.position,
        velocity: state.velocity,
        frame: Frame {
            forward: rotation.get_forward().as_dvec3(),
            right: rotation.get_right().as_dvec3(),
            up: rotation.get_up().as_dvec3(),
        },
        angular_velocity: state.angular_velocity,
        radius: state.radius,
    }
}

// Ids are `u32` and lengths are slices because the FFI generator maps types by their text.
#[luajit_ffi_gen::luajit_ffi]
impl NavWorld {
    /// Creates an empty world that reads its bodies from `physics`.
    #[bind(name = "Create")]
    pub fn new(physics: &Physics) -> Self {
        Self {
            physics: physics.world(),
            world: ::nav::World::new(),
            agents: vec![],
            sensing: SENSING_OFF,
            excluded: HashSet::new(),
            sensed: BTreeMap::new(),
            queried: Vec::new(),
            obstacle_spheres: vec![],
        }
    }

    /// The number of control axes per agent: the stride of the axes buffer.
    pub fn axis_count() -> u32 {
        u32::try_from(AXIS_COUNT).expect("the axis count fits u32")
    }

    /// The number of slots: the agents and the free slots between them. It is the number of
    /// entries, per agent, in the buffers `copy_axes` and `copy_arrived` fill.
    pub fn slot_count(&self) -> u32 {
        u32::try_from(self.agents.len()).expect("the slot count fits u32")
    }

    /// Adds an agent steered from `body` and returns its id, its slot: the lowest one that no
    /// agent holds, so 0 for the first agent and then consecutive until an agent is removed.
    /// The drags are in 1/s; the thrusts (forward, right, up) and torques (pitch, yaw,
    /// roll) are the forces and torques the ship applies along and about its own axes at full
    /// input. Accelerations follow from the body's mass and inertia at each step, and the top
    /// speed is the forward acceleration over the linear drag.
    ///
    /// # Panics
    /// If `body` is not a root body of this world's physics.
    // The generator maps types by their text and has no struct arguments, so the limits stay as
    // separate numbers.
    pub fn add_agent(
        &mut self,
        body: &RigidBody,
        linear_drag: f64,
        angular_drag: f64,
        thrust_forward: f64,
        thrust_right: f64,
        thrust_up: f64,
        torque_pitch: f64,
        torque_yaw: f64,
        torque_roll: f64,
    ) -> u32 {
        let handles = self.handles_of(body, "agent");
        let limits = FlightLimits {
            speed_cap: NO_SPEED_CAP,
            linear_drag,
            angular_drag,
            thrust: [thrust_forward, thrust_right, thrust_up],
            torque: [torque_pitch, torque_yaw, torque_roll],
        };
        let state = Self::state_of(&self.physics.as_ref(), &handles, "agent");
        let id = self.world.add_agent(profile_of(&limits, &state));
        self.excluded.insert(handles.key());
        let agent = Some(Agent {
            id,
            handles,
            limits,
            frame_velocity: DVec3::ZERO,
            tracking: None,
        });
        // `nav` hands out the lowest free slot, so the slots stay in step with it.
        let slot = id.index();
        if slot == self.agents.len() {
            self.agents.push(agent);
        } else {
            self.agents[slot] = agent;
        }
        u32::try_from(slot).expect("the slot count fits u32")
    }

    /// Removes the agent and frees its id, which the next [`add_agent`](NavWorld::add_agent) may
    /// hand out again, so the caller drops it. The agent's body is left alone in physics, where
    /// the other agents sense it as an obstacle from the next step on. An agent that follows or
    /// pursues the body keeps doing so, but no longer leaves the body out of its avoidance.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn remove_agent(&mut self, id: u32) {
        let agent = self.agent_mut(id);
        let (agent, key) = (agent.id, agent.handles.key());
        self.world.remove_agent(agent);
        self.excluded.remove(&key);
        self.agents[agent.index()] = None;
    }

    /// Turns sensing on or off. With a non-zero `mask`, each step finds the bodies whose collision
    /// group shares a bit with it within each agent's sensing range and treats them as
    /// obstacles; 0 turns it off (the default). Agents and their own parts are never sensed.
    /// Bodies are found as of the last `Physics::update`.
    // The mask is a `u32` because the generator maps types by their text (`CollisionMask`).
    pub fn set_sensing(&mut self, mask: u32) {
        self.sensing = mask;
    }

    /// Sends the agent to `target`, in world coordinates, and stops it within the arrival radius
    /// of it (game units). The radius is at least two of the agent's own radii in every
    /// behaviour below.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn set_arrive(&mut self, id: u32, target: &Position, arrival_radius: f64) {
        self.set_fixed(id, Behaviour::Arrive(Goal::fixed(target.v, arrival_radius)));
    }

    /// Sends the agent to the point `offset` (game units, world axes) from `body`'s centre, which
    /// moves with `body`, and stops it on that point: the goal has the body's velocity, so the
    /// agent comes to rest relative to the body. `body` is read from physics at each step. It is
    /// only a frame: the agent still avoids it. The offset keeps its world direction while the
    /// body turns. Pair it with [`set_frame_velocity`](NavWorld::set_frame_velocity) when the
    /// body is carried along by something that drag doesn't act on.
    ///
    /// # Panics
    /// If `id` isn't an agent, or `body` isn't a root body of this world's physics.
    pub fn set_arrive_relative(
        &mut self,
        id: u32,
        body: &RigidBody,
        offset: &DVec3,
        arrival_radius: f64,
    ) {
        self.set_tracking(id, body, false, *offset, |_| {
            Behaviour::Arrive(Goal::fixed(DVec3::ZERO, arrival_radius))
        });
    }

    /// Sets the velocity (game units per second, world axes) that carries the agent along with its
    /// surroundings, such as a gravity well's, until it is set again; zero by default. The agent
    /// is steered relative to it: drag, top speed, braking and avoidance all follow the velocity
    /// the agent has relative to it. Set it before each step it changes.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn set_frame_velocity(&mut self, id: u32, velocity: &DVec3) {
        let agent = self.agent_mut(id);
        agent.frame_velocity = *velocity;
        let agent = agent.id;
        self.world.set_frame_velocity(agent, *velocity);
    }

    /// Makes the agent's nose hold toward `point`, in world coordinates, instead of the facing its
    /// behaviour gives, until it is set again or cleared ([`clear_facing`](NavWorld::clear_facing)).
    /// It is a point and not a body, so the caller sets it each step for a target that moves. The
    /// nose still follows the course while the agent speeds up toward it harder than its side
    /// thrusters can, and a point within the agent's own radius gives no facing. A new behaviour
    /// leaves the facing as it is.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn set_facing(&mut self, id: u32, point: &Position) {
        let agent = self.agent_mut(id).id;
        self.world.set_facing(agent, Some(point.v));
    }

    /// Gives the agent's facing back to its behaviour.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn clear_facing(&mut self, id: u32) {
        let agent = self.agent_mut(id).id;
        self.world.set_facing(agent, None);
    }

    /// Sends the agent through `target` at top speed and does not stop it there. It slows so as to
    /// pass `target` at `pass_speed` (game units per second, relative to the agent's frame), on
    /// the braking curve Arrive uses; an infinite `pass_speed`, or one at or above the agent's top
    /// speed, never slows it.
    ///
    /// # Panics
    /// If `id` isn't an agent, or `pass_speed` is negative or NaN.
    // The generator maps types by their text, so the speed stays a bare `f64` (`Speed`).
    pub fn set_seek(&mut self, id: u32, target: &Position, arrival_radius: f64, pass_speed: f64) {
        assert!(
            pass_speed >= 0.0,
            "a pass speed must be zero or more, got {pass_speed}"
        );
        self.set_fixed(
            id,
            Behaviour::Seek {
                goal: Goal::fixed(target.v, arrival_radius),
                pass_speed,
            },
        );
    }

    /// Limits the speed the agent is planned to fly at, in every behaviour and in avoidance, to
    /// `cap` (game units per second, relative to the agent's frame; see
    /// [`set_frame_velocity`](NavWorld::set_frame_velocity)), until it is set again. An agent
    /// above the cap is braked down to it at full thrust. An infinite cap, the default, lifts
    /// the limit.
    ///
    /// # Panics
    /// If `id` isn't an agent, or `cap` is negative or NaN.
    // The generator maps types by their text, so the speed stays a bare `f64` (`Speed`).
    pub fn set_speed_cap(&mut self, id: u32, cap: f64) {
        assert!(cap >= 0.0, "a speed cap must be zero or more, got {cap}");
        let agent = self.agent_mut(id);
        agent.limits.speed_cap = cap;
        let agent = agent.id;
        self.world.set_speed_cap(agent, cap);
    }

    /// Keeps the agent where its body is now.
    ///
    /// # Panics
    /// If `id` isn't an agent, or its body has left the physics world.
    pub fn set_hold(&mut self, id: u32, arrival_radius: f64) {
        let handles = self.agent_mut(id).handles;
        let state = Self::state_of(&self.physics.as_ref(), &handles, "agent");
        self.set_fixed(id, Behaviour::hold(&nav_body(&state), arrival_radius));
    }

    /// Meets `target` `stand_off` from its hull (game units), matching its velocity. `target` is
    /// read from physics at each step, and the agent doesn't avoid it (it still avoids everything
    /// else; if `target` is an agent, it ignores this one while this one is behind it, and avoids
    /// it with the half share of any steering agent otherwise).
    ///
    /// # Panics
    /// If `id` isn't an agent, or `target` isn't a root body of this world's physics.
    pub fn set_pursue(&mut self, id: u32, target: &RigidBody, stand_off: f64, arrival_radius: f64) {
        self.set_tracking(id, target, true, DVec3::ZERO, |target| Behaviour::Pursue {
            target,
            stand_off,
            arrival_radius,
        });
    }

    /// Holds the slot `right`, `up`, `forward` (game units) away from `leader`, in the leader's
    /// own axes, matching its velocity. `leader` is read from physics at each step, and the agent
    /// doesn't avoid it (it still avoids everything else; if `leader` is an agent, it ignores
    /// this one while this one is behind it, and avoids it with the half share of any steering
    /// agent otherwise).
    ///
    /// # Panics
    /// If `id` isn't an agent, or `leader` isn't a root body of this world's physics.
    // The generator has no struct arguments, so the offset stays as three numbers.
    pub fn set_follow(
        &mut self,
        id: u32,
        leader: &RigidBody,
        right: f64,
        up: f64,
        forward: f64,
        arrival_radius: f64,
    ) {
        self.set_tracking(id, leader, true, DVec3::ZERO, |leader| Behaviour::Follow {
            leader,
            offset: FrameOffset { right, up, forward },
            arrival_radius,
        });
    }

    /// Circles `centre` at `radius` (game units) in the plane through it normal to `normal`,
    /// anticlockwise about `normal`, moving with the centre. `centre` is read from physics at each
    /// step, and the agent still avoids it, so an orbit that passes through it is bent.
    ///
    /// # Panics
    /// If `id` isn't an agent, `centre` isn't a root body of this world's physics, `radius` isn't
    /// positive, or `normal` is zero or not finite.
    pub fn set_orbit(
        &mut self,
        id: u32,
        centre: &RigidBody,
        radius: f64,
        normal: &DVec3,
        arrival_radius: f64,
    ) {
        assert!(radius > 0.0, "NavWorld orbit: the radius must be positive");
        assert!(
            normal.is_finite() && normal.length_squared() > 0.0,
            "NavWorld orbit: the normal must be finite and not zero"
        );
        let normal = *normal;
        self.set_tracking(id, centre, false, DVec3::ZERO, |centre| Behaviour::Orbit {
            centre,
            radius,
            normal,
            arrival_radius,
        });
    }

    /// Leaves the agent without input.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn clear_behaviour(&mut self, id: u32) {
        let agent = self.agent_mut(id);
        agent.tracking = None;
        let agent = agent.id;
        self.world.set_behaviour(agent, None);
        self.world.set_tracked(agent, None);
    }

    /// Sets how agents avoid obstacles: the look-ahead in seconds, the largest clearance in agent
    /// radii (the clearance itself grows with the closing speed) and the aim factor (see
    /// `doc/engine/nav.md`).
    pub fn set_avoidance_tuning(&mut self, horizon_s: f64, clearance_radii: f64, aim_factor: f64) {
        self.world.set_tuning(AvoidanceTuning {
            horizon_s,
            clearance_radii,
            aim_factor,
        });
    }

    /// Sets how often the agent's avoidance, and the sensing it needs, runs, in seconds; 0 (the
    /// default) runs it at every step. The behaviour, the controller and the arrival test run at
    /// every step either way, and between runs the agent keeps the detour the last run gave it.
    /// Agents with one period run in different steps. A goal change doesn't bring the next run
    /// forward: call [`refresh`](NavWorld::refresh) for that.
    ///
    /// # Panics
    /// If `id` isn't an agent, or `seconds` is negative or not finite.
    pub fn set_avoidance_period(&mut self, id: u32, seconds: f64) {
        let agent = self.agent_mut(id).id;
        self.world.set_avoidance_period(agent, seconds);
    }

    /// Makes the agent's avoidance run at the next step and drops the detour it kept. Call it when
    /// the agent's goal changes to somewhere else, so the new course isn't flown on the old
    /// detour; the goal setters don't, because a caller may repeat them every frame.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn refresh(&mut self, id: u32) {
        let agent = self.agent_mut(id).id;
        self.world.refresh(agent);
    }

    /// The distance (game units) from the agent's centre within which it senses bodies now, from
    /// its velocity relative to its frame: the longer it takes to stop, the farther it looks. It
    /// is the range [`step`](NavWorld::step) queries, whatever the sensing mask.
    ///
    /// # Panics
    /// If `id` isn't an agent, or its body has left the physics world.
    pub fn sensing_range(&self, id: u32) -> f64 {
        let agent = usize::try_from(id)
            .ok()
            .and_then(|index| self.agents.get(index))
            .and_then(Option::as_ref)
            .unwrap_or_else(|| panic!("NavWorld has no agent {id}"));
        let state = Self::state_of(&self.physics.as_ref(), &agent.handles, "agent");
        let profile = profile_of(&agent.limits, &state);
        range_of(
            &nav_body(&state),
            &profile,
            agent.frame_velocity,
            self.world.tuning(),
        )
    }

    /// Reads every agent and tracked body from physics, senses obstacles for the agents whose
    /// avoidance is due, and computes each agent's control axes and arrival. `dt` is the time in
    /// seconds since the last step.
    ///
    /// # Panics
    /// If an agent's body or a tracked body has left the physics world, or `dt` is negative or
    /// not finite.
    pub fn step(&mut self, dt: f64) {
        let physics = self.physics.clone();
        {
            let physics = physics.as_ref();
            self.sensed.clear();
            self.queried.clear();
            self.read_agents(&physics, dt);
        }
        self.collect_obstacles();
        self.world.set_obstacles(&self.obstacle_spheres);
        self.world.step(dt);
    }

    /// Copies the control axes of the last step into `out`: one value per axis for each slot, in
    /// slot order and in the order of `NavAxis`. A free slot's are zero.
    ///
    /// # Panics
    /// If `out` isn't exactly the number of slots times the number of axes long.
    pub fn copy_axes(&self, out: &mut [f64]) {
        let axes = self.world.axes();
        assert_eq!(
            out.len(),
            axes.len(),
            "NavWorld::copy_axes needs slot_count · axis_count values"
        );
        out.copy_from_slice(axes);
    }

    /// Copies, for each slot in order, whether its agent was within its goal's arrival radius (at
    /// least two of its own radii) at the last step. A free slot's is false.
    ///
    /// # Panics
    /// If `out` isn't exactly one value per slot long.
    pub fn copy_arrived(&self, out: &mut [bool]) {
        assert_eq!(
            out.len(),
            self.agents.len(),
            "NavWorld::copy_arrived needs slot_count values"
        );
        for (flag, agent) in out.iter_mut().zip(&self.agents) {
            *flag = agent
                .as_ref()
                .is_some_and(|agent| self.world.arrived(agent.id));
        }
    }
}

#[cfg(test)]
mod tests;
