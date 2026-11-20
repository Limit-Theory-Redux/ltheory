//! The braking model: how far an agent runs before it stops, and the speed it can stop within a
//! distance from. The model is a damped double integrator braking against its own motion:
//! `dv/dt = −a_b − k·v`, with reverse thrust `a_b` and linear drag `k`. Design notes:
//! `doc/engine/nav.md`.

use crate::agent::Profile;
use crate::controller::VELOCITY_GAIN;

/// The share of the forward thrust that braking is planned on. The velocity loop lags the wanted
/// velocity, so planning on the full thrust would leave no room to catch up and the agent would
/// overshoot its stop. Avoidance brakes on the same curve, and a larger share lets it close on
/// obstacles faster than a detour can turn, so the share stays small.
pub const BRAKING_THRUST_SHARE: f64 = 0.1;

/// The ratio of the velocity loop's bandwidth to the gain of a position loop around it that is
/// critically damped: a loop `s² + ω·s + G·ω = 0` is critical at `G = ω / 4`.
const CRITICAL_DAMPING_RATIO: f64 = 4.0;

/// Below this `x`, `x − ln(1 + x)` is summed as a series: the difference of the two terms loses
/// all its digits as `x` goes to 0.
const SERIES_LIMIT: f64 = 0.1;

/// The last power of `x` in the series of `x − ln(1 + x)`, which alternates and shrinks by at
/// least `SERIES_LIMIT` per term: below the limit the cut-off is under double precision.
const SERIES_LAST_POWER: i32 = 16;

/// The most steps the inverse takes. Halley's method converges cubically from its start, so a
/// handful reaches double precision; the cap only bounds a bad input.
const MAX_INVERSE_STEPS: usize = 12;

/// The relative step below which the inverse has converged.
const INVERSE_TOLERANCE: f64 = 1.0e-14;

/// The deceleration the braking model plans on, in length units per second squared.
fn braking_acceleration(profile: &Profile) -> f64 {
    BRAKING_THRUST_SHARE * profile.accel_forward
}

/// `x − ln(1 + x)` for `x ≥ 0`: the stopping distance in units of `a_b / k²` at the speed
/// `x · a_b / k`.
fn excess(x: f64) -> f64 {
    if x >= SERIES_LIMIT {
        return x - x.ln_1p();
    }
    // x²/2 − x³/3 + x⁴/4 − …, summed from the smallest term up.
    (2..=SERIES_LAST_POWER)
        .rev()
        .map(|power| {
            let sign = if power % 2 == 0 { 1.0 } else { -1.0 };
            sign * x.powi(power) / f64::from(power)
        })
        .sum()
}

/// The `x ≥ 0` with `excess(x) = c`. The function is convex and increasing, so Halley's method
/// from a start on the high side converges monotonically.
///
/// A closed form exists through the Lambert W function (`1 + x = −W₋₁(−e^(−1−c))`), but it loses
/// its digits near `c = 0`, the end of the braking that matters most, and needs a dependency for
/// one scalar function.
fn inverse_excess(c: f64) -> f64 {
    // The root is `√(2c)` for small `c` and `c + ln(1 + c)` for large `c`; the larger of the
    // two is never below it.
    let mut x = (2.0 * c).sqrt().max(c + c.ln_1p());
    for _ in 0..MAX_INVERSE_STEPS {
        let error = excess(x) - c;
        let slope = x / (1.0 + x);
        let curvature = 1.0 / ((1.0 + x) * (1.0 + x));
        let step = 2.0 * error * slope / (2.0 * slope * slope - error * curvature);
        x -= step;
        if step.abs() <= INVERSE_TOLERANCE * x {
            break;
        }
    }
    x
}

