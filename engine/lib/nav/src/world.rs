use glam::DVec3;

use crate::agent::{AT_REST_RADII_PER_S, AXIS_COUNT, Agent, AgentId, Body, Goal, Profile, Speed};
use crate::avoidance::{AvoidanceTuning, Peer, Peers, Sphere, avoid};
use crate::behaviour::{Behaviour, Wanted, facing_toward, rest_facing, steer};
use crate::controller::control;
use crate::obstacle::{Obstacle, ObstacleKey};

/// The navigation state of one simulated space: its agents, the obstacles around them, and the
/// control axes the last [`step`](World::step) gave each agent.
///
/// The caller snapshots bodies into it before a step ([`set_body`](World::set_body),
/// [`set_obstacles`](World::set_obstacles)) and reads the axes after it.
///
/// An agent's id is its slot: [`remove_agent`](World::remove_agent) frees it and
/// [`add_agent`](World::add_agent) reuses the lowest free one. The per-agent buffers have one
/// entry per slot, and a free slot's entry is inert (zero axes, not arrived).
#[derive(Debug, Default)]
pub struct World {
    /// One entry per slot; `None` for a free one.
    agents: Vec<Option<Agent>>,
    obstacles: Vec<Obstacle>,
    /// The obstacles and the live agents' own spheres of the step in progress, reused between
    /// steps.
    scene: Vec<Obstacle>,
    /// `scene` as seen from the frame of the agent being steered, reused between agents and steps.
    shifted: Vec<Obstacle>,
    /// The per-obstacle data of the agent being steered, reused between agents and steps.
    spheres: Vec<Sphere>,
    /// What each slot is to the others, in slot order (a free slot is neither steering nor
    /// tracking), reused between steps.
    peers: Vec<Peer>,
    /// The axes of each slot.
    axes: Vec<[f64; AXIS_COUNT]>,
    arrived: Vec<bool>,
    /// `arrived` of the live agents as it was before the step in progress, in slot order with the
    /// free slots left out, reused between steps.
    was_arrived: Vec<bool>,
    tuning: AvoidanceTuning,
    /// The seconds the steps have passed so far.
    clock: f64,
}

/// How far from the hull of the agent that holds its goal an agent still counts as having reached
/// it, in the agent's own radii: the room the arrival floor leaves around a goal.
const HELD_GOAL_GAP_RADII: f64 = 3.0;

/// The step of the golden ratio: `fract(slot · GOLDEN_STEP)` spreads any run of slots evenly over
/// `[0, 1)`, whatever the number of agents, so agents with one avoidance period don't run in the
/// same step.
const GOLDEN_STEP: f64 = 0.618_033_988_749_894_9;

/// The share of an avoidance period that passes before the agent in `slot` first runs avoidance.
fn stagger_of(slot: usize) -> f64 {
    (slot as f64 * GOLDEN_STEP).fract()
}

/// What `World::step` shares between the agents of one step.
struct StepScene<'a> {
    /// The obstacles and one sphere per live agent.
    scene: &'a mut Vec<Obstacle>,
    /// `scene` as seen from the frame of the agent being steered.
    shifted: &'a mut Vec<Obstacle>,
    spheres: &'a mut Vec<Sphere>,
    peers: &'a [Peer],
    tuning: &'a AvoidanceTuning,
}

impl StepScene<'_> {
    /// The course `agent`, the one in `index`, wants after avoiding the scene: its sphere is the
    /// one at `own` and the scene ends at `last`. An agent in a moving frame works in a still one:
    /// every velocity it looks at loses the frame's. The shifted scene is a copy, because
    /// shifting in place and back would round.
    fn avoid(
        &mut self,
        agent: &mut Agent,
        index: usize,
        (own, last): (usize, usize),
        (body, toward_goal, goal): (&Body, Wanted, Goal),
    ) -> Wanted {
        let frame = agent.frame_velocity;
        let scene = if frame == DVec3::ZERO {
            &mut *self.scene
        } else {
            self.shifted.clear();
            self.shifted
                .extend(self.scene.iter().map(|sphere| sphere.seen_from(frame)));
            &mut *self.shifted
        };
        scene.swap(own, last);
        let wanted = avoid(
            body,
            &agent.profile,
            toward_goal,
            &scene[..last],
            self.tuning,
            Some(goal),
            Peers {
                own: AgentId::new(index),
                tracked: agent.tracked,
                all: self.peers,
            },
            &mut agent.commitment,
            self.spheres,
        );
        scene.swap(own, last);
        wanted
    }
}

