use crate::agent::{AXIS_COUNT, Agent, AgentId, Body, Profile};
use crate::avoidance::{AvoidanceTuning, Sphere, avoid};
use crate::behaviour::{Behaviour, steer};
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
    /// The obstacles and the agents' own spheres of the step in progress, reused between steps.
    scene: Vec<Obstacle>,
    /// The per-obstacle data of the agent being steered, reused between agents and steps.
    spheres: Vec<Sphere>,
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

    /// Adds an agent that is at rest at the origin, facing along -z, with no behaviour.
    pub fn add_agent(&mut self, profile: Profile) -> AgentId {
        let id = AgentId::new(self.agents.len());
        self.agents.push(Agent {
            body: Body::AT_REST,
            profile,
            behaviour: None,
            commitment: None,
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

    /// Sets what limits an agent's motion; the caller refreshes it when the body's mass changes.
    /// Panics if `id` isn't an agent of this world.
    pub fn set_profile(&mut self, id: AgentId, profile: Profile) {
        self.agents[id.index()].profile = profile;
    }

    /// Sets what an agent does; `None` leaves it without input. Target bodies inside the
    /// behaviour are snapshots, so set it again each step for a target that moves. Panics if `id`
    /// isn't an agent of this world.
    pub fn set_behaviour(&mut self, id: AgentId, behaviour: Option<Behaviour>) {
        self.agents[id.index()].behaviour = behaviour;
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
        // The set obstacles, then one sphere per agent, each keeping its velocity. An agent
        // leaves its own sphere out by swapping it behind the end of the slice it avoids, so the
        // slice is stable between steps and the avoidance memory keeps its indices.
        self.scene.clear();
        self.scene.extend_from_slice(&self.obstacles);
        self.scene.extend(self.agents.iter().map(|agent| Obstacle {
            position: agent.body.position,
            velocity: agent.body.velocity,
            radius: agent.body.radius,
        }));
        let last = self.scene.len().saturating_sub(1);
        let slots = self.axes.iter_mut().zip(&mut self.arrived);
        for (index, (agent, (axes, arrived))) in self.agents.iter_mut().zip(slots).enumerate() {
            let own = self.obstacles.len() + index;
            let Some(behaviour) = agent.behaviour else {
                *axes = [0.0; AXIS_COUNT];
                *arrived = false;
                continue;
            };
            let (toward_goal, goal) = steer(&behaviour, &agent.body, &agent.profile);
            self.scene.swap(own, last);
            let wanted = avoid(
                &agent.body,
                &agent.profile,
                toward_goal,
                &self.scene[..last],
                &self.tuning,
                Some(goal),
                &mut agent.commitment,
                &mut self.spheres,
            );
            self.scene.swap(own, last);
            *axes = control(&agent.body, &agent.profile, &wanted);
            *arrived = agent.body.position.distance(goal.target)
                <= goal.effective_arrival_radius(agent.body.radius);
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
    use crate::agent::{Axis, Goal};

    const PROFILE: Profile = Profile::TEST;

    fn goal(target: DVec3) -> Behaviour {
        Behaviour::Arrive(Goal {
            target,
            arrival_radius: 5.0,
        })
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
        world.set_behaviour(to_right, Some(goal(DVec3::new(500.0, 0.0, 0.0))));
        world.set_behaviour(to_left, Some(goal(DVec3::new(-500.0, 0.0, 0.0))));
        world.step();

        let yaw_of = |id: AgentId| world.axes()[id.index() * AXIS_COUNT + Axis::Yaw as usize];
        assert_eq!(yaw_of(to_right), 1.0);
        assert_eq!(yaw_of(to_left), -1.0);
    }

    #[test]
    fn arrival_is_reported_within_the_goal_radius() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -4.0))));
        world.step();
        assert!(world.arrived(id));

        world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -6.0))));
        world.step();
        assert!(!world.arrived(id));
    }

    #[test]
    fn arrival_is_held_to_at_least_two_agent_radii() {
        let radius = 10.0;
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_body(
            id,
            Body {
                radius,
                ..Body::AT_REST
            },
        );
        // The goal's own radius is 5, less than the agent's two radii.
        let at = |distance: f64| goal(DVec3::new(0.0, 0.0, -distance));
        world.set_behaviour(id, Some(at(2.0 * radius - 0.5)));
        world.step();
        assert!(world.arrived(id));

        world.set_behaviour(id, Some(at(2.0 * radius + 0.5)));
        world.step();
        assert!(!world.arrived(id));
    }

    #[test]
    fn an_agent_avoids_the_other_agents_but_never_itself() {
        let blocker_body = Body {
            position: DVec3::new(0.0, 0.0, -200.0),
            radius: 30.0,
            ..Body::AT_REST
        };
        // The agent that flies is first in one world and last in the other.
        for flier_first in [true, false] {
            let mut world = World::new();
            let (flier, blocker) = if flier_first {
                let flier = world.add_agent(PROFILE);
                (flier, world.add_agent(PROFILE))
            } else {
                let blocker = world.add_agent(PROFILE);
                (world.add_agent(PROFILE), blocker)
            };
            world.set_behaviour(flier, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
            let axes_of = |world: &World, id: AgentId| {
                let start = id.index() * AXIS_COUNT;
                world.axes()[start..start + AXIS_COUNT].to_vec()
            };
            world.step();
            let alone = axes_of(&world, flier);

            world.set_body(blocker, blocker_body);
            world.step();
            assert_ne!(axes_of(&world, flier), alone, "flier first: {flier_first}");
            assert_eq!(axes_of(&world, blocker), [0.0; AXIS_COUNT]);
        }
    }

    #[test]
    fn an_obstacle_on_the_course_changes_the_axes() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
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
