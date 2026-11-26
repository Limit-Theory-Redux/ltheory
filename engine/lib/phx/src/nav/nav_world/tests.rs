use rapier3d_f64::prelude::nalgebra as na;

use super::*;
use crate::nav::NavAxis;

/// The length of a step: a frame at 60 Hz.
const TICK_S: f64 = 1.0 / 60.0;
const SHIP_RADIUS: f32 = 1.0;
const LINEAR_DRAG: f64 = 2.0;
const ANGULAR_DRAG: f64 = 20.0;
/// Forward thrust that holds a top speed of 100 on a unit-mass body.
const THRUST_FORWARD: f64 = 100.0 * LINEAR_DRAG;
const THRUST_SIDE: f64 = THRUST_FORWARD / 4.0;
const TORQUE: f64 = 60.0;
const ARRIVAL_RADIUS: f64 = 5.0;
const GOAL_DISTANCE: f64 = 500.0;
const ROCK_RADIUS: f32 = 30.0;
const ROCK_DISTANCE: f64 = 200.0;
const LONG_GOAL_DISTANCE: f64 = 1000.0;
const STAND_OFF: f64 = 8.0;
const SLOT_DISTANCE: f64 = 100.0;
const ORBIT_RADIUS: f64 = 100.0;

fn body_in(physics: &mut Physics, radius: f32, at: Position) -> Box<RigidBody> {
    let mut body = RigidBody::new_sphere();
    body.set_scale(radius);
    physics.add_rigid_body(&mut body);
    body.set_position(&at);
    body
}

fn ship_in(physics: &mut Physics) -> Box<RigidBody> {
    body_in(physics, SHIP_RADIUS, Position::ZERO)
}

fn goal_at(x: f64, z: f64) -> Position {
    Position::new(x, 0.0, z)
}

fn axes_of(nav: &NavWorld) -> Vec<f64> {
    let mut out = vec![0.0; nav.agents.len() * AXIS_COUNT];
    nav.copy_axes(&mut out);
    out
}

fn axis(axes: &[f64], agent: usize, axis: NavAxis) -> f64 {
    axes[agent * AXIS_COUNT + axis as usize]
}

fn add_ship(nav: &mut NavWorld, body: &RigidBody) -> u32 {
    nav.add_agent(
        body,
        LINEAR_DRAG,
        ANGULAR_DRAG,
        THRUST_FORWARD,
        THRUST_SIDE,
        THRUST_SIDE,
        TORQUE,
        TORQUE,
        TORQUE,
    )
}

fn state_in(physics: &Physics, body: &RigidBody) -> BodyState {
    let handles = body.handles_in(&physics.world()).expect("a root body");
    physics
        .world()
        .as_ref()
        .body_state(&handles)
        .expect("in the world")
}

#[test]
fn accelerations_are_the_forces_over_the_mass_and_inertia_of_the_body() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let state = state_in(&physics, &ship);
    assert!(state.mass > 0.0, "a ship body has mass");
    let limits = FlightLimits {
        speed_cap: NO_SPEED_CAP,
        linear_drag: LINEAR_DRAG,
        angular_drag: ANGULAR_DRAG,
        thrust: [THRUST_FORWARD, THRUST_SIDE, THRUST_SIDE],
        torque: [TORQUE, 2.0 * TORQUE, 3.0 * TORQUE],
    };
    let profile = profile_of(&limits, &state);
    assert_eq!(profile.accel_forward, THRUST_FORWARD / state.mass);
    assert_eq!(profile.accel_right, THRUST_SIDE / state.mass);
    assert_eq!(profile.turn_pitch, TORQUE * state.inverse_inertia.x);
    assert_eq!(profile.turn_yaw, 2.0 * TORQUE * state.inverse_inertia.y);
    assert_eq!(profile.turn_roll, 3.0 * TORQUE * state.inverse_inertia.z);
    assert!(profile.turn_pitch > 0.0 && profile.turn_roll > 0.0);
    assert_eq!(
        profile.max_speed(),
        THRUST_FORWARD / state.mass / LINEAR_DRAG
    );
}

#[test]
fn a_body_without_mass_gets_no_acceleration() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let state = BodyState {
        mass: 0.0,
        inverse_inertia: DVec3::ZERO,
        ..state_in(&physics, &ship)
    };
    let limits = FlightLimits {
        speed_cap: NO_SPEED_CAP,
        linear_drag: LINEAR_DRAG,
        angular_drag: ANGULAR_DRAG,
        thrust: [THRUST_FORWARD; 3],
        torque: [TORQUE; 3],
    };
    let profile = profile_of(&limits, &state);
    assert_eq!(profile.accel_forward, 0.0);
    assert_eq!(profile.turn_yaw, 0.0);
}

#[test]
fn an_agent_is_steered_to_its_goal() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert_eq!(axis(&axes_of(&nav), 0, NavAxis::Yaw), 1.0);
}

