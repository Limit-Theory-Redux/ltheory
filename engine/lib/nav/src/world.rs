use crate::agent::{AXIS_COUNT, Agent, AgentId, Body, Goal, Profile};
use crate::avoidance::{AvoidanceTuning, avoid};
use crate::behaviour::arrive;
use crate::controller::control;
use crate::obstacle::Obstacle;

/// The navigation state of one simulated space: its agents, the obstacles around them, and the
/// control axes the last [`step`](World::step) gave each agent.
///
/// The caller snapshots bodies into it before a step ([`set_body`](World::set_body),
/// [`set_obstacles`](World::set_obstacles)) and reads the axes after it.
#[derive(Debug, Default)]
pub struct World {
    agents: Vec<Agent>,
    obstacles: Vec<Obstacle>,
    /// The axes of each agent, in agent order.
    axes: Vec<[f64; AXIS_COUNT]>,
    arrived: Vec<bool>,
    tuning: AvoidanceTuning,
}

impl World {
    /// Creates an empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an agent that is at rest at the origin, facing along -z, with no goal.
    pub fn add_agent(&mut self, profile: Profile) -> AgentId {
        let id = AgentId::new(self.agents.len());
        self.agents.push(Agent {
            body: Body::AT_REST,
            profile,
            goal: None,
        });
        self.axes.push([0.0; AXIS_COUNT]);
        self.arrived.push(false);
        id
    }

    /// The number of agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Sets the body an agent is steered from. Panics if `id` isn't an agent of this world.
    pub fn set_body(&mut self, id: AgentId, body: Body) {
        self.agents[id.index()].body = body;
    }

    /// Sets an agent's goal; `None` leaves it without input. Panics if `id` isn't an agent of
    /// this world.
    pub fn set_goal(&mut self, id: AgentId, goal: Option<Goal>) {
        self.agents[id.index()].goal = goal;
    }

    /// Replaces the obstacles.
    pub fn set_obstacles(&mut self, obstacles: &[Obstacle]) {
        self.obstacles.clear();
        self.obstacles.extend_from_slice(obstacles);
    }

    /// Replaces the avoidance tuning.
    pub fn set_tuning(&mut self, tuning: AvoidanceTuning) {
        self.tuning = tuning;
    }

    /// Computes every agent's control axes and arrival from the current bodies and obstacles.
    pub fn step(&mut self) {
        let slots = self.axes.iter_mut().zip(&mut self.arrived);
        for (agent, (axes, arrived)) in self.agents.iter().zip(slots) {
            let Some(goal) = &agent.goal else {
                *axes = [0.0; AXIS_COUNT];
                *arrived = false;
                continue;
            };
            let toward_goal = arrive(&agent.body, &agent.profile, goal);
            let wanted = avoid(
                &agent.body,
                agent.profile.max_speed,
                toward_goal,
                &self.obstacles,
                &self.tuning,
            );
            *axes = control(&agent.body, &agent.profile, &wanted);
            *arrived = agent.body.position.distance(goal.target) <= goal.arrival_radius;
        }
    }

    /// The control axes of every agent, `AXIS_COUNT` consecutive values each, in agent order and
    /// in the order of [`Axis`](crate::Axis).
    pub fn axes(&self) -> &[f64] {
        self.axes.as_flattened()
    }

    /// Whether the agent was within its goal's arrival radius at the last step. Panics if `id`
    /// isn't an agent of this world.
    pub fn arrived(&self, id: AgentId) -> bool {
        self.arrived[id.index()]
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use super::*;
    use crate::agent::Axis;

    const PROFILE: Profile = Profile {
        max_speed: 100.0,
        linear_drag: 2.0,
    };

    fn goal(target: DVec3) -> Goal {
        Goal {
            target,
            arrival_radius: 5.0,
        }
    }

    #[test]
    fn new_world_has_no_agents() {
        let world = World::new();
        assert_eq!(world.agent_count(), 0);
        assert!(world.axes().is_empty());
    }

    #[test]
    fn agents_get_consecutive_ids_and_axis_slots() {
        let mut world = World::new();
        let first = world.add_agent(PROFILE);
        let second = world.add_agent(PROFILE);
        assert_eq!((first.index(), second.index()), (0, 1));
        assert_eq!(world.axes().len(), 2 * AXIS_COUNT);
    }

    #[test]
    fn an_agent_without_a_goal_gets_no_input() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.step();
        assert_eq!(world.axes(), [0.0; AXIS_COUNT]);
        assert!(!world.arrived(id));
    }

    #[test]
    fn each_agent_is_steered_to_its_own_goal() {
        let mut world = World::new();
        let to_right = world.add_agent(PROFILE);
        let to_left = world.add_agent(PROFILE);
        world.set_goal(to_right, Some(goal(DVec3::new(500.0, 0.0, 0.0))));
        world.set_goal(to_left, Some(goal(DVec3::new(-500.0, 0.0, 0.0))));
        world.step();

        let yaw_of = |id: AgentId| world.axes()[id.index() * AXIS_COUNT + Axis::Yaw as usize];
        assert_eq!(yaw_of(to_right), 1.0);
        assert_eq!(yaw_of(to_left), -1.0);
    }

    #[test]
    fn arrival_is_reported_within_the_goal_radius() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_goal(id, Some(goal(DVec3::new(0.0, 0.0, -4.0))));
        world.step();
        assert!(world.arrived(id));

        world.set_goal(id, Some(goal(DVec3::new(0.0, 0.0, -6.0))));
        world.step();
        assert!(!world.arrived(id));
    }

    #[test]
    fn an_obstacle_on_the_course_changes_the_axes() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_goal(id, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        world.step();
        let straight = world.axes().to_vec();

        world.set_obstacles(&[Obstacle {
            position: DVec3::new(0.0, 0.0, -200.0),
            velocity: DVec3::ZERO,
            radius: 30.0,
        }]);
        world.step();
        assert_ne!(world.axes(), straight);
        world.set_obstacles(&[]);
        world.step();
        assert_eq!(world.axes(), straight);
    }
}