/// Whether `body` has reached `goal`: it is within the goal's arrival radius, or another agent
/// that had arrived and is at rest holds the goal (the goal lies within the two hulls' reach) and
/// `body` is next to it. Goals closer together than hulls can't all be sat on, so the agents that
/// come last would otherwise circle those that settled and never arrive.
///
/// `agents` holds one sphere per agent, in agent order, in world axes, `was_arrived` the arrival
/// flags of the step before, and `own` is the index of `body`'s agent. `frame` is the velocity of
/// `body`'s frame, which the other agents' rest is judged against.
fn has_reached(
    body: &Body,
    goal: Goal,
    agents: &[Obstacle],
    frame: DVec3,
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
                && (other.velocity - frame).length() <= other.radius * AT_REST_RADII_PER_S
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

    /// Adds an agent that is at rest at the origin, facing along -z, with no behaviour. It takes
    /// the lowest free slot, or a new one after the last.
    pub fn add_agent(&mut self, profile: Profile) -> AgentId {
        let agent = Agent {
            body: Body::AT_REST,
            profile,
            frame_velocity: DVec3::ZERO,
            behaviour: None,
            tracked: None,
            commitment: None,
            facing: None,
            avoidance_period: 0.0,
            next_avoidance: 0.0,
            held: None,
        };
        if let Some(index) = self.agents.iter().position(Option::is_none) {
            self.agents[index] = Some(agent);
            return AgentId::new(index);
        }
        self.agents.push(Some(agent));
        self.axes.push([0.0; AXIS_COUNT]);
        self.arrived.push(false);
        AgentId::new(self.agents.len() - 1)
    }

    /// Removes an agent and frees its slot, which a later [`add_agent`](World::add_agent) reuses.
    /// The other agents forget the removed one: a tracked key or a remembered side that named it
    /// is cleared, so the slot's next agent inherits nothing. The caller drops `id`: it names
    /// whichever agent takes the slot next. Panics if `id` isn't an agent of this world.
    pub fn remove_agent(&mut self, id: AgentId) {
        self.agent_mut(id);
        self.agents[id.index()] = None;
        self.axes[id.index()] = [0.0; AXIS_COUNT];
        self.arrived[id.index()] = false;
        let removed = ObstacleKey::Agent(id);
        for agent in self.agents.iter_mut().flatten() {
            if agent.tracked == Some(removed) {
                agent.tracked = None;
            }
            if agent.commitment.is_some_and(|side| side.is_about(removed)) {
                agent.commitment = None;
            }
        }
    }

    /// The number of slots: the agents and the free slots between them. It is the length, in
    /// agents, of what [`axes`](World::axes) holds.
    pub fn slot_count(&self) -> usize {
        self.agents.len()
    }

    fn agent_mut(&mut self, id: AgentId) -> &mut Agent {
        self.agents
            .get_mut(id.index())
            .and_then(Option::as_mut)
            .unwrap_or_else(|| panic!("{id:?} isn't an agent of this world"))
    }

    /// Sets the body an agent is steered from. Panics if `id` isn't an agent of this world.
    pub fn set_body(&mut self, id: AgentId, body: Body) {
        self.agent_mut(id).body = body;
    }

    /// Sets what limits an agent's motion; the caller refreshes it when the body's mass changes.
    /// Panics if `id` isn't an agent of this world.
    pub fn set_profile(&mut self, id: AgentId, profile: Profile) {
        self.agent_mut(id).profile = profile;
    }

    /// Sets the largest speed the agent's behaviour and avoidance plan at, relative to its frame
    /// ([`NO_SPEED_CAP`](crate::NO_SPEED_CAP) for none, the default). It lowers the top speed
    /// only: the thrust limits are unchanged, so an agent over a cap it was just given is steered
    /// down to it at full authority. It is the profile's `speed_cap`, so a caller that replaces
    /// the profile ([`set_profile`](World::set_profile)) carries the cap in it.
    ///
    /// # Panics
    /// If `id` isn't an agent of this world, or `cap` is negative or NaN.
    pub fn set_speed_cap(&mut self, id: AgentId, cap: Speed) {
        assert!(cap >= 0.0, "a speed cap must be zero or more, got {cap}");
        self.agent_mut(id).profile.speed_cap = cap;
    }

    /// Sets the velocity the agent's surroundings carry it at (zero by default), in length units
    /// per second: the part of its velocity that drag doesn't act on. The agent is steered, and
    /// judged at rest, relative to it, and the controller's drag, top speed and braking follow.
    /// It is a translation only: the frame's acceleration and turning are ignored. Set it again
    /// each step it changes. Panics if `id` isn't an agent of this world.
    pub fn set_frame_velocity(&mut self, id: AgentId, velocity: DVec3) {
        self.agent_mut(id).frame_velocity = velocity;
    }

    /// Sets what an agent does; `None` leaves it without input. Target bodies inside the
    /// behaviour are snapshots, so set it again each step for a target that moves. Panics if `id`
    /// isn't an agent of this world.
    pub fn set_behaviour(&mut self, id: AgentId, behaviour: Option<Behaviour>) {
        self.agent_mut(id).behaviour = behaviour;
    }

    /// Names the obstacle the agent's behaviour tracks (a pursued or followed body), or `None`.
    /// The agent leaves it out of its scene. If it is an agent, that agent leaves this one out
    /// too while this one lies behind it (relative to its wanted course), and avoids it with
    /// the half share of any steering agent otherwise. The key stays until it is set again, so
    /// the caller clears it when it sets a behaviour that tracks nothing. Panics if `id` isn't
    /// an agent of this world.
    pub fn set_tracked(&mut self, id: AgentId, target: Option<ObstacleKey>) {
        self.agent_mut(id).tracked = target;
    }

    /// Sets a world point the agent's nose holds toward, or `None` to leave the facing to its
    /// behaviour. It replaces the behaviour's own facing, and keeps until it is set again, so the
    /// caller sets it each step for a point that moves. The nose follows the course only while
    /// the agent speeds up toward it harder than its side thrusters can, as with any facing; a
    /// point within the agent's own radius gives no facing. Panics if `id` isn't an agent of this
    /// world.
    pub fn set_facing(&mut self, id: AgentId, point: Option<DVec3>) {
        self.agent_mut(id).facing = point;
    }

    /// Sets how often the agent's avoidance runs, in seconds; 0 (the default) runs it at every
    /// step. The behaviour, the controller and the arrival test run at every step either way.
    /// Between runs the agent keeps the course the last run gave it, if that changed the
    /// behaviour's course, and otherwise flies the behaviour's fresh course. Agents with one period
    /// first run at different times, staggered by slot, so they don't all run in one step.
    ///
    /// # Panics
    /// If `id` isn't an agent of this world, or `seconds` is negative or not finite.
    pub fn set_avoidance_period(&mut self, id: AgentId, seconds: f64) {
        assert!(
            seconds.is_finite() && seconds >= 0.0,
            "an avoidance period must be zero or more seconds, got {seconds}"
        );
        let clock = self.clock;
        let agent = self.agent_mut(id);
        agent.avoidance_period = seconds;
        agent.next_avoidance = clock + seconds * stagger_of(id.index());
        agent.held = None;
    }

    /// Makes the agent's avoidance run at the next step, and drops the course it kept from the
    /// last run. A caller that changes the agent's goal calls it, so the new goal is not flown
    /// on a stale course. Panics if `id` isn't an agent of this world.
    pub fn refresh(&mut self, id: AgentId) {
        let clock = self.clock;
        let agent = self.agent_mut(id);
        agent.next_avoidance = clock;
        agent.held = None;
    }

    /// Whether the agent's avoidance runs in a step of `dt` seconds taken now: a caller that
    /// senses obstacles only for the agents that need them asks it before the step. Panics if
    /// `id` isn't an agent of this world.
    pub fn avoidance_due(&self, id: AgentId, dt: f64) -> bool {
        let agent = self
            .agents
            .get(id.index())
            .and_then(Option::as_ref)
            .unwrap_or_else(|| panic!("{id:?} isn't an agent of this world"));
        agent.is_due(self.clock + dt)
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

    /// Computes every agent's control axes and arrival from the current bodies and obstacles,
    /// after `dt` seconds have passed since the last step. The agents whose avoidance period has
    /// run out avoid the obstacles; see [`set_avoidance_period`](World::set_avoidance_period).
    ///
    /// # Panics
    /// If `dt` is negative or not finite.
    pub fn step(&mut self, dt: f64) {
        assert!(
            dt.is_finite() && dt >= 0.0,
            "a step must be zero or more seconds long, got {dt}"
        );
        self.clock += dt;
        // The set obstacles, then one sphere per live agent, each keeping its velocity. An agent
        // leaves its own sphere out by swapping it behind the end of the slice it avoids.
        self.scene.clear();
        self.scene.extend_from_slice(&self.obstacles);
        let agent_spheres = self
            .agents
            .iter()
            .enumerate()
            .filter_map(|(index, agent)| Some((index, agent.as_ref()?)))
            .map(|(index, agent)| Obstacle {
                key: ObstacleKey::Agent(AgentId::new(index)),
                position: agent.body.position,
                velocity: agent.body.velocity,
                radius: agent.body.radius,
            });
        self.scene.extend(agent_spheres);
        let last = self.scene.len().saturating_sub(1);
        // Whether an agent holds its goal is judged on the arrivals of the step before, so the
        // result doesn't depend on the order the agents are stepped in. Like the agents' spheres,
        // it has no entry for a free slot.
        self.was_arrived.clear();
        self.was_arrived.extend(
            self.agents
                .iter()
                .zip(&self.arrived)
                .filter_map(|(agent, &arrived)| agent.as_ref().map(|_| arrived)),
        );
        self.peers.clear();
        self.peers.extend(self.agents.iter().map(|agent| {
            agent.as_ref().map_or(Peer::ABSENT, |agent| Peer {
                steering: agent.behaviour.is_some(),
                tracks: agent.tracked,
            })
        }));
        let mut shared = StepScene {
            scene: &mut self.scene,
            shifted: &mut self.shifted,
            spheres: &mut self.spheres,
            peers: &self.peers,
            tuning: &self.tuning,
        };
        let slots = self.axes.iter_mut().zip(&mut self.arrived);
        // The place of the agent being steered among the live ones: its sphere's place in the
        // scene after the obstacles, and its entry in `was_arrived`.
        let mut rank = 0;
        for (index, (agent, (axes, arrived))) in self.agents.iter_mut().zip(slots).enumerate() {
            let Some(agent) = agent else {
                *axes = [0.0; AXIS_COUNT];
                *arrived = false;
                continue;
            };
            let own = self.obstacles.len() + rank;
            rank += 1;
            let Some(behaviour) = agent.behaviour else {
                *axes = [0.0; AXIS_COUNT];
                *arrived = false;
                // A behaviour set later starts with a run of avoidance and no kept course.
                agent.next_avoidance = self.clock;
                agent.held = None;
                continue;
            };
            let frame = agent.frame_velocity;
            let body = agent.body.seen_from(frame);
            let behaviour = behaviour.seen_from(frame);
            let (toward_goal, goal) = steer(&behaviour, &body, &agent.profile);
            let wanted = if agent.is_due(self.clock) {
                let wanted = shared.avoid(agent, index, (own, last), (&body, toward_goal, goal));
                agent.held = (wanted != toward_goal).then_some(wanted);
                if agent.avoidance_period > 0.0 {
                    agent.scheduled_after_run(self.clock);
                }
                wanted
            } else {
                agent.held.unwrap_or(toward_goal)
            };
            let facing = agent
                .facing
                .and_then(|point| facing_toward(&body, point))
                .or_else(|| rest_facing(&behaviour, &body, &goal));
            *axes = control(&body, &agent.profile, &wanted, facing);
            *arrived = has_reached(
                &body,
                goal,
                &shared.scene[self.obstacles.len()..],
                frame,
                &self.was_arrived,
                rank - 1,
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
        assert!(
            self.agents.get(id.index()).is_some_and(Option::is_some),
            "{id:?} isn't an agent of this world"
        );
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

    /// The length of a step in the tests: a frame at 60 Hz.
    const TICK_S: f64 = 1.0 / 60.0;

    fn goal(target: DVec3) -> Behaviour {
        Behaviour::Arrive(Goal::fixed(target, 5.0))
    }

    /// The axes of one agent that seeks a far point ahead at `relative_speed` along -z relative to
    /// its frame, carried at `carry`, under `cap`.
    fn capped_axes(cap: Speed, carry: DVec3, relative_speed: f64) -> [f64; AXIS_COUNT] {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_speed_cap(id, cap);
        world.set_frame_velocity(id, carry);
        world.set_body(
            id,
            Body {
                velocity: carry + DVec3::NEG_Z * relative_speed,
                ..Body::AT_REST
            },
        );
        let far = DVec3::new(0.0, 0.0, -1.0e6);
        world.set_behaviour(
            id,
            Some(Behaviour::Seek {
                goal: Goal::fixed(far, 5.0),
                pass_speed: f64::INFINITY,
            }),
        );
        world.step(TICK_S);
        world.axes().try_into().expect("one agent")
    }

    #[test]
    fn a_speed_cap_is_relative_to_the_frame_velocity() {
        const CAP: f64 = 40.0;
        let carry = DVec3::new(300.0, 0.0, -500.0);
        for relative_speed in [0.0, CAP, 2.0 * CAP] {
            assert_eq!(
                capped_axes(CAP, carry, relative_speed),
                capped_axes(CAP, DVec3::ZERO, relative_speed),
                "at {relative_speed} relative to the frame"
            );
        }
        let forward = |speed| capped_axes(CAP, DVec3::ZERO, speed)[Axis::ThrustForward as usize];
        assert!(forward(0.0) > forward(CAP), "accelerates below the cap");
        assert!(forward(2.0 * CAP) < forward(CAP), "brakes above the cap");
    }

    #[test]
    #[should_panic(expected = "a speed cap must be zero or more")]
    fn a_negative_speed_cap_is_refused() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_speed_cap(id, -1.0);
    }

    #[test]
    #[should_panic(expected = "a speed cap must be zero or more")]
    fn a_nan_speed_cap_is_refused() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_speed_cap(id, f64::NAN);
    }

    #[test]
    fn new_world_has_no_agents() {
        let world = World::new();
        assert_eq!(world.slot_count(), 0);
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

    /// A flier heading for a goal 1000 ahead, and a resting agent of radius 30 on its course 200
    /// ahead, in slots 0 and 1.
    fn flier_and_blocker() -> (World, AgentId, AgentId) {
        let mut world = World::new();
        let flier = world.add_agent(PROFILE);
        let blocker = world.add_agent(PROFILE);
        world.set_behaviour(flier, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        world.set_body(
            blocker,
            Body {
                position: DVec3::new(0.0, 0.0, -200.0),
                radius: 30.0,
                ..Body::AT_REST
            },
        );
        (world, flier, blocker)
    }

    fn axes_of(world: &World, id: AgentId) -> Vec<f64> {
        let start = id.index() * AXIS_COUNT;
        world.axes()[start..start + AXIS_COUNT].to_vec()
    }

    #[test]
    fn a_removed_agents_slot_is_the_lowest_free_one_and_is_reused() {
        let mut world = World::new();
        let ids: Vec<_> = (0..4).map(|_| world.add_agent(PROFILE)).collect();
        world.remove_agent(ids[2]);
        world.remove_agent(ids[1]);
        assert_eq!(
            world.slot_count(),
            4,
            "removal frees a slot, it doesn't shrink"
        );
        assert_eq!(world.add_agent(PROFILE), ids[1]);
        assert_eq!(world.add_agent(PROFILE), ids[2]);
        assert_eq!(
            world.add_agent(PROFILE).index(),
            4,
            "no free slot left: a new one"
        );
        assert_eq!(world.axes().len(), 5 * AXIS_COUNT);
    }

    #[test]
    fn a_free_slot_has_no_input_and_has_not_arrived() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        world.step(TICK_S);
        assert_ne!(world.axes(), [0.0; AXIS_COUNT], "the agent steers");
        world.remove_agent(id);
        assert_eq!(world.axes(), [0.0; AXIS_COUNT]);
        world.step(TICK_S);
        assert_eq!(world.axes(), [0.0; AXIS_COUNT]);
        let reused = world.add_agent(PROFILE);
        world.step(TICK_S);
        assert_eq!(
            axes_of(&world, reused),
            [0.0; AXIS_COUNT],
            "the new agent has no goal"
        );
        assert!(!world.arrived(reused));
    }

    #[test]
    #[should_panic(expected = "isn't an agent of this world")]
    fn a_free_slot_is_not_an_agent() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.remove_agent(id);
        world.set_behaviour(id, None);
    }

    #[test]
    #[should_panic(expected = "isn't an agent of this world")]
    fn an_agent_can_not_be_removed_twice() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.remove_agent(id);
        world.remove_agent(id);
    }

    #[test]
    fn a_removed_agent_is_no_longer_in_the_scene() {
        let (mut world, flier, blocker) = flier_and_blocker();
        world.step(TICK_S);
        let blocked = axes_of(&world, flier);
        world.remove_agent(blocker);
        world.step(TICK_S);
        let mut alone = World::new();
        let only = alone.add_agent(PROFILE);
        alone.set_behaviour(only, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        alone.step(TICK_S);
        assert_ne!(axes_of(&world, flier), blocked);
        assert_eq!(axes_of(&world, flier), axes_of(&alone, only));
    }

    #[test]
    fn a_free_slot_before_an_agent_does_not_change_how_it_is_steered() {
        let (mut compact, flier, _) = flier_and_blocker();
        compact.step(TICK_S);

        let mut holed = World::new();
        let hole = holed.add_agent(PROFILE);
        let moved = holed.add_agent(PROFILE);
        let blocker = holed.add_agent(PROFILE);
        holed.remove_agent(hole);
        holed.set_behaviour(moved, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        holed.set_body(
            blocker,
            Body {
                position: DVec3::new(0.0, 0.0, -200.0),
                radius: 30.0,
                ..Body::AT_REST
            },
        );
        holed.step(TICK_S);
        assert_eq!(axes_of(&holed, moved), axes_of(&compact, flier));
    }

    #[test]
    fn an_agent_that_takes_a_freed_slot_inherits_no_tracked_target() {
        let (mut world, flier, blocker) = flier_and_blocker();
        world.set_tracked(flier, Some(ObstacleKey::Agent(blocker)));
        world.step(TICK_S);
        let tracked = axes_of(&world, flier);
        world.remove_agent(blocker);
        let newcomer = world.add_agent(PROFILE);
        assert_eq!(newcomer, blocker);
        world.set_body(
            newcomer,
            Body {
                position: DVec3::new(0.0, 0.0, -200.0),
                radius: 30.0,
                ..Body::AT_REST
            },
        );
        world.step(TICK_S);
        assert_eq!(world.agents[flier.index()].as_ref().unwrap().tracked, None);
        assert_ne!(axes_of(&world, flier), tracked, "the newcomer is avoided");
    }

    #[test]
    fn the_side_remembered_about_a_removed_agent_is_forgotten() {
        let (mut world, flier, blocker) = flier_and_blocker();
        world.step(TICK_S);
        let commitment = |world: &World| world.agents[flier.index()].as_ref().unwrap().commitment;
        assert!(commitment(&world).is_some(), "the flier remembers its side");
        world.remove_agent(blocker);
        assert!(commitment(&world).is_none());
    }

    #[test]
    fn an_agent_without_a_goal_gets_no_input() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.step(TICK_S);
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
        world.step(TICK_S);

        let yaw_of = |id: AgentId| world.axes()[id.index() * AXIS_COUNT + Axis::Yaw as usize];
        assert_eq!(yaw_of(to_right), 1.0);
        assert_eq!(yaw_of(to_left), -1.0);
    }

    #[test]
    fn arrival_is_reported_within_the_goal_radius() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -4.0))));
        world.step(TICK_S);
        assert!(world.arrived(id));

        world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -6.0))));
        world.step(TICK_S);
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
        world.step(TICK_S);
        assert!(world.arrived(id));

        world.set_behaviour(id, Some(at(2.0 * radius + 0.5)));
        world.step(TICK_S);
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
        world.step(TICK_S);
        world.step(TICK_S);
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
            world.step(TICK_S);
            let alone = axes_of(&world, flier);

            world.set_body(blocker, blocker_body);
            world.step(TICK_S);
            assert_ne!(axes_of(&world, flier), alone, "flier first: {flier_first}");
            assert_eq!(axes_of(&world, blocker), [0.0; AXIS_COUNT]);
        }
    }

    #[test]
    fn an_obstacle_on_the_course_changes_the_axes() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        world.step(TICK_S);
        let straight = world.axes().to_vec();

        world.set_obstacles(&[Obstacle {
            key: ObstacleKey::Body(0),
            position: DVec3::new(0.0, 0.0, -200.0),
            velocity: DVec3::ZERO,
            radius: 30.0,
        }]);
        world.step(TICK_S);
        assert_ne!(world.axes(), straight);
        world.set_obstacles(&[]);
        world.step(TICK_S);
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
        world.step(TICK_S);
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
            still.step(TICK_S);
            moving.step(TICK_S);
            assert_same_axes(&still, &moving, seed);

            let mut open = carried_world(seed, DVec3::ZERO, false);
            open.step(TICK_S);
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
        still.step(TICK_S);
        unaware.step(TICK_S);
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
        world.step(TICK_S);
        assert!(world.arrived(id));
        let thrust = &world.axes()[Axis::ThrustForward as usize..AXIS_COUNT];
        assert_eq!(thrust, [0.0; 3]);
    }

    /// A point to the right of an agent that faces -z.
    const RIGHT_POINT: DVec3 = DVec3::new(500.0, 0.0, 0.0);

    /// The yaw axis after one step of an agent at the origin, at rest, that holds its place (its
    /// goal is on it) or flies to a goal 1000 ahead, with `facing` set.
    fn yaw_with_facing(on_station: bool, facing: Option<DVec3>) -> f64 {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        let target = if on_station {
            DVec3::ZERO
        } else {
            DVec3::new(0.0, 0.0, -1000.0)
        };
        world.set_behaviour(id, Some(goal(target)));
        world.set_facing(id, facing);
        world.step(TICK_S);
        world.axes()[Axis::Yaw as usize]
    }

    #[test]
    fn an_agent_on_station_turns_its_nose_to_a_facing_point() {
        assert_eq!(yaw_with_facing(true, None), 0.0);
        assert!(
            yaw_with_facing(true, Some(RIGHT_POINT)) > 0.0,
            "turns right"
        );
        assert!(
            yaw_with_facing(true, Some(-RIGHT_POINT)) < 0.0,
            "turns left"
        );
    }

    #[test]
    fn a_facing_point_replaces_the_behaviours_rest_facing() {
        // A follower on station would face where its leader faces (+x here); the point wins.
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        let leader = Body {
            position: DVec3::ZERO,
            frame: crate::agent::Frame {
                forward: DVec3::X,
                right: DVec3::Z,
                up: DVec3::Y,
            },
            ..Body::AT_REST
        };
        world.set_behaviour(
            id,
            Some(Behaviour::Follow {
                leader,
                offset: FrameOffset {
                    right: 0.0,
                    up: 0.0,
                    forward: 0.0,
                },
                arrival_radius: 5.0,
            }),
        );
        world.step(TICK_S);
        assert!(
            world.axes()[Axis::Yaw as usize] > 0.0,
            "faces the leader's heading"
        );
        world.set_facing(id, Some(-RIGHT_POINT));
        world.step(TICK_S);
        assert!(world.axes()[Axis::Yaw as usize] < 0.0, "faces the point");
    }

    #[test]
    fn an_agent_speeding_up_hard_keeps_the_main_thruster_aimed_along_its_course() {
        assert_eq!(yaw_with_facing(false, Some(RIGHT_POINT)), 0.0);
    }

    #[test]
    fn a_facing_point_on_the_agent_gives_no_facing() {
        assert_eq!(yaw_with_facing(true, Some(DVec3::ZERO)), 0.0);
    }

    #[test]
    fn a_removed_agents_facing_is_not_inherited_by_the_next_agent_in_its_slot() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_facing(id, Some(RIGHT_POINT));
        world.remove_agent(id);
        let next = world.add_agent(PROFILE);
        assert_eq!(next.index(), id.index());
        world.set_behaviour(next, Some(goal(DVec3::ZERO)));
        world.step(TICK_S);
        assert_eq!(world.axes()[Axis::Yaw as usize], 0.0);
    }

    #[test]
    #[should_panic(expected = "isn't an agent")]
    fn a_facing_for_a_free_slot_is_refused() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.remove_agent(id);
        world.set_facing(id, None);
    }

    /// A step and a period that are exact in binary, so the clock reaches a due time exactly: the
    /// period is `PERIOD_STEPS` steps.
    const EXACT_STEP_S: f64 = 1.0 / 64.0;
    const PERIOD_STEPS: usize = 8;
    const PERIOD_S: f64 = EXACT_STEP_S * PERIOD_STEPS as f64;

    /// Moves the blocker of `flier_and_blocker` far off the flier's course.
    fn move_blocker_away(world: &mut World, blocker: AgentId) {
        world.set_body(
            blocker,
            Body {
                position: DVec3::new(5000.0, 0.0, -200.0),
                radius: 30.0,
                ..Body::AT_REST
            },
        );
    }

    /// The axes of the flier alone, on the same course with nothing in its way.
    fn free_axes() -> Vec<f64> {
        let mut alone = World::new();
        let only = alone.add_agent(PROFILE);
        alone.set_behaviour(only, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        alone.step(EXACT_STEP_S);
        axes_of(&alone, only)
    }

    #[test]
    fn a_detour_is_held_until_the_agent_is_due_again() {
        let (mut world, flier, blocker) = flier_and_blocker();
        world.set_avoidance_period(flier, PERIOD_S);
        world.step(EXACT_STEP_S);
        let detour = axes_of(&world, flier);
        assert_ne!(detour, free_axes(), "the first step runs avoidance");
        move_blocker_away(&mut world, blocker);
        for step in 2..PERIOD_STEPS {
            world.step(EXACT_STEP_S);
            assert_eq!(axes_of(&world, flier), detour, "step {step} keeps it");
        }
        world.step(EXACT_STEP_S);
        assert_eq!(
            axes_of(&world, flier),
            free_axes(),
            "the run a period after the first sees the way clear"
        );
    }

    #[test]
    fn an_agent_whose_last_run_found_nothing_flies_the_fresh_course() {
        let (mut world, flier, blocker) = flier_and_blocker();
        move_blocker_away(&mut world, blocker);
        world.set_avoidance_period(flier, PERIOD_S);
        world.step(EXACT_STEP_S);
        assert_eq!(axes_of(&world, flier), free_axes());
        assert!(
            world.agents[flier.index()]
                .as_ref()
                .is_some_and(|agent| agent.held.is_none()),
            "nothing is held"
        );
        // The goal changes between runs: the fresh course is flown at once, not a held one.
        world.set_behaviour(flier, Some(goal(DVec3::new(500.0, 0.0, 0.0))));
        world.step(EXACT_STEP_S);
        assert_eq!(
            world.axes()[Axis::Yaw as usize],
            1.0,
            "turns to the new goal"
        );
    }

    #[test]
    fn a_refresh_runs_avoidance_at_the_next_step_and_drops_the_detour() {
        let (mut world, flier, blocker) = flier_and_blocker();
        world.set_avoidance_period(flier, PERIOD_S);
        world.step(EXACT_STEP_S);
        move_blocker_away(&mut world, blocker);
        world.step(EXACT_STEP_S);
        assert_ne!(axes_of(&world, flier), free_axes(), "the detour is held");
        world.refresh(flier);
        assert!(world.avoidance_due(flier, EXACT_STEP_S));
        world.step(EXACT_STEP_S);
        assert_eq!(axes_of(&world, flier), free_axes());
    }

    #[test]
    fn avoidance_runs_once_a_period_for_each_agent_and_not_all_in_one_step() {
        const AGENTS: usize = 8;
        const STEPS: usize = 4 * PERIOD_STEPS;
        let mut world = World::new();
        let ids: Vec<_> = (0..AGENTS).map(|_| world.add_agent(PROFILE)).collect();
        for &id in &ids {
            world.set_avoidance_period(id, PERIOD_S);
            world.set_behaviour(id, Some(goal(DVec3::new(0.0, 0.0, -1000.0))));
        }
        let next_of =
            |world: &World, id: AgentId| world.agents[id.index()].as_ref().unwrap().next_avoidance;
        let mut runs = vec![0; AGENTS];
        let mut most_in_a_step = 0;
        // The first period is the stagger's, in which each agent runs once.
        for step in 0..PERIOD_STEPS + STEPS {
            let before: Vec<f64> = ids.iter().map(|&id| next_of(&world, id)).collect();
            let due: Vec<bool> = ids
                .iter()
                .map(|&id| world.avoidance_due(id, EXACT_STEP_S))
                .collect();
            world.step(EXACT_STEP_S);
            for (index, &id) in ids.iter().enumerate() {
                let ran = next_of(&world, id) != before[index];
                assert_eq!(ran, due[index], "avoidance_due predicts the step");
                if step >= PERIOD_STEPS {
                    runs[index] += usize::from(ran);
                }
            }
            most_in_a_step = most_in_a_step.max(due.iter().filter(|&&due| due).count());
        }
        assert_eq!(runs, vec![STEPS / PERIOD_STEPS; AGENTS], "once a period");
        assert!(
            most_in_a_step < AGENTS / 2,
            "the agents are staggered, but {most_in_a_step} ran in one step"
        );
    }

    #[test]
    fn a_period_of_zero_runs_avoidance_at_every_step() {
        let (mut world, flier, blocker) = flier_and_blocker();
        world.set_avoidance_period(flier, PERIOD_S);
        world.set_avoidance_period(flier, 0.0);
        world.step(EXACT_STEP_S);
        move_blocker_away(&mut world, blocker);
        world.step(EXACT_STEP_S);
        assert_eq!(axes_of(&world, flier), free_axes());
    }

    #[test]
    #[should_panic(expected = "an avoidance period must be zero or more")]
    fn a_negative_avoidance_period_is_refused() {
        let mut world = World::new();
        let id = world.add_agent(PROFILE);
        world.set_avoidance_period(id, -1.0);
    }

    #[test]
    #[should_panic(expected = "a step must be zero or more seconds long")]
    fn a_negative_step_is_refused() {
        World::new().step(-EXACT_STEP_S);
    }
}