#[test]
fn ids_are_consecutive_and_each_agent_has_its_own_goal() {
    let mut physics = Physics::new();
    let first = ship_in(&mut physics);
    // Apart and behind the first ship, so neither is on the other's course.
    let second = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, ROCK_DISTANCE));
    let mut nav = NavWorld::new(&physics);
    assert_eq!(add_ship(&mut nav, &first), 0);
    assert_eq!(add_ship(&mut nav, &second), 1);
    assert_eq!(nav.slot_count(), 2);
    nav.set_arrive(0, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
    nav.set_arrive(1, &goal_at(-GOAL_DISTANCE, ROCK_DISTANCE), ARRIVAL_RADIUS);
    nav.step(TICK_S);
    let axes = axes_of(&nav);
    assert_eq!(axis(&axes, 0, NavAxis::Yaw), 1.0);
    assert_eq!(axis(&axes, 1, NavAxis::Yaw), -1.0);
}

#[test]
fn the_next_step_follows_the_body_in_physics() {
    let mut physics = Physics::new();
    let mut ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert_eq!(axis(&axes_of(&nav), 0, NavAxis::Yaw), 1.0);

    ship.set_position(&goal_at(2.0 * GOAL_DISTANCE, 0.0));
    nav.step(TICK_S);
    assert_eq!(axis(&axes_of(&nav), 0, NavAxis::Yaw), -1.0);
}

#[test]
fn an_agent_without_a_goal_gets_no_input() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
    nav.clear_behaviour(id);
    nav.step(TICK_S);
    assert!(axes_of(&nav).iter().all(|&value| value == 0.0));
}

#[test]
fn arrival_is_reported_per_agent() {
    let mut physics = Physics::new();
    let near = ship_in(&mut physics);
    let far = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let near_id = add_ship(&mut nav, &near);
    let far_id = add_ship(&mut nav, &far);
    nav.set_arrive(
        near_id,
        &goal_at(0.0, -ARRIVAL_RADIUS / 2.0),
        ARRIVAL_RADIUS,
    );
    nav.set_arrive(far_id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
    nav.step(TICK_S);
    let mut arrived = [false; 2];
    nav.copy_arrived(&mut arrived);
    assert_eq!(arrived, [true, false]);
}

fn yaw_of(nav: &NavWorld) -> f64 {
    axis(&axes_of(nav), 0, NavAxis::Yaw)
}

fn arrived_of(nav: &NavWorld) -> bool {
    let mut arrived = [false];
    nav.copy_arrived(&mut arrived);
    arrived[0]
}

#[test]
fn seek_steers_at_its_point() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_seek(
        id,
        &goal_at(GOAL_DISTANCE, 0.0),
        ARRIVAL_RADIUS,
        f64::INFINITY,
    );
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 1.0);
}

fn forward_thrust_of(nav: &NavWorld) -> f64 {
    axis(&axes_of(nav), 0, NavAxis::ThrustForward)
}

#[test]
fn a_speed_cap_holds_through_the_profile_refresh_of_every_step() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    let far = goal_at(0.0, -LONG_GOAL_DISTANCE);
    nav.set_seek(id, &far, ARRIVAL_RADIUS, f64::INFINITY);
    nav.step(TICK_S);
    assert!(
        forward_thrust_of(&nav) > 0.0,
        "an uncapped agent at rest accelerates"
    );

    nav.set_speed_cap(id, 0.0);
    for _ in 0..3 {
        nav.step(TICK_S);
        assert_eq!(forward_thrust_of(&nav), 0.0, "a cap of zero wants rest");
    }

    nav.set_speed_cap(id, f64::INFINITY);
    nav.step(TICK_S);
    assert!(
        forward_thrust_of(&nav) > 0.0,
        "an infinite cap lifts the limit"
    );
}

#[test]
#[should_panic(expected = "a speed cap must be zero or more")]
fn a_negative_speed_cap_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_speed_cap(id, -1.0);
}

#[test]
#[should_panic(expected = "a speed cap must be zero or more")]
fn a_nan_speed_cap_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_speed_cap(id, f64::NAN);
}

#[test]
fn a_facing_turns_a_held_ship_to_a_point_until_it_is_cleared() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_hold(id, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 0.0, "a held ship keeps its own heading");

    nav.set_facing(id, &goal_at(GOAL_DISTANCE, 0.0));
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 1.0, "turns to a point on its right");

    // A new behaviour leaves the facing as it is.
    nav.set_hold(id, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 1.0);

    nav.clear_facing(id);
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 0.0);
}

#[test]
#[should_panic(expected = "NavWorld has no agent")]
fn a_facing_for_an_unknown_agent_is_refused() {
    let physics = Physics::new();
    let mut nav = NavWorld::new(&physics);
    nav.set_facing(0, &goal_at(GOAL_DISTANCE, 0.0));
}

