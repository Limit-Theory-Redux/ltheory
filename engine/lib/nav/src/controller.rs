use glam::DVec3;

use crate::agent::{AXIS_COUNT, Axis, Body, DragRate, Frame, Profile};
use crate::behaviour::{TINY_SQ, Wanted};

/// Wanted acceleration per unit of velocity error, per second: the velocity loop's gain.
const VELOCITY_GAIN: f64 = 10.0;
/// The heading error, in radians, at which the turn asks for its full angular acceleration.
const TURN_SATURATION_ANGLE: f64 = 0.5;
/// Upper bound of the turn loop's natural frequency, in radians per second. It keeps the loop
/// stable for turning gear that is much stronger than the drag.
const MAX_TURN_FREQUENCY: f64 = 10.0;
/// Damping ratio of the turn loop, counting the angular drag.
const TURN_DAMPING_RATIO: f64 = 1.0;
/// The roll rate, in radians per second, at which the roll axis reaches full input.
const ROLL_RATE_SPAN: f64 = 1.0;

fn clamp_axis(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

/// `value / limit`, or 0 for an axis without authority.
fn fraction_of(value: f64, limit: f64) -> f64 {
    if limit > 0.0 { value / limit } else { 0.0 }
}

/// The acceleration that brings the agent to the wanted velocity: a proportional term on the
/// velocity error, plus the acceleration that holds the wanted velocity against linear drag.
fn wanted_acceleration(body: &Body, profile: &Profile, wanted: &Wanted) -> DVec3 {
    VELOCITY_GAIN * (wanted.velocity - body.velocity) + profile.linear_drag * wanted.velocity
}

/// Thrust inputs (forward, right, up) that give `acceleration`. When an axis can't give its share,
/// all three are scaled down together, so the agent still accelerates in the wanted direction.
fn thrust_inputs(frame: &Frame, profile: &Profile, acceleration: DVec3) -> [f64; 3] {
    let inputs = [
        fraction_of(acceleration.dot(frame.forward), profile.accel_forward),
        fraction_of(acceleration.dot(frame.right), profile.accel_right),
        fraction_of(acceleration.dot(frame.up), profile.accel_up),
    ];
    let peak = inputs
        .iter()
        .fold(0.0_f64, |peak, input| peak.max(input.abs()));
    // Dividing, not multiplying by the reciprocal, so the peak axis is exactly full.
    inputs.map(|input| input / peak.max(1.0))
}

/// The unit direction the nose points along. While the agent is gaining speed toward its course
/// with more acceleration than the side thrusters give, it is the acceleration, so the main
/// thruster does the work. Otherwise it is the course, and strafing and braking do the rest.
fn aim_direction(profile: &Profile, wanted: &Wanted, acceleration: DVec3) -> DVec3 {
    let side_limit = profile.accel_right.min(profile.accel_up);
    let speeding_up = acceleration.dot(wanted.direction) > 0.0;
    if speeding_up && acceleration.length_squared() > side_limit * side_limit {
        acceleration.normalize()
    } else {
        wanted.direction
    }
}

/// The rotation vector (axis times angle) that brings the nose onto the unit direction `course`.
fn heading_error(frame: &Frame, course: DVec3) -> DVec3 {
    let axis = frame.forward.cross(course);
    let sin_angle = axis.length();
    let angle = sin_angle.atan2(frame.forward.dot(course));
    if sin_angle * sin_angle < TINY_SQ {
        // On course, or exactly reversed: any axis across the nose turns it.
        frame.right * angle
    } else {
        axis * (angle / sin_angle)
    }
}

/// The input of one turning axis: a PD law on the heading error and the measured turn rate, in
/// units of the axis's angular acceleration limit. The stiffness gives the full limit at
/// `TURN_SATURATION_ANGLE`; the damping makes the loop critically damped with the angular drag
/// counted, and is 0 when the drag alone damps it more.
fn turn_input(error: f64, rate: f64, limit: f64, angular_drag: DragRate) -> f64 {
    if limit <= 0.0 {
        return 0.0;
    }
    let stiffness = (limit / TURN_SATURATION_ANGLE).min(MAX_TURN_FREQUENCY * MAX_TURN_FREQUENCY);
    let damping = (2.0 * TURN_DAMPING_RATIO * stiffness.sqrt() - angular_drag).max(0.0);
    clamp_axis((stiffness * error - damping * rate) / limit)
}

/// The control axes that bring the agent's velocity to the wanted one and turn its nose to the
/// aim direction, within the profile's limits.
pub(crate) fn control(body: &Body, profile: &Profile, wanted: &Wanted) -> [f64; AXIS_COUNT] {
    let mut axes = [0.0; AXIS_COUNT];
    if wanted.direction.length_squared() < TINY_SQ {
        return axes;
    }
    let frame = &body.frame;
    let acceleration = wanted_acceleration(body, profile, wanted);

    let [forward, right, up] = thrust_inputs(frame, profile, acceleration);
    axes[Axis::ThrustForward as usize] = forward;
    axes[Axis::ThrustRight as usize] = right;
    axes[Axis::ThrustUp as usize] = up;

    // Positive pitch turns about the right axis, positive yaw about the down axis and positive
    // roll about the forward axis.
    let error = heading_error(frame, aim_direction(profile, wanted, acceleration));
    let rate = body.angular_velocity;
    axes[Axis::Pitch as usize] = turn_input(
        error.dot(frame.right),
        rate.dot(frame.right),
        profile.turn_pitch,
        profile.angular_drag,
    );
    axes[Axis::Yaw as usize] = turn_input(
        -error.dot(frame.up),
        -rate.dot(frame.up),
        profile.turn_yaw,
        profile.angular_drag,
    );
    if profile.turn_roll > 0.0 {
        axes[Axis::Roll as usize] = clamp_axis(-rate.dot(frame.forward) / ROLL_RATE_SPAN);
    }
    axes
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: Profile = Profile::TEST;

    fn level_body(velocity: DVec3) -> Body {
        Body {
            velocity,
            radius: 1.0,
            ..Body::AT_REST
        }
    }

    fn wanted_toward(direction: DVec3, speed: f64) -> Wanted {
        Wanted {
            velocity: direction * speed,
            direction,
        }
    }

    fn axis(axes: &[f64; AXIS_COUNT], which: Axis) -> f64 {
        axes[which as usize]
    }

    #[test]
    fn nothing_wanted_gives_no_input() {
        let moving = level_body(DVec3::new(0.0, 0.0, -50.0));
        assert_eq!(control(&moving, &PROFILE, &Wanted::NONE), [0.0; AXIS_COUNT]);
    }

    #[test]
    fn a_course_to_the_right_yaws_right() {
        let axes = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(DVec3::X, 50.0),
        );
        assert_eq!(axis(&axes, Axis::Yaw), 1.0);
        assert_eq!(axis(&axes, Axis::Pitch), 0.0);
    }

    #[test]
    fn a_course_above_pitches_up() {
        let axes = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(DVec3::Y, 50.0),
        );
        assert_eq!(axis(&axes, Axis::Pitch), 1.0);
        assert_eq!(axis(&axes, Axis::Yaw), 0.0);
    }

    #[test]
    fn a_small_heading_error_gives_a_proportional_turn() {
        let angle = 0.1_f64;
        let course = DVec3::new(angle.sin(), 0.0, -angle.cos());
        let axes = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(course, 50.0),
        );
        assert!((axis(&axes, Axis::Yaw) - angle / TURN_SATURATION_ANGLE).abs() < 1e-12);
    }

    #[test]
    fn a_reversed_course_turns_about_the_right_axis() {
        let axes = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(DVec3::Z, 50.0),
        );
        assert_eq!(axis(&axes, Axis::Pitch), 1.0);
        assert_eq!(axis(&axes, Axis::Yaw), 0.0);
    }

    #[test]
    fn a_turn_already_under_way_is_damped() {
        let angle = 0.1_f64;
        let course = DVec3::new(angle.sin(), 0.0, -angle.cos());
        let wanted = wanted_toward(course, 50.0);
        let yaw_with = |rate: f64| {
            // A positive yaw rate is a turn about the down axis.
            let turning = Body {
                angular_velocity: DVec3::new(0.0, -rate, 0.0),
                ..level_body(DVec3::ZERO)
            };
            axis(&control(&turning, &PROFILE, &wanted), Axis::Yaw)
        };
        assert!(yaw_with(0.5) < yaw_with(0.0));
        assert!(yaw_with(2.0) < 0.0, "a fast turn onto the course brakes");
    }

    #[test]
    fn roll_is_damped() {
        let wanted = wanted_toward(DVec3::NEG_Z, 50.0);
        let rolling = Body {
            // About the forward axis (-z) by +0.5 rad/s.
            angular_velocity: DVec3::new(0.0, 0.0, -0.5),
            ..level_body(DVec3::ZERO)
        };
        let axes = control(&rolling, &PROFILE, &wanted);
        assert!((axis(&axes, Axis::Roll) + 0.5 / ROLL_RATE_SPAN).abs() < 1e-12);
        let still = control(&level_body(DVec3::ZERO), &PROFILE, &wanted);
        assert_eq!(axis(&still, Axis::Roll), 0.0);
    }

    #[test]
    fn thrust_forward_is_proportional_then_clamped() {
        // At rest, wanted speed `v` gives (VELOCITY_GAIN + drag) · v of acceleration.
        let speed = 5.0;
        let accel = (VELOCITY_GAIN + PROFILE.linear_drag) * speed;
        let half = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(DVec3::NEG_Z, speed),
        );
        assert!((axis(&half, Axis::ThrustForward) - accel / PROFILE.accel_forward).abs() < 1e-12);

        let full = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(DVec3::NEG_Z, 100.0),
        );
        assert_eq!(axis(&full, Axis::ThrustForward), 1.0);
    }

    #[test]
    fn holding_the_wanted_velocity_thrusts_against_drag() {
        let speed = 50.0;
        let cruising = level_body(DVec3::new(0.0, 0.0, -speed));
        let axes = control(&cruising, &PROFILE, &wanted_toward(DVec3::NEG_Z, speed));
        let hold = PROFILE.linear_drag * speed / PROFILE.accel_forward;
        assert!((axis(&axes, Axis::ThrustForward) - hold).abs() < 1e-12);
        assert_eq!(axis(&axes, Axis::ThrustRight), 0.0);
    }

    #[test]
    fn thrust_off_course_keeps_the_direction_of_the_wanted_acceleration() {
        // Halfway between the nose and the right: both axes are asked for the same acceleration,
        // the weaker side thruster saturates, and the main thruster is scaled to match it.
        let course = DVec3::new(1.0, 0.0, -1.0).normalize();
        let axes = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(course, 50.0),
        );
        assert_eq!(axis(&axes, Axis::ThrustRight), 1.0);
        let forward = axis(&axes, Axis::ThrustForward) * PROFILE.accel_forward;
        let right = axis(&axes, Axis::ThrustRight) * PROFILE.accel_right;
        assert!((forward - right).abs() < 1e-9, "{forward} against {right}");
    }

    #[test]
    fn too_fast_brakes_with_reverse_thrust() {
        let fast = level_body(DVec3::new(0.0, 0.0, -100.0));
        let axes = control(&fast, &PROFILE, &wanted_toward(DVec3::NEG_Z, 20.0));
        assert_eq!(axis(&axes, Axis::ThrustForward), -1.0);
        assert_eq!(axis(&axes, Axis::Yaw), 0.0, "the nose stays on the course");
    }

    #[test]
    fn braking_needs_no_heading() {
        // Tail to the direction of travel: the nose thrusts along +z, which slows the agent.
        let reversed = Body {
            frame: Frame {
                forward: DVec3::Z,
                right: DVec3::NEG_X,
                up: DVec3::Y,
            },
            ..level_body(DVec3::new(0.0, 0.0, -100.0))
        };
        let axes = control(&reversed, &PROFILE, &wanted_toward(DVec3::NEG_Z, 20.0));
        assert_eq!(axis(&axes, Axis::ThrustForward), 1.0);
    }

    #[test]
    fn sideways_drift_is_cancelled_by_strafing() {
        let drifting = level_body(DVec3::new(10.0, -10.0, -50.0));
        let axes = control(&drifting, &PROFILE, &wanted_toward(DVec3::NEG_Z, 50.0));
        assert_eq!(axis(&axes, Axis::ThrustRight), -1.0);
        assert_eq!(axis(&axes, Axis::ThrustUp), 1.0);
    }

    #[test]
    fn an_axis_without_authority_gets_no_input() {
        let weak = Profile {
            accel_right: 0.0,
            turn_yaw: 0.0,
            turn_roll: 0.0,
            ..PROFILE
        };
        let rolling = Body {
            angular_velocity: DVec3::new(0.0, 0.0, -1.0),
            ..level_body(DVec3::ZERO)
        };
        let axes = control(&rolling, &weak, &wanted_toward(DVec3::X, 50.0));
        assert_eq!(axis(&axes, Axis::ThrustRight), 0.0);
        assert_eq!(axis(&axes, Axis::Yaw), 0.0);
        assert_eq!(axis(&axes, Axis::Roll), 0.0);
    }

    #[test]
    fn every_axis_stays_in_range() {
        let wild = Body {
            angular_velocity: DVec3::new(30.0, -20.0, 10.0),
            ..level_body(DVec3::new(900.0, -400.0, 300.0))
        };
        let axes = control(
            &wild,
            &PROFILE,
            &wanted_toward(DVec3::new(-1.0, 2.0, 3.0).normalize(), 100.0),
        );
        assert!(axes.iter().all(|a| a.abs() <= 1.0), "{axes:?}");
    }
}
