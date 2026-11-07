//! Closed-loop checks: agents flown by the control axes through a simple model of the ship.
//!
//! The model is a point mass with linear drag, thrust along its own axes, and turn rates
//! proportional to the pitch and yaw axes. It stands in for the physics engine, so the tests
//! check that behaviour, avoidance and controller work together, not the ship's real response.

use glam::{DQuat, DVec3};
use nav::{
    AXIS_COUNT, Axis, Behaviour, Body, Frame, FrameOffset, Goal, Obstacle, ObstacleKey, Profile,
    World,
};

const STEP_S: f64 = 1.0 / 60.0;
const TIME_LIMIT_S: f64 = 60.0;
const LINEAR_DRAG: f64 = 2.0;
const ANGULAR_DRAG: f64 = 20.0;
/// Strafe and vertical thrust, as fractions of the forward thrust.
const SIDE_THRUST_RATIO: f64 = 0.25;
/// Turn rate at full pitch or yaw input once the drag balances it, in radians per second.
const TURN_RATE: f64 = 3.0;
/// Roll authority, as a fraction of the pitch and yaw authority.
const ROLL_RATIO: f64 = 0.3;
const PROFILE: Profile = profile(100.0);
const AGENT_RADIUS: f64 = 2.0;
const ARRIVAL_RADIUS: f64 = 5.0;
const ARRIVAL_SPEED: f64 = 2.0;

/// A profile whose forward thrust holds `max_speed` against the drag, with side thrust a fraction
/// of it and turning gear that reaches `TURN_RATE` against the angular drag.
const fn profile(max_speed: f64) -> Profile {
    let accel_forward = max_speed * LINEAR_DRAG;
    let turn = TURN_RATE * ANGULAR_DRAG;
    Profile {
        linear_drag: LINEAR_DRAG,
        angular_drag: ANGULAR_DRAG,
        accel_forward,
        accel_right: accel_forward * SIDE_THRUST_RATIO,
        accel_up: accel_forward * SIDE_THRUST_RATIO,
        turn_pitch: turn,
        turn_yaw: turn,
        turn_roll: turn * ROLL_RATIO,
    }
}

struct Ship {
    position: DVec3,
    velocity: DVec3,
    orientation: DQuat,
    /// Radians per second, about the world axes.
    angular_velocity: DVec3,
    /// The smallest distance from the ship's centre to an obstacle's surface so far.
    clearance: f64,
    start: DVec3,
    /// The farthest the ship has been from where it started.
    range: f64,
    /// The farthest the ship has been from the line through where it started along z.
    sway: f64,
    /// The ship's bounding radius.
    radius: f64,
}

impl Ship {
    fn new(position: DVec3) -> Self {
        Self {
            radius: AGENT_RADIUS,
            position,
            velocity: DVec3::ZERO,
            orientation: DQuat::IDENTITY,
            angular_velocity: DVec3::ZERO,
            clearance: f64::INFINITY,
            start: position,
            range: 0.0,
            sway: 0.0,
        }
    }

    fn body(&self) -> Body {
        Body {
            position: self.position,
            velocity: self.velocity,
            angular_velocity: self.angular_velocity,
            frame: Frame {
                forward: self.orientation * DVec3::NEG_Z,
                right: self.orientation * DVec3::X,
                up: self.orientation * DVec3::Y,
            },
            radius: self.radius,
        }
    }

    fn apply(&mut self, profile: Profile, axes: &[f64]) {
        let axis = |which: Axis| axes[which as usize];
        let body = self.body();
        // Pitch turns about the right axis, yaw about the down axis, roll about the forward axis.
        let turn = body.frame.right * axis(Axis::Pitch) * profile.turn_pitch
            - body.frame.up * axis(Axis::Yaw) * profile.turn_yaw
            + body.frame.forward * axis(Axis::Roll) * profile.turn_roll;
        self.angular_velocity += (turn - self.angular_velocity * profile.angular_drag) * STEP_S;
        let spin = self.angular_velocity.length() * STEP_S;
        if spin > 0.0 {
            self.orientation = (DQuat::from_axis_angle(self.angular_velocity.normalize(), spin)
                * self.orientation)
                .normalize();
        }
        let thrust = body.frame.forward * axis(Axis::ThrustForward) * profile.accel_forward
            + body.frame.right * axis(Axis::ThrustRight) * profile.accel_right
            + body.frame.up * axis(Axis::ThrustUp) * profile.accel_up;
        self.velocity += (thrust - self.velocity * profile.linear_drag) * STEP_S;
        self.position += self.velocity * STEP_S;
        self.range = self.range.max(self.position.distance(self.start));
        let off_line = self.position - self.start;
        self.sway = self.sway.max(off_line.x.hypot(off_line.y));
    }
}

