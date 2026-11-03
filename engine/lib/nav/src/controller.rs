use glam::DVec3;

use crate::agent::{AXIS_COUNT, Axis, Body, Frame, Profile};
use crate::behaviour::{TINY_SQ, Wanted};

/// Turn axis input per radian between the nose and the course; full input from half a radian.
const TURN_GAIN: f64 = 2.0;
/// Thrust axes reach full input at this fraction of the profile's top speed of velocity error.
const THRUST_ERROR_SPAN: f64 = 0.1;
/// Forward thrust only when the cosine between the nose and the course is above this.
const THRUST_ALIGNMENT: f64 = 0.7;

fn clamp_axis(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

/// Pitch and yaw that turn the nose toward the unit direction `course`.
fn turn_toward(frame: &Frame, course: DVec3) -> (f64, f64) {
    // The axis that brings the nose onto the course, and the angle between them.
    let axis = frame.forward.cross(course);
    let sin_angle = axis.length();
    let angle = sin_angle.atan2(frame.forward.dot(course));
    let axis = if sin_angle * sin_angle < TINY_SQ {
        // On course, or exactly reversed: any axis across the nose turns it.
        frame.right
    } else {
        axis / sin_angle
    };
    // Positive pitch turns about the right axis; positive yaw about the down axis.
    let pitch = clamp_axis(TURN_GAIN * angle * axis.dot(frame.right));
    let yaw = clamp_axis(-TURN_GAIN * angle * axis.dot(frame.up));
    (pitch, yaw)
}

/// The control axes that turn the agent toward the wanted course and bring its velocity to the
/// wanted one. A damping controller, with no model of the ship's response.
pub(crate) fn control(body: &Body, profile: &Profile, wanted: &Wanted) -> [f64; AXIS_COUNT] {
    let mut axes = [0.0; AXIS_COUNT];
    if wanted.direction.length_squared() < TINY_SQ {
        return axes;
    }
    let error = wanted.velocity - body.velocity;
    let error_sq = error.length_squared();

    // Too slow along the course: keep the nose on the velocity change, so thrust adds to it.
    // Otherwise keep it on the course, and drag and reverse thrust slow the agent down.
    let course = if error.dot(wanted.direction) > 0.0 && error_sq > TINY_SQ {
        error / error_sq.sqrt()
    } else {
        wanted.direction
    };
    let (pitch, yaw) = turn_toward(&body.frame, course);
    axes[Axis::Pitch as usize] = pitch;
    axes[Axis::Yaw as usize] = yaw;

    let span = THRUST_ERROR_SPAN * profile.max_speed;
    if body.frame.forward.dot(course) > THRUST_ALIGNMENT {
        axes[Axis::ThrustForward as usize] = clamp_axis(error.dot(body.frame.forward) / span);
    }
    axes[Axis::ThrustRight as usize] = clamp_axis(error.dot(body.frame.right) / span);
    axes[Axis::ThrustUp as usize] = clamp_axis(error.dot(body.frame.up) / span);
    axes
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: Profile = Profile {
        max_speed: 100.0,
        linear_drag: 2.0,
    };

    fn level_body(velocity: DVec3) -> Body {
        Body {
            position: DVec3::ZERO,
            velocity,
            frame: Frame {
                forward: DVec3::NEG_Z,
                right: DVec3::X,
                up: DVec3::Y,
            },
            radius: 1.0,
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
        assert!((axis(&axes, Axis::Yaw) - TURN_GAIN * angle).abs() < 1e-12);
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
    fn thrust_forward_is_proportional_then_clamped() {
        let span = THRUST_ERROR_SPAN * PROFILE.max_speed;
        let wanted_slow = wanted_toward(DVec3::NEG_Z, span / 2.0);
        let half = control(&level_body(DVec3::ZERO), &PROFILE, &wanted_slow);
        assert!((axis(&half, Axis::ThrustForward) - 0.5).abs() < 1e-12);

        let full = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(DVec3::NEG_Z, 100.0),
        );
        assert_eq!(axis(&full, Axis::ThrustForward), 1.0);
    }

    #[test]
    fn no_forward_thrust_while_the_nose_is_off_course() {
        let axes = control(
            &level_body(DVec3::ZERO),
            &PROFILE,
            &wanted_toward(DVec3::X, 100.0),
        );
        assert_eq!(axis(&axes, Axis::ThrustForward), 0.0);
        assert_eq!(
            axis(&axes, Axis::ThrustRight),
            1.0,
            "strafing still adds the velocity"
        );
    }

    #[test]
    fn too_fast_brakes_with_reverse_thrust() {
        let fast = level_body(DVec3::new(0.0, 0.0, -100.0));
        let axes = control(&fast, &PROFILE, &wanted_toward(DVec3::NEG_Z, 20.0));
        assert_eq!(axis(&axes, Axis::ThrustForward), -1.0);
        assert_eq!(axis(&axes, Axis::Yaw), 0.0, "the nose stays on the course");
    }

    #[test]
    fn sideways_drift_is_cancelled_by_strafing() {
        let drifting = level_body(DVec3::new(10.0, -10.0, -50.0));
        let axes = control(&drifting, &PROFILE, &wanted_toward(DVec3::NEG_Z, 50.0));
        assert_eq!(axis(&axes, Axis::ThrustRight), -1.0);
        assert_eq!(axis(&axes, Axis::ThrustUp), 1.0);
    }

    #[test]
    fn every_axis_stays_in_range() {
        let wild = level_body(DVec3::new(900.0, -400.0, 300.0));
        let axes = control(
            &wild,
            &PROFILE,
            &wanted_toward(DVec3::new(-1.0, 2.0, 3.0).normalize(), 100.0),
        );
        assert!(axes.iter().all(|a| a.abs() <= 1.0), "{axes:?}");
    }
}
