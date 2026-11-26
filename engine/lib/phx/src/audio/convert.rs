use std::time::Duration;

use kira::{Decibels, Easing, StartTime, Tween};

/// Converts a linear amplitude (`1.0` leaves a sound unchanged) to decibels.
///
/// Amplitudes at or below `0.001` map to [`Decibels::SILENCE`], which kira plays as silence.
pub(crate) fn amplitude_to_decibels(amplitude: f64) -> Decibels {
    if amplitude <= 0.0 {
        return Decibels::SILENCE;
    }

    Decibels(((20.0 * amplitude.log10()) as f32).max(Decibels::SILENCE.0))
}

/// A linear tween that starts immediately and lasts `millis` milliseconds.
pub(crate) fn linear_tween(millis: u64) -> Tween {
    Tween {
        start_time: StartTime::Immediate,
        duration: Duration::from_millis(millis),
        easing: Easing::Linear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_amplitude_is_identity() {
        assert_eq!(amplitude_to_decibels(1.0), Decibels::IDENTITY);
    }

    #[test]
    fn zero_and_negative_amplitudes_are_silent() {
        assert_eq!(amplitude_to_decibels(0.0), Decibels::SILENCE);
        assert_eq!(amplitude_to_decibels(-1.0), Decibels::SILENCE);
        assert_eq!(amplitude_to_decibels(0.0001), Decibels::SILENCE);
    }

    #[test]
    fn decibels_round_trip_to_the_same_amplitude() {
        for amplitude in [0.01, 0.25, 0.5, 0.8, 1.0, 2.0] {
            let round_trip = amplitude_to_decibels(amplitude).as_amplitude() as f64;
            assert!(
                (round_trip - amplitude).abs() < 1e-5,
                "{amplitude} came back as {round_trip}"
            );
        }
    }
}
