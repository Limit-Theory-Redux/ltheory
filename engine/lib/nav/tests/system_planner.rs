//! Checks of the system planner kernel (`script/Shared/Helpers/SystemPlanner.lua`), run through
//! the kernel harness. A Rust oracle of the orbits judges the paths: a plan that reports `Ok`
//! must keep clear of every body's safe sphere at the time the ship is there.

mod common;

use common::KernelVm;
use mlua::{Function, Table, Value};

const BODY_STRIDE: usize = 10;
const OUT_STRIDE: usize = 4;
const STATUS_OK: i64 = 0;
const STATUS_DEPTH_EXCEEDED: i64 = 1;
const STATUS_CAPACITY_EXCEEDED: i64 = 2;
const STATUS_ENDPOINT_INSIDE_BODY: i64 = 3;
const STATUS_INVALID_INPUT: i64 = 4;
const STATUS_NO_DETOUR: i64 = 5;

const SPEED: f64 = 1000.0;
const DEPTH: i64 = 4;
const CAPACITY: usize = 64;
/// Samples per leg when a path is checked against the bodies.
const CLEARANCE_SAMPLES: usize = 1500;
/// A path may enter a safe sphere this fraction of its radius: the kernel's search skips a graze
/// shallower than 1 %, and the clearance samples are discrete.
const CLEARANCE_TOLERANCE: f64 = 0.97;

type Vec3 = [f64; 3];

#[derive(Clone, Copy)]
struct Body {
    parent: i32,
    position: Vec3,
    orbit_radius: f64,
    phase: f64,
    angular_speed: f64,
    inclination: f64,
    radius: f64,
    safe: f64,
}

/// A body that stays where it is, with a safe radius of `safe`.
fn fixed(position: Vec3, radius: f64, safe: f64) -> Body {
    Body {
        parent: -1,
        position,
        orbit_radius: 0.0,
        phase: 0.0,
        angular_speed: 0.0,
        inclination: 0.0,
        radius,
        safe,
    }
}

/// A body on a circle around `parent` (-1 for its own position, the origin here).
fn orbiting(parent: i32, orbit_radius: f64, phase: f64, angular_speed: f64, radius: f64) -> Body {
    Body {
        parent,
        position: [0.0; 3],
        orbit_radius,
        phase,
        angular_speed,
        inclination: 0.0,
        radius,
        safe: radius * 1.5,
    }
}

/// The oracle: where body `i` is `t` seconds from the snapshot.
fn body_position(bodies: &[Body], i: usize, t: f64) -> Vec3 {
    let body = &bodies[i];
    if body.orbit_radius <= 0.0 {
        return body.position;
    }
    let parent = if body.parent >= 0 {
        body_position(bodies, body.parent as usize, t)
    } else {
        body.position
    };
    let angle = body.phase + body.angular_speed * t;
    let x = angle.cos() * body.orbit_radius;
    let z = angle.sin() * body.orbit_radius;
    [
        parent[0] + x,
        parent[1] + z * body.inclination.sin(),
        parent[2] + z * body.inclination.cos(),
    ]
}

fn flatten(bodies: &[Body]) -> Vec<f64> {
    let mut values = Vec::with_capacity(bodies.len() * BODY_STRIDE);
    for b in bodies {
        values.extend_from_slice(&[
            f64::from(b.parent),
            b.position[0],
            b.position[1],
            b.position[2],
            b.orbit_radius,
            b.phase,
            b.angular_speed,
            b.inclination,
            b.radius,
            b.safe,
        ]);
    }
    values
}

