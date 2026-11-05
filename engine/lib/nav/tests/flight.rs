//! Closed-loop checks: agents flown by the control axes through a simple model of the ship.
//!
//! The model is a point mass with linear drag, thrust along its own axes, and turn rates
//! proportional to the pitch and yaw axes. It stands in for the physics engine, so the tests
//! check that behaviour, avoidance and controller work together, not the ship's real response.

use glam::{DQuat, DVec3};
use nav::{AXIS_COUNT, Axis, Body, Frame, Goal, Obstacle, Profile, World};

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
}

impl Ship {
    fn new(position: DVec3) -> Self {
        Self {
            position,
            velocity: DVec3::ZERO,
            orientation: DQuat::IDENTITY,
            angular_velocity: DVec3::ZERO,
            clearance: f64::INFINITY,
            start: position,
            range: 0.0,
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
            radius: AGENT_RADIUS,
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
    world.set_goal(
        id,
        Some(Goal {
            target,
            arrival_radius: ARRIVAL_RADIUS,
        }),
    );
    let mut obstacles = obstacles.to_vec();

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
