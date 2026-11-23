//! Checks of the system planner kernel (`script/Shared/Helpers/SystemPlanner.lua`), run through
//! the kernel harness. A Rust oracle of the orbits judges the paths: a plan that reports `Ok`
//! must keep clear of every body's safe sphere at the time the ship is there.

#[path = "../common/mod.rs"]
mod common;

mod arrival;
mod drive;
mod legs;
mod random;
mod stand_off;
mod well;
mod zones;

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

/// A cost zone: a sphere that rides on body `frame` (`SYSTEM_FRAME` for none), centred `centre`
/// from it, where each metre travelled costs `cost` extra.
#[derive(Clone, Copy)]
struct Zone {
    frame: i32,
    centre: Vec3,
    radius: f64,
    cost: f64,
}

/// A zone fixed in the system.
fn zone(centre: Vec3, radius: f64, cost: f64) -> Zone {
    Zone {
        frame: SYSTEM_FRAME,
        centre,
        radius,
        cost,
    }
}

/// The values of a zone row, in the order of `SystemPlanner.Zone`.
const ZONE_STRIDE: usize = 6;

fn flatten_zones(zones: &[Zone]) -> Vec<f64> {
    let mut values = Vec::with_capacity(zones.len() * ZONE_STRIDE);
    for z in zones {
        values.extend_from_slice(&[
            f64::from(z.frame),
            z.centre[0],
            z.centre[1],
            z.centre[2],
            z.radius,
            z.cost,
        ]);
    }
    values
}

/// The oracle: where zone `z` is `t` seconds from the snapshot.
fn zone_position(bodies: &[Body], z: &Zone, t: f64) -> Vec3 {
    if z.frame == SYSTEM_FRAME {
        return z.centre;
    }
    plus(body_position(bodies, z.frame as usize, t), z.centre)
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
        self.run_arrival(
            bodies, start, frame, goal, speed, ship, depth, capacity, drive, None,
        )
    }

    /// Plans as `run_driven` does, with the arrival radius `arrival` (none for no radius).
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the kernel's plan inputs"
    )]
    fn run_arrival(
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
        arrival: Option<f64>,
    ) -> Plan {
        self.run_zone_buffer(
            bodies, start, frame, goal, speed, ship, depth, capacity, drive, arrival, None, None,
        )
    }

    /// Plans as `run_arrival` does, with the cost zones `zones` (none for no zones).
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the kernel's plan inputs"
    )]
    fn run_zoned(
        &self,
        bodies: &[Body],
        start: Vec3,
        frame: i32,
        goal: Vec3,
        speed: f64,
        depth: i64,
        drive: Option<Drive>,
        zones: &[Zone],
    ) -> Plan {
        let flat = flatten_zones(zones);
        self.run_zone_buffer(
            bodies,
            start,
            frame,
            goal,
            speed,
            SHIP,
            depth,
            CAPACITY,
            drive,
            None,
            Some(&flat),
            Some(zones.len()),
        )
    }

    /// Plans with the zone buffer and the zone count exactly as given, which need not agree.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the kernel's plan inputs"
    )]
    fn run_zone_buffer(
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
        arrival: Option<f64>,
        zone_values: Option<&[f64]>,
        zone_count: Option<usize>,
    ) -> Plan {
        self.run_all(
            bodies,
            start,
            frame,
            goal,
            speed,
            ship,
            depth,
            capacity,
            drive,
            arrival,
            zone_values,
            zone_count,
            None,
        )
    }

    /// Plans with every input of the kernel, `stand_off` (the distance the last leg stops short of
    /// the goal; none for zero) included.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the kernel's plan inputs"
    )]
    fn run_all(
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
        arrival: Option<f64>,
        zone_values: Option<&[f64]>,
        zone_count: Option<usize>,
        stand_off: Option<f64>,
    ) -> Plan {
        let flat = flatten(bodies);
        let drive_buffer: Option<Value> = drive.map(|d| self.vm.buffer(&d.block()).unwrap());
        // A zero-length buffer cannot be built, so no zones in a buffer get one unused element.
        let zone_buffer: Option<Value> = zone_values.map(|values| {
            self.vm
                .buffer(if values.is_empty() { &[0.0] } else { values })
                .unwrap()
        });
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
                // A tuple converts to Lua arguments up to 16 elements; its last element may
                // itself be a tuple, which carries the arguments after the 15th.
                (
                    drive_buffer.as_ref(),
                    arrival,
                    zone_buffer.as_ref(),
                    zone_count,
                    stand_off,
                ),
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
    min_sphere_ratio(plan, speed, bodies[i].safe, |t| body_position(bodies, i, t))
}

/// The smallest ratio of distance to `radius` between the ship and a sphere whose centre is
/// `centre_at(t)`, along the path flown at `speed`.
fn min_sphere_ratio(plan: &Plan, speed: f64, radius: f64, centre_at: impl Fn(f64) -> Vec3) -> f64 {
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
            let ratio = distance(position, centre_at(at)) / radius;
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

// ---- Helpers that more than one topic uses ----

/// The radius of the well the planet in these tests carries ships in, and the share of it that the
/// entry waypoint lies at (the rule of `doc/script/nav/system-planner.md`).
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

/// Whether `point` is outside every body's safe sphere (times of 0 and `at`), by a margin.
fn clear_of_all(bodies: &[Body], point: Vec3, at: f64) -> bool {
    const MARGIN: f64 = 1.2;
    (0..bodies.len())
        .all(|i| distance(point, body_position(bodies, i, at)) > bodies[i].safe * MARGIN)
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
    drive_run_leg(bodies, from, to, t0, drive, 0.0, None).0
}

/// The state a drive run carries from one leg to the next: the multiplier and the speed.
type RunState = (f64, f64);

/// `drive_leg` for a leg of a drive run: it starts from the state `carried` with no charge (a
/// fresh leg charges), and a leg that the run goes on from for `beyond` more length is not released
/// but ends with the state it has, which is returned.
fn drive_run_leg(
    bodies: &[Body],
    from: Vec3,
    to: Vec3,
    t0: f64,
    drive: Drive,
    beyond: f64,
    carried: Option<RunState>,
) -> (Vec<(f64, f64)>, RunState) {
    let length = distance(from, to);
    let direction = unit(sub(to, from));
    let braking_time = BRAKING_LAG_MARGIN * (1.0 / drive.ramp_down + 1.0 / SHIP.drag);
    let continues = beyond > 0.0;
    let release = if continues {
        length
    } else {
        length - HANDOVER_MULT * braking_time * SPEED
    };
    let (mut t, mut s, mut m, mut v) = (t0 + drive.charge, SPEED * drive.charge, 1.0, SPEED);
    let mut samples = vec![(t0, 0.0), (t, s)];
    if let Some((carried_m, carried_v)) = carried {
        (t, s, m, v) = (t0, 0.0, carried_m, carried_v);
        samples = vec![(t0, 0.0)];
    }
    while s < release {
        let point = plus(from, scaled(direction, s));
        let wanted = (length - s + beyond) / (braking_time * SPEED);
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
    if !continues {
        samples.push((t_release + (length - s_release) / SPEED, length));
    }
    (samples, (m, v))
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

/// A planet whose zone the leg passes through, clear of its safe sphere.
fn planet_with_zone(centre: Vec3) -> Body {
    fixed(centre, 5e5, 1e6).with_zone(5e7, 2000.0)
}
