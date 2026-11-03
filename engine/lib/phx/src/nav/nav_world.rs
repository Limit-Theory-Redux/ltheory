use ::nav::{AXIS_COUNT, AgentId, AvoidanceTuning, Body, Frame, Goal, Obstacle, Profile};

use crate::math::*;
use crate::physics::*;
use crate::rf::Rf;

/// Lua handle on the navigation state of one simulated space.
///
/// The only wrapper over the `nav` crate; the algorithms live there. It is bound to one
/// `Physics`: agents and obstacles are bodies of that physics world, registered once, and each
/// [`step`](NavWorld::step) reads their state from it. Registered bodies must stay in the physics
/// world for as long as the `NavWorld` steps.
pub struct NavWorld {
    physics: Rf<PhysicsWorld>,
    world: ::nav::World,
    /// The agents in `world`'s order, with the body each one is steered from.
    agents: Vec<(AgentId, BodyHandles)>,
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

    fn agent(&self, id: u32) -> AgentId {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.agents.get(index))
            .unwrap_or_else(|| panic!("NavWorld has no agent {id}"))
            .0
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
    /// consecutive. The speed limit is in game units per second and the linear drag in 1/s.
    ///
    /// # Panics
    /// If `body` is not a root body of this world's physics.
    pub fn add_agent(&mut self, body: &RigidBody, max_speed: f64, linear_drag: f64) -> u32 {
        let handles = self.handles_of(body, "agent");
        let id = self.world.add_agent(Profile {
            max_speed,
            linear_drag,
        });
        self.agents.push((id, handles));
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
    /// of it (game units).
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn set_goal(&mut self, id: u32, target: &Position, arrival_radius: f64) {
        let id = self.agent(id);
        self.world.set_goal(
            id,
            Some(Goal {
                target: target.v,
                arrival_radius,
            }),
        );
    }

    /// Leaves the agent without input.
    ///
    /// # Panics
    /// If `id` isn't an agent.
    pub fn clear_goal(&mut self, id: u32) {
        let id = self.agent(id);
        self.world.set_goal(id, None);
    }

    /// Sets how agents avoid obstacles: the look-ahead in seconds, the clearance in agent radii
    /// and the aim factor (see `doc/engine/nav.md`).
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
            for (index, (id, handles)) in self.agents.iter().enumerate() {
                let state = Self::state_of(&physics, handles, &format!("agent {index}"));
                self.world.set_body(*id, nav_body(&state));
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

    /// Copies, for each agent in order, whether it was within its goal's arrival radius at the
    /// last step.
    ///
    /// # Panics
    /// If `out` isn't exactly one value per agent long.
    pub fn copy_arrived(&self, out: &mut [bool]) {
        assert_eq!(
            out.len(),
            self.agents.len(),
            "NavWorld::copy_arrived needs agent_count values"
        );
        for (flag, (id, _)) in out.iter_mut().zip(&self.agents) {
            *flag = self.world.arrived(*id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::NavAxis;

    const SHIP_RADIUS: f32 = 1.0;
    const MAX_SPEED: f64 = 100.0;
    const LINEAR_DRAG: f64 = 2.0;
    const ARRIVAL_RADIUS: f64 = 5.0;
    const GOAL_DISTANCE: f64 = 500.0;
    const ROCK_RADIUS: f32 = 30.0;
    const ROCK_DISTANCE: f64 = 200.0;
    const LONG_GOAL_DISTANCE: f64 = 1000.0;

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

    #[test]
    fn an_agent_is_steered_to_its_goal() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let id = nav.add_agent(&ship, MAX_SPEED, LINEAR_DRAG);
        nav.set_goal(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.step();
        assert_eq!(axis(&axes_of(&nav), 0, NavAxis::Yaw), 1.0);
    }

    #[test]
    fn ids_are_consecutive_and_each_agent_has_its_own_goal() {
        let mut physics = Physics::new();
        let first = ship_in(&mut physics);
        let second = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        assert_eq!(nav.add_agent(&first, MAX_SPEED, LINEAR_DRAG), 0);
        assert_eq!(nav.add_agent(&second, MAX_SPEED, LINEAR_DRAG), 1);
        assert_eq!(nav.agent_count(), 2);
        nav.set_goal(0, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.set_goal(1, &goal_at(-GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
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
        let id = nav.add_agent(&ship, MAX_SPEED, LINEAR_DRAG);
        nav.set_goal(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
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
        let id = nav.add_agent(&ship, MAX_SPEED, LINEAR_DRAG);
        nav.set_goal(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
        nav.clear_goal(id);
        nav.step();
        assert!(axes_of(&nav).iter().all(|&value| value == 0.0));
    }

    #[test]
    fn arrival_is_reported_per_agent() {
        let mut physics = Physics::new();
        let near = ship_in(&mut physics);
        let far = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        let near_id = nav.add_agent(&near, MAX_SPEED, LINEAR_DRAG);
        let far_id = nav.add_agent(&far, MAX_SPEED, LINEAR_DRAG);
        nav.set_goal(
            near_id,
            &goal_at(0.0, -ARRIVAL_RADIUS / 2.0),
            ARRIVAL_RADIUS,
        );
        nav.set_goal(far_id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
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
        let id = nav.add_agent(&ship, MAX_SPEED, LINEAR_DRAG);
        nav.set_goal(id, &goal_at(0.0, -LONG_GOAL_DISTANCE), ARRIVAL_RADIUS);
        nav.step();
        let straight = axes_of(&nav);

        nav.add_obstacle(&rock);
        nav.step();
        assert_ne!(axes_of(&nav), straight);
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
        nav.add_agent(&stranger, MAX_SPEED, LINEAR_DRAG);
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
        nav.set_goal(3, &Position::ZERO, ARRIVAL_RADIUS);
    }

    #[test]
    #[should_panic(expected = "needs agent_count · axis_count values")]
    fn a_wrong_axes_buffer_is_refused() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        nav.add_agent(&ship, MAX_SPEED, LINEAR_DRAG);
        nav.copy_axes(&mut [0.0; AXIS_COUNT + 1]);
    }

    #[test]
    #[should_panic(expected = "needs agent_count values")]
    fn a_wrong_arrived_buffer_is_refused() {
        let mut physics = Physics::new();
        let ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        nav.add_agent(&ship, MAX_SPEED, LINEAR_DRAG);
        nav.copy_arrived(&mut [false; 2]);
    }

    #[test]
    #[should_panic(expected = "agent 0: its body has left the physics world")]
    fn a_removed_agent_body_stops_the_step() {
        let mut physics = Physics::new();
        let mut ship = ship_in(&mut physics);
        let mut nav = NavWorld::new(&physics);
        nav.add_agent(&ship, MAX_SPEED, LINEAR_DRAG);
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
