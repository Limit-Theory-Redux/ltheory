use glam::DVec3;

use crate::agent::{AT_REST_RADII_PER_S, AXIS_COUNT, Agent, AgentId, Body, Goal, Profile};
use crate::avoidance::{AvoidanceTuning, Peer, Peers, Sphere, avoid};
use crate::behaviour::{Behaviour, rest_facing, steer};
use crate::controller::control;
use crate::obstacle::{Obstacle, ObstacleKey};

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
    /// `scene` as seen from the frame of the agent being steered, reused between agents and steps.
    shifted: Vec<Obstacle>,
    /// The per-obstacle data of the agent being steered, reused between agents and steps.
    spheres: Vec<Sphere>,
    /// What each agent is to the others, in agent order, reused between steps.
    peers: Vec<Peer>,
    /// The axes of each agent, in agent order.
    axes: Vec<[f64; AXIS_COUNT]>,
    arrived: Vec<bool>,
    /// `arrived` as it was before the step in progress, reused between steps.
    was_arrived: Vec<bool>,
    tuning: AvoidanceTuning,
}

/// How far from the hull of the agent that holds its goal an agent still counts as having reached
/// it, in the agent's own radii: the room the arrival floor leaves around a goal.
const HELD_GOAL_GAP_RADII: f64 = 3.0;