#[test]
#[should_panic(expected = "NavWorld has no agent")]
fn clearing_the_facing_of_an_unknown_agent_is_refused() {
    let physics = Physics::new();
    let mut nav = NavWorld::new(&physics);
    nav.clear_facing(0);
}

#[test]
#[should_panic(expected = "a pass speed must be zero or more")]
fn a_negative_pass_speed_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_seek(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS, -1.0);
}

#[test]
#[should_panic(expected = "NavWorld has no agent 3")]
fn a_speed_cap_for_an_unknown_agent_is_refused() {
    let physics = Physics::new();
    let mut nav = NavWorld::new(&physics);
    nav.set_speed_cap(3, 1.0);
}

#[test]
fn hold_keeps_the_position_the_body_had_at_the_call() {
    let mut physics = Physics::new();
    let mut ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_hold(id, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert!(arrived_of(&nav));

    // The held point stays behind: it doesn't follow the body.
    ship.set_position(&goal_at(GOAL_DISTANCE, 0.0));
    nav.step(TICK_S);
    assert!(!arrived_of(&nav));
    assert_eq!(yaw_of(&nav), -1.0);
}

#[test]
fn a_pursued_body_is_read_again_at_each_step() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut target = body_in(&mut physics, SHIP_RADIUS, goal_at(GOAL_DISTANCE, 0.0));
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 1.0);

    target.set_position(&goal_at(-GOAL_DISTANCE, 0.0));
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), -1.0);
}

#[test]
fn a_pursuer_arrives_at_its_stand_off_from_the_hull() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    // The stand-off point is the two hulls' radii plus the stand-off from the target's centre.
    let gap = f64::from(SHIP_RADIUS) * 2.0 + STAND_OFF;
    let target = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, -gap));
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert!(arrived_of(&nav));
}

#[test]
fn a_follower_reads_its_leader_and_its_slot_at_each_step() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    // The leader faces -z, so a slot 100 ahead of it, in its own axes, is at the origin.
    let mut leader = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, SLOT_DISTANCE));
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_follow(id, &leader, 0.0, 0.0, SLOT_DISTANCE, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert!(arrived_of(&nav));

    leader.set_position(&goal_at(GOAL_DISTANCE, SLOT_DISTANCE));
    nav.step(TICK_S);
    assert!(!arrived_of(&nav));
    assert_eq!(yaw_of(&nav), 1.0);
}

#[test]
fn an_orbiting_ship_on_its_circle_has_arrived_and_one_far_off_is_steered_to_it() {
    let mut physics = Physics::new();
    let on_circle = ship_in(&mut physics);
    let far = body_in(&mut physics, SHIP_RADIUS, goal_at(-GOAL_DISTANCE, 0.0));
    let centre = body_in(&mut physics, SHIP_RADIUS, goal_at(ORBIT_RADIUS, 0.0));
    let mut nav = NavWorld::new(&physics);
    let first = add_ship(&mut nav, &on_circle);
    let second = add_ship(&mut nav, &far);
    for id in [first, second] {
        nav.set_orbit(id, &centre, ORBIT_RADIUS, &DVec3::Y, ARRIVAL_RADIUS);
    }
    nav.step(TICK_S);
    let mut arrived = [false; 2];
    nav.copy_arrived(&mut arrived);
    assert_eq!(arrived, [true, false]);
    // The far ship is left of the centre and faces -z, so the circle is on its right.
    assert_eq!(axis(&axes_of(&nav), 1, NavAxis::Yaw), 1.0);
}

#[test]
fn a_later_behaviour_drops_the_target_of_an_earlier_one() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut target = body_in(&mut physics, SHIP_RADIUS, goal_at(GOAL_DISTANCE, 0.0));
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
    nav.set_arrive(id, &goal_at(GOAL_DISTANCE, 0.0), ARRIVAL_RADIUS);
    physics.remove_rigid_body(&mut target);
    // The step would stop on a removed target if the pursuit were still held.
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 1.0);
}

#[test]
#[should_panic(expected = "target: the body is not a root body of its physics world")]
fn a_target_of_another_physics_world_is_refused() {
    let mut physics = Physics::new();
    let mut other = Physics::new();
    let stranger = ship_in(&mut other);
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_pursue(id, &stranger, STAND_OFF, ARRIVAL_RADIUS);
}

#[test]
#[should_panic(expected = "agent 0's target: its body has left the physics world")]
fn a_removed_target_stops_the_step_and_names_the_agent() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut target = body_in(&mut physics, SHIP_RADIUS, goal_at(GOAL_DISTANCE, 0.0));
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_pursue(id, &target, STAND_OFF, ARRIVAL_RADIUS);
    physics.remove_rigid_body(&mut target);
    nav.step(TICK_S);
}