/// Flies one ship to `target` past `obstacles`, which move at their own velocity. Returns the
/// ship and the time it arrived at, or `None` if it didn't within the limit.
fn fly(start: DVec3, target: DVec3, obstacles: &[Obstacle]) -> (Ship, Option<f64>) {
    fly_with(PROFILE, start, target, obstacles)
}

fn fly_with(
    profile: Profile,
    start: DVec3,
    target: DVec3,
    obstacles: &[Obstacle],
) -> (Ship, Option<f64>) {
    fly_ship(profile, Ship::new(start), target, obstacles)
}

/// Flies `ship`, in whatever state it starts, like [`fly`].
fn fly_ship(
    profile: Profile,
    mut ship: Ship,
    target: DVec3,
    obstacles: &[Obstacle],
) -> (Ship, Option<f64>) {
    let mut world = World::new();
    let id = world.add_agent(profile);
    world.set_behaviour(
        id,
        Some(Behaviour::Arrive(Goal {
            target,
            arrival_radius: ARRIVAL_RADIUS,
        })),
    );
    // Each rock gets its own key, so the side memory tells them apart.
    let mut obstacles = obstacles.to_vec();
    for (key, obstacle) in (0..).zip(&mut obstacles) {
        obstacle.key = ObstacleKey::Body(key);
    }

    let mut time = 0.0;
    while time < TIME_LIMIT_S {
        world.set_body(id, ship.body());
        world.set_obstacles(&obstacles);
        world.step();
        let axes = &world.axes()[id.index() * AXIS_COUNT..][..AXIS_COUNT];
        ship.apply(profile, axes);
        for obstacle in &mut obstacles {
            obstacle.position += obstacle.velocity * STEP_S;
            let gap = ship.position.distance(obstacle.position) - obstacle.radius;
            ship.clearance = ship.clearance.min(gap);
        }
        time += STEP_S;
        if ship.position.distance(target) <= ARRIVAL_RADIUS
            && ship.velocity.length() <= ARRIVAL_SPEED
        {
            return (ship, Some(time));
        }
    }
    (ship, None)
}

fn rock(position: DVec3, velocity: DVec3, radius: f64) -> Obstacle {
    Obstacle {
        key: ObstacleKey::Body(0),
        position,
        velocity,
        radius,
    }
}

#[test]
fn a_ship_behind_and_beside_its_goal_turns_flies_and_stops() {
    let target = DVec3::new(300.0, 0.0, 400.0);
    let (ship, arrived) = fly(DVec3::ZERO, target, &[]);
    assert!(
        arrived.is_some(),
        "stopped {} from the goal",
        ship.position.distance(target)
    );
}

#[test]
fn a_ship_at_cruise_speed_brakes_with_thrust_when_its_goal_is_where_it_started() {
    let cruise = 8000.0;
    let ship = Ship {
        velocity: DVec3::new(0.0, 0.0, -cruise),
        ..Ship::new(DVec3::ZERO)
    };
    // The goal is just behind the ship, so the range is how far it runs on before it turns back.
    let target = DVec3::new(0.0, 0.0, 10.0);
    let (ship, arrived) = fly_ship(profile(8800.0), ship, target, &[]);
    assert!(
        arrived.is_some(),
        "stopped {} away",
        ship.position.distance(target)
    );
    // Drag alone carries the ship `cruise / drag` past where it started; thrust must beat that.
    assert!(
        ship.range < 0.5 * cruise / LINEAR_DRAG,
        "went {} past the start",
        ship.range
    );
}

#[test]
fn a_ship_stops_on_a_far_goal_without_a_large_overshoot() {
    let target = DVec3::new(0.0, 0.0, -5000.0);
    let (ship, arrived) = fly_with(profile(8800.0), DVec3::ZERO, target, &[]);
    assert!(arrived.is_some());
    // The ship's farthest point from its start is at most a little past the goal.
    assert!(
        ship.range < target.length() + 10.0 * ARRIVAL_RADIUS,
        "went {} for a goal at {}",
        ship.range,
        target.length()
    );
}

/// How much sooner a ship must arrive on a long leg than it did when it braked on drag alone,
/// in seconds. Reverse thrust shortens the stop by about this much at top speed.
const THRUST_BRAKING_GAIN_S: f64 = 1.0;