fn distance(a: Vec3, b: Vec3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

struct Plan {
    status: i64,
    start: Vec3,
    points: Vec<Vec3>,
    times: Vec<f64>,
}

impl Plan {
    /// The corners of the path, the start first.
    fn corners(&self) -> Vec<Vec3> {
        std::iter::once(self.start)
            .chain(self.points.iter().copied())
            .collect()
    }

    /// Waypoints before the goal.
    fn detours(&self) -> usize {
        self.points.len().saturating_sub(1)
    }
}

struct Planner {
    vm: KernelVm,
    kernel: Table,
}

impl Planner {
    fn new() -> Self {
        let vm = KernelVm::new(None).expect("create the kernel state");
        let kernel = vm
            .require("Shared.Helpers.SystemPlanner")
            .expect("load the kernel");
        Self { vm, kernel }
    }

    fn run(
        &self,
        bodies: &[Body],
        start: Vec3,
        goal: Vec3,
        speed: f64,
        depth: i64,
        capacity: usize,
    ) -> Plan {
        let flat = flatten(bodies);
        // A zero-length buffer cannot be built, so an empty system gets one unused element.
        let body_buffer: Value = self
            .vm
            .buffer(if flat.is_empty() { &[0.0] } else { &flat })
            .unwrap();
        let out_buffer: Value = self
            .vm
            .buffer(&vec![0.0; capacity.max(1) * OUT_STRIDE])
            .unwrap();
        let (status, count): (f64, f64) = self
            .kernel
            .get::<Function>("plan")
            .unwrap()
            .call((
                &body_buffer,
                bodies.len(),
                start[0],
                start[1],
                start[2],
                goal[0],
                goal[1],
                goal[2],
                speed,
                depth,
                &out_buffer,
                capacity,
            ))
            .expect("run the kernel");
        let count = count as usize;
        let values = self
            .vm
            .read_buffer(&out_buffer, capacity.max(1) * OUT_STRIDE)
            .unwrap();
        let mut points = Vec::new();
        let mut times = Vec::new();
        for waypoint in values.chunks(OUT_STRIDE).take(count) {
            points.push([waypoint[0], waypoint[1], waypoint[2]]);
            times.push(waypoint[3]);
        }
        Plan {
            status: status as i64,
            start,
            points,
            times,
        }
    }

    fn plan(&self, bodies: &[Body], start: Vec3, goal: Vec3) -> Plan {
        self.run(bodies, start, goal, SPEED, DEPTH, CAPACITY)
    }
}

/// The times are the path length so far over the planning speed, and so strictly increase.
fn assert_times(plan: &Plan, speed: f64) {
    let corners = plan.corners();
    let mut length = 0.0;
    let mut previous = 0.0;
    for (i, time) in plan.times.iter().enumerate() {
        length += distance(corners[i], corners[i + 1]);
        assert!(
            (time - length / speed).abs() <= 1e-6 * (1.0 + time.abs()),
            "time {time} against {}",
            length / speed
        );
        assert!(*time >= previous, "times must not decrease");
        previous = *time;
    }
}

/// The smallest ratio of distance to safe radius between the ship and body `i` along the path.
fn min_clearance_ratio(plan: &Plan, bodies: &[Body], i: usize, speed: f64) -> f64 {
    let corners = plan.corners();
    let mut time = 0.0;
    let mut smallest = f64::INFINITY;
    for leg in corners.windows(2) {
        let length = distance(leg[0], leg[1]);
        for step in 0..=CLEARANCE_SAMPLES {
            let fraction = step as f64 / CLEARANCE_SAMPLES as f64;
            let position = [
                leg[0][0] + (leg[1][0] - leg[0][0]) * fraction,
                leg[0][1] + (leg[1][1] - leg[0][1]) * fraction,
                leg[0][2] + (leg[1][2] - leg[0][2]) * fraction,
            ];
            let at = time + length * fraction / speed;
            let ratio = distance(position, body_position(bodies, i, at)) / bodies[i].safe;
            smallest = smallest.min(ratio);
        }
        time += length / speed;
    }
    smallest
}

fn assert_clear(plan: &Plan, bodies: &[Body], speed: f64) {
    for i in 0..bodies.len() {
        let ratio = min_clearance_ratio(plan, bodies, i, speed);
        assert!(
            ratio >= CLEARANCE_TOLERANCE,
            "body {i}: path enters the safe sphere to {ratio} of its radius"
        );
    }
}

#[test]
fn a_clear_line_is_the_goal_alone() {
    let planner = Planner::new();
    let bodies = [fixed([0.0, 0.0, 9000.0], 500.0, 750.0)];
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[10000.0, 0.0, 0.0]]);
    assert!((plan.times[0] - 20.0).abs() < 1e-9);
}

#[test]
fn no_bodies_is_the_goal_alone() {
    let plan = Planner::new().plan(&[], [0.0; 3], [3000.0, 4000.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[3000.0, 4000.0, 0.0]]);
    assert!((plan.times[0] - 5.0).abs() < 1e-9);
}

#[test]
fn start_equal_to_goal_is_the_goal_alone() {
    let plan = Planner::new().plan(&[], [5.0; 3], [5.0; 3]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[5.0; 3]]);
    assert_eq!(plan.times, vec![0.0]);
}

