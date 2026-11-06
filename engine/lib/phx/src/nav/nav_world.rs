use ::nav::{
    AXIS_COUNT, AgentId, AvoidanceTuning, Behaviour, Body, Frame, FrameOffset, Goal, Obstacle,
    Profile,
};

use crate::math::*;
use crate::physics::*;
use crate::rf::Rf;

/// What an agent's ship can do, as forces and torques along and about its own axes (forward, right,
/// up; pitch, yaw, roll), with its drags per second.
#[derive(Clone, Copy)]
struct FlightLimits {
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
}

/// An agent: its id in `nav`, the body it is steered from and the limits of its ship.
struct Agent {
    id: AgentId,
    handles: BodyHandles,
    limits: FlightLimits,
    /// `Some` while the agent's behaviour names a target body.
    tracking: Option<Tracking>,
}

/// Writes the current state of the target body into the body that `behaviour` names; the
/// behaviours with a fixed point have none.
fn refresh_target(behaviour: &mut Behaviour, body: Body) {
    match behaviour {
        Behaviour::Pursue { target, .. } => *target = body,
        Behaviour::Follow { leader, .. } => *leader = body,
        Behaviour::Orbit { centre, .. } => *centre = body,
        Behaviour::Seek(_) | Behaviour::Arrive(_) => {}
    }
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
/// `Physics`: agents and obstacles are bodies of that physics world, registered once, and each
/// [`step`](NavWorld::step) reads their state from it. Registered bodies must stay in the physics
/// world for as long as the `NavWorld` steps.
pub struct NavWorld {
    physics: Rf<PhysicsWorld>,
    world: ::nav::World,
    /// The agents in `world`'s order.
    agents: Vec<Agent>,
    obstacles: Vec<BodyHandles>,
    /// The obstacles of the current step, reused between steps.
    obstacle_spheres: Vec<Obstacle>,
}

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
            .unwrap_or_else(|| panic!("NavWorld has no agent {id}"))
    }

    /// Gives the agent a behaviour that names no body.
    fn set_fixed(&mut self, id: u32, behaviour: Behaviour) {
        let agent = self.agent_mut(id);
        agent.tracking = None;
        let agent = agent.id;
        self.world.set_behaviour(agent, Some(behaviour));
    }

    /// Gives the agent a behaviour built by `build` from the state of `target`, which each step
    /// reads again.
    fn set_tracking(&mut self, id: u32, target: &RigidBody, build: impl FnOnce(Body) -> Behaviour) {
        // The agent is looked up first, so an unknown id is reported before a bad target.
        self.agent_mut(id);
        let handles = self.handles_of(target, "target");
        let state = Self::state_of(&self.physics.as_ref(), &handles, "target");
        let behaviour = build(nav_body(&state));
        let agent = self.agent_mut(id);
        agent.tracking = Some(Tracking { handles, behaviour });
        let agent = agent.id;
        self.world.set_behaviour(agent, Some(behaviour));
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
            obstacles: vec![],
            obstacle_spheres: vec![],
        }
    }

    /// The number of control axes per agent: the stride of the axes buffer.
    pub fn axis_count() -> u32 {
        u32::try_from(AXIS_COUNT).expect("the axis count fits u32")
    }

    /// The number of agents.
    pub fn agent_count(&self) -> u32 {
        u32::try_from(self.agents.len()).expect("the agent count fits u32")
    }

    /// Adds an agent steered from `body` and returns its id: 0 for the first agent, then
    /// consecutive. The drags are in 1/s; the thrusts (forward, right, up) and torques (pitch, yaw,
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
            linear_drag,
            angular_drag,
            thrust: [thrust_forward, thrust_right, thrust_up],
            torque: [torque_pitch, torque_yaw, torque_roll],
        };
        let state = Self::state_of(&self.physics.as_ref(), &handles, "agent");
        let id = self.world.add_agent(profile_of(&limits, &state));
        self.agents.push(Agent {
            id,
            handles,
            limits,
            tracking: None,
        });
        self.agent_count() - 1
    }

    /// Adds a sphere that every agent steers around: the body's position, velocity and bounding
    /// radius at each step.
    ///
    /// # Panics
    /// If `body` is not a root body of this world's physics.
    pub fn add_obstacle(&mut self, body: &RigidBody) {
        let handles = self.handles_of(body, "obstacle");
        self.obstacles.push(handles);
    }

    /// Sends the agent to `target`, in world coordinates, and stops it within the arrival radius
    /// of it (game units). The radius is at least two of the agent's own radii in every
    /// behaviour below.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn set_arrive(&mut self, id: u32, target: &Position, arrival_radius: f64) {
        self.set_fixed(
            id,
            Behaviour::Arrive(Goal {
                target: target.v,
                arrival_radius,
            }),
        );
    }

    /// Sends the agent through `target` at top speed, without braking.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn set_seek(&mut self, id: u32, target: &Position, arrival_radius: f64) {
        self.set_fixed(
            id,
            Behaviour::Seek(Goal {
                target: target.v,
                arrival_radius,
            }),
        );
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
    /// read from physics at each step.
    ///
    /// # Panics
    /// If `id` isn't an agent, or `target` isn't a root body of this world's physics.
    pub fn set_pursue(&mut self, id: u32, target: &RigidBody, stand_off: f64, arrival_radius: f64) {
        self.set_tracking(id, target, |target| Behaviour::Pursue {
            target,
            stand_off,
            arrival_radius,
        });
    }

    /// Holds the slot `right`, `up`, `forward` (game units) away from `leader`, in the leader's
    /// own axes, matching its velocity. `leader` is read from physics at each step.
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
        self.set_tracking(id, leader, |leader| Behaviour::Follow {
            leader,
            offset: FrameOffset { right, up, forward },
            arrival_radius,
        });
    }

    /// Circles `centre` at `radius` (game units) in the plane through it normal to `normal`,
    /// anticlockwise about `normal`, moving with the centre. `centre` is read from physics at each
    /// step.
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
        self.set_tracking(id, centre, |centre| Behaviour::Orbit {
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

    /// Reads every registered body from physics and computes each agent's control axes and
    /// arrival.
    ///
    /// # Panics
    /// If a registered body has left the physics world.
    pub fn step(&mut self) {
        {
            let physics = self.physics.as_ref();
            for (index, agent) in self.agents.iter_mut().enumerate() {
                let state = Self::state_of(&physics, &agent.handles, &format!("agent {index}"));
                self.world.set_body(agent.id, nav_body(&state));
                // Read each step, so a change of mass reaches the controller.
                self.world
                    .set_profile(agent.id, profile_of(&agent.limits, &state));
                // A target is a snapshot in `nav`, so it is read again for every step.
                if let Some(tracking) = &mut agent.tracking {
                    let target = Self::state_of(
                        &physics,
                        &tracking.handles,
                        &format!("agent {index}'s target"),
                    );
                    refresh_target(&mut tracking.behaviour, nav_body(&target));
                    self.world.set_behaviour(agent.id, Some(tracking.behaviour));
                }
            }
            self.obstacle_spheres.clear();
            for (index, handles) in self.obstacles.iter().enumerate() {
                let state = Self::state_of(&physics, handles, &format!("obstacle {index}"));
                self.obstacle_spheres.push(Obstacle {
                    position: state.position,
                    velocity: state.velocity,
                    radius: state.radius,
                });
            }
        }
        self.world.set_obstacles(&self.obstacle_spheres);
        self.world.step();
    }

    /// Copies the control axes of the last step into `out`: one value per axis for each agent, in
    /// agent order and in the order of `NavAxis`.
    ///
    /// # Panics
    /// If `out` isn't exactly the number of agents times the number of axes long.
    pub fn copy_axes(&self, out: &mut [f64]) {
        let axes = self.world.axes();
        assert_eq!(
            out.len(),
            axes.len(),
            "NavWorld::copy_axes needs agent_count · axis_count values"
        );
        out.copy_from_slice(axes);
    }

    /// Copies, for each agent in order, whether it was within its goal's arrival radius (at least
    /// two of its own radii) at the last step.
    ///
    /// # Panics
    /// If `out` isn't exactly one value per agent long.
    pub fn copy_arrived(&self, out: &mut [bool]) {
        assert_eq!(
            out.len(),
            self.agents.len(),
            "NavWorld::copy_arrived needs agent_count values"
        );
        for (flag, agent) in out.iter_mut().zip(&self.agents) {
            *flag = self.world.arrived(agent.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::NavAxis;

    const SHIP_RADIUS: f32 = 1.0;
    const LINEAR_DRAG: f64 = 2.0;
    const ANGULAR_DRAG: f64 = 20.0;
    /// Forward thrust that holds a top speed of 100 on a unit-mass body.
    const THRUST_FORWARD: f64 = 100.0 * LINEAR_DRAG;
    const THRUST_SIDE: f64 = THRUST_FORWARD / 4.0;
    const TORQUE: f64 = 60.0;
    const ARRIVAL_RADIUS: f64 = 5.0;
    const GOAL_DISTANCE: f64 = 500.0;
    const ROCK_RADIUS: f32 = 30.0;
    const ROCK_DISTANCE: f64 = 200.0;
    const LONG_GOAL_DISTANCE: f64 = 1000.0;
    const STAND_OFF: f64 = 8.0;
    const SLOT_DISTANCE: f64 = 100.0;
    const ORBIT_RADIUS: f64 = 100.0;

    fn body_in(physics: &mut Physics, radius: f32, at: Position) -> Box<RigidBody> {
        let mut body = RigidBody::new_sphere();
        body.set_scale(radius);
        physics.add_rigid_body(&mut body);
        body.set_position(&at);
        body
    }

    fn ship_in(physics: &mut Physics) -> Box<RigidBody> {
        body_in(physics, SHIP_RADIUS, Position::ZERO)
    }

    fn goal_at(x: f64, z: f64) -> Position {
        Position::new(x, 0.0, z)
    }

    fn axes_of(nav: &NavWorld) -> Vec<f64> {
        let mut out = vec![0.0; nav.agents.len() * AXIS_COUNT];
        nav.copy_axes(&mut out);
        out
    }

    fn axis(axes: &[f64], agent: usize, axis: NavAxis) -> f64 {
        axes[agent * AXIS_COUNT + axis as usize]
    }

    fn add_ship(nav: &mut NavWorld, body: &RigidBody) -> u32 {
        nav.add_agent(
            body,
            LINEAR_DRAG,
            ANGULAR_DRAG,
            THRUST_FORWARD,
            THRUST_SIDE,
            THRUST_SIDE,
            TORQUE,
            TORQUE,
            TORQUE,
        )
    }

    fn state_in(physics: &Physics, body: &RigidBody) -> BodyState {
        let handles = body.handles_in(&physics.world()).expect("a root body");
        physics
            .world()
            .as_ref()
            .body_state(&handles)
            .expect("in the world")
    }

    #[test]
    fn accelerations_are_the_forces_over_the_mass_and_inertia_of_the_body() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let state = state_in(&physics, &ship);
        assert!(state.mass > 0.0, "a ship body has mass");
        let limits = FlightLimits {
            linear_drag: LINEAR_DRAG,
            angular_drag: ANGULAR_DRAG,
            thrust: [THRUST_FORWARD, THRUST_SIDE, THRUST_SIDE],
            torque: [TORQUE, 2.0 * TORQUE, 3.0 * TORQUE],
        };
        let profile = profile_of(&limits, &state);
        assert_eq!(profile.accel_forward, THRUST_FORWARD / state.mass);
        assert_eq!(profile.accel_right, THRUST_SIDE / state.mass);
        assert_eq!(profile.turn_pitch, TORQUE * state.inverse_inertia.x);
        assert_eq!(profile.turn_yaw, 2.0 * TORQUE * state.inverse_inertia.y);
        assert_eq!(profile.turn_roll, 3.0 * TORQUE * state.inverse_inertia.z);
        assert!(profile.turn_pitch > 0.0 && profile.turn_roll > 0.0);
        assert_eq!(
            profile.max_speed(),
            THRUST_FORWARD / state.mass / LINEAR_DRAG
        );
    }

    #[test]
    fn a_body_without_mass_gets_no_acceleration() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let state = BodyState {
            mass: 0.0,
            inverse_inertia: DVec3::ZERO,
            ..state_in(&physics, &ship)
        };
        let limits = FlightLimits {
            linear_drag: LINEAR_DRAG,
            angular_drag: ANGULAR_DRAG,
            thrust: [THRUST_FORWARD; 3],
            torque: [TORQUE; 3],
        };
        let profile = profile_of(&limits, &state);
        assert_eq!(profile.accel_forward, 0.0);
        assert_eq!(profile.turn_yaw, 0.0);
    }

    #[test]
    fn an_agent_is_steered_to_its_goal() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.step();
        assert_eq!(axis(&axes_of(&nav), 0, NavAxis::Yaw), 1.0);
    }

    #[test]
    fn ids_are_consecutive_and_each_agent_has_its_own_goal() {
        let mut physics = Physics::new();
        let first = ship_in(&mut physics);
        // Apart and behind the first ship, so neither is on the other's course.
        let second = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, ROCK_DISTANCE));
        let mut nav = NavWorld::new(&physics);
        assert_eq!(add_ship(&mut nav, &first), 0);
        assert_eq!(add_ship(&mut nav, &second), 1);
        assert_eq!(nav.agent_count(), 2);
        nav.set_arrive(0, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.set_arrive(1, &goal_at(-GOAL_DISTANCE, ROCK_DISTANCE), ARRIVAL_RADIUS);
        nav.step();
        let axes = axes_of(&nav);
        assert_eq!(axis(&axes, 0, NavAxis::Yaw), 1.0);
        assert_eq!(axis(&axes, 1, NavAxis::Yaw), -1.0);
    }

    #[test]
    fn the_next_step_follows_the_body_in_physics() {
        let mut physics = Physics::new();
        let mut ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.step();
        assert_eq!(axis(&axes_of(&nav), 0, NavAxis::Yaw), 1.0);

        ship.set_position(&goal_at(2.0 * GOAL_DISTANCE, 0.0));
        nav.step();
        assert_eq!(axis(&axes_of(&nav), 0, NavAxis::Yaw), -1.0);
    }

    #[test]
    fn an_agent_without_a_goal_gets_no_input() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.clear_behaviour(id);
        nav.step();
        assert!(axes_of(&nav).iter().all(|&value| value == 0.0));
    }

    #[test]
    fn arrival_is_reported_per_agent() {
        let mut physics = Physics::new();
        let near = ship_in(&mut physics);
        let far = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let near_id = add_ship(&mut nav, &near);
        let far_id = add_ship(&mut nav, &far);
        nav.set_arrive(
            near_id,
            &goal_at(0.0, -ARRIVAL_RADIUS / 2.0),
            ARRIVAL_RADIUS,
        );
        nav.set_arrive(far_id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.step();
        let mut arrived = [false; 2];
        nav.copy_arrived(&mut arrived);
        assert_eq!(arrived, [true, false]);
    }

    #[test]
    fn an_obstacle_body_on_the_course_changes_the_axes() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let rock = body_in(&mut physics, ROCK_RADIUS, goal_at(0.0, -ROCK_DISTANCE));
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_arrive(id, &goal_at(0.0, -LONG_GOAL_DISTANCE), ARRIVAL_RADIUS);
        nav.step();
        let straight = axes_of(&nav);

        nav.add_obstacle(&rock);
        nav.step();
        assert_ne!(axes_of(&nav), straight);
    }

    fn yaw_of(nav: &NavWorld) -> f64 {
        axis(&axes_of(nav), 0, NavAxis::Yaw)
    }

    fn arrived_of(nav: &NavWorld) -> bool {
        let mut arrived = [false];
        nav.copy_arrived(&mut arrived);
        arrived[0]
    }

    #[test]
    fn seek_steers_at_its_point() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_seek(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.step();
        assert_eq!(yaw_of(&nav), 1.0);
    }

    #[test]
    fn hold_keeps_the_position_the_body_had_at_the_call() {
        let mut physics = Physics::new();
        let mut ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_hold(id, ARRIVAL_RADIUS);
        nav.step();
        assert!(arrived_of(&nav));

        // The held point stays behind: it doesn't follow the body.
        ship.set_position(&goal_at(GOAL_DISTANCE, 0.0));
        nav.step();
        assert!(!arrived_of(&nav));
        assert_eq!(yaw_of(&nav), -1.0);
    }

    #[test]
    fn a_pursued_body_is_read_again_at_each_step() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut target = body_in(&mut physics, SHIP_RADIUS, goal_at(GOAL_DISTANCE, 0.0));
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
        nav.step();
        assert_eq!(yaw_of(&nav), 1.0);

        target.set_position(&goal_at(-GOAL_DISTANCE, 0.0));
        nav.step();
        assert_eq!(yaw_of(&nav), -1.0);
    }

    #[test]
    fn a_pursuer_arrives_at_its_stand_off_from_the_hull() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        // The stand-off point is the two hulls' radii plus the stand-off from the target's centre.
        let gap = f64::from(SHIP_RADIUS) * 2.0 + STAND_OFF;
        let target = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, -gap));
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
        nav.step();
        assert!(arrived_of(&nav));
    }

    #[test]
    fn a_follower_reads_its_leader_and_its_slot_at_each_step() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        // The leader faces -z, so a slot 100 ahead of it, in its own axes, is at the origin.
        let mut leader = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, SLOT_DISTANCE));
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_follow(id, &leader, 0.0, 0.0, SLOT_DISTANCE, ARRIVAL_RADIUS);
        nav.step();
        assert!(arrived_of(&nav));

        leader.set_position(&goal_at(GOAL_DISTANCE, SLOT_DISTANCE));
        nav.step();
        assert!(!arrived_of(&nav));
        assert_eq!(yaw_of(&nav), 1.0);
    }

    #[test]
    fn an_orbiting_ship_on_its_circle_has_arrived_and_one_far_off_is_steered_to_it() {
        let mut physics = Physics::new();
        let on_circle = ship_in(&mut physics);
        let far = body_in(&mut physics, SHIP_RADIUS, goal_at(-GOAL_DISTANCE, 0.0));
        let centre = body_in(&mut physics, SHIP_RADIUS, goal_at(ORBIT_RADIUS, 0.0));
        let mut nav = NavWorld::new(&physics);
        let first = add_ship(&mut nav, &on_circle);
        let second = add_ship(&mut nav, &far);
        for id in [first, second] {
            nav.set_orbit(id, &centre, ORBIT_RADIUS, &DVec3::Y, ARRIVAL_RADIUS);
        }
        nav.step();
        let mut arrived = [false; 2];
        nav.copy_arrived(&mut arrived);
        assert_eq!(arrived, [true, false]);
        // The far ship is left of the centre and faces -z, so the circle is on its right.
        assert_eq!(axis(&axes_of(&nav), 1, NavAxis::Yaw), 1.0);
    }

    #[test]
    fn a_later_behaviour_drops_the_target_of_an_earlier_one() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut target = body_in(&mut physics, SHIP_RADIUS, goal_at(GOAL_DISTANCE, 0.0));
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
        nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        physics.remove_rigid_body(&mut target);
        // The step would stop on a removed target if the pursuit were still held.
        nav.step();
        assert_eq!(yaw_of(&nav), 1.0);
    }

    #[test]
    #[should_panic(expected = "target: the body is not a root body of its physics world")]
    fn a_target_of_another_physics_world_is_refused() {
        let mut physics = Physics::new();
        let mut other = Physics::new();
        let stranger = ship_in(&mut other);
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_pursue(id, &stranger, STAND_OFF, ARRIVAL_RADIUS);
    }

    #[test]
    #[should_panic(expected = "agent 0's target: its body has left the physics world")]
    fn a_removed_target_stops_the_step_and_names_the_agent() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut target = body_in(&mut physics, SHIP_RADIUS, goal_at(GOAL_DISTANCE, 0.0));
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
        physics.remove_rigid_body(&mut target);
        nav.step();
    }

    #[test]
    #[should_panic(expected = "the normal must be finite and not zero")]
    fn an_orbit_without_a_normal_is_refused() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_orbit(id, &ship, ORBIT_RADIUS, &DVec3::ZERO, ARRIVAL_RADIUS);
    }

    #[test]
    #[should_panic(expected = "the radius must be positive")]
    fn an_orbit_without_a_radius_is_refused() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = add_ship(&mut nav, &ship);
        nav.set_orbit(id, &ship, 0.0, &DVec3::Y, ARRIVAL_RADIUS);
    }

    #[test]
    fn axis_values_are_the_positions_in_the_nav_buffer() {
        assert_eq!(NavAxis::Pitch as usize, ::nav::Axis::Pitch as usize);
        assert_eq!(NavAxis::Yaw as usize, ::nav::Axis::Yaw as usize);
        assert_eq!(NavAxis::Roll as usize, ::nav::Axis::Roll as usize);
        assert_eq!(
            NavAxis::ThrustForward as usize,
            ::nav::Axis::ThrustForward as usize
        );
        assert_eq!(
            NavAxis::ThrustRight as usize,
            ::nav::Axis::ThrustRight as usize
        );
        assert_eq!(NavAxis::ThrustUp as usize, ::nav::Axis::ThrustUp as usize);
        assert_eq!(NavWorld::axis_count() as usize, AXIS_COUNT);
    }

    #[test]
    #[should_panic(expected = "not a root body of its physics world")]
    fn a_body_of_another_physics_world_is_refused() {
        let mut physics = Physics::new();
        let mut other = Physics::new();
        let stranger = ship_in(&mut other);
        let mut nav = NavWorld::new(&physics);
        let _ = ship_in(&mut physics);
        add_ship(&mut nav, &stranger);
    }

    #[test]
    #[should_panic(expected = "not a root body of its physics world")]
    fn a_body_that_is_not_in_physics_is_refused() {
        let physics = Physics::new();
        let loose = RigidBody::new_sphere();
        let mut nav = NavWorld::new(&physics);
        nav.add_obstacle(&loose);
    }

    #[test]
    #[should_panic(expected = "NavWorld has no agent 3")]
    fn an_unknown_agent_id_is_refused() {
        let physics = Physics::new();
        let mut nav = NavWorld::new(&physics);
        nav.set_arrive(3, &Position::ZERO, ARRIVAL_RADIUS);
    }

    #[test]
    #[should_panic(expected = "needs agent_count · axis_count values")]
    fn a_wrong_axes_buffer_is_refused() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        add_ship(&mut nav, &ship);
        nav.copy_axes(&mut [0.0; AXIS_COUNT + 1]);
    }

    #[test]
    #[should_panic(expected = "needs agent_count values")]
    fn a_wrong_arrived_buffer_is_refused() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        add_ship(&mut nav, &ship);
        nav.copy_arrived(&mut [false; 2]);
    }

    #[test]
    #[should_panic(expected = "agent 0: its body has left the physics world")]
    fn a_removed_agent_body_stops_the_step() {
        let mut physics = Physics::new();
        let mut ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        add_ship(&mut nav, &ship);
        physics.remove_rigid_body(&mut ship);
        nav.step();
    }

    #[test]
    #[should_panic(expected = "obstacle 0: its body has left the physics world")]
    fn a_removed_obstacle_body_stops_the_step() {
        let mut physics = Physics::new();
        let mut rock = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        nav.add_obstacle(&rock);
        physics.remove_rigid_body(&mut rock);
        nav.step();
    }
}
