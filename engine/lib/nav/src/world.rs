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
mod tests;