#[test]
#[should_panic(expected = "the normal must be finite and not zero")]
fn an_orbit_without_a_normal_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_orbit(id, &ship, ORBIT_RADIUS, &DVec3::ZERO, ARRIVAL_RADIUS);
}

#[test]
#[should_panic(expected = "the radius must be positive")]
fn an_orbit_without_a_radius_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_orbit(id, &ship, 0.0, &DVec3::Y, ARRIVAL_RADIUS);
}

#[test]
fn axis_values_are_the_positions_in_the_nav_buffer() {
    assert_eq!(NavAxis::Pitch as usize, ::nav::Axis::Pitch as usize);
    assert_eq!(NavAxis::Yaw as usize, ::nav::Axis::Yaw as usize);
    assert_eq!(NavAxis::Roll as usize, ::nav::Axis::Roll as usize);
    assert_eq!(
        NavAxis::ThrustForward as usize,
        ::nav::Axis::ThrustForward as usize
    );
    assert_eq!(
        NavAxis::ThrustRight as usize,
        ::nav::Axis::ThrustRight as usize
    );
    assert_eq!(NavAxis::ThrustUp as usize, ::nav::Axis::ThrustUp as usize);
    assert_eq!(NavWorld::axis_count() as usize, AXIS_COUNT);
}

#[test]
#[should_panic(expected = "not a root body of its physics world")]
fn a_body_of_another_physics_world_is_refused() {
    let mut physics = Physics::new();
    let mut other = Physics::new();
    let stranger = ship_in(&mut other);
    let mut nav = NavWorld::new(&physics);
    let _ = ship_in(&mut physics);
    add_ship(&mut nav, &stranger);
}

#[test]
#[should_panic(expected = "not a root body of its physics world")]
fn a_body_that_is_not_in_physics_is_refused() {
    let physics = Physics::new();
    let loose = RigidBody::new_sphere();
    let mut nav = NavWorld::new(&physics);
    add_ship(&mut nav, &loose);
}

#[test]
#[should_panic(expected = "NavWorld has no agent 3")]
fn an_unknown_agent_id_is_refused() {
    let physics = Physics::new();
    let mut nav = NavWorld::new(&physics);
    nav.set_arrive(3, &Position::ZERO, ARRIVAL_RADIUS);
}

#[test]
#[should_panic(expected = "needs slot_count · axis_count values")]
fn a_wrong_axes_buffer_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    add_ship(&mut nav, &ship);
    nav.copy_axes(&mut [0.0; AXIS_COUNT + 1]);
}

#[test]
#[should_panic(expected = "needs slot_count values")]
fn a_wrong_arrived_buffer_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    add_ship(&mut nav, &ship);
    nav.copy_arrived(&mut [false; 2]);
}

#[test]
#[should_panic(expected = "agent 0: its body has left the physics world")]
fn a_removed_agent_body_stops_the_step() {
    let mut physics = Physics::new();
    let mut ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    add_ship(&mut nav, &ship);
    physics.remove_rigid_body(&mut ship);
    nav.step(TICK_S);
}

const ALL_BITS: u32 = u32::MAX;
const ONE_STEP: f32 = 1.0 / 60.0;

/// A ship that arrives at a far goal past a rock on its course, with physics updated so
/// that queries see the rock.
struct RockFlight {
    physics: Physics,
    nav: NavWorld,
    /// Held so the body stays in physics.
    _ship: Box<RigidBody>,
    rock: Box<RigidBody>,
}

fn flight_past_a_rock() -> RockFlight {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let rock = body_in(&mut physics, ROCK_RADIUS, goal_at(0.0, -ROCK_DISTANCE));
    physics.update(ONE_STEP);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive(id, &goal_at(0.0, -LONG_GOAL_DISTANCE), ARRIVAL_RADIUS);
    RockFlight {
        physics,
        nav,
        _ship: ship,
        rock,
    }
}

#[test]
fn a_sensed_body_is_the_sphere_of_its_state_and_steers_the_agent() {
    let mut flight = flight_past_a_rock();
    flight.nav.step(TICK_S);
    let straight = axes_of(&flight.nav);

    flight.nav.set_sensing(ALL_BITS);
    flight.nav.step(TICK_S);
    let state = state_in(&flight.physics, &flight.rock);
    let key = flight.nav.handles_of(&flight.rock, "rock").key();
    assert_eq!(
        flight.nav.obstacle_spheres,
        [Obstacle {
            key: ObstacleKey::Body(key),
            position: state.position,
            velocity: state.velocity,
            radius: state.radius,
        }]
    );
    assert_ne!(axes_of(&flight.nav), straight, "the rock is on the course");
}

/// Longer than the steps of the tests below, so a second run of avoidance is out of reach
/// unless it is asked for.
const LONG_PERIOD_S: f64 = 10.0;