#[test]
fn a_ship_stops_on_a_long_fast_leg_soon_and_without_a_large_overshoot() {
    // Each leg with the seconds the ship took to arrive when it braked on drag alone.
    for (length, drag_only_s) in [(10_000.0, 4.62), (15_000.0, 5.22), (20_000.0, 5.80)] {
        let target = DVec3::new(0.0, 0.0, -length);
        let (ship, arrived) = fly_with(profile(8800.0), DVec3::ZERO, target, &[]);
        let time = arrived.unwrap_or_else(|| panic!("leg {length}: no arrival"));
        assert!(
            time < drag_only_s - THRUST_BRAKING_GAIN_S,
            "leg {length}: arrived after {time} s"
        );
        assert!(
            ship.range < length + 10.0 * ARRIVAL_RADIUS,
            "leg {length}: went {}",
            ship.range
        );
    }
}

#[test]
fn a_ship_flies_around_a_static_obstacle_on_its_course() {
    let target = DVec3::new(0.0, 0.0, -1000.0);
    let rocks = [rock(DVec3::new(0.0, 0.0, -500.0), DVec3::ZERO, 60.0)];
    let (ship, arrived) = fly(DVec3::ZERO, target, &rocks);
    assert!(arrived.is_some());
    assert!(
        ship.clearance > AGENT_RADIUS,
        "came within {} of the surface",
        ship.clearance
    );
}

#[test]
fn a_ship_flies_around_an_obstacle_crossing_its_course() {
    let target = DVec3::new(0.0, 0.0, -1000.0);
    let rocks = [rock(
        DVec3::new(200.0, 0.0, -500.0),
        DVec3::new(-40.0, 0.0, 0.0),
        40.0,
    )];
    let (ship, arrived) = fly(DVec3::ZERO, target, &rocks);
    assert!(arrived.is_some());
    assert!(
        ship.clearance > AGENT_RADIUS,
        "came within {} of the surface",
        ship.clearance
    );
}

#[test]
fn a_ship_flies_around_several_obstacles() {
    let target = DVec3::new(0.0, 0.0, -2000.0);
    let rocks = [
        rock(DVec3::new(0.0, 0.0, -400.0), DVec3::ZERO, 50.0),
        rock(DVec3::new(30.0, 20.0, -900.0), DVec3::ZERO, 80.0),
        rock(
            DVec3::new(-120.0, 0.0, -1300.0),
            DVec3::new(25.0, 0.0, 0.0),
            60.0,
        ),
        rock(DVec3::new(0.0, -40.0, -1700.0), DVec3::ZERO, 70.0),
    ];
    let (ship, arrived) = fly(DVec3::ZERO, target, &rocks);
    assert!(arrived.is_some());
    assert!(
        ship.clearance > AGENT_RADIUS,
        "came within {} of the surface",
        ship.clearance
    );
}

#[test]
fn a_ship_flies_round_a_pair_of_rocks_with_no_gap_between_them() {
    let target = DVec3::new(0.0, 0.0, -1000.0);
    // The course leans right of the first rock's centre, and the second rock closes that side.
    let rocks = [
        rock(DVec3::new(-5.0, 0.0, -400.0), DVec3::ZERO, 50.0),
        rock(DVec3::new(85.0, 0.0, -400.0), DVec3::ZERO, 50.0),
    ];
    let (ship, arrived) = fly(DVec3::ZERO, target, &rocks);
    assert!(arrived.is_some());
    assert!(
        ship.clearance > AGENT_RADIUS,
        "came within {} of the surface",
        ship.clearance
    );
}

#[test]
fn a_ship_flies_round_the_end_of_a_row_of_rocks() {
    let target = DVec3::new(0.0, 0.0, -1000.0);
    let rocks = [-100.0, 0.0, 100.0].map(|x| rock(DVec3::new(x, 0.0, -400.0), DVec3::ZERO, 60.0));
    let (ship, arrived) = fly(DVec3::ZERO, target, &rocks);
    assert!(arrived.is_some());
    assert!(
        ship.clearance > AGENT_RADIUS,
        "came within {} of the surface",
        ship.clearance
    );
}

#[test]
fn a_ship_at_cruise_speed_flies_round_a_planet_that_crosses_its_course() {
    // Distances of a ship that cruises at several km/s past a planet of several km.
    let profile = profile(8800.0);
    let target = DVec3::new(0.0, 0.0, -40000.0);
    for planet_speed in [0.0, 628.0] {
        for offset in [0.0, 500.0, 1500.0, 2900.0] {
            let planet = rock(
                DVec3::new(offset, 0.0, -20000.0),
                DVec3::new(planet_speed, 0.0, 0.0),
                3000.0,
            );
            let (ship, arrived) = fly_with(profile, DVec3::ZERO, target, &[planet]);
            assert!(
                arrived.is_some(),
                "planet at {offset} moving {planet_speed}: no arrival"
            );
            assert!(
                ship.clearance > AGENT_RADIUS,
                "planet at {offset} moving {planet_speed}: clearance {}",
                ship.clearance
            );
        }
    }
}