#[test]
fn a_sphere_on_the_line_gets_one_symmetric_detour() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let start = [-10000.0, 0.0, 0.0];
    let goal = [10000.0, 0.0, 0.0];
    let plan = planner.plan(&bodies, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let waypoint = plan.points[0];
    assert!(waypoint[0].abs() < 1.0, "symmetric: {waypoint:?}");
    let off_line = (waypoint[1].powi(2) + waypoint[2].powi(2)).sqrt();
    assert!(off_line > 1500.0, "outside the safe sphere: {off_line}");
    assert_eq!(plan.points[1], goal);
    assert!(plan.times[1] > 20.0, "the detour is longer than the line");
    assert_times(&plan, SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_sphere_on_a_line_along_any_axis_is_cleared() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    for (start, goal) in [
        ([0.0, -9000.0, 0.0], [0.0, 12000.0, 0.0]),
        ([0.0, 0.0, 9000.0], [0.0, 0.0, -9000.0]),
        ([-5000.0, -5000.0, -5000.0], [5000.0, 5000.0, 5000.0]),
    ] {
        let plan = planner.plan(&bodies, start, goal);
        assert_eq!(plan.status, STATUS_OK, "{start:?} to {goal:?}");
        assert_eq!(plan.detours(), 1);
        assert_times(&plan, SPEED);
        assert_clear(&plan, &bodies, SPEED);
    }
}

#[test]
fn a_line_that_only_grazes_the_sphere_is_still_cleared() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.plan(&bodies, [-10000.0, 1400.0, 0.0], [10000.0, 1400.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_body_on_the_line_now_that_will_have_left_costs_no_detour() {
    let planner = Planner::new();
    // On the line at the start, 10 km ahead; it circles off before the ship gets there.
    let moving = [orbiting(-1, 10000.0, 0.0, 0.1, 400.0)];
    let plan = planner.plan(&moving, [-20000.0, 0.0, 0.0], [20000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points.len(), 1, "no detour: {:?}", plan.points);
    assert_clear(&plan, &moving, SPEED);

    // The same body held still where it is now is in the way.
    let still = [fixed([10000.0, 0.0, 0.0], 400.0, 600.0)];
    let plan = planner.plan(&still, [-20000.0, 0.0, 0.0], [20000.0, 0.0, 0.0]);
    assert_eq!(plan.detours(), 1);
}

#[test]
fn a_body_off_the_line_now_that_will_be_on_it_costs_a_detour() {
    let planner = Planner::new();
    // A moon of a small planet beside the line: it crosses the origin 20 s from now, when the
    // ship (starting at x = -20 km at 1 km/s) is there.
    let angular_speed = 0.02;
    let planet = [-10000.0, 0.0, 8000.0];
    let orbit_radius = (10000.0f64.powi(2) + 8000.0f64.powi(2)).sqrt();
    let phase = (-8000.0f64).atan2(10000.0) - angular_speed * 20.0;
    let moon = orbiting(0, orbit_radius, phase, angular_speed, 600.0);
    let system = [fixed(planet, 10.0, 10.0), moon];
    let start = [-20000.0, 0.0, 0.0];
    let goal = [20000.0, 0.0, 0.0];
    let plan = planner.plan(&system, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1, "the moon is in the way at t = 20");
    assert_times(&plan, SPEED);
    assert_clear(&plan, &system, SPEED);

    // Frozen at its position now, the moon is nowhere near the line: no detour.
    let mut frozen = system;
    frozen[1] = fixed(body_position(&system, 1, 0.0), 600.0, 900.0);
    let plan = planner.plan(&frozen, start, goal);
    assert_eq!(plan.points.len(), 1);
}

#[test]
fn a_moon_follows_a_moving_parent() {
    let planner = Planner::new();
    let system = [
        orbiting(-1, 20000.0, 0.5, 0.01, 1500.0),
        orbiting(0, 4000.0, 1.0, 0.2, 300.0),
    ];
    // Aim at the moon where it will be 40 s from now.
    let target = body_position(&system, 1, 40.0);
    let start = [target[0] - 40000.0, target[1], target[2]];
    let goal = [target[0] + 40000.0, target[1], target[2]];
    let plan = planner.plan(&system, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1);
    assert_times(&plan, SPEED);
    assert_clear(&plan, &system, SPEED);
}

#[test]
fn an_endpoint_inside_a_safe_sphere_shrinks_it() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 2000.0)];

    // The start is inside the safe sphere (1.5 km from the centre) and outside the hull.
    let plan = planner.plan(&bodies, [-1500.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let shrunk = 1500.0 * 0.99 / bodies[0].safe;
    assert!(min_clearance_ratio(&plan, &bodies, 0, SPEED) >= shrunk * CLEARANCE_TOLERANCE);

    // The goal is inside it, on the far side.
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [1500.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert!(min_clearance_ratio(&plan, &bodies, 0, SPEED) >= shrunk * CLEARANCE_TOLERANCE);
}

#[test]
fn an_endpoint_inside_a_hull_is_an_error() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.plan(&bodies, [500.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
    assert!(plan.points.is_empty());
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [0.0, 999.0, 0.0]);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
}

#[test]
fn the_depth_limit_is_reported_and_the_leg_left_straight() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.run(
        &bodies,
        [-10000.0, 0.0, 0.0],
        [10000.0, 0.0, 0.0],
        SPEED,
        0,
        CAPACITY,
    );
    assert_eq!(plan.status, STATUS_DEPTH_EXCEEDED);
    assert_eq!(plan.points, vec![[10000.0, 0.0, 0.0]]);
}

#[test]
fn a_full_output_buffer_is_reported() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.run(
        &bodies,
        [-10000.0, 0.0, 0.0],
        [10000.0, 0.0, 0.0],
        SPEED,
        DEPTH,
        1,
    );
    assert_eq!(plan.status, STATUS_CAPACITY_EXCEEDED);
    assert_eq!(plan.points.len(), 1);
}

#[test]
fn a_body_that_arrives_on_the_goal_in_time_is_an_error() {
    let planner = Planner::new();
    // The goal is outside the body now, but the body is on it when the straight flight ends.
    let moon = orbiting(-1, 10000.0, 0.0, 0.05, 500.0);
    let goal = body_position(&[moon], 0, 20.0);
    let start = [goal[0] - 20000.0, goal[1], goal[2]];
    let plan = planner.plan(&[moon], start, goal);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
}

#[test]
fn a_body_that_crosses_the_leg_is_passed_behind() {
    let planner = Planner::new();
    // The body crosses the line at the origin going toward +z; the ship passes on the -z side.
    let moon = orbiting(-1, 10000.0, -0.05 * 20.0, 0.05, 600.0);
    let crossing = body_position(&[moon], 0, 20.0);
    let start = [crossing[0] - 20000.0, crossing[1], crossing[2]];
    let goal = [crossing[0] + 20000.0, crossing[1], crossing[2]];
    let plan = planner.plan(&[moon], start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let waypoint = plan.points[0];
    let body_then = body_position(&[moon], 0, plan.times[0]);
    assert!(
        distance(waypoint, body_then) > moon.safe,
        "the waypoint is clear of the body when the ship gets there"
    );
    assert_clear(&plan, &[moon], SPEED);
}

#[test]
fn bad_input_is_rejected() {
    let planner = Planner::new();
    let good = [fixed([0.0, 0.0, 5000.0], 100.0, 150.0)];
    let start = [-1000.0, 0.0, 0.0];
    let goal = [1000.0, 0.0, 0.0];

    let rejected = |plan: Plan, what: &str| {
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "{what}");
        assert!(plan.points.is_empty(), "{what}");
    };
    rejected(
        planner.run(&good, start, goal, 0.0, DEPTH, CAPACITY),
        "zero speed",
    );
    rejected(
        planner.run(&good, start, goal, SPEED, -1, CAPACITY),
        "negative depth",
    );
    rejected(
        planner.run(&good, start, goal, SPEED, DEPTH, 0),
        "no capacity",
    );
    rejected(planner.plan(&good, [f64::NAN, 0.0, 0.0], goal), "NaN start");
    rejected(
        planner.plan(&good, start, [0.0, f64::INFINITY, 0.0]),
        "infinite goal",
    );

    let mut bad = good;
    bad[0].safe = 50.0;
    rejected(
        planner.plan(&bad, start, goal),
        "safe radius under the radius",
    );
    let mut bad = good;
    bad[0].radius = 0.0;
    rejected(planner.plan(&bad, start, goal), "zero radius");
    let mut bad = good;
    bad[0].parent = 0;
    rejected(
        planner.plan(&bad, start, goal),
        "a body that is its own parent",
    );
    let mut bad = good;
    bad[0].phase = f64::NAN;
    rejected(planner.plan(&bad, start, goal), "NaN phase");
}

/// A small seeded generator (xorshift64*): the tests need a repeatable spread, not quality.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let bits = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11;
        bits as f64 / (1u64 << 53) as f64
    }

    fn range(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.next()
    }

    fn point(&mut self, half_extent: f64) -> Vec3 {
        [
            self.range(-half_extent, half_extent),
            self.range(-half_extent, half_extent),
            self.range(-half_extent, half_extent),
        ]
    }
}