#[test]
fn an_agent_whose_avoidance_is_not_due_senses_nothing_and_keeps_its_detour() {
    let mut flight = flight_past_a_rock();
    flight.nav.set_sensing(ALL_BITS);
    flight.nav.set_avoidance_period(0, LONG_PERIOD_S);
    flight.nav.step(TICK_S);
    let detour = axes_of(&flight.nav);
    assert_eq!(flight.nav.obstacle_spheres.len(), 1, "the first run senses");

    flight.nav.step(TICK_S);
    assert!(
        flight.nav.obstacle_spheres.is_empty(),
        "an agent that isn't due is not sensed for"
    );
    assert_eq!(axes_of(&flight.nav), detour, "the detour is held");
}

#[test]
fn a_refresh_senses_and_avoids_again_at_the_next_step() {
    let mut flight = flight_past_a_rock();
    flight.nav.set_sensing(ALL_BITS);
    flight.nav.set_avoidance_period(0, LONG_PERIOD_S);
    flight.nav.step(TICK_S);
    flight.nav.step(TICK_S);
    assert!(flight.nav.obstacle_spheres.is_empty());

    flight.nav.refresh(0);
    flight.nav.step(TICK_S);
    assert_eq!(flight.nav.obstacle_spheres.len(), 1);
}

#[test]
fn a_goal_setter_does_not_bring_the_next_avoidance_forward() {
    let mut flight = flight_past_a_rock();
    flight.nav.set_sensing(ALL_BITS);
    flight.nav.set_avoidance_period(0, LONG_PERIOD_S);
    flight.nav.step(TICK_S);
    for _ in 0..3 {
        flight
            .nav
            .set_arrive(0, &goal_at(0.0, -LONG_GOAL_DISTANCE), ARRIVAL_RADIUS);
        flight.nav.step(TICK_S);
        assert!(
            flight.nav.obstacle_spheres.is_empty(),
            "a goal sent again every frame leaves the period alone"
        );
    }
}

#[test]
fn the_sensing_range_of_an_agent_is_the_range_nav_gives_its_state() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    let state = state_in(&physics, &ship);
    let profile = profile_of(&nav.agents[0].as_ref().unwrap().limits, &state);
    let expected = sensing_range(&nav_body(&state), &profile, nav.world.tuning());
    assert!(expected > 0.0, "an agent at rest still looks ahead");
    assert_eq!(nav.sensing_range(id), expected);
}

#[test]
#[should_panic(expected = "NavWorld has no agent")]
fn the_sensing_range_of_an_unknown_agent_is_refused() {
    let physics = Physics::new();
    let nav = NavWorld::new(&physics);
    nav.sensing_range(0);
}

#[test]
#[should_panic(expected = "an avoidance period must be zero or more")]
fn a_negative_avoidance_period_is_refused() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_avoidance_period(id, -1.0);
}

#[test]
#[should_panic(expected = "NavWorld has no agent")]
fn an_avoidance_period_for_an_unknown_agent_is_refused() {
    let physics = Physics::new();
    let mut nav = NavWorld::new(&physics);
    nav.set_avoidance_period(0, 1.0);
}

#[test]
#[should_panic(expected = "NavWorld has no agent")]
fn a_refresh_for_an_unknown_agent_is_refused() {
    let physics = Physics::new();
    let mut nav = NavWorld::new(&physics);
    nav.refresh(0);
}

#[test]
fn sensing_is_off_by_default_and_by_a_zero_mask_or_a_foreign_one() {
    let RockFlight {
        mut nav, mut rock, ..
    } = flight_past_a_rock();
    nav.step(TICK_S);
    let straight = axes_of(&nav);
    assert!(nav.obstacle_spheres.is_empty());

    rock.set_collision_group(0b01);
    for mask in [0, 0b10] {
        nav.set_sensing(mask);
        nav.step(TICK_S);
        assert_eq!(axes_of(&nav), straight, "mask {mask:#b}");
    }
    nav.set_sensing(0b01);
    nav.step(TICK_S);
    assert_ne!(axes_of(&nav), straight);
}

#[test]
fn sensing_does_not_count_agents() {
    let mut flight = flight_past_a_rock();
    let other = body_in(
        &mut flight.physics,
        SHIP_RADIUS,
        goal_at(SLOT_DISTANCE, 0.0),
    );
    add_ship(&mut flight.nav, &other);
    flight.physics.update(ONE_STEP);
    flight.nav.set_sensing(ALL_BITS);
    flight.nav.step(TICK_S);
    assert_eq!(flight.nav.obstacle_spheres.len(), 1, "the rock alone");
}

#[test]
fn a_removed_agents_body_is_sensed_as_an_obstacle_again() {
    let mut flight = flight_past_a_rock();
    let other = body_in(
        &mut flight.physics,
        SHIP_RADIUS,
        goal_at(SLOT_DISTANCE, 0.0),
    );
    let other_id = add_ship(&mut flight.nav, &other);
    flight.physics.update(ONE_STEP);
    flight.nav.set_sensing(ALL_BITS);
    flight.nav.step(TICK_S);
    assert_eq!(flight.nav.obstacle_spheres.len(), 1, "the rock alone");

    flight.nav.remove_agent(other_id);
    flight.nav.step(TICK_S);
    assert_eq!(
        flight.nav.obstacle_spheres.len(),
        2,
        "the rock and the removed agent's body"
    );
}

