//! Checks of the system planner kernel (`script/Shared/Helpers/SystemPlanner.lua`), run through
//! the kernel harness. A Rust oracle of the orbits judges the paths: a plan that reports `Ok`
//! must keep clear of every body's safe sphere at the time the ship is there.

mod common;

use common::KernelVm;
use mlua::{Function, Table, Value};
use nav::{NO_SPEED_CAP, Profile, circle_speed};

const BODY_STRIDE: usize = 12;
/// A waypoint is `frame, x, y, z, t, radius, speed, drive, through`.
const OUT_STRIDE: usize = 9;
const SYSTEM_FRAME: i32 = -1;
const STATUS_OK: i64 = 0;
const STATUS_DEPTH_EXCEEDED: i64 = 1;
const STATUS_CAPACITY_EXCEEDED: i64 = 2;
const STATUS_ENDPOINT_INSIDE_BODY: i64 = 3;
const STATUS_INVALID_INPUT: i64 = 4;
const STATUS_NO_DETOUR: i64 = 5;
const STATUS_GOAL_OUTRUN: i64 = 6;
const STATUS_GOAL_NOT_SETTLED: i64 = 7;
const STATUS_DRIVE_NOT_TIMED: i64 = 8;

const SPEED: f64 = 1000.0;
/// A ship whose top speed, `accel / drag`, is the planning speed.
const SHIP: Ship = Ship {
    accel: 500.0,
    drag: 0.5,
};
const DEPTH: i64 = 4;
const CAPACITY: usize = 64;
/// Samples per leg when a path is checked against the bodies.
const CLEARANCE_SAMPLES: usize = 1500;
/// A path may enter a safe sphere this fraction of its radius: the kernel's search skips a graze
/// shallower than 1 %, and the clearance samples are discrete.
const CLEARANCE_TOLERANCE: f64 = 0.97;

type Vec3 = [f64; 3];

/// The ship's forward acceleration and linear drag, which set the corner speeds.
#[derive(Clone, Copy)]
struct Ship {
    accel: f64,
    drag: f64,
}

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
    /// The radius of the body's travel-drive zone, zero for none.
    zone_radius: f64,
    /// The drive multiplier at the body's centre.
    zone_cap: f64,
}

impl Body {
    /// The body with a travel-drive zone of `radius`, capping the drive at `cap` at its centre.
    fn with_zone(self, radius: f64, cap: f64) -> Self {
        Self {
            zone_radius: radius,
            zone_cap: cap,
            ..self
        }
    }
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
        zone_radius: 0.0,
        zone_cap: 0.0,
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
        zone_radius: 0.0,
        zone_cap: 0.0,
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
            b.zone_radius,
            b.zone_cap,
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
    /// The waypoints as world points: each frame's position at the waypoint's time plus its offset.
    points: Vec<Vec3>,
    times: Vec<f64>,
    /// The frame body of each waypoint, `SYSTEM_FRAME` for none.
    frames: Vec<i32>,
    /// The offset of each waypoint from its frame, as the kernel wrote it.
    offsets: Vec<Vec3>,
    /// The arc radius of each corner, zero for a sharp one and for the goal.
    radii: Vec<f64>,
    /// The speed each corner is passed at.
    speeds: Vec<f64>,
    /// Whether the leg that ends at each waypoint is flown under the travel drive.
    drives: Vec<bool>,
    /// Whether the drive goes on through each waypoint: the leg that ends there and the next one
    /// are one run.
    throughs: Vec<bool>,
}

/// The travel drive as the kernel takes it: the block of `SystemPlanner.Drive`.
#[derive(Clone, Copy)]
struct Drive {
    open_cap: f64,
    charge: f64,
    ramp_up: f64,
    ramp_down: f64,
    min_length: f64,
}

impl Drive {
    fn block(self) -> [f64; 5] {
        [
            self.open_cap,
            self.charge,
            self.ramp_up,
            self.ramp_down,
            self.min_length,
        ]
    }
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
        self.run_framed(
            bodies,
            start,
            SYSTEM_FRAME,
            goal,
            speed,
            SHIP,
            depth,
            capacity,
        )
    }

    /// Plans to `goal`, a point in the system for `SYSTEM_FRAME`, else an offset from that body.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the kernel's plan inputs"
    )]
    fn run_framed(
        &self,
        bodies: &[Body],
        start: Vec3,
        frame: i32,
        goal: Vec3,
        speed: f64,
        ship: Ship,
        depth: i64,
        capacity: usize,
    ) -> Plan {
        self.run_driven(
            bodies, start, frame, goal, speed, ship, depth, capacity, None,
        )
    }

    /// Plans as `run_framed` does, under the travel drive `drive` (none for no drive).
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the kernel's plan inputs"
    )]
    fn run_driven(
        &self,
        bodies: &[Body],
        start: Vec3,
        frame: i32,
        goal: Vec3,
        speed: f64,
        ship: Ship,
        depth: i64,
        capacity: usize,
        drive: Option<Drive>,
    ) -> Plan {
        let flat = flatten(bodies);
        let drive_buffer: Option<Value> = drive.map(|d| self.vm.buffer(&d.block()).unwrap());
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
                frame,
                goal[0],
                goal[1],
                goal[2],
                speed,
                ship.accel,
                ship.drag,
                depth,
                &out_buffer,
                capacity,
                drive_buffer.as_ref(),
            ))
            .expect("run the kernel");
        let count = count as usize;
        let values = self
            .vm
            .read_buffer(&out_buffer, capacity.max(1) * OUT_STRIDE)
            .unwrap();
        let mut plan = Plan {
            status: status as i64,
            start,
            points: Vec::new(),
            times: Vec::new(),
            frames: Vec::new(),
            offsets: Vec::new(),
            radii: Vec::new(),
            speeds: Vec::new(),
            drives: Vec::new(),
            throughs: Vec::new(),
        };
        for waypoint in values.chunks(OUT_STRIDE).take(count) {
            let frame = waypoint[0] as i32;
            let offset = [waypoint[1], waypoint[2], waypoint[3]];
            let time = waypoint[4];
            let centre = if frame == SYSTEM_FRAME {
                [0.0; 3]
            } else {
                body_position(bodies, frame as usize, time)
            };
            plan.points.push([
                centre[0] + offset[0],
                centre[1] + offset[1],
                centre[2] + offset[2],
            ]);
            plan.times.push(time);
            plan.frames.push(frame);
            plan.offsets.push(offset);
            plan.radii.push(waypoint[5]);
            plan.speeds.push(waypoint[6]);
            plan.drives.push(waypoint[7] != 0.0);
            plan.throughs.push(waypoint[8] != 0.0);
        }
        plan
    }

    fn plan(&self, bodies: &[Body], start: Vec3, goal: Vec3) -> Plan {
        self.run(bodies, start, goal, SPEED, DEPTH, CAPACITY)
    }

    fn plan_framed(&self, bodies: &[Body], start: Vec3, frame: i32, offset: Vec3) -> Plan {
        self.run_framed(bodies, start, frame, offset, SPEED, SHIP, DEPTH, CAPACITY)
    }
}

/// A goal on a moving body is planned for an arrival time that is right to within the kernel's
/// tolerance (`GOAL_TIME_TOLERANCE`), so the last time of such a plan may differ by that much.
const GOAL_TIME_SLACK: f64 = 1e-4;