/// Counts of what the random cases produced.
#[derive(Default, Debug)]
struct Tally {
    cases: usize,
    ok: usize,
    detoured: usize,
    multiple_detours: usize,
    depth_exceeded: usize,
    no_detour: usize,
}

impl Tally {
    fn record(&mut self, plan: &Plan) {
        self.cases += 1;
        if plan.status == STATUS_OK {
            self.ok += 1;
            if plan.detours() >= 1 {
                self.detoured += 1;
            }
            if plan.detours() >= 2 {
                self.multiple_detours += 1;
            }
        } else if plan.status == STATUS_DEPTH_EXCEEDED {
            self.depth_exceeded += 1;
        } else if plan.status == STATUS_NO_DETOUR {
            self.no_detour += 1;
        }
    }
}

/// Whether `point` is outside every body's safe sphere (times of 0 and `at`), by a margin.
fn clear_of_all(bodies: &[Body], point: Vec3, at: f64) -> bool {
    const MARGIN: f64 = 1.2;
    (0..bodies.len())
        .all(|i| distance(point, body_position(bodies, i, at)) > bodies[i].safe * MARGIN)
}

#[test]
fn random_static_spheres_are_cleared_or_the_status_says_why() {
    const CASES: usize = 400;
    const EXTENT: f64 = 20000.0;
    let planner = Planner::new();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut tally = Tally::default();
    while tally.cases < CASES {
        let bodies: Vec<Body> = (0..20)
            .map(|_| {
                let radius = rng.range(1000.0, 3000.0);
                fixed(rng.point(EXTENT), radius, radius * 1.5)
            })
            .collect();
        let start = rng.point(EXTENT);
        let goal = rng.point(EXTENT);
        if !clear_of_all(&bodies, start, 0.0) || !clear_of_all(&bodies, goal, 0.0) {
            continue;
        }
        let plan = planner.plan(&bodies, start, goal);
        assert!(
            matches!(
                plan.status,
                STATUS_OK | STATUS_DEPTH_EXCEEDED | STATUS_NO_DETOUR
            ),
            "status {}",
            plan.status
        );
        assert!(!plan.points.is_empty());
        assert_eq!(*plan.points.last().unwrap(), goal);
        assert_times(&plan, SPEED);
        if plan.status == STATUS_OK {
            assert_clear(&plan, &bodies, SPEED);
        }
        tally.record(&plan);
    }
    println!("static: {tally:?}");
    assert!(tally.ok * 10 >= CASES * 7, "most cases plan: {tally:?}");
    assert!(tally.detoured >= 40, "detours occur: {tally:?}");
    assert!(
        tally.multiple_detours >= 5,
        "nested detours occur: {tally:?}"
    );
}