#[test]
fn every_rock_placed_on_the_course_is_cleared() {
    let target = DVec3::new(0.0, 0.0, -1000.0);
    // Offsets from the course line, and distances along it, with a fixed spread of radii.
    let mut cases = 0;
    for (i, offset) in [-30.0, -10.0, 0.0, 10.0, 30.0].into_iter().enumerate() {
        for (j, along) in [200.0, 400.0, 700.0].into_iter().enumerate() {
            let radius = 30.0 + 15.0 * ((i + j) % 4) as f64;
            let rocks = [rock(
                DVec3::new(offset, offset / 2.0, -along),
                DVec3::ZERO,
                radius,
            )];
            let (ship, arrived) = fly(DVec3::ZERO, target, &rocks);
            assert!(arrived.is_some(), "rock at {offset}, {along}: no arrival");
            assert!(
                ship.clearance > AGENT_RADIUS,
                "rock at {offset}, {along}: clearance {}",
                ship.clearance
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 15);
}

/// Ships of real size cross a field of rocks that block each other's detours. A ship that
/// swapped the side it passes a rock on, whenever a farther rock blocked the kept side, slid
/// sideways between the two rocks without getting through.
#[test]
fn ships_cross_a_field_of_rocks_without_swapping_sides() {
    const SHIP_RADIUS: f64 = 150.0;
    const MAX_SPEED: f64 = 8800.0;
    let rocks = [
        rock(DVec3::new(-4000.0, 0.0, 0.0), DVec3::ZERO, 400.0),
        rock(DVec3::new(-1000.0, 0.0, 300.0), DVec3::ZERO, 500.0),
        rock(DVec3::new(2000.0, 0.0, -300.0), DVec3::ZERO, 300.0),
        rock(DVec3::new(5000.0, 0.0, 0.0), DVec3::ZERO, 800.0),
        rock(DVec3::new(0.0, 0.0, 1500.0), DVec3::ZERO, 800.0),
        rock(DVec3::new(3000.0, 0.0, -1500.0), DVec3::ZERO, 600.0),
    ];
    let mut cases = 0;
    for lane in [-600.0, 0.0, 600.0] {
        for jitter in [0.0, 7.0, 13.0, 29.0, 47.0] {
            for turn in [0.0, std::f64::consts::FRAC_PI_2] {
                let start = DVec3::new(-10000.0, 0.0, lane + jitter);
                let target = DVec3::new(10000.0, 0.0, lane);
                let ship = Ship {
                    radius: SHIP_RADIUS,
                    orientation: DQuat::from_rotation_y(-turn),
                    ..Ship::new(start)
                };
                let (ship, arrived) = fly_ship(profile(MAX_SPEED), ship, target, &rocks);
                let case = format!("lane {lane}, jitter {jitter}, turned {turn}");
                assert!(arrived.is_some(), "{case}: stuck at {:?}", ship.position);
                assert!(
                    ship.clearance > SHIP_RADIUS,
                    "{case}: clearance {}",
                    ship.clearance
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 30);
}

/// How far from the ring of goals the crowd's ships start, in metres. The ships' order of arrival
/// changes with it, and with it which ship finds its goal held by the others.
const CROWD_START_DISTANCES_M: [f64; 6] = [2000.0, 3000.0, 4000.0, 6250.0, 8000.0, 9000.0];

/// Ships of real size fly to goals that lie closer together than their hulls, with arrival
/// circles smaller than a hull. None of them can sit at its own goal while the others do, so
/// without a floor on the arrival radius, and the arrival of an agent beside the one that holds
/// its goal, they circle each other and never arrive.
#[test]
fn ships_with_goals_closer_than_their_hulls_all_arrive() {
    for start_distance_m in CROWD_START_DISTANCES_M {
        let stuck = crowd_ships_left_out(start_distance_m);
        assert!(
            stuck.is_empty(),
            "start {start_distance_m} m: ships {stuck:?} never arrived"
        );
    }
}

/// Flies the crowd of ships to their goals from `start_distance_m` away; returns the indices of
/// the ships that never arrived within the time limit.
fn crowd_ships_left_out(start_distance_m: f64) -> Vec<usize> {
    const SHIP_RADIUS: f64 = 150.0;
    const SHIPS: usize = 8;
    const GOAL_RING_M: f64 = 220.0;
    const SMALL_ARRIVAL_M: f64 = 100.0;
    let angle = |i: usize| std::f64::consts::TAU * i as f64 / SHIPS as f64;
    let mut world = World::new();
    let mut ships = Vec::new();
    for i in 0..SHIPS {
        let id = world.add_agent(profile(8800.0));
        let target = DVec3::new(angle(i).cos(), 0.0, angle(i).sin()) * GOAL_RING_M;
        // Each ship comes in from outside the ring of goals, along the line through its goal.
        let start = target.normalize() * start_distance_m + DVec3::new(0.0, 30.0 * i as f64, 0.0);
        world.set_behaviour(
            id,
            Some(Behaviour::Arrive(Goal {
                target,
                arrival_radius: SMALL_ARRIVAL_M,
            })),
        );
        let ship = Ship {
            radius: SHIP_RADIUS,
            ..Ship::new(start)
        };
        ships.push((id, ship, None::<f64>));
    }
    let mut time = 0.0;
    while time < TIME_LIMIT_S && ships.iter().any(|(_, _, arrived)| arrived.is_none()) {
        for (id, ship, _) in &ships {
            world.set_body(*id, ship.body());
        }
        world.step();
        for (id, ship, arrived) in &mut ships {
            let axes = &world.axes()[id.index() * AXIS_COUNT..][..AXIS_COUNT];
            ship.apply(profile(8800.0), axes);
            if arrived.is_none() && world.arrived(*id) {
                *arrived = Some(time);
            }
        }
        time += STEP_S;
    }
    ships
        .iter()
        .enumerate()
        .filter(|(_, (_, _, arrived))| arrived.is_none())
        .map(|(index, _)| index)
        .collect()
}

/// One step of a ship's flight: when it was, and where it was going.
struct Sample {
    time: f64,
    position: DVec3,
    velocity: DVec3,
}

/// Flies `ships` together for `seconds`. Each step, `behave(index, time, ships)` gives a ship's
/// behaviour from the state of them all, so a target can be another ship. Returns each ship's
/// samples, taken before the step's thrust is applied.
fn fly_fleet(
    ships: &mut [Ship],
    seconds: f64,
    behave: impl FnMut(usize, f64, &[Ship]) -> Behaviour,
) -> Vec<Vec<Sample>> {
    fly_fleet_tracking(ships, seconds, behave, |_| None)
}

/// Like [`fly_fleet`], with `tracked(index)` the ship that ship `index` tracks, which it leaves
/// out of its avoidance.
fn fly_fleet_tracking(
    ships: &mut [Ship],
    seconds: f64,
    mut behave: impl FnMut(usize, f64, &[Ship]) -> Behaviour,
    tracked: impl Fn(usize) -> Option<usize>,
) -> Vec<Vec<Sample>> {
    let mut world = World::new();
    let ids: Vec<_> = ships.iter().map(|_| world.add_agent(PROFILE)).collect();
    let mut samples: Vec<Vec<Sample>> = ships.iter().map(|_| Vec::new()).collect();
    let mut time = 0.0;
    while time < seconds {
        for (index, id) in ids.iter().enumerate() {
            world.set_body(*id, ships[index].body());
            world.set_behaviour(*id, Some(behave(index, time, ships)));
            world.set_tracked(
                *id,
                tracked(index).map(|target| ObstacleKey::Agent(ids[target])),
            );
            samples[index].push(Sample {
                time,
                position: ships[index].position,
                velocity: ships[index].velocity,
            });
        }
        world.step();
        for (index, id) in ids.iter().enumerate() {
            let axes = &world.axes()[id.index() * AXIS_COUNT..][..AXIS_COUNT];
            ships[index].apply(PROFILE, axes);
        }
        time += STEP_S;
    }
    samples
}

/// A body for a behaviour's target: a sphere at `position` moving at `velocity`.
fn target_body(position: DVec3, velocity: DVec3, radius: f64) -> Body {
    Body {
        position,
        velocity,
        radius,
        ..Body::AT_REST
    }
}

fn arrive_at_point(target: DVec3) -> Behaviour {
    Behaviour::Arrive(Goal {
        target,
        arrival_radius: ARRIVAL_RADIUS,
    })
}

#[test]
fn a_seeking_ship_passes_its_point_at_top_speed() {
    const POINT_DISTANCE: f64 = 3000.0;
    const PASS_DISTANCE: f64 = 10.0;
    let point = DVec3::new(0.0, 0.0, -POINT_DISTANCE);
    let mut ships = [Ship::new(DVec3::ZERO)];
    let samples = fly_fleet(&mut ships, TIME_LIMIT_S, |_, _, _| {
        Behaviour::Seek(Goal {
            target: point,
            arrival_radius: ARRIVAL_RADIUS,
        })
    });
    let pass = samples[0]
        .iter()
        .find(|sample| sample.position.distance(point) <= PASS_DISTANCE)
        .expect("the ship reaches its point");
    assert!(
        pass.velocity.length() > 0.8 * PROFILE.max_speed(),
        "passes at {}",
        pass.velocity.length()
    );
}

/// The first time after which `holds` is true of every later sample, if there is one.
fn settled_from(samples: &[Sample], holds: impl Fn(&Sample) -> bool) -> Option<f64> {
    let failing = samples.iter().rposition(|sample| !holds(sample));
    match failing {
        None => samples.first().map(|sample| sample.time),
        Some(last) => samples.get(last + 1).map(|sample| sample.time),
    }
}

#[test]
fn a_pursuer_leads_a_crossing_target_and_settles_beside_it_sooner_than_a_chase() {
    const TARGET_RADIUS: f64 = 5.0;
    const STAND_OFF: f64 = 20.0;
    const TARGET_SPEED: f64 = 40.0;
    const TOLERANCE: f64 = 10.0;
    const MATCHED_SPEED: f64 = 5.0;
    const SECONDS: f64 = 90.0;
    let start = DVec3::new(0.0, 0.0, -2000.0);
    let velocity = DVec3::X * TARGET_SPEED;
    let hull_gap = TARGET_RADIUS + AGENT_RADIUS + STAND_OFF;
    let run = |lead: bool| {
        let mut ships = [Ship::new(DVec3::ZERO)];
        let samples = fly_fleet(&mut ships, SECONDS, |_, time, _| {
            let target = target_body(start + velocity * time, velocity, TARGET_RADIUS);
            if lead {
                Behaviour::Pursue {
                    target,
                    stand_off: STAND_OFF,
                    arrival_radius: ARRIVAL_RADIUS,
                }
            } else {
                // The same meeting point, but with no feed-forward of the target's velocity.
                arrive_at_point(target.position)
            }
        });
        settled_from(&samples[0], |sample| {
            let target = start + velocity * sample.time;
            let gap = sample.position.distance(target);
            (gap - hull_gap).abs() < TOLERANCE
                && (sample.velocity - velocity).length() < MATCHED_SPEED
        })
    };
    let led = run(true).expect("the pursuer settles beside its target");
    // A chase with no lead stops short of the target's hull, so it can't settle at the stand-off.
    assert!(run(false).is_none_or(|chased| led < chased));
}

#[test]
fn a_follower_holds_its_slot_in_the_leaders_frame_without_touching_it() {
    const SLOT: DVec3 = DVec3::new(30.0, 0.0, -60.0);
    const SECONDS: f64 = 60.0;
    const SETTLED_S: f64 = 45.0;
    const SLOT_TOLERANCE: f64 = 12.0;
    let leader_goal = DVec3::new(0.0, 0.0, -1500.0);
    let mut ships = [
        Ship::new(DVec3::ZERO),
        Ship::new(DVec3::new(-200.0, 0.0, 300.0)),
    ];
    let samples = fly_fleet(&mut ships, SECONDS, |index, _, ships| {
        if index == 0 {
            return arrive_at_point(leader_goal);
        }
        Behaviour::Follow {
            leader: ships[0].body(),
            offset: FrameOffset {
                right: SLOT.x,
                up: SLOT.y,
                // The ship faces -z, so forward in its frame is -z in the world.
                forward: -SLOT.z,
            },
            arrival_radius: ARRIVAL_RADIUS,
        }
    });
    let mut nearest = f64::INFINITY;
    for (leader, follower) in samples[0].iter().zip(&samples[1]) {
        nearest = nearest.min(leader.position.distance(follower.position));
        if leader.time > SETTLED_S {
            // The leader flies along -z with its frame unrotated, so the slot is a fixed offset.
            let slot = leader.position + SLOT;
            assert!(
                follower.position.distance(slot) < SLOT_TOLERANCE,
                "t = {}: {} off its slot",
                leader.time,
                follower.position.distance(slot)
            );
        }
    }
    assert!(
        nearest > 2.0 * AGENT_RADIUS,
        "came within {nearest} of the leader"
    );
}

#[test]
fn a_follower_keeps_its_slot_while_the_leader_is_still_flying() {
    const SLOT: DVec3 = DVec3::new(30.0, 0.0, -60.0);
    const SECONDS: f64 = 25.0;
    // The leader has reached cruise by then, and is still far from its goal.
    const CRUISE_FROM_S: f64 = 10.0;
    const CRUISE_FRACTION: f64 = 0.9;
    const SLOT_LAG_TOLERANCE: f64 = 10.0;
    let leader_goal = DVec3::new(0.0, 0.0, -5000.0);
    // The follower starts in its slot, so what it does afterwards is the lag of the rule alone.
    let mut ships = [Ship::new(DVec3::ZERO), Ship::new(SLOT)];
    let samples = fly_fleet(&mut ships, SECONDS, |index, _, ships| {
        if index == 0 {
            return arrive_at_point(leader_goal);
        }
        Behaviour::Follow {
            leader: ships[0].body(),
            offset: FrameOffset {
                right: SLOT.x,
                up: SLOT.y,
                forward: -SLOT.z,
            },
            arrival_radius: ARRIVAL_RADIUS,
        }
    });
    let mut worst: f64 = 0.0;
    for (leader, follower) in samples[0].iter().zip(&samples[1]) {
        if leader.time < CRUISE_FROM_S {
            continue;
        }
        assert!(
            leader.velocity.length() > CRUISE_FRACTION * PROFILE.max_speed(),
            "t = {}: the leader is not cruising",
            leader.time
        );
        worst = worst.max(follower.position.distance(leader.position + SLOT));
    }
    // Without the leader's velocity fed forward, the follower trails its slot by the speed over
    // the position loop's gain, far more than this.
    assert!(worst < SLOT_LAG_TOLERANCE, "{worst} off its slot");
}

#[test]
fn a_held_ship_returns_to_its_point_after_a_push() {
    const PUSH_SPEED: f64 = 60.0;
    const SECONDS: f64 = 30.0;
    let mut ships = [Ship {
        velocity: DVec3::new(0.0, 0.0, -PUSH_SPEED),
        ..Ship::new(DVec3::ZERO)
    }];
    let samples = fly_fleet(&mut ships, SECONDS, |_, _, _| {
        Behaviour::hold(
            &target_body(DVec3::ZERO, DVec3::ZERO, AGENT_RADIUS),
            ARRIVAL_RADIUS,
        )
    });
    let overshoot = samples[0]
        .iter()
        .map(|sample| sample.position.length())
        .fold(0.0, f64::max);
    assert!(overshoot > ARRIVAL_RADIUS, "the push carries it away");
    let last = samples[0].last().expect("samples");
    assert!(
        last.position.length() < ARRIVAL_RADIUS,
        "back at {}",
        last.position.length()
    );
    assert!(last.velocity.length() < ARRIVAL_SPEED);
}

/// Where the orbit tests put the circle, and what they accept of a ship that has settled on it.
const ORBIT_RADIUS: f64 = 300.0;
const ORBIT_SECONDS: f64 = 160.0;
const ORBIT_SETTLED_S: f64 = 100.0;
const ORBIT_RADIAL_TOLERANCE: f64 = 15.0;
const ORBIT_HEIGHT_TOLERANCE: f64 = 5.0;

/// Orbits a centre that moves at `centre_velocity` from `start`, and checks, relative to the centre,
/// that the ship ends on the circle in its plane, going anticlockwise about +y at the circle speed.
fn assert_settles_on_orbit(start: DVec3, centre_velocity: DVec3) {
    let mut ships = [Ship::new(start)];
    let samples = fly_fleet(&mut ships, ORBIT_SECONDS, |_, time, _| Behaviour::Orbit {
        centre: target_body(centre_velocity * time, centre_velocity, 0.0),
        radius: ORBIT_RADIUS,
        normal: DVec3::Y,
        arrival_radius: ARRIVAL_RADIUS,
    });
    let circle_speed = 0.5 * PROFILE.accel_forward / PROFILE.linear_drag;
    let mut turned = 0.0;
    let mut previous: Option<DVec3> = None;
    for sample in samples[0].iter().filter(|s| s.time > ORBIT_SETTLED_S) {
        let offset = sample.position - centre_velocity * sample.time;
        let relative = sample.velocity - centre_velocity;
        let ring = DVec3::new(offset.x, 0.0, offset.z).length();
        assert!(
            (ring - ORBIT_RADIUS).abs() < ORBIT_RADIAL_TOLERANCE,
            "t = {}: {ring} from the axis",
            sample.time
        );
        assert!(
            offset.y.abs() < ORBIT_HEIGHT_TOLERANCE,
            "{} off the plane",
            offset.y
        );
        let speed = relative.length();
        assert!(
            (0.8 * circle_speed..1.2 * circle_speed).contains(&speed),
            "t = {}: speed {speed} against {circle_speed}",
            sample.time
        );
        if let Some(before) = previous {
            // Anticlockwise about +y is a positive turn about it, from +x toward -z.
            turned += before.cross(offset).y;
        }
        previous = Some(offset);
    }
    assert!(turned > 0.0, "goes the wrong way round");
}

#[test]
fn an_orbiting_ship_settles_on_the_circle_from_outside() {
    assert_settles_on_orbit(DVec3::new(3.0 * ORBIT_RADIUS, 0.0, 0.0), DVec3::ZERO);
}

#[test]
fn an_orbiting_ship_settles_on_the_circle_from_inside() {
    assert_settles_on_orbit(DVec3::new(0.3 * ORBIT_RADIUS, 0.0, 40.0), DVec3::ZERO);
}

#[test]
fn an_orbiting_ship_settles_on_the_circle_from_above_the_plane() {
    assert_settles_on_orbit(DVec3::new(ORBIT_RADIUS, 500.0, 0.0), DVec3::ZERO);
}

#[test]
fn an_orbiting_ship_follows_a_moving_centre() {
    assert_settles_on_orbit(
        DVec3::new(2.0 * ORBIT_RADIUS, 100.0, 0.0),
        DVec3::new(10.0, 0.0, 5.0),
    );
}

/// Where the head-on ships start, along z on either side of the origin, and fly to the other's.
const HEAD_ON_HALF_LENGTH: f64 = 400.0;

#[test]
fn ships_flying_head_on_pass_without_touching_and_arrive() {
    const SECONDS: f64 = 40.0;
    let ends = [
        DVec3::new(0.0, 0.0, HEAD_ON_HALF_LENGTH),
        DVec3::new(0.0, 0.0, -HEAD_ON_HALF_LENGTH),
    ];
    let mut ships = [
        Ship::new(ends[0]),
        Ship {
            orientation: DQuat::from_rotation_y(std::f64::consts::PI),
            ..Ship::new(ends[1])
        },
    ];
    let samples = fly_fleet(&mut ships, SECONDS, |index, _, _| {
        arrive_at_point(ends[1 - index])
    });
    let nearest = samples[0]
        .iter()
        .zip(&samples[1])
        .map(|(first, second)| first.position.distance(second.position))
        .fold(f64::INFINITY, f64::min);
    assert!(
        nearest > 2.0 * AGENT_RADIUS,
        "the ships touched: {nearest} apart"
    );
    for (index, ship) in ships.iter().enumerate() {
        let left = ship.position.distance(ends[1 - index]);
        assert!(left < ARRIVAL_RADIUS, "ship {index} stopped {left} short");
    }
    for (index, ship) in ships.iter().enumerate() {
        assert!(ship.sway > 0.0, "ship {index} didn't turn aside");
    }
}

#[test]
fn a_follower_closing_on_a_slot_just_behind_its_leader_reaches_it_without_touching() {
    // The slot is close behind the hull, inside the reach of a ship that closes in at speed.
    const SLOT_BEHIND: f64 = 4.3;
    const SLOT_TOLERANCE: f64 = 0.5;
    const SECONDS: f64 = 40.0;
    const START_BEHIND: f64 = 300.0;
    let mut ships = [Ship::new(DVec3::ZERO), Ship::new(DVec3::Z * START_BEHIND)];
    let samples = fly_fleet_tracking(
        &mut ships,
        SECONDS,
        |index, _, ships| {
            if index == 0 {
                return arrive_at_point(DVec3::ZERO);
            }
            Behaviour::Follow {
                leader: ships[0].body(),
                offset: FrameOffset {
                    right: 0.0,
                    up: 0.0,
                    forward: -SLOT_BEHIND,
                },
                arrival_radius: ARRIVAL_RADIUS,
            }
        },
        |index| (index == 1).then_some(0),
    );
    let nearest = samples[0]
        .iter()
        .zip(&samples[1])
        .map(|(leader, follower)| leader.position.distance(follower.position))
        .fold(f64::INFINITY, f64::min);
    assert!(
        nearest > 2.0 * AGENT_RADIUS,
        "came within {nearest} of the leader"
    );
    let slot = ships[0].position + DVec3::Z * SLOT_BEHIND;
    let stopped = ships[1].position.distance(slot);
    assert!(
        stopped < SLOT_TOLERANCE,
        "the follower stopped {stopped} from its slot"
    );
}