#[test]
fn a_removed_agents_id_goes_to_the_next_agent_and_the_buffers_keep_their_slots() {
    let mut physics = Physics::new();
    let bodies: Vec<_> = (0..3)
        .map(|index| {
            body_in(
                &mut physics,
                SHIP_RADIUS,
                goal_at(0.0, ROCK_DISTANCE * f64::from(index)),
            )
        })
        .collect();
    let mut nav = NavWorld::new(&physics);
    let ids: Vec<_> = bodies.iter().map(|body| add_ship(&mut nav, body)).collect();
    assert_eq!(ids, [0, 1, 2]);
    for &id in &ids {
        nav.set_arrive(id, &goal_at(GOAL_DISTANCE, ROCK_DISTANCE), ARRIVAL_RADIUS);
    }
    nav.step(TICK_S);
    let before = axes_of(&nav);

    nav.remove_agent(1);
    assert_eq!(nav.slot_count(), 3, "the slot stays");
    nav.step(TICK_S);
    let after = axes_of(&nav);
    assert_eq!(after.len(), before.len());
    let slot = |axes: &[f64], id: usize| axes[id * AXIS_COUNT..(id + 1) * AXIS_COUNT].to_vec();
    assert_eq!(slot(&after, 1), [0.0; AXIS_COUNT], "a free slot is inert");
    let mut arrived = [true; 3];
    nav.copy_arrived(&mut arrived);
    assert!(!arrived[1]);

    let newcomer = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, -ROCK_DISTANCE));
    assert_eq!(add_ship(&mut nav, &newcomer), 1, "the freed slot is reused");
    assert_eq!(nav.slot_count(), 3);
    assert_eq!(
        slot(&axes_of(&nav), 1),
        [0.0; AXIS_COUNT],
        "it starts with no goal"
    );
}

#[test]
#[should_panic(expected = "NavWorld has no agent 0")]
fn a_removed_agent_is_not_an_agent() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.remove_agent(id);
    nav.set_arrive(id, &Position::ZERO, ARRIVAL_RADIUS);
}

#[test]
fn a_removed_agents_body_may_leave_physics() {
    let mut physics = Physics::new();
    let mut ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.remove_agent(id);
    physics.remove_rigid_body(&mut ship);
    nav.step(TICK_S);
}

const FIRST_AGENT: u32 = 0;

/// The axes of one step with sensing set to `mask`.
fn axes_sensing(flight: &mut RockFlight, mask: u32) -> Vec<f64> {
    flight.nav.set_sensing(mask);
    flight.nav.step(TICK_S);
    axes_of(&flight.nav)
}

#[test]
fn a_pursued_body_is_left_out_of_the_pursuers_scene() {
    let mut flight = flight_past_a_rock();
    flight
        .nav
        .set_pursue(FIRST_AGENT, &flight.rock, STAND_OFF, ARRIVAL_RADIUS);
    let blind = axes_sensing(&mut flight, 0);
    assert_eq!(axes_sensing(&mut flight, ALL_BITS), blind);
}

#[test]
fn an_orbit_still_avoids_its_centre() {
    let mut flight = flight_past_a_rock();
    // A circle inside the rock: the course to it runs into the rock's hull first.
    flight.nav.set_orbit(
        FIRST_AGENT,
        &flight.rock,
        f64::from(ROCK_RADIUS) / 2.0,
        &DVec3::Y,
        ARRIVAL_RADIUS,
    );
    let blind = axes_sensing(&mut flight, 0);
    assert_ne!(axes_sensing(&mut flight, ALL_BITS), blind);
}

#[test]
fn a_later_behaviour_avoids_the_body_an_earlier_pursuit_left_out() {
    let mut flight = flight_past_a_rock();
    let avoided = axes_sensing(&mut flight, ALL_BITS);
    flight
        .nav
        .set_pursue(FIRST_AGENT, &flight.rock, STAND_OFF, ARRIVAL_RADIUS);
    flight.nav.step(TICK_S);
    assert_ne!(
        axes_of(&flight.nav),
        avoided,
        "the pursuit ignores the rock"
    );

    flight.nav.set_arrive(
        FIRST_AGENT,
        &goal_at(0.0, -LONG_GOAL_DISTANCE),
        ARRIVAL_RADIUS,
    );
    flight.nav.step(TICK_S);
    assert_eq!(axes_of(&flight.nav), avoided);
}