/// The times are the path length so far over the planning speed, and so strictly increase.
fn assert_times(plan: &Plan, speed: f64) {
    let corners = plan.corners();
    let slack = if plan.frames.last() == Some(&SYSTEM_FRAME) {
        0.0
    } else {
        GOAL_TIME_SLACK
    };
    let mut length = 0.0;
    let mut previous = 0.0;
    for (i, time) in plan.times.iter().enumerate() {
        length += distance(corners[i], corners[i + 1]);
        assert!(
            (time - length / speed).abs() <= 1e-6 * (1.0 + time.abs()) + slack,
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

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scaled(a: Vec3, factor: f64) -> Vec3 {
    a.map(|component| component * factor)
}

fn plus(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn unit(a: Vec3) -> Vec3 {
    scaled(a, 1.0 / distance(a, [0.0; 3]))
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// An arc built as the oracle sees it: the circle of `radius` tangent to both legs, found from
/// the corner's bisector, and its points by spherical interpolation between the radii to the
/// entry and the exit. Independent of the kernel's construction.
struct ArcOracle {
    centre: Vec3,
    to_entry: Vec3,
    to_exit: Vec3,
    radius: f64,
    turn: f64,
    /// Distance from the corner to the entry and the exit.
    tangent: f64,
}

impl ArcOracle {
    fn new(previous: Vec3, corner: Vec3, next: Vec3, radius: f64) -> Self {
        let before = unit(sub(corner, previous));
        let after = unit(sub(next, corner));
        let turn = dot(before, after).clamp(-1.0, 1.0).acos();
        let half = turn / 2.0;
        let inside = unit(sub(after, before));
        let centre = plus(corner, scaled(inside, radius / half.cos()));
        let tangent = radius * half.tan();
        let entry = sub(corner, scaled(before, tangent));
        let exit = plus(corner, scaled(after, tangent));
        Self {
            centre,
            to_entry: scaled(sub(entry, centre), 1.0 / radius),
            to_exit: scaled(sub(exit, centre), 1.0 / radius),
            radius,
            turn,
            tangent,
        }
    }

    /// The point `fraction` (0 to 1) of the way along the arc.
    fn point(&self, fraction: f64) -> Vec3 {
        let weight_entry = ((1.0 - fraction) * self.turn).sin() / self.turn.sin();
        let weight_exit = (fraction * self.turn).sin() / self.turn.sin();
        plus(
            self.centre,
            scaled(
                plus(
                    scaled(self.to_entry, weight_entry),
                    scaled(self.to_exit, weight_exit),
                ),
                self.radius,
            ),
        )
    }
}

/// The arc of every corner that has one, with the time the ship reaches its entry at the
/// planning speed.
fn arcs(plan: &Plan, speed: f64) -> Vec<(usize, ArcOracle, f64)> {
    let corners = plan.corners();
    let mut found = Vec::new();
    for k in 0..plan.points.len().saturating_sub(1) {
        if plan.radii[k] > 0.0 {
            let arc = ArcOracle::new(corners[k], corners[k + 1], corners[k + 2], plan.radii[k]);
            let entry_time = plan.times[k] - arc.tangent / speed;
            found.push((k, arc, entry_time));
        }
    }
    found
}

/// The smallest ratio of distance to safe radius between the arcs of the plan and body `i`.
fn min_arc_clearance_ratio(plan: &Plan, bodies: &[Body], i: usize, speed: f64) -> f64 {
    let mut smallest = f64::INFINITY;
    for (_, arc, entry_time) in arcs(plan, speed) {
        for step in 0..=CLEARANCE_SAMPLES {
            let fraction = step as f64 / CLEARANCE_SAMPLES as f64;
            let at = entry_time + fraction * arc.radius * arc.turn / speed;
            let ratio =
                distance(arc.point(fraction), body_position(bodies, i, at)) / bodies[i].safe;
            smallest = smallest.min(ratio);
        }
    }
    smallest
}

/// The share of each leg that an arc's tangent length may take: the kernel's `ARC_LEG_SHARE`.
const ARC_LEG_SHARE: f64 = 0.5;
/// The kernel's `TURN_SPEED_KNEE`, the fraction of the turn-speed limit where the arc stops growing.
const TURN_SPEED_KNEE: f64 = 0.9;

/// The largest arc radius the legs and the turn-speed knee allow at corner `k`: where the kernel
/// starts before the bodies make it smaller. `share` is the kernel's turn share.
fn unconstrained_radius(plan: &Plan, k: usize, share: f64, ship: Ship) -> f64 {
    let corners = plan.corners();
    let before = unit(sub(corners[k + 1], corners[k]));
    let after = unit(sub(corners[k + 2], corners[k + 1]));
    let tan_half = (dot(before, after).clamp(-1.0, 1.0).acos() / 2.0).tan();
    let shorter =
        distance(corners[k], corners[k + 1]).min(distance(corners[k + 1], corners[k + 2]));
    let knee = TURN_SPEED_KNEE;
    let knee_radius =
        knee * knee * share * ship.accel / (ship.drag * ship.drag * (1.0 - knee * knee).sqrt());
    knee_radius.min(ARC_LEG_SHARE * shorter / tan_half)
}

/// Both the legs and the arcs of an `Ok` plan stay out of the safe spheres, and each arc sits
/// inside its corner: the entry and the exit lie on the legs, within their lengths.
fn assert_clear(plan: &Plan, bodies: &[Body], speed: f64) {
    for i in 0..bodies.len() {
        let ratio = min_clearance_ratio(plan, bodies, i, speed);
        assert!(
            ratio >= CLEARANCE_TOLERANCE,
            "body {i}: path enters the safe sphere to {ratio} of its radius"
        );
        let ratio = min_arc_clearance_ratio(plan, bodies, i, speed);
        assert!(
            ratio >= CLEARANCE_TOLERANCE,
            "body {i}: an arc enters the safe sphere to {ratio} of its radius"
        );
    }
    let corners = plan.corners();
    for (k, arc, _) in arcs(plan, speed) {
        let shorter =
            distance(corners[k], corners[k + 1]).min(distance(corners[k + 1], corners[k + 2]));
        assert!(
            arc.tangent <= shorter * ARC_LEG_SHARE * (1.0 + 1e-9),
            "arc {k}: tangent {} against legs of at least {}",
            arc.tangent,
            shorter
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

/// The profile of a ship with the forward acceleration and drag of `ship`, for `nav`'s own
/// turn model.
fn profile_of(ship: Ship) -> Profile {
    Profile {
        speed_cap: NO_SPEED_CAP,
        linear_drag: ship.drag,
        angular_drag: 0.0,
        accel_forward: ship.accel,
        accel_right: 0.0,
        accel_up: 0.0,
        turn_pitch: 0.0,
        turn_yaw: 0.0,
        turn_roll: 0.0,
    }
}

/// The kernel's constant `TURN_THRUST_SHARE`.
fn turn_share(planner: &Planner) -> f64 {
    planner.kernel.get::<f64>("TURN_THRUST_SHARE").unwrap()
}

#[test]
fn the_kernels_turn_speed_is_the_circle_speed_of_nav() {
    let planner = Planner::new();
    let share = turn_share(&planner);
    let turn_speed: Function = planner.kernel.get("turnSpeed").unwrap();
    for ship in [
        SHIP,
        Ship {
            accel: 17_600.0,
            drag: 2.0,
        },
        Ship {
            accel: 40.0,
            drag: 0.02,
        },
        // Without drag the top speed is infinite and the turn speed is `√(share · a · r)`.
        Ship {
            accel: 300.0,
            drag: 0.0,
        },
    ] {
        let profile = profile_of(ship);
        let top = if ship.drag > 0.0 {
            ship.accel / ship.drag
        } else {
            f64::INFINITY
        };
        // From a radius that the top speed caps to one that the thrust does.
        for radius in [0.0, 1.0, 30.0, 500.0, 4_000.0, 90_000.0, 5.0e6] {
            let kernel: f64 = turn_speed
                .call((ship.accel, ship.drag, radius, top))
                .unwrap();
            let expected = circle_speed(&profile, radius, share);
            assert!(
                (kernel - expected).abs() <= 1e-9 * (1.0 + expected),
                "radius {radius}, ship {} / {}: kernel {kernel}, nav {expected}",
                ship.accel,
                ship.drag
            );
        }
    }
}

#[test]
fn a_corner_gets_an_arc_the_ship_can_hold() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let radius = plan.radii[0];
    assert!(radius > 0.0, "the detour corner is rounded");
    // The arc is passed at the speed `nav` holds a circle of that radius at, below the planning
    // speed here.
    let held = circle_speed(&profile_of(SHIP), radius, turn_share(&planner));
    assert!(held < SPEED, "the turn is the limit: {held}");
    assert!((plan.speeds[0] - held).abs() <= 1e-9 * held);
    // The goal is where the ship stops.
    assert_eq!((plan.radii[1], plan.speeds[1]), (0.0, 0.0));
    assert_times(&plan, SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_corner_that_barely_turns_is_passed_at_the_planning_speed() {
    let planner = Planner::new();
    // A sphere that the line only just enters: the detour is a shallow bend.
    let bodies = [fixed([0.0, 1400.0, 0.0], 1000.0, 1500.0)];
    let plan = planner.run(
        &bodies,
        [-10000.0, 0.0, 0.0],
        [10000.0, 0.0, 0.0],
        SPEED,
        DEPTH,
        CAPACITY,
    );
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert!(plan.radii[0] > 0.0 || plan.speeds[0] == SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_plan_without_a_detour_has_no_corner() {
    let plan = Planner::new().plan(&[], [0.0; 3], [3000.0, 4000.0, 0.0]);
    assert_eq!(plan.radii, vec![0.0]);
    assert_eq!(plan.speeds, vec![0.0]);
}

#[test]
fn a_ship_without_drag_gets_an_arc_without_a_knee() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let ship = Ship {
        accel: 300.0,
        drag: 0.0,
    };
    let plan = planner.run_framed(
        &bodies,
        [-10000.0, 0.0, 0.0],
        SYSTEM_FRAME,
        [10000.0, 0.0, 0.0],
        SPEED,
        ship,
        DEPTH,
        CAPACITY,
    );
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.radii[0] > 0.0);
    let held = circle_speed(&profile_of(ship), plan.radii[0], turn_share(&planner));
    assert!((plan.speeds[0] - held.min(SPEED)).abs() <= 1e-9 * SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

const NARROW_START: Vec3 = [-3000.0, 0.0, 0.0];
const NARROW_GOAL: Vec3 = [3000.0, 0.0, 0.0];

/// A ship whose turns are wide: its full-size arc at the narrow detour's corner is larger than
/// the sphere's clearance from the corner.
const WIDE_TURNING_SHIP: Ship = Ship {
    accel: 5000.0,
    drag: 0.5,
};

/// One sphere of safe radius `safe` between a start and a goal, with one corner just outside it.
fn narrow_detour(planner: &Planner, safe: f64, ship: Ship) -> (Plan, Vec<Body>) {
    let bodies = vec![fixed([0.0; 3], safe / 1.5, safe)];
    let plan = planner.run_framed(
        &bodies,
        NARROW_START,
        SYSTEM_FRAME,
        NARROW_GOAL,
        SPEED,
        ship,
        DEPTH,
        CAPACITY,
    );
    (plan, bodies)
}

#[test]
fn an_arc_that_cuts_into_a_body_is_made_smaller() {
    let planner = Planner::new();
    let (plan, bodies) = narrow_detour(&planner, 600.0, WIDE_TURNING_SHIP);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let full = unconstrained_radius(&plan, 0, turn_share(&planner), WIDE_TURNING_SHIP);
    assert!(plan.radii[0] > 0.0, "a smaller arc is clear");
    assert!(plan.radii[0] < full, "the full arc was blocked");
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_corner_with_no_clear_arc_is_left_sharp() {
    let planner = Planner::new();
    let (plan, sphere) = narrow_detour(&planner, 600.0, SHIP);
    assert_eq!(plan.detours(), 1);
    // A body just inside the corner, as far from the legs as it is wide: the legs clear it by a
    // hair, and every arc, however small, passes closer.
    const INSIDE_DISTANCE: f64 = 50.0;
    const LEG_CLEARANCE: f64 = 1.005;
    let corner = plan.points[0];
    let to_start = unit(sub(NARROW_START, corner));
    let to_goal = unit(sub(NARROW_GOAL, corner));
    let inside = unit(plus(to_start, to_goal));
    let leg_distance = INSIDE_DISTANCE * (1.0 - dot(inside, to_start).powi(2)).sqrt();
    let blocker = fixed(
        plus(corner, scaled(inside, INSIDE_DISTANCE)),
        1.0,
        leg_distance / LEG_CLEARANCE,
    );
    let bodies = [sphere[0], blocker];
    let plan = planner.plan(&bodies, NARROW_START, NARROW_GOAL);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1, "the blocker leaves the legs alone");
    assert_eq!(
        (plan.radii[0], plan.speeds[0]),
        (0.0, 0.0),
        "the corner is sharp"
    );
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

#[test]
fn a_detour_waypoint_is_in_the_frame_of_the_body_it_goes_around() {
    let planner = Planner::new();
    // The moon of the earlier test: off the line now, on it at t = 20.
    let angular_speed = 0.02;
    let planet = [-10000.0, 0.0, 8000.0];
    let orbit_radius = (10000.0f64.powi(2) + 8000.0f64.powi(2)).sqrt();
    let phase = (-8000.0f64).atan2(10000.0) - angular_speed * 20.0;
    let moon = orbiting(0, orbit_radius, phase, angular_speed, 600.0);
    let system = [fixed(planet, 10.0, 10.0), moon];
    let goal = [20000.0, 0.0, 0.0];
    let plan = planner.plan(&system, [-20000.0, 0.0, 0.0], goal);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1);

    let (goal_frame, detour_frames) = plan.frames.split_last().unwrap();
    assert_eq!(*goal_frame, SYSTEM_FRAME, "the goal keeps its frame");
    assert_eq!(*plan.offsets.last().unwrap(), goal);
    assert!(
        detour_frames.iter().all(|frame| *frame == 1),
        "{detour_frames:?}"
    );
    // Whenever the ship gets there, a point at this offset is as far from the body as planned.
    for offset in &plan.offsets[..plan.detours()] {
        let length = distance(*offset, [0.0; 3]);
        assert!(
            length >= moon.safe * CLEARANCE_TOLERANCE,
            "offset {length} against the safe radius {}",
            moon.safe
        );
    }
}

#[test]
fn a_goal_on_an_orbiting_body_keeps_its_offset_and_meets_the_body() {
    let planner = Planner::new();
    let moon = orbiting(-1, 10000.0, 0.0, 0.05, 600.0);
    let offset = [3000.0, 500.0, 0.0];
    let start = [-20000.0, 0.0, 0.0];
    let plan = planner.plan_framed(&[moon], start, 0, offset);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.frames.last(), Some(&0));
    assert_eq!(plan.offsets.last(), Some(&offset));
    let goal = *plan.points.last().unwrap();
    assert_ne!(
        goal,
        [offset[0] + 10000.0, offset[1], offset[2]],
        "the body has moved on from where it is now"
    );
    // The times are the path's length over the speed, to the goal where the body is on arrival.
    assert_times(&plan, SPEED);
    assert_clear(&plan, &[moon], SPEED);
}

#[test]
fn a_goal_on_a_static_body_is_the_body_plus_the_offset() {
    let planner = Planner::new();
    let system = [fixed([5000.0, 0.0, 0.0], 500.0, 750.0)];
    let plan = planner.plan_framed(&system, [-5000.0, 0.0, 0.0], 0, [0.0, 2000.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[5000.0, 2000.0, 0.0]]);
    assert_eq!(plan.frames, vec![0]);
    let expected = distance([-5000.0, 0.0, 0.0], [5000.0, 2000.0, 0.0]) / SPEED;
    assert!((plan.times[0] - expected).abs() < 1e-9);
}

#[test]
fn a_goal_in_the_system_frame_keeps_its_world_point() {
    let planner = Planner::new();
    let goal = [3000.0, 4000.0, 0.0];
    let plan = planner.plan_framed(&[], [0.0; 3], SYSTEM_FRAME, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.frames, vec![SYSTEM_FRAME]);
    assert_eq!(plan.offsets, vec![goal]);
}

#[test]
fn a_detour_toward_a_moving_goal_is_planned_for_the_later_arrival() {
    let planner = Planner::new();
    // A sphere sits on the line to a goal beside a moon that circles it; the detour makes the
    // flight longer, so the moon is somewhere else when the ship gets there than at the
    // straight flight's end.
    let moon = orbiting(-1, 10000.0, -1.5, 0.05, 300.0);
    let system = [fixed([0.0; 3], 1000.0, 1500.0), moon];
    let start = [-20000.0, 0.0, 0.0];
    let plan = planner.plan_framed(&system, start, 1, [2000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    assert!(
        plan.detours() >= 1,
        "the sphere is in the way: {:?}",
        plan.points
    );
    let arrival = *plan.times.last().unwrap();
    let straight = distance(start, *plan.points.last().unwrap()) / SPEED;
    assert!(
        arrival > straight,
        "the detour is longer: {arrival} against {straight}"
    );
    assert_times(&plan, SPEED);
    assert_clear(&plan, &system, SPEED);
}

#[test]
fn a_goal_on_a_body_that_can_outrun_the_ship_is_an_error() {
    let planner = Planner::new();
    // The moon moves at 2 km/s along its orbit, the ship at 1 km/s.
    let moon = orbiting(-1, 10000.0, 0.0, 0.2, 300.0);
    let plan = planner.plan_framed(&[moon], [-20000.0, 0.0, 0.0], 0, [2000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
    assert!(plan.points.is_empty());
}

// ---- The goal body's gravity well ----

/// The radius of the well the planet in these tests carries ships in, and the share of it that the
/// entry waypoint lies at (the rule of `doc/script/system-planner.md`).
const WELL_RADIUS: f64 = 10000.0;
const WELL_ENTRY_SHARE: f64 = 0.95;
const WELL_ORBIT_RADIUS: f64 = 100000.0;
/// The drive multiplier at the planet's centre.
const WELL_ZONE_CAP: f64 = 2000.0;
const PLANET_RADIUS: f64 = 1000.0;

/// A planet on a circle round the origin at `speed`, with a well of `WELL_RADIUS`.
fn well_planet(speed: f64) -> Body {
    orbiting(
        -1,
        WELL_ORBIT_RADIUS,
        0.0,
        speed / WELL_ORBIT_RADIUS,
        PLANET_RADIUS,
    )
    .with_zone(WELL_RADIUS, WELL_ZONE_CAP)
}

/// The planner's answer for a goal at `offset` from the planet (body 0), under `drive`.
fn plan_in_well(
    planner: &Planner,
    planet: Body,
    start: Vec3,
    offset: Vec3,
    drive: Option<Drive>,
) -> Plan {
    planner.run_driven(
        &[planet],
        start,
        0,
        offset,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        drive,
    )
}

/// The distance from the origin to the segment `a`-`b`.
fn segment_distance_to_origin(a: Vec3, b: Vec3) -> f64 {
    let ab = sub(b, a);
    let along = (-dot(a, ab) / dot(ab, ab)).clamp(0.0, 1.0);
    distance(plus(a, scaled(ab, along)), [0.0; 3])
}

#[test]
fn a_body_faster_than_the_ship_is_reached_through_an_entry_in_its_well() {
    let planner = Planner::new();
    let planet = well_planet(3.0 * SPEED);
    let offset = [3000.0, 0.0, 500.0];
    let start = [-100000.0, 0.0, 50000.0];

    // Without a well the same planet outruns the ship, which is the failure this fixes.
    let bare = Body {
        zone_radius: 0.0,
        zone_cap: 0.0,
        ..planet
    };
    let plan = plan_in_well(&planner, bare, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);

    let plan = plan_in_well(&planner, planet, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    let n = plan.frames.len();
    assert_eq!(plan.frames[n - 1], 0, "the goal rides on the planet");
    assert_eq!(plan.offsets[n - 1], offset, "the goal keeps its offset");
    let e = entry_index(&plan).expect("the entry, in the planet's frame at the entry radius");
    assert_eq!(e, 0, "the system part is one leg");
    assert_eq!(plan.radii[e], 0.0, "the entry corner is not rounded");
    assert!(plan.speeds[e] > 0.0, "and not passed at rest");
    assert!(plan.drives[e], "the long leg to the well is driven");
    assert!(
        !plan.throughs[e],
        "the part in the well is shorter than the release distance, so the drive is released"
    );
    assert!(plan.times[e] > 0.0 && plan.times[n - 1] > plan.times[e]);
    // The entry is further than the planning speed gets in its time: the drive reached it.
    assert!(
        distance(start, plan.points[e]) > SPEED * plan.times[e],
        "the entry is further than the planning speed gets in its time"
    );
}

/// The waypoint at the entry radius in the planet's frame: the entry that the kernel placed.
fn entry_index(plan: &Plan) -> Option<usize> {
    entry_index_in(plan, WELL_RADIUS)
}

/// The waypoint at the entry radius of a well of `well_radius` in the planet's frame.
fn entry_index_in(plan: &Plan, well_radius: f64) -> Option<usize> {
    plan.frames
        .iter()
        .zip(&plan.offsets)
        .position(|(frame, offset)| {
            *frame == 0
                && (distance(*offset, [0.0; 3]) - well_radius * WELL_ENTRY_SHARE).abs()
                    <= 1e-9 * well_radius
        })
}

#[test]
fn a_ship_inside_the_goals_well_is_planned_in_the_frame_whatever_the_bodys_speed() {
    let planner = Planner::new();
    // The planet moves ten times the ship's speed, which the ship in its well does not feel.
    let planet = well_planet(10.0 * SPEED);
    let centre = body_position(&[planet], 0, 0.0);
    let start_offset = [-6000.0, 0.0, 2000.0];
    let start = plus(centre, start_offset);
    // The line from the start to the goal passes within the planet's safe sphere.
    let offset = [4000.0, 0.0, 0.0];
    let plan = plan_in_well(&planner, planet, start, offset, None);
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
    assert!(
        plan.frames.iter().all(|frame| *frame == 0),
        "{:?}",
        plan.frames
    );
    assert_eq!(plan.offsets.last(), Some(&offset));
    assert!(
        plan.detours() >= 1,
        "the planet is in the way: {:?}",
        plan.offsets
    );

    // The path is the same as for a planet that stands still: lengths, times and clearance are
    // those of the offsets.
    let mut corners = vec![start_offset];
    corners.extend(plan.offsets.iter().copied());
    let mut length = 0.0;
    for (k, time) in plan.times.iter().enumerate() {
        length += distance(corners[k], corners[k + 1]);
        assert_close(
            *time,
            length / SPEED,
            1e-6 * (1.0 + time) + GOAL_TIME_SLACK,
            "time",
        );
        assert!(
            segment_distance_to_origin(corners[k], corners[k + 1])
                >= planet.safe * CLEARANCE_TOLERANCE,
            "leg {k} cuts the planet's safe sphere"
        );
    }
}

#[test]
fn a_body_that_outruns_the_part_before_its_well_is_an_error() {
    let planner = Planner::new();
    let offset = [3000.0, 0.0, 500.0];
    let start = [-100000.0, 0.0, 50000.0];
    // Faster than the planning speed, with no drive to catch it.
    let plan = plan_in_well(&planner, well_planet(3.0 * SPEED), start, offset, None);
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
    assert!(plan.points.is_empty());
    // Faster than the drive's top speed.
    let fast = well_planet(2.0 * DRIVE.open_cap * SPEED);
    let plan = plan_in_well(&planner, fast, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
    assert!(plan.points.is_empty());
}

#[test]
fn a_part_before_the_well_that_is_too_short_to_drive_cannot_catch_a_fast_body() {
    let planner = Planner::new();
    let never = Drive {
        min_length: 1e9,
        ..DRIVE
    };
    let planet = well_planet(5.0 * SPEED);
    let start = [-100000.0, 0.0, 50000.0];
    let offset = [3000.0, 0.0, 500.0];
    // The part before the well is flown at the planning speed, which the planet outruns: the
    // arrival at the entry does not settle, and the ship cannot close on it.
    let plan = plan_in_well(&planner, planet, start, offset, Some(never));
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN, "{:?}", plan.offsets);
    assert!(plan.points.is_empty());
    // The drive that the same part is flown under catches it.
    let plan = plan_in_well(&planner, planet, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
}

#[test]
fn a_goal_outside_the_bodys_well_is_planned_as_before() {
    let planner = Planner::new();
    let start = [-100000.0, 0.0, 50000.0];
    // Beyond the well: nothing changes for a body that has one.
    let outside = [2.0 * WELL_RADIUS, 0.0, 0.0];
    let slow = well_planet(0.5 * SPEED);
    let bare = Body {
        zone_radius: 0.0,
        zone_cap: 0.0,
        ..slow
    };
    let with_well = plan_in_well(&planner, slow, start, outside, None);
    let without = plan_in_well(&planner, bare, start, outside, None);
    assert_eq!(with_well.status, STATUS_OK);
    assert_eq!(with_well.status, without.status);
    assert_eq!(with_well.frames, without.frames);
    assert_eq!(with_well.offsets, without.offsets);
    assert_eq!(with_well.times, without.times);
    // And a fast body still outruns the ship there.
    let plan = plan_in_well(
        &planner,
        well_planet(3.0 * SPEED),
        start,
        outside,
        Some(DRIVE),
    );
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
}

#[test]
fn a_slow_body_with_a_well_gets_the_same_entry_without_the_drive() {
    let planner = Planner::new();
    let planet = well_planet(0.4 * SPEED);
    let offset = [3000.0, 0.0, 500.0];
    let plan = plan_in_well(&planner, planet, [-100000.0, 0.0, 50000.0], offset, None);
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    let n = plan.frames.len();
    assert_eq!(plan.offsets[n - 1], offset);
    let e = entry_index(&plan).expect("the entry");
    assert_eq!(e, 0, "the system part is one leg");
    // The system part is flown at the planning speed, to where the planet is on entering.
    assert_close(
        distance([-100000.0, 0.0, 50000.0], plan.points[e]) / SPEED,
        plan.times[e],
        1e-6 * plan.times[e],
        "entry time",
    );
    // The part in the well is a line of offsets, timed as one.
    let mut length = 0.0;
    for k in e + 1..n {
        length += distance(plan.offsets[k - 1], plan.offsets[k]);
    }
    assert_close(
        length / SPEED,
        plan.times[n - 1] - plan.times[e],
        1e-6 * plan.times[n - 1] + GOAL_TIME_SLACK,
        "time in the well",
    );
}

// ---- A drive run through the well's entry ----

/// A well wider than the drive's release distance (`HANDOVER_MULT · braking time · SPEED`, about
/// 21 000 here), so that a run can reach a goal inside it.
const BIG_WELL_RADIUS: f64 = 200000.0;
const BIG_ORBIT_RADIUS: f64 = 5.0e6;

/// A planet that stands still, with the wide well.
fn standing_big_planet() -> Body {
    fixed([0.0; 3], PLANET_RADIUS, 1.5 * PLANET_RADIUS).with_zone(BIG_WELL_RADIUS, WELL_ZONE_CAP)
}

/// A planet on a circle round the origin at `speed`, with the wide well.
fn big_well_planet(speed: f64) -> Body {
    orbiting(
        -1,
        BIG_ORBIT_RADIUS,
        0.0,
        speed / BIG_ORBIT_RADIUS,
        PLANET_RADIUS,
    )
    .with_zone(BIG_WELL_RADIUS, WELL_ZONE_CAP)
}

/// A start far outside the standing planet's well, and a goal inside it on the near side, so that
/// the straight line meets the well's entry and goes on to the goal.
const RUN_START: Vec3 = [-2.0e6, 0.0, 5.0e5];
const RUN_GOAL_OFFSET: Vec3 = [-1.0e5, 0.0, 2.0e4];

/// The checks that a plan of `RUN_START` to `RUN_GOAL_OFFSET` under `drive` is one drive run, and
/// takes the time that one leg takes under the same drive.
fn assert_one_run_to_the_goal(drive: Drive) {
    let planner = Planner::new();
    let planet = standing_big_planet();
    let plan = plan_in_well(&planner, planet, RUN_START, RUN_GOAL_OFFSET, Some(drive));
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
    let n = plan.frames.len();
    let e = entry_index_in(&plan, BIG_WELL_RADIUS).expect("the entry");
    assert_eq!((e, n), (0, 2), "a system leg and a framed leg");
    assert_eq!(
        plan.throughs,
        vec![true, false],
        "the run goes through the entry"
    );
    assert!(plan.drives[e] && plan.drives[n - 1], "both legs are driven");
    assert!(
        plan.speeds[e] > SPEED,
        "the entry is passed at the run's speed: {}",
        plan.speeds[e]
    );
    // One leg from the start to the goal, timed again from the oracle's own integration, takes the
    // time of the run. A second charge, or a release at the entry, would add much more.
    let along = drive_leg(&[planet], RUN_START, RUN_GOAL_OFFSET, 0.0, drive);
    assert_close(
        plan.times[n - 1],
        along.last().unwrap().0,
        DRIVE_TIME_TOLERANCE,
        "arrival",
    );
    let framed = plan.times[n - 1] - plan.times[e];
    let framed_length = distance(plan.offsets[e], RUN_GOAL_OFFSET);
    assert!(
        framed < 0.5 * framed_length / SPEED,
        "the framed leg is driven: {framed} s over {framed_length}"
    );
}

#[test]
fn a_drive_run_goes_on_through_the_entry_to_the_goal() {
    assert_one_run_to_the_goal(DRIVE);
}

#[test]
fn the_framed_leg_of_a_run_has_no_minimum_length_and_no_charge() {
    // The framed leg is about 90 000 long and the system leg is about 1.8 million.
    assert_one_run_to_the_goal(Drive {
        min_length: 5.0e5,
        ..DRIVE
    });
}

#[test]
fn a_detour_in_the_well_ends_the_run_at_the_entry() {
    let planner = Planner::new();
    // A rock on the line, inside the well and past the entry.
    let rock = fixed([-1.5e5, 0.0, 32632.0], 2000.0, 3000.0);
    let bodies = [standing_big_planet(), rock];
    let plan = planner.run_driven(
        &bodies,
        RUN_START,
        0,
        RUN_GOAL_OFFSET,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(DRIVE),
    );
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
    assert!(
        plan.detours() >= 2,
        "the entry and a detour: {:?}",
        plan.offsets
    );
    assert!(
        plan.throughs.iter().all(|through| !through),
        "{:?}",
        plan.throughs
    );
    let e = entry_index_in(&plan, BIG_WELL_RADIUS).expect("the entry");
    assert_eq!(e, 0, "the system part is one leg");
    assert_eq!(
        plan.speeds[e], SPEED,
        "the entry is passed at the planning speed"
    );
    // The system leg is released before the entry, as a leg that ends there always was.
    let released = drive_leg(&bodies, RUN_START, plan.points[e], 0.0, DRIVE);
    assert_close(
        plan.times[e],
        released.last().unwrap().0,
        DRIVE_TIME_TOLERANCE,
        "time at the entry",
    );
}

#[test]
fn a_plan_without_a_driven_system_leg_or_room_for_a_run_has_none() {
    let planner = Planner::new();
    let planet = standing_big_planet();
    let no_run = |plan: &Plan, what: &str| {
        assert_eq!(plan.status, STATUS_OK, "{what}: {:?}", plan.offsets);
        assert!(plan.throughs.iter().all(|through| !through), "{what}");
        let e = entry_index_in(plan, BIG_WELL_RADIUS).expect("the entry");
        assert_eq!(plan.speeds[e], SPEED, "{what}");
    };
    // No drive at all.
    let plan = plan_in_well(&planner, planet, RUN_START, RUN_GOAL_OFFSET, None);
    no_run(&plan, "no drive");
    // A system leg shorter than the drive's minimum is flown at the planning speed.
    let never = Drive {
        min_length: 1.0e9,
        ..DRIVE
    };
    let plan = plan_in_well(&planner, planet, RUN_START, RUN_GOAL_OFFSET, Some(never));
    no_run(&plan, "system leg below the minimum");
    // A goal so near the entry that the release distance would fall before it.
    let plan = plan_in_well(
        &planner,
        planet,
        RUN_START,
        [-1.8e5, 0.0, 3.0e4],
        Some(DRIVE),
    );
    no_run(&plan, "framed leg below the release distance");
}

#[test]
fn random_runs_through_wide_wells_hold_their_contract() {
    const CASES: usize = 150;
    const EXTENT: f64 = 3.0e6;
    let planner = Planner::new();
    let mut rng = Rng(0x2D6B_41C9_73E5_A810);
    let (mut planned, mut runs, mut driven_entries) = (0, 0, 0);
    for _ in 0..CASES {
        let planet = Body {
            phase: rng.range(0.0, std::f64::consts::TAU),
            ..big_well_planet(rng.range(0.2, 6.0) * SPEED)
        };
        let centre = body_position(&[planet], 0, 0.0);
        let start = rng.point(EXTENT);
        let direction = rng.point(1.0);
        let norm = distance(direction, [0.0; 3]);
        if norm < 0.1 || distance(start, centre) < 1.2 * BIG_WELL_RADIUS {
            continue;
        }
        let reach = rng.range(0.1, 0.8) * BIG_WELL_RADIUS / norm;
        let offset = direction.map(|component| component * reach);
        let plan = plan_in_well(&planner, planet, start, offset, Some(DRIVE));
        if plan.status != STATUS_OK {
            continue;
        }
        planned += 1;
        let n = plan.frames.len();
        let entry = entry_index_in(&plan, BIG_WELL_RADIUS).expect("the entry");
        driven_entries += usize::from(plan.drives[entry]);
        for (k, through) in plan.throughs.iter().enumerate() {
            if !*through {
                continue;
            }
            runs += 1;
            assert_eq!(
                k, entry,
                "a run goes through the entry only: {:?}",
                plan.offsets
            );
            assert_eq!(
                k + 2,
                n,
                "and then is one leg to the goal: {:?}",
                plan.offsets
            );
            assert!(plan.drives[k] && plan.drives[k + 1], "both legs are driven");
            assert!(
                plan.speeds[k] > SPEED,
                "at the run's speed: {}",
                plan.speeds[k]
            );
        }
        assert!(!plan.throughs[n - 1], "the goal ends the run");
    }
    println!("wide wells: planned {planned}, driven entries {driven_entries}, runs {runs}");
    assert!(planned >= CASES / 3, "many cases plan: {planned}");
    assert!(runs >= planned / 4, "runs occur: {runs} of {planned}");
}

#[test]
fn random_goals_in_a_wells_are_reached_or_the_status_says_why() {
    const CASES: usize = 200;
    const EXTENT: f64 = 250000.0;
    let planner = Planner::new();
    let mut rng = Rng(0x7C21_90AE_5B33_F0D1);
    let (mut ok, mut inside, mut entered, mut outrun, mut not_settled) = (0, 0, 0, 0, 0);
    let mut cases = 0;
    while cases < CASES {
        let planet = Body {
            phase: rng.range(0.0, std::f64::consts::TAU),
            ..well_planet(rng.range(0.2, 6.0) * SPEED)
        };
        let centre = body_position(&[planet], 0, 0.0);
        // One start in three is inside the well.
        let start = if rng.next() < 1.0 / 3.0 {
            plus(centre, rng.point(0.5 * WELL_RADIUS))
        } else {
            rng.point(EXTENT)
        };
        let direction = rng.point(1.0);
        let norm = distance(direction, [0.0; 3]);
        let to_planet = distance(start, centre);
        if norm < 0.1 || to_planet < 2.0 * planet.safe {
            continue;
        }
        let reach = rng.range(2.0, 0.9 * WELL_RADIUS / planet.safe) * planet.safe / norm;
        let offset = direction.map(|component| component * reach);
        let drive = (rng.next() < 0.5).then_some(DRIVE);
        let plan = plan_in_well(&planner, planet, start, offset, drive);
        cases += 1;
        match plan.status {
            STATUS_OK => {
                ok += 1;
                let n = plan.frames.len();
                assert_eq!(plan.frames[n - 1], 0);
                assert_eq!(plan.offsets[n - 1], offset);
                assert!(
                    drive.is_some() || plan.throughs.iter().all(|through| !through),
                    "no run without a drive"
                );
                let started_inside = to_planet < WELL_RADIUS;
                if started_inside {
                    inside += 1;
                    assert!(plan.frames.iter().all(|frame| *frame == 0));
                } else {
                    entered += 1;
                    assert!(
                        entry_index(&plan).is_some(),
                        "the entry: {:?}",
                        plan.offsets
                    );
                }
                // Every leg after the entry, or from the start inside the well, keeps clear of
                // the planet in its frame.
                let first_framed = entry_index(&plan).map_or(0, |e| e + 1);
                for k in first_framed.max(1)..n {
                    assert!(
                        segment_distance_to_origin(plan.offsets[k - 1], plan.offsets[k])
                            >= planet.safe * CLEARANCE_TOLERANCE,
                        "leg {k} cuts the planet: {:?}",
                        plan.offsets
                    );
                }
            }
            STATUS_GOAL_OUTRUN => outrun += 1,
            STATUS_GOAL_NOT_SETTLED => not_settled += 1,
            other => panic!("status {other}: {:?}", plan.offsets),
        }
    }
    println!(
        "well goals: ok {ok} (inside {inside}, entered {entered}), outrun {outrun}, not settled {not_settled}"
    );
    assert!(ok * 10 >= CASES * 5, "many cases plan: {ok}");
    assert!(entered >= 20, "entries occur: {entered}");
    assert!(outrun >= 1, "a body that is too fast occurs");
    assert!(
        not_settled * 10 <= CASES,
        "few cases do not settle: {not_settled}"
    );
}

#[test]
fn a_goal_on_the_safe_sphere_of_its_body_is_reached_straight_whatever_the_rounding() {
    const DIRECTIONS: usize = 200;
    const SAFE: f64 = 150.0;
    const START_DISTANCE: f64 = 600.0;
    let planner = Planner::new();
    // The numbers of a ship flying to a planet's stand-off point, which is its safe radius: the
    // positions are large next to the offsets, so the rounding of the world positions differs from
    // that of the offsets.
    let ship = Ship {
        accel: 1.76,
        drag: 2.0,
    };
    let star = fixed([0.0; 3], 1.0, 2.0);
    // A planet that outruns the ship, with a well that holds the start, and a slow one with none.
    let fast = orbiting(0, 10000.0, 0.0, 1.76 / 10000.0, 100.0).with_zone(1000.0, 2000.0);
    let slow = orbiting(0, 10000.0, 0.0, 0.1 / 10000.0, 100.0);
    assert_eq!(fast.safe, SAFE);
    let mut rng = Rng(0x51A7_0E22_4D9B_C3F5);
    let mut cases = 0;
    while cases < DIRECTIONS {
        let direction = rng.point(1.0);
        if distance(direction, [0.0; 3]) < 0.1 {
            continue;
        }
        cases += 1;
        let direction = unit(direction);
        let goal = scaled(direction, SAFE);
        for planet in [fast, slow] {
            let system = [star, planet];
            let start = plus(
                body_position(&system, 1, 0.0),
                scaled(direction, START_DISTANCE),
            );
            let plan = planner.run_driven(&system, start, 1, goal, 0.88, ship, 5, 32, None);
            assert_eq!(plan.status, STATUS_OK, "direction {direction:?}");
            assert_eq!(plan.detours(), 0, "a straight line to the sphere's surface");
        }
    }
}

#[test]
fn a_frame_that_is_not_a_body_is_rejected() {
    let planner = Planner::new();
    let system = [fixed([0.0, 0.0, 5000.0], 100.0, 150.0)];
    for frame in [-2, 1, 7] {
        let plan = planner.plan_framed(&system, [0.0; 3], frame, [100.0, 0.0, 0.0]);
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "frame {frame}");
        assert!(plan.points.is_empty(), "frame {frame}");
    }
    let plan = planner.plan_framed(&[], [0.0; 3], 0, [100.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_INVALID_INPUT, "a frame with no bodies");
}

#[test]
fn a_goal_offset_inside_the_frame_bodys_hull_is_an_error() {
    let planner = Planner::new();
    let system = [fixed([5000.0, 0.0, 0.0], 500.0, 750.0)];
    let plan = planner.plan_framed(&system, [-5000.0, 0.0, 0.0], 0, [100.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
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
    goal_outrun: usize,
    goal_not_settled: usize,
    /// Corners before the goal, by what became of them.
    corners: usize,
    arcs: usize,
    /// Arcs smaller than the legs and the knee allow, because the bodies were in the way.
    halved: usize,
    /// Corners left sharp, to be passed at rest.
    sharp: usize,
    /// Corners that barely turn, passed at the planning speed.
    straight: usize,
    /// The kernel's turn share, which sets the unconstrained arc radius.
    share: f64,
}

impl Tally {
    fn new(share: f64) -> Self {
        Self {
            share,
            ..Self::default()
        }
    }

    fn record_corners(&mut self, plan: &Plan) {
        for k in 0..plan.points.len().saturating_sub(1) {
            self.corners += 1;
            if plan.radii[k] > 0.0 {
                self.arcs += 1;
                if plan.radii[k] < unconstrained_radius(plan, k, self.share, SHIP) * (1.0 - 1e-6) {
                    self.halved += 1;
                }
            } else if plan.speeds[k] == 0.0 {
                self.sharp += 1;
            } else {
                self.straight += 1;
            }
        }
    }

    fn record(&mut self, plan: &Plan) {
        self.cases += 1;
        if plan.status == STATUS_GOAL_OUTRUN {
            self.goal_outrun += 1;
        } else if plan.status == STATUS_GOAL_NOT_SETTLED {
            self.goal_not_settled += 1;
        }
        if plan.status == STATUS_OK {
            self.ok += 1;
            self.record_corners(plan);
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
    let mut tally = Tally::new(turn_share(&planner));
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

/// Eight planets on circles about the origin, and a moon for each of the first two.
fn random_orbiting_system(rng: &mut Rng) -> Vec<Body> {
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
    bodies
}

#[test]
fn random_orbiting_systems_are_cleared_or_the_status_says_why() {
    const CASES: usize = 200;
    const EXTENT: f64 = 25000.0;
    let planner = Planner::new();
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let mut tally = Tally::new(turn_share(&planner));
    while tally.cases < CASES {
        let bodies = random_orbiting_system(&mut rng);
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

#[test]
fn random_goals_on_moving_bodies_are_reached_or_the_status_says_why() {
    const CASES: usize = 200;
    const EXTENT: f64 = 25000.0;
    let planner = Planner::new();
    let mut rng = Rng(0x2F0B_3A55_91C4_7E6D);
    let mut tally = Tally::new(turn_share(&planner));
    let mut framed_detours = 0;
    while tally.cases < CASES {
        let bodies = random_orbiting_system(&mut rng);
        let start = rng.point(EXTENT);
        let frame = (rng.next() * bodies.len() as f64) as usize;
        // A point 2 to 4 safe radii from the frame body, in a random direction.
        let direction = rng.point(1.0);
        let norm = distance(direction, [0.0; 3]);
        if norm < 0.1 || !clear_of_all(&bodies, start, 0.0) {
            continue;
        }
        let reach = rng.range(2.0, 4.0) * bodies[frame].safe / norm;
        let offset = direction.map(|component| component * reach);

        let plan = planner.plan_framed(&bodies, start, frame as i32, offset);
        if plan.status == STATUS_ENDPOINT_INSIDE_BODY {
            continue; // another body holds the point
        }
        if plan.status != STATUS_GOAL_OUTRUN {
            assert_eq!(plan.frames.last(), Some(&(frame as i32)));
            assert_eq!(plan.offsets.last(), Some(&offset));
        }
        if plan.status != STATUS_GOAL_OUTRUN && plan.status != STATUS_GOAL_NOT_SETTLED {
            assert_times(&plan, SPEED);
        }
        // A goal inside another body's safe sphere shrinks that sphere for the plan, so only a
        // goal clear of every body is judged against the full radii.
        let goal_clear = plan.status == STATUS_OK
            && clear_of_all(
                &bodies,
                *plan.points.last().unwrap(),
                *plan.times.last().unwrap(),
            );
        if goal_clear {
            assert_clear(&plan, &bodies, SPEED);
            if plan.detours() >= 1 {
                framed_detours += 1;
            }
        }
        tally.record(&plan);
    }
    println!("framed goals: {tally:?}, with detours {framed_detours}");
    assert!(tally.ok * 10 >= CASES * 4, "many cases plan: {tally:?}");
    assert!(
        framed_detours >= 10,
        "detours to a moving goal occur: {tally:?}"
    );
    assert!(
        tally.goal_outrun >= 1,
        "a goal on a fast body occurs: {tally:?}"
    );
    // A detour that appears or disappears as the goal moves leaves no arrival time that holds,
    // which is reported; it stays a minority of the cases.
    assert!(
        tally.goal_not_settled * 5 <= CASES,
        "few goals do not settle: {tally:?}"
    );
}

// ---- The travel drive ----

/// The drive as the game configures it, for a ship of top speed `SPEED`.
const DRIVE: Drive = Drive {
    open_cap: 40000.0,
    charge: 3.0,
    ramp_up: 0.3,
    ramp_down: 1.5,
    min_length: 0.0,
};
/// The multiplier the drive is released below, and the braking law's margin and the zone's reach:
/// the values of `TravelDriveMath`, which `the_zone_cap_is_the_games` holds the kernel's to.
const HANDOVER_MULT: f64 = 2.0;
const BRAKING_LAG_MARGIN: f64 = 4.0;
const INFLUENCE_RADIUS_FACTOR: f64 = 2.0;
/// The oracle's step, seconds. The kernel's steps are adaptive and coarser.
const ORACLE_STEP: f64 = 2e-3;
/// How far the kernel's drive time may be from the oracle's, as a fraction.
const DRIVE_TIME_TOLERANCE: f64 = 0.01;

/// The zone cap, written out again from the formula of `doc/script/zones.md`.
fn zone_cap(open_cap: f64, zone_min: f64, zone_radius: f64, distance: f64) -> f64 {
    let influence = zone_radius * INFLUENCE_RADIUS_FACTOR;
    if distance >= influence {
        return open_cap;
    }
    let t = distance / influence;
    open_cap.min(zone_min + (open_cap - zone_min) * t * t)
}

/// The drive multiplier the zones allow at `point`, `t` seconds in.
fn zones_allow(bodies: &[Body], point: Vec3, t: f64, drive: Drive) -> f64 {
    let mut cap = drive.open_cap;
    for (i, body) in bodies.iter().enumerate() {
        if body.zone_radius > 0.0 {
            let d = distance(point, body_position(bodies, i, t));
            cap = cap.min(zone_cap(drive.open_cap, body.zone_cap, body.zone_radius, d));
        }
    }
    cap
}

/// Where the ship is along the leg from `from` to `to` under the drive, as (time, distance)
/// samples every `ORACLE_STEP`, from time `t0`: it charges at the planning speed, then the drive
/// follows the lowest of the zones' cap and the braking law's multiplier (rising at
/// `(target - 1) · ramp_up`, falling exponentially at `ramp_down`), the speed follows its drive
/// speed with the ship's drag, and the drive is released `HANDOVER_MULT · T · SPEED` before the
/// end. The kernel's model, integrated again here in fixed small steps.
fn drive_leg(bodies: &[Body], from: Vec3, to: Vec3, t0: f64, drive: Drive) -> Vec<(f64, f64)> {
    let length = distance(from, to);
    let direction = unit(sub(to, from));
    let braking_time = BRAKING_LAG_MARGIN * (1.0 / drive.ramp_down + 1.0 / SHIP.drag);
    let release = length - HANDOVER_MULT * braking_time * SPEED;
    let (mut t, mut s, mut m, mut v) = (t0 + drive.charge, SPEED * drive.charge, 1.0, SPEED);
    let mut samples = vec![(t0, 0.0), (t, s)];
    while s < release {
        let point = plus(from, scaled(direction, s));
        let wanted = (length - s) / (braking_time * SPEED);
        let target = zones_allow(bodies, point, t, drive).min(wanted).max(1.0);
        m = if m < target {
            target.min(m + (target - 1.0) * drive.ramp_up * ORACLE_STEP)
        } else {
            target + (m - target) * (-drive.ramp_down * ORACLE_STEP).exp()
        };
        let v_new = SPEED * m + (v - SPEED * m) * (-SHIP.drag * ORACLE_STEP).exp();
        s += 0.5 * (v + v_new) * ORACLE_STEP;
        v = v_new;
        t += ORACLE_STEP;
        samples.push((t, s.min(length)));
    }
    let (t_release, s_release) = *samples.last().unwrap();
    samples.push((t_release + (length - s_release) / SPEED, length));
    samples
}

impl Planner {
    /// Plans a goal in the system frame under the drive.
    fn plan_driven(&self, bodies: &[Body], start: Vec3, goal: Vec3) -> Plan {
        self.run_driven(
            bodies,
            start,
            SYSTEM_FRAME,
            goal,
            SPEED,
            SHIP,
            DEPTH,
            CAPACITY,
            Some(DRIVE),
        )
    }
}

/// The time the oracle gets for the one leg from `start` to `goal`.
fn oracle_time(bodies: &[Body], start: Vec3, goal: Vec3) -> f64 {
    drive_leg(bodies, start, goal, 0.0, DRIVE).last().unwrap().0
}

fn assert_close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance * expected.abs(),
        "{what}: {actual} against {expected}"
    );
}

#[test]
fn the_zone_cap_is_the_games() {
    let planner = Planner::new();
    let math = planner
        .vm
        .require("Shared.Helpers.TravelDriveMath")
        .expect("load the drive maths");
    let function: Function = math.get("zoneCap").unwrap();
    for distance in [0.0, 100.0, 1500.0, 4999.0, 5000.0, 9999.0, 10000.0, 1e6] {
        let lua: f64 = function.call((40000.0, 2000.0, 5000.0, distance)).unwrap();
        assert_eq!(lua, zone_cap(40000.0, 2000.0, 5000.0, distance));
    }
    let influence: f64 = math.get("INFLUENCE_RADIUS_FACTOR").unwrap();
    let handover: f64 = math.get("HANDOVER_MULT").unwrap();
    assert_eq!(
        (influence, handover),
        (INFLUENCE_RADIUS_FACTOR, HANDOVER_MULT)
    );
}

#[test]
fn a_leg_is_driven_when_that_is_sooner() {
    let planner = Planner::new();
    let braking_time = BRAKING_LAG_MARGIN * (1.0 / DRIVE.ramp_down + 1.0 / SHIP.drag);
    let shortest = SPEED * (DRIVE.charge + HANDOVER_MULT * braking_time);
    let start = [0.0; 3];
    for length in [1e3, 0.5 * shortest, 0.99 * shortest] {
        let plan = planner.plan_driven(&[], start, [length, 0.0, 0.0]);
        assert_eq!(plan.status, STATUS_OK);
        assert_eq!(plan.drives, vec![false], "length {length}");
        assert_eq!(plan.times, vec![length / SPEED]);
    }
    for length in [1.2 * shortest, 3.0 * shortest, 1e6, 1e8, 1e10] {
        let goal = [length, 0.0, 0.0];
        let plan = planner.plan_driven(&[], start, goal);
        assert_eq!(plan.status, STATUS_OK, "length {length}");
        assert_eq!(plan.drives, vec![true], "length {length}");
        assert!(plan.times[0] < length / SPEED);
        assert_close(
            plan.times[0],
            oracle_time(&[], start, goal),
            DRIVE_TIME_TOLERANCE,
            &format!("time of {length}"),
        );
    }
}

/// Plans the one leg from the origin to `[length, 0, 0]` under `drive`.
fn plan_leg_under(planner: &Planner, length: f64, drive: Drive) -> Plan {
    planner.run_driven(
        &[],
        [0.0; 3],
        SYSTEM_FRAME,
        [length, 0.0, 0.0],
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(drive),
    )
}

#[test]
fn a_leg_shorter_than_the_minimum_is_not_driven() {
    let planner = Planner::new();
    let length = 1e8;
    let free = plan_leg_under(&planner, length, DRIVE);
    assert_eq!((free.status, free.drives.clone()), (STATUS_OK, vec![true]));

    // The same leg, with a minimum just above it: flown at the planning speed.
    let held = Drive {
        min_length: length * 1.001,
        ..DRIVE
    };
    let plan = plan_leg_under(&planner, length, held);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.drives, vec![false]);
    assert_eq!(plan.times, vec![length / SPEED]);

    // A minimum at the leg's own length still drives it.
    let exact = Drive {
        min_length: length,
        ..DRIVE
    };
    let plan = plan_leg_under(&planner, length, exact);
    assert_eq!(plan.drives, vec![true]);
    assert_eq!(plan.times, free.times);
}

#[test]
fn the_minimum_applies_to_each_leg_of_a_detour() {
    let planner = Planner::new();
    let bodies = vec![planet_with_zone([5e8, 0.0, 0.0])];
    let start = [0.0; 3];
    let goal = [1e9, 0.0, 0.0];
    let both = planner.plan_driven(&bodies, start, goal);
    assert_eq!(both.drives, vec![true, true]);
    // Each leg is a little over half the way; a minimum above that leaves both at normal speed.
    let long_legs = Drive {
        min_length: 0.6e9,
        ..DRIVE
    };
    let none = planner.run_driven(
        &bodies,
        start,
        SYSTEM_FRAME,
        goal,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(long_legs),
    );
    assert_eq!(none.status, STATUS_OK);
    assert_eq!(none.drives, vec![false, false]);
}

#[test]
fn a_bad_drive_minimum_is_invalid_input() {
    let planner = Planner::new();
    for (what, min_length) in [
        ("negative", -1.0),
        ("NaN", f64::NAN),
        ("infinite", f64::INFINITY),
    ] {
        let drive = Drive {
            min_length,
            ..DRIVE
        };
        let plan = plan_leg_under(&planner, 1e8, drive);
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "{what}");
    }
}

#[test]
fn a_long_free_leg_gains_time_as_the_autopilots_eta_law_says() {
    let planner = Planner::new();
    let math = planner
        .vm
        .require("Shared.Helpers.TravelDriveMath")
        .expect("load the drive maths");
    let eta: Function = math.get("travelTime").unwrap();
    let law = |length: f64| -> f64 {
        eta.call((
            length,
            0.0,
            SPEED,
            DRIVE.ramp_down,
            SHIP.drag,
            DRIVE.open_cap,
            true,
        ))
        .unwrap()
    };
    let kernel = |length: f64| planner.plan_driven(&[], [0.0; 3], [length, 0.0, 0.0]).times[0];
    // Charging and the climb to the cap are the same for both lengths: the cruise between them
    // is what the law has to match.
    let (near, far) = (1e9, 3e9);
    assert_close(
        kernel(far) - kernel(near),
        law(far) - law(near),
        DRIVE_TIME_TOLERANCE,
        "extra cruise time",
    );
}

/// A planet whose zone the leg passes through, clear of its safe sphere.
fn planet_with_zone(centre: Vec3) -> Body {
    fixed(centre, 5e5, 1e6).with_zone(5e7, 2000.0)
}

#[test]
fn a_leg_that_needs_too_many_steps_is_planned_without_the_drive() {
    let planner = Planner::new();
    // A drive that falls a million times a second is held to steps of microseconds while its
    // multiplier climbs, so the leg's timeline does not fit in the step limit.
    let jittery = Drive {
        ramp_down: 1e6,
        ..DRIVE
    };
    let length = 1e8;
    let plan = planner.run_driven(
        &[],
        [0.0; 3],
        SYSTEM_FRAME,
        [length, 0.0, 0.0],
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(jittery),
    );
    assert_eq!(plan.status, STATUS_DRIVE_NOT_TIMED);
    assert_eq!(plan.drives, vec![false]);
    assert_eq!(plan.times, vec![length / SPEED]);
}

#[test]
fn a_leg_through_a_zone_is_slower_and_the_oracle_agrees() {
    let planner = Planner::new();
    let start = [0.0; 3];
    let goal = [1e9, 0.0, 0.0];
    let in_zone = vec![planet_with_zone([5e8, 0.0, 3e6])];
    let open = vec![fixed([5e8, 0.0, 3e6], 5e5, 1e6)];
    let slow = planner.plan_driven(&in_zone, start, goal);
    let free = planner.plan_driven(&open, start, goal);
    assert_eq!((slow.status, slow.detours()), (STATUS_OK, 0));
    assert_eq!(slow.drives, vec![true]);
    assert!(
        slow.times[0] > free.times[0] + 0.5,
        "{} against {}",
        slow.times[0],
        free.times[0]
    );
    assert_close(
        slow.times[0],
        oracle_time(&in_zone, start, goal),
        DRIVE_TIME_TOLERANCE,
        "time through the zone",
    );
    assert_close(
        free.times[0],
        oracle_time(&open, start, goal),
        DRIVE_TIME_TOLERANCE,
        "time past the body",
    );
}

#[test]
fn a_drive_ends_before_every_corner() {
    let planner = Planner::new();
    let bodies = vec![planet_with_zone([5e8, 0.0, 0.0])];
    let plan = planner.plan_driven(&bodies, [0.0; 3], [1e9, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    // Both legs of the detour are long enough to drive; each one ends at a corner.
    assert_eq!(plan.drives, vec![true, true]);
}

#[test]
fn a_detour_is_planned_for_where_the_body_is_when_the_drive_gets_there() {
    const PERIOD: f64 = 200.0;
    const CROSSING: f64 = 6e8;
    let planner = Planner::new();
    let start = [0.0; 3];
    let goal = [1e9, 0.0, 0.0];
    // The ship under the drive is at the crossing point at this time.
    let timeline = drive_leg(&[], start, goal, 0.0, DRIVE);
    let k = timeline.partition_point(|&(_, s)| s < CROSSING);
    let arrival = timeline[k].0;
    let angular_speed = std::f64::consts::TAU / PERIOD;
    // A planet on a circle about [5e8, 0, 0] that is at [6e8, 0, 0], on the line, at that time. At
    // the time a ship flying at the planning speed would be there it is somewhere else.
    let mut planet = orbiting(-1, 1e8, -angular_speed * arrival, angular_speed, 1e6);
    planet.position = [5e8, 0.0, 0.0];
    planet.safe = 5e6;
    let bodies = vec![planet];
    let normal_arrival = CROSSING / SPEED;
    let there = body_position(&bodies, 0, normal_arrival);
    assert!(distance(there, [CROSSING, 0.0, 0.0]) > 10.0 * planet.safe);

    let plan = planner.plan_driven(&bodies, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(
        plan.detours(),
        1,
        "the planet is on the line when the drive gets there"
    );
    assert!(plan.drives.iter().all(|&driven| driven));
    let ratio = min_driven_clearance_ratio(&plan, &bodies, 0);
    assert!(ratio >= CLEARANCE_TOLERANCE, "clearance {ratio}");
}

#[test]
fn a_goal_on_a_slow_moving_body_settles_under_the_drive() {
    const CASES: usize = 30;
    let planner = Planner::new();
    let mut rng = Rng(0x00D0_17ED_600D_5EED);
    // A planet whose orbit speed, 400 game units per second, is below the planning speed.
    let planet = orbiting(-1, 2e8, 0.0, 2e-6, 1e6);
    let bodies = vec![planet];
    let offset = [4e6, 0.0, 0.0];
    let (mut settled, mut driven) = (0, 0);
    for _ in 0..CASES {
        let start = rng.point(1.5e9);
        if distance(start, body_position(&bodies, 0, 0.0)) < 1e8 {
            continue;
        }
        let plan = planner.run_driven(
            &bodies,
            start,
            0,
            offset,
            SPEED,
            SHIP,
            DEPTH,
            CAPACITY,
            Some(DRIVE),
        );
        assert!(
            matches!(plan.status, STATUS_OK | STATUS_GOAL_NOT_SETTLED),
            "status {}",
            plan.status
        );
        assert_eq!(plan.frames.last(), Some(&0));
        assert_eq!(plan.offsets.last(), Some(&offset));
        // The goal's world point at the time the plan arrives is the body then plus the offset.
        let arrival = *plan.times.last().unwrap();
        let there = plus(body_position(&bodies, 0, arrival), offset);
        assert!(
            distance(*plan.points.last().unwrap(), there)
                < 1e-6 * (1.0 + distance(there, [0.0; 3]))
        );
        if plan.status == STATUS_OK {
            settled += 1;
        }
        if plan.drives.iter().any(|&leg| leg) {
            driven += 1;
        }
    }
    println!("moving goal under the drive: settled {settled}, driven {driven}");
    assert!(settled >= CASES / 2, "most plans settle: {settled}");
    assert!(driven >= CASES / 2, "most plans drive: {driven}");
}

/// The time a plan's leg `k` starts: the previous waypoint's, zero for the first leg.
fn leg_start_time(plan: &Plan, k: usize) -> f64 {
    if k == 0 { 0.0 } else { plan.times[k - 1] }
}

/// The smallest ratio of distance to safe radius between the ship and body `i` along the corner
/// path, with each driven leg timed by the oracle and the others at the planning speed.
fn min_driven_clearance_ratio(plan: &Plan, bodies: &[Body], i: usize) -> f64 {
    let corners = plan.corners();
    let mut smallest = f64::INFINITY;
    for (k, leg) in corners.windows(2).enumerate() {
        let t0 = leg_start_time(plan, k);
        let length = distance(leg[0], leg[1]);
        let timeline = if plan.drives[k] {
            drive_leg(bodies, leg[0], leg[1], t0, DRIVE)
        } else {
            vec![(t0, 0.0), (t0 + length / SPEED, length)]
        };
        let direction = unit(sub(leg[1], leg[0]));
        for &(t, s) in &timeline {
            let position = plus(leg[0], scaled(direction, s.min(length)));
            let ratio = distance(position, body_position(bodies, i, t)) / bodies[i].safe;
            smallest = smallest.min(ratio);
        }
    }
    smallest
}

/// Eight planets with zones on circles, a moon for each of the first two, scaled up so that a
/// leg is long enough to drive and a body crosses it in the seconds a drive flight takes.
fn drive_scale_system(rng: &mut Rng) -> Vec<Body> {
    const SCALE: f64 = 1e4;
    let mut bodies = random_orbiting_system(rng);
    for body in &mut bodies {
        body.orbit_radius *= SCALE;
        body.radius *= SCALE;
        body.safe *= SCALE;
        // A moon's period is short next to a planet's; the planets' linear speed stays below the
        // drive's.
        let cap = if body.parent >= 0 { 200.0 } else { 2000.0 };
        *body = body.with_zone(body.safe * 3.0, cap);
    }
    bodies
}

#[test]
fn random_drive_plans_clear_the_bodies_at_the_times_the_drive_gives() {
    const CASES: usize = 60;
    const EXTENT: f64 = 2.5e8;
    let planner = Planner::new();
    let mut rng = Rng(0x5EED_D41E_0B1D_C0DE);
    let (mut cases, mut driven, mut driven_detours, mut ok) = (0, 0, 0, 0);
    let mut closest = f64::INFINITY;
    while cases < CASES {
        let bodies = drive_scale_system(&mut rng);
        let start = rng.point(EXTENT);
        let goal = rng.point(EXTENT);
        let straight_time = distance(start, goal) / DRIVE.open_cap / SPEED;
        if !clear_of_all(&bodies, start, 0.0) || !clear_of_all(&bodies, goal, straight_time) {
            continue;
        }
        cases += 1;
        let plan = planner.plan_driven(&bodies, start, goal);
        assert!(!plan.points.is_empty());
        if plan.status != STATUS_OK {
            continue;
        }
        ok += 1;
        if plan.drives.iter().any(|&driven| driven) {
            driven += 1;
            if plan.detours() >= 1 {
                driven_detours += 1;
            }
        }
        for i in 0..bodies.len() {
            let ratio = min_driven_clearance_ratio(&plan, &bodies, i);
            closest = closest.min(ratio);
            assert!(
                ratio >= CLEARANCE_TOLERANCE,
                "case {cases}, body {i}: path enters the safe sphere to {ratio} of its radius"
            );
        }
    }
    println!(
        "drive: ok {ok}, driven {driven}, driven with detours {driven_detours}, closest {closest}"
    );
    assert!(ok * 10 >= CASES * 5, "most cases plan");
    assert!(driven * 10 >= CASES * 5, "most plans use the drive");
    assert!(driven_detours >= 5, "detours under the drive occur");
}