#[test]
fn random_orbiting_systems_are_cleared_or_the_status_says_why() {
    const CASES: usize = 200;
    const EXTENT: f64 = 25000.0;
    let planner = Planner::new();
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let mut tally = Tally::default();
    while tally.cases < CASES {
        let mut bodies: Vec<Body> = Vec::new();
        for _ in 0..8 {
            let radius = rng.range(1500.0, 4000.0);
            let mut planet = orbiting(
                -1,
                rng.range(8000.0, 20000.0),
                rng.range(0.0, std::f64::consts::TAU),
                rng.range(-0.05, 0.05),
                radius,
            );
            planet.inclination = rng.range(-0.5, 0.5);
            bodies.push(planet);
        }
        for planet in 0..2 {
            let radius = rng.range(150.0, 400.0);
            bodies.push(orbiting(
                planet,
                rng.range(3000.0, 5000.0),
                rng.range(0.0, std::f64::consts::TAU),
                rng.range(-0.2, 0.2),
                radius,
            ));
        }
        let start = rng.point(EXTENT);
        let goal = rng.point(EXTENT);
        let straight_time = distance(start, goal) / SPEED;
        if !clear_of_all(&bodies, start, 0.0) || !clear_of_all(&bodies, goal, straight_time) {
            continue;
        }
        let plan = planner.plan(&bodies, start, goal);
        assert!(!plan.points.is_empty());
        assert_times(&plan, SPEED);
        if plan.status == STATUS_OK {
            assert_clear(&plan, &bodies, SPEED);
        }
        tally.record(&plan);
    }
    println!("orbiting: {tally:?}");
    assert!(tally.ok * 10 >= CASES * 6, "most cases plan: {tally:?}");
    assert!(tally.detoured >= 20, "detours occur: {tally:?}");
}