#[test]
fn a_followed_agent_is_left_out_of_the_followers_scene_and_an_unfollowed_one_is_not() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let leader = body_in(&mut physics, ROCK_RADIUS, goal_at(0.0, -ROCK_DISTANCE));
    let mut nav = NavWorld::new(&physics);
    let follower = add_ship(&mut nav, &ship);
    add_ship(&mut nav, &leader);
    // The leader faces -z, so the slot lies beyond it, and the leader is on the course to it.
    nav.set_follow(follower, &leader, 0.0, 0.0, ROCK_DISTANCE, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert!(yaw_of(&nav).abs() < 1e-9, "straight at the slot");

    nav.set_arrive(
        follower,
        &goal_at(0.0, -2.0 * ROCK_DISTANCE),
        ARRIVAL_RADIUS,
    );
    nav.step(TICK_S);
    assert!(
        yaw_of(&nav).abs() > 0.0,
        "the same course, around the leader"
    );
}

#[test]
fn a_sensed_body_that_leaves_physics_is_not_a_problem() {
    let mut flight = flight_past_a_rock();
    flight.nav.set_sensing(ALL_BITS);
    flight.nav.step(TICK_S);
    assert_eq!(flight.nav.obstacle_spheres.len(), 1);
    flight.physics.remove_rigid_body(&mut flight.rock);
    flight.physics.update(ONE_STEP);
    flight.nav.step(TICK_S);
    assert!(flight.nav.obstacle_spheres.is_empty());
}

#[test]
fn a_body_beyond_the_sensing_range_is_not_sensed() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive(id, &goal_at(0.0, -LONG_GOAL_DISTANCE), ARRIVAL_RADIUS);
    nav.set_sensing(ALL_BITS);
    nav.step(TICK_S);
    let tuning = *nav.world.tuning();
    let state = state_in(&physics, &ship);
    let profile = profile_of(&nav.agents[0].as_ref().unwrap().limits, &state);
    let range = sensing_range(&nav_body(&state), &profile, &tuning);
    let _beyond = body_in(
        &mut physics,
        ROCK_RADIUS,
        goal_at(0.0, -(range + f64::from(ROCK_RADIUS) + 1.0)),
    );
    let within = body_in(&mut physics, ROCK_RADIUS, goal_at(range - 1.0, 0.0));
    physics.update(ONE_STEP);
    nav.step(TICK_S);
    assert_eq!(
        nav.obstacle_spheres.len(),
        1,
        "only the body inside the range"
    );
    assert_eq!(
        nav.obstacle_spheres[0].position,
        state_in(&physics, &within).position
    );
}

/// A velocity far above the ships' top speed.
const CARRY: f64 = 400.0;
const ANCHOR_DISTANCE: f64 = 500.0;

fn carry() -> DVec3 {
    DVec3::new(CARRY, 0.0, 0.0)
}

fn fly_at(body: &mut RigidBody, velocity: DVec3) {
    body.set_linear_velocity(&na::Vector3::new(velocity.x, velocity.y, velocity.z));
}

/// A ship and an anchor body flying together at `CARRY`, and an agent for the ship whose goal
/// is its own position, kept relative to the anchor. The fields only keep the bodies and
/// their physics world alive.
struct CarriedShip {
    nav: NavWorld,
    id: u32,
    _physics: Physics,
    _ship: Box<RigidBody>,
    _anchor: Box<RigidBody>,
}

fn carried_ship_on_its_goal() -> CarriedShip {
    let mut physics = Physics::new();
    let mut ship = ship_in(&mut physics);
    let mut anchor = body_in(&mut physics, SHIP_RADIUS, goal_at(0.0, -ANCHOR_DISTANCE));
    fly_at(&mut ship, carry());
    fly_at(&mut anchor, carry());
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    let offset = DVec3::new(0.0, 0.0, ANCHOR_DISTANCE);
    nav.set_arrive_relative(id, &anchor, &offset, ARRIVAL_RADIUS);
    CarriedShip {
        nav,
        id,
        _physics: physics,
        _ship: ship,
        _anchor: anchor,
    }
}

fn thrust_of(nav: &NavWorld) -> [f64; 3] {
    let axes = axes_of(nav);
    [
        axis(&axes, 0, NavAxis::ThrustForward),
        axis(&axes, 0, NavAxis::ThrustRight),
        axis(&axes, 0, NavAxis::ThrustUp),
    ]
}

#[test]
fn a_ship_at_rest_in_its_frame_on_its_goal_has_no_thrust() {
    let CarriedShip { mut nav, id, .. } = carried_ship_on_its_goal();
    nav.set_frame_velocity(id, &carry());
    nav.step(TICK_S);
    assert!(arrived_of(&nav));
    assert_eq!(thrust_of(&nav), [0.0; 3]);
}

#[test]
fn a_ship_that_is_not_told_its_frame_thrusts_against_the_drag_of_the_carry() {
    let CarriedShip { mut nav, .. } = carried_ship_on_its_goal();
    nav.step(TICK_S);
    assert_ne!(thrust_of(&nav), [0.0; 3]);
}