/// The distance an agent at `speed` (length units per second) runs while it brakes to rest, in
/// length units. 0 for a speed of 0 or less.
pub(crate) fn stopping_distance(profile: &Profile, speed: f64) -> f64 {
    let drag = profile.linear_drag;
    let thrust = braking_acceleration(profile);
    if speed <= 0.0 {
        return 0.0;
    }
    if thrust <= 0.0 {
        // Drag alone: it covers `speed / k` and never stops an agent without drag.
        return if drag > 0.0 {
            speed / drag
        } else {
            f64::INFINITY
        };
    }
    if drag <= 0.0 {
        return speed * speed / (2.0 * thrust);
    }
    thrust / (drag * drag) * excess(drag * speed / thrust)
}

/// The gain, per second, of the wanted speed on the distance close to the stop. It is the
/// critically damped gain of a position loop around the velocity loop, never below the drag
/// rate, which keeps the wanted speed at least what drag alone stops.
fn tail_gain(profile: &Profile) -> f64 {
    let drag = profile.linear_drag;
    ((VELOCITY_GAIN + drag) / CRITICAL_DAMPING_RATIO).max(drag)
}

/// The speed from which the agent can brake to rest within `distance`, in length units per
/// second; 0 for a distance of 0 or less. It is the inverse of [`stopping_distance`], limited near
/// the stop to `tail_gain · distance`: the braking curve's slope is unbounded at the stop, and
/// a velocity loop of finite gain can't follow it. Never below the speed drag alone stops from
/// within `distance` (`distance · k`).
pub(crate) fn stopping_speed(profile: &Profile, distance: f64) -> f64 {
    let drag = profile.linear_drag;
    let thrust = braking_acceleration(profile);
    if distance <= 0.0 {
        return 0.0;
    }
    if thrust <= 0.0 {
        return drag * distance;
    }
    let curve = if drag <= 0.0 {
        (2.0 * thrust * distance).sqrt()
    } else {
        thrust / drag * inverse_excess(distance * drag * drag / thrust)
    };
    curve.min(tail_gain(profile) * distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: Profile = Profile::TEST;

    fn braking(profile: &Profile) -> f64 {
        braking_acceleration(profile)
    }

    #[test]
    fn the_excess_matches_the_logarithm_and_its_series_where_they_meet() {
        let x: f64 = SERIES_LIMIT * (1.0 - 1.0e-9);
        let direct = x - x.ln_1p();
        assert!(
            (excess(x) - direct).abs() < 1.0e-9 * direct,
            "{} against {direct}",
            excess(x)
        );
        let x: f64 = 3.0;
        assert!((excess(x) - (x - (1.0 + x).ln())).abs() < 1.0e-15);
        let tiny = 1.0e-6;
        assert!((excess(tiny) - tiny * tiny / 2.0).abs() < 1.0e-6 * tiny * tiny);
        assert_eq!(excess(0.0), 0.0);
    }

    #[test]
    fn stopping_distance_solves_the_braking_equation() {
        // From speed v under deceleration a_b + k·v, a ship covers v/k − (a_b/k²)·ln(1 + k·v/a_b).
        let speed = 80.0;
        let (drag, thrust) = (PROFILE.linear_drag, braking(&PROFILE));
        let expected = speed / drag - thrust / (drag * drag) * (1.0 + drag * speed / thrust).ln();
        assert!((stopping_distance(&PROFILE, speed) - expected).abs() < 1.0e-9);
        assert_eq!(stopping_distance(&PROFILE, 0.0), 0.0);
        assert_eq!(stopping_distance(&PROFILE, -3.0), 0.0);
    }

    #[test]
    fn thrust_shortens_the_stop_that_drag_alone_would_take() {
        let speed = 100.0;
        let drag_only = speed / PROFILE.linear_drag;
        let braked = stopping_distance(&PROFILE, speed);
        // A tenth of the thrust that holds the top speed shortens the stop of a ship at it by a
        // fifth or more.
        assert!(braked < 0.8 * drag_only, "{braked} against {drag_only}");
    }

    #[test]
    fn without_drag_the_stop_is_the_constant_deceleration_one() {
        let no_drag = Profile {
            linear_drag: 0.0,
            ..PROFILE
        };
        let thrust = braking(&no_drag);
        assert!((stopping_distance(&no_drag, 30.0) - 900.0 / (2.0 * thrust)).abs() < 1.0e-9);
        // Far enough from the stop that the tail's limit doesn't bind.
        let distance = 1.0e3;
        assert!(
            (stopping_speed(&no_drag, distance) - (2.0 * thrust * distance).sqrt()).abs() < 1.0e-9
        );
    }

    #[test]
    fn without_thrust_the_stop_is_the_drag_one() {
        let no_thrust = Profile {
            accel_forward: 0.0,
            ..PROFILE
        };
        assert_eq!(stopping_distance(&no_thrust, 30.0), 15.0);
        assert_eq!(stopping_speed(&no_thrust, 15.0), 30.0);
        let nothing = Profile {
            linear_drag: 0.0,
            ..no_thrust
        };
        assert_eq!(stopping_distance(&nothing, 30.0), f64::INFINITY);
        assert_eq!(stopping_speed(&nothing, 15.0), 0.0);
    }

    #[test]
    fn stopping_speed_inverts_the_distance_away_from_the_stop() {
        // Past the tail's reach (about 20 for this profile) the speed is the curve's, so the round
        // trip returns the distance.
        let mut curve_cases = 0;
        for distance in [150.0, 300.0, 1.0e3, 1.0e4, 1.0e6] {
            let speed = stopping_speed(&PROFILE, distance);
            let back = stopping_distance(&PROFILE, speed);
            assert!(
                (back - distance).abs() < 1.0e-9 * distance,
                "{distance}: {back}"
            );
            curve_cases += 1;
        }
        assert_eq!(curve_cases, 5);
    }

    #[test]
    fn stopping_speed_never_asks_for_more_than_braking_gives() {
        // Whatever the distance, braking from the wanted speed stops within it.
        let mut tail_cases = 0;
        let mut curve_cases = 0;
        let mut distance = 1.0e-9;
        while distance < 1.0e7 {
            let speed = stopping_speed(&PROFILE, distance);
            assert!(
                stopping_distance(&PROFILE, speed) <= distance * (1.0 + 1.0e-9),
                "{distance}"
            );
            if (speed - tail_gain(&PROFILE) * distance).abs() <= 1.0e-12 * speed {
                tail_cases += 1;
            } else {
                curve_cases += 1;
            }
            distance *= 1.7;
        }
        assert!(
            tail_cases > 10 && curve_cases > 10,
            "{tail_cases}, {curve_cases}"
        );
    }

    #[test]
    fn stopping_speed_is_at_least_what_drag_alone_stops_from() {
        for distance in [1.0e-6, 0.01, 1.0, 10.0, 1.0e3, 1.0e5] {
            assert!(
                stopping_speed(&PROFILE, distance) >= PROFILE.linear_drag * distance,
                "{distance}"
            );
        }
        assert_eq!(stopping_speed(&PROFILE, 0.0), 0.0);
        assert_eq!(stopping_speed(&PROFILE, -1.0), 0.0);
    }

    #[test]
    fn stopping_speed_grows_with_the_distance() {
        let mut last = 0.0;
        let mut distance = 1.0e-6;
        while distance < 1.0e6 {
            let speed = stopping_speed(&PROFILE, distance);
            assert!(speed > last, "{distance}: {speed} after {last}");
            last = speed;
            distance *= 1.3;
        }
    }

    #[test]
    fn near_the_stop_the_wanted_speed_is_linear_in_the_distance() {
        let distance = 1.0e-3;
        let speed = stopping_speed(&PROFILE, distance);
        assert!((speed - tail_gain(&PROFILE) * distance).abs() < 1.0e-12);
        let strong = Profile {
            linear_drag: 20.0,
            ..PROFILE
        };
        assert_eq!(
            tail_gain(&strong),
            strong.linear_drag,
            "never below the drag rate"
        );
    }
}