/// Whether `body` has reached `goal`: it is within the goal's arrival radius, or another agent
/// that had arrived and is at rest holds the goal (the goal lies within the two hulls' reach) and
/// `body` is next to it. Goals closer together than hulls can't all be sat on, so the agents that
/// come last would otherwise circle those that settled and never arrive.
///
/// `agents` holds one sphere per agent, in agent order, `was_arrived` the arrival flags of the
/// step before, and `own` is the index of `body`'s agent.
fn has_reached(
    body: &Body,
    goal: Goal,
    agents: &[Obstacle],
    was_arrived: &[bool],
    own: usize,
) -> bool {
    if body.position.distance(goal.target) <= goal.effective_arrival_radius(body.radius) {
        return true;
    }
    agents
        .iter()
        .zip(was_arrived)
        .enumerate()
        .any(|(index, (other, &arrived))| {
            let hull = other.radius + body.radius;
            index != own
                && arrived
                && other.velocity.length() <= other.radius * AT_REST_RADII_PER_S
                && other.position.distance(goal.target) <= hull
                && body.position.distance(other.position)
                    <= hull + HELD_GOAL_GAP_RADII * body.radius
        })
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
            frame_velocity: DVec3::ZERO,
            behaviour: None,
            tracked: None,
            commitment: None,
        });
        self.axes.push([0.0; AXIS_COUNT]);
        self.arrived.push(false);
        self.was_arrived.push(false);
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

    /// Sets the velocity the agent's surroundings carry it at (zero by default), in length units
    /// per second: the part of its velocity that drag doesn't act on. The agent is steered, and
    /// judged at rest, relative to it, and the controller's drag, top speed and braking follow.
    /// It is a translation only: the frame's acceleration and turning are ignored. Set it again
    /// each step it changes. Panics if `id` isn't an agent of this world.
    pub fn set_frame_velocity(&mut self, id: AgentId, velocity: DVec3) {
        self.agents[id.index()].frame_velocity = velocity;
    }

    /// Sets what an agent does; `None` leaves it without input. Target bodies inside the
    /// behaviour are snapshots, so set it again each step for a target that moves. Panics if `id`
    /// isn't an agent of this world.
    pub fn set_behaviour(&mut self, id: AgentId, behaviour: Option<Behaviour>) {
        self.agents[id.index()].behaviour = behaviour;
    }

    /// Names the obstacle the agent's behaviour tracks (a pursued or followed body), or `None`.
    /// The agent leaves it out of its scene. If it is an agent, that agent leaves this one out
    /// too while this one lies behind it (relative to its wanted course), and avoids it with
    /// the half share of any steering agent otherwise. The key stays until it is set again, so
    /// the caller clears it when it sets a behaviour that tracks nothing. Panics if `id` isn't
    /// an agent of this world.
    pub fn set_tracked(&mut self, id: AgentId, target: Option<ObstacleKey>) {
        self.agents[id.index()].tracked = target;
    }

    /// Replaces the obstacles.
    pub fn set_obstacles(&mut self, obstacles: &[Obstacle]) {
        self.obstacles.clear();
        self.obstacles.extend_from_slice(obstacles);
    }

    /// The avoidance tuning in force.
    pub fn tuning(&self) -> &AvoidanceTuning {
        &self.tuning
    }

    /// Replaces the avoidance tuning.
    pub fn set_tuning(&mut self, tuning: AvoidanceTuning) {
        self.tuning = tuning;
    }

    /// Computes every agent's control axes and arrival from the current bodies and obstacles.
    pub fn step(&mut self) {
        // The set obstacles, then one sphere per agent, each keeping its velocity. An agent
        // leaves its own sphere out by swapping it behind the end of the slice it avoids.
        self.scene.clear();
        self.scene.extend_from_slice(&self.obstacles);
        let agent_spheres = self
            .agents
            .iter()
            .enumerate()
            .map(|(index, agent)| Obstacle {
                key: ObstacleKey::Agent(AgentId::new(index)),
                position: agent.body.position,
                velocity: agent.body.velocity,
                radius: agent.body.radius,
            });
        self.scene.extend(agent_spheres);
        let last = self.scene.len().saturating_sub(1);
        // Whether an agent holds its goal is judged on the arrivals of the step before, so the
        // result doesn't depend on the order the agents are stepped in.
        self.was_arrived.clear();
        self.was_arrived.extend_from_slice(&self.arrived);
        self.peers.clear();
        self.peers.extend(self.agents.iter().map(|agent| Peer {
            steering: agent.behaviour.is_some(),
            tracks: agent.tracked,
        }));
        let slots = self.axes.iter_mut().zip(&mut self.arrived);
        for (index, (agent, (axes, arrived))) in self.agents.iter_mut().zip(slots).enumerate() {
            let own = self.obstacles.len() + index;
            let Some(behaviour) = agent.behaviour else {
                *axes = [0.0; AXIS_COUNT];
                *arrived = false;
                continue;
            };
            // An agent in a moving frame works in a still one: every velocity it looks at loses
            // the frame's. The shifted scene is a copy, because shifting in place and back would
            // round.
            let frame = agent.frame_velocity;
            let scene = if frame == DVec3::ZERO {
                &mut self.scene
            } else {
                self.shifted.clear();
                self.shifted
                    .extend(self.scene.iter().map(|sphere| sphere.seen_from(frame)));
                &mut self.shifted
            };
            let body = agent.body.seen_from(frame);
            let behaviour = behaviour.seen_from(frame);
            let (toward_goal, goal) = steer(&behaviour, &body, &agent.profile);
            scene.swap(own, last);
            let wanted = avoid(
                &body,
                &agent.profile,
                toward_goal,
                &scene[..last],
                &self.tuning,
                Some(goal),
                Peers {
                    own: AgentId::new(index),
                    tracked: agent.tracked,
                    all: &self.peers,
                },
                &mut agent.commitment,
                &mut self.spheres,
            );
            scene.swap(own, last);
            let facing = rest_facing(&behaviour, &body, &goal);
            *axes = control(&body, &agent.profile, &wanted, facing);
            *arrived = has_reached(
                &body,
                goal,
                &scene[self.obstacles.len()..],
                &self.was_arrived,
                index,
            );
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
    use crate::behaviour::FrameOffset;

    const PROFILE: Profile = Profile::TEST;

    fn goal(target: DVec3) -> Behaviour {
        Behaviour::Arrive(Goal::fixed(target, 5.0))
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

    /// Two agents of radius 10: the first sits at rest on its own goal, the second heads for that
    /// same point from `distance` away, with `holder_speed` the first's speed along z. Returns
    /// whether the second has reached its goal after two steps (the first arrives in the first).
    fn second_agent_reaches_a_held_goal(
        distance: f64,
        holder_speed: f64,
        holder_arrives: bool,
    ) -> bool {
        const RADIUS: f64 = 10.0;
        let goal_point = DVec3::ZERO;
        let mut world = World::new();
        let holder = world.add_agent(PROFILE);
        let waiting = world.add_agent(PROFILE);
        world.set_body(
            holder,
            Body {
                velocity: DVec3::new(0.0, 0.0, holder_speed),
                radius: RADIUS,
                ..Body::AT_REST
            },
        );
        world.set_body(
            waiting,
            Body {
                position: DVec3::new(distance, 0.0, 0.0),
                radius: RADIUS,
                ..Body::AT_REST
            },
        );
        let holder_goal = if holder_arrives {
            goal_point
        } else {
            DVec3::new(0.0, 5000.0, 0.0)
        };
        world.set_behaviour(holder, Some(goal(holder_goal)));
        world.set_behaviour(waiting, Some(goal(goal_point)));
        world.step();
        world.step();
        world.arrived(waiting)
    }

    #[test]
    fn an_agent_next_to_the_resting_agent_that_holds_its_goal_has_reached_it() {
        // The arrival circle is held to 2 radii = 20, so 45 is outside it but within the hulls
        // (20) plus 3 radii (30) of the holder.
        assert!(second_agent_reaches_a_held_goal(45.0, 0.0, true));
    }

    #[test]
    fn a_held_goal_is_not_reached_from_beyond_the_hulls_gap() {
        assert!(!second_agent_reaches_a_held_goal(60.0, 0.0, true));
    }

    #[test]
    fn a_goal_is_not_held_by_an_agent_that_moves_or_has_not_arrived() {
        assert!(!second_agent_reaches_a_held_goal(45.0, 50.0, true));
        assert!(!second_agent_reaches_a_held_goal(45.0, 0.0, false));
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
            key: ObstacleKey::Body(0),
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

    /// How the second agent of `flier_axes` relates to the first.
    #[derive(Clone, Copy, Default)]
    struct Blocker {
        /// It has a behaviour: it holds its place with a goal there.
        steers: bool,
        /// It tracks the flier, so it leaves the flier out of its own scene. The flier still
        /// avoids it while it lies ahead of the flier.
        tracks_flier: bool,
        /// The flier tracks it, and leaves it out of its scene.
        tracked_by_flier: bool,
    }

    /// The control axes of an agent flying at 30 toward a goal 1000 ahead, with a resting agent of
    /// radius 30 on its course 200 ahead that is related to it as `blocker` says. The flier moves
    /// so that the mean velocity of the pair differs from its own.
    fn flier_axes(blocker: Option<Blocker>) -> Vec<f64> {
        let mut world = World::new();
        let flier = world.add_agent(PROFILE);
        world.set_body(
            flier,
            Body {
                velocity: DVec3::NEG_Z * 30.0,
                ..Body::AT_REST
            },
        );
        world.set_behaviour(flier, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        if let Some(blocker) = blocker {
            let position = DVec3::new(0.0, 0.0, -200.0);
            let other = world.add_agent(PROFILE);
            world.set_body(
                other,
                Body {
                    position,
                    radius: 30.0,
                    ..Body::AT_REST
                },
            );
            if blocker.steers {
                world.set_behaviour(other, Some(goal(position)));
            }
            if blocker.tracks_flier {
                world.set_tracked(other, Some(ObstacleKey::Agent(flier)));
            }
            if blocker.tracked_by_flier {
                world.set_tracked(flier, Some(ObstacleKey::Agent(other)));
            }
        }
        world.step();
        world.axes()[..AXIS_COUNT].to_vec()
    }

    #[test]
    fn an_agent_leaves_the_agent_it_tracks_out_of_its_scene() {
        let alone = flier_axes(None);
        let tracked = Blocker {
            tracked_by_flier: true,
            ..Blocker::default()
        };
        assert_eq!(flier_axes(Some(tracked)), alone);
        assert_ne!(flier_axes(Some(Blocker::default())), alone);
    }

    #[test]
    fn an_agent_takes_half_of_the_dodge_of_an_agent_that_steers() {
        let resting = flier_axes(Some(Blocker::default()));
        let steering = flier_axes(Some(Blocker {
            steers: true,
            ..Blocker::default()
        }));
        assert_ne!(steering, flier_axes(None), "the blocker is still avoided");
        assert_ne!(steering, resting, "the dodge is shared");
    }

    #[test]
    fn the_agent_that_is_tracked_takes_half_of_the_dodge_of_a_tracker_ahead_of_it() {
        let steering = Blocker {
            steers: true,
            ..Blocker::default()
        };
        let tracker = Blocker {
            tracks_flier: true,
            ..steering
        };
        assert_eq!(flier_axes(Some(tracker)), flier_axes(Some(steering)));
        assert_ne!(
            flier_axes(Some(tracker)),
            flier_axes(Some(Blocker::default())),
            "the tracker is not a resting sphere"
        );
    }

    /// A fixed pseudo-random sequence in `[-1, 1)`.
    struct Sequence(u64);

    impl Sequence {
        fn next(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
        }

        fn vector(&mut self, scale: f64) -> DVec3 {
            DVec3::new(self.next(), self.next(), self.next()) * scale
        }
    }

    const FRAME_AGENTS: usize = 5;
    const FRAME_OBSTACLES: usize = 4;
    const FRAME_SPREAD: f64 = 200.0;
    const FRAME_SPEED: f64 = 60.0;
    /// A velocity far above the test profile's top speed, which is 100.
    const CARRY_SPEED: f64 = 900.0;
    const AXES_TOLERANCE: f64 = 1e-6;

    /// A world of `FRAME_AGENTS` agents flying every kind of behaviour among obstacles, drawn from
    /// `seed`, in which every velocity (bodies, goals, targets, obstacles) has `carry` added and
    /// every agent's frame velocity is `carry`. Pursuers and followers track their target, which
    /// is among the obstacles. Without `with_obstacles` the world has none of them.
    fn carried_world(seed: u64, carry: DVec3, with_obstacles: bool) -> World {
        let mut rng = Sequence(seed);
        let mut world = World::new();
        let mut obstacles = Vec::new();
        for index in 0..FRAME_OBSTACLES {
            obstacles.push(Obstacle {
                key: ObstacleKey::Body(index as u64),
                position: rng.vector(FRAME_SPREAD),
                velocity: rng.vector(FRAME_SPEED) + carry,
                radius: 30.0 + 30.0 * rng.next().abs(),
            });
        }
        for index in 0..FRAME_AGENTS {
            let id = world.add_agent(PROFILE);
            let body = Body {
                position: rng.vector(FRAME_SPREAD),
                velocity: rng.vector(FRAME_SPEED) + carry,
                radius: 1.0 + 4.0 * rng.next().abs(),
                ..Body::AT_REST
            };
            world.set_body(id, body);
            world.set_frame_velocity(id, carry);
            let target = Body {
                position: rng.vector(FRAME_SPREAD),
                velocity: rng.vector(FRAME_SPEED) + carry,
                radius: 5.0,
                ..Body::AT_REST
            };
            let arrival_radius = 10.0;
            let behaviour = match index % 4 {
                0 => Behaviour::Arrive(Goal {
                    target: target.position,
                    velocity: target.velocity,
                    arrival_radius,
                }),
                1 => Behaviour::Pursue {
                    target,
                    stand_off: 30.0,
                    arrival_radius,
                },
                2 => Behaviour::Follow {
                    leader: target,
                    offset: FrameOffset {
                        right: 20.0,
                        up: 0.0,
                        forward: -40.0,
                    },
                    arrival_radius,
                },
                _ => Behaviour::Orbit {
                    centre: target,
                    radius: 150.0,
                    normal: DVec3::Y,
                    arrival_radius,
                },
            };
            world.set_behaviour(id, Some(behaviour));
            if matches!(
                behaviour,
                Behaviour::Pursue { .. } | Behaviour::Follow { .. }
            ) {
                let key = ObstacleKey::Body((FRAME_OBSTACLES + index) as u64);
                obstacles.push(Obstacle {
                    key,
                    position: target.position,
                    velocity: target.velocity,
                    radius: target.radius,
                });
                world.set_tracked(id, Some(key));
            }
        }
        if with_obstacles {
            world.set_obstacles(&obstacles);
        }
        world
    }

    fn assert_same_axes(left: &World, right: &World, seed: u64) {
        for (index, (a, b)) in left.axes().iter().zip(right.axes()).enumerate() {
            assert!(
                (a - b).abs() <= AXES_TOLERANCE,
                "seed {seed}, axis {index}: {a} against {b}"
            );
        }
        for agent in 0..FRAME_AGENTS {
            let id = AgentId::new(agent);
            assert_eq!(left.arrived(id), right.arrived(id), "seed {seed}");
        }
    }

    #[test]
    fn a_world_carried_with_its_frame_steers_as_a_still_world() {
        const SEEDS: u64 = 200;
        let mut avoided = 0;
        for seed in 1..=SEEDS {
            let carry = Sequence(seed ^ 0xC0FFEE).vector(CARRY_SPEED);
            let mut still = carried_world(seed, DVec3::ZERO, true);
            let mut moving = carried_world(seed, carry, true);
            still.step();
            moving.step();
            assert_same_axes(&still, &moving, seed);

            let mut open = carried_world(seed, DVec3::ZERO, false);
            open.step();
            avoided += usize::from(open.axes() != still.axes());
        }
        // The obstacles and the other agents must change what the agents do in most worlds,
        // or the equality above says nothing about the shifted scene.
        assert!(avoided > SEEDS as usize / 2, "avoiding in {avoided} worlds");
    }

    #[test]
    fn without_its_frame_a_carried_world_is_steered_differently() {
        let carry = DVec3::new(CARRY_SPEED, 0.0, 0.0);
        let mut still = carried_world(1, DVec3::ZERO, true);
        let mut unaware = carried_world(1, carry, true);
        for index in 0..FRAME_AGENTS {
            unaware.set_frame_velocity(AgentId::new(index), DVec3::ZERO);
        }
        still.step();
        unaware.step();
        assert_ne!(still.axes(), unaware.axes());
    }

    #[test]
    fn an_agent_at_rest_in_its_frame_with_its_goal_on_it_gets_no_thrust() {
        let carry = DVec3::new(0.0, 0.0, -CARRY_SPEED);
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_body(
            id,
            Body {
                velocity: carry,
                ..Body::AT_REST
            },
        );
        world.set_frame_velocity(id, carry);
        world.set_behaviour(
            id,
            Some(Behaviour::Arrive(Goal {
                target: DVec3::ZERO,
                velocity: carry,
                arrival_radius: 5.0,
            })),
        );
        world.step();
        assert!(world.arrived(id));
        let thrust = &world.axes()[Axis::ThrustForward as usize..AXIS_COUNT];
        assert_eq!(thrust, [0.0; 3]);
    }
}