#[test]
fn an_arrive_relative_goal_follows_the_body_and_keeps_its_offset() {
    let mut physics = Physics::new();
    let ship = ship_in(&mut physics);
    let mut anchor = body_in(&mut physics, SHIP_RADIUS, goal_at(GOAL_DISTANCE, 0.0));
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive_relative(id, &anchor, &DVec3::ZERO, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 1.0);

    anchor.set_position(&goal_at(-GOAL_DISTANCE, 0.0));
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), -1.0);

    // The goal lies the offset from the body: back on the right of the ship.
    let offset = DVec3::new(2.0 * GOAL_DISTANCE, 0.0, 0.0);
    nav.set_arrive_relative(id, &anchor, &offset, ARRIVAL_RADIUS);
    nav.step(TICK_S);
    assert_eq!(yaw_of(&nav), 1.0);
}

#[test]
fn an_arrive_relative_goal_does_not_leave_its_body_out_of_the_scene() {
    let mut flight = flight_past_a_rock();
    let offset = DVec3::new(0.0, 0.0, -(LONG_GOAL_DISTANCE - ROCK_DISTANCE));
    flight
        .nav
        .set_arrive_relative(FIRST_AGENT, &flight.rock, &offset, ARRIVAL_RADIUS);
    let blind = axes_sensing(&mut flight, 0);
    assert_ne!(axes_sensing(&mut flight, ALL_BITS), blind);
}

#[test]
fn the_sensing_range_follows_the_velocity_relative_to_the_frame() {
    let mut physics = Physics::new();
    let mut ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive(id, &goal_at(0.0, -LONG_GOAL_DISTANCE), ARRIVAL_RADIUS);
    nav.set_sensing(ALL_BITS);
    let state = state_in(&physics, &ship);
    let profile = profile_of(&nav.agents[0].as_ref().unwrap().limits, &state);
    let range = sensing_range(&nav_body(&state), &profile, nav.world.tuning());
    let beyond = goal_at(0.0, -(range + f64::from(ROCK_RADIUS) + 1.0));
    let _beyond = body_in(&mut physics, ROCK_RADIUS, beyond);
    physics.update(ONE_STEP);
    // Carried at a speed that would stretch the range if the frame were ignored.
    fly_at(&mut ship, carry());
    nav.set_frame_velocity(id, &carry());
    nav.step(TICK_S);
    assert!(nav.obstacle_spheres.is_empty(), "the range is that at rest");

    nav.set_frame_velocity(id, &DVec3::ZERO);
    nav.step(TICK_S);
    assert_eq!(nav.obstacle_spheres.len(), 1, "the range grows with speed");
}

#[test]
#[should_panic(expected = "NavWorld has no agent 3")]
fn a_frame_velocity_for_an_unknown_agent_is_refused() {
    let physics = Physics::new();
    let mut nav = NavWorld::new(&physics);
    nav.set_frame_velocity(3, &DVec3::ZERO);
}

#[test]
#[should_panic(expected = "target: the body is not a root body of its physics world")]
fn an_arrive_relative_body_of_another_physics_world_is_refused() {
    let mut physics = Physics::new();
    let mut other = Physics::new();
    let stranger = ship_in(&mut other);
    let ship = ship_in(&mut physics);
    let mut nav = NavWorld::new(&physics);
    let id = add_ship(&mut nav, &ship);
    nav.set_arrive_relative(id, &stranger, &DVec3::ZERO, ARRIVAL_RADIUS);
}

#[test]
fn near_balls_share_one_query_that_contains_both() {
    let a = Ball::new(DVec3::ZERO, 100.0);
    let b = Ball::new(DVec3::new(30.0, 0.0, 0.0), 100.0);
    let mut queries = Vec::new();
    add_to_queries(&mut queries, a);
    add_to_queries(&mut queries, b);
    assert_eq!(queries.len(), 1);
    for ball in [a, b] {
        assert!(queries[0].centre.distance(ball.centre) + ball.radius <= queries[0].radius + 1e-9);
    }
}

#[test]
fn far_balls_keep_their_own_queries() {
    let near = Ball::new(DVec3::ZERO, 100.0);
    let far = Ball::new(DVec3::new(1000.0, 0.0, 0.0), 100.0);
    let mut queries = Vec::new();
    add_to_queries(&mut queries, near);
    add_to_queries(&mut queries, far);
    assert_eq!(queries, vec![near, far]);
}

#[test]
fn a_ball_inside_another_adds_no_query_and_changes_none() {
    let big = Ball::new(DVec3::ZERO, 100.0);
    let mut queries = vec![big];
    add_to_queries(&mut queries, Ball::new(DVec3::new(10.0, 0.0, 0.0), 20.0));
    assert_eq!(queries, vec![big]);
}
