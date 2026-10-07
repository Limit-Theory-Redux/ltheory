use std::collections::HashMap;
use std::sync::LazyLock;

use strum::IntoEnumIterator;

use super::FrameStage;
use crate::system::TimeStamp;

static FIXED_DT: LazyLock<bool> = LazyLock::new(|| std::env::var_os("LTHEORY_CAPTURE").is_some());

pub struct FrameTimer {
    last_update: HashMap<FrameStage, TimeStamp>,
}

impl FrameTimer {
    pub fn new() -> Self {
        let now = TimeStamp::now();
        let last_update = FrameStage::iter().map(|stage| (stage, now)).collect();
        FrameTimer { last_update }
    }

    pub fn update(&mut self, stage: FrameStage) -> f64 {
        // Deterministic mode for render validation captures (LTHEORY_CAPTURE):
        // every stage sees a fixed 60 Hz delta regardless of wall-clock time.
        if *FIXED_DT {
            return 1.0 / 60.0;
        }
        let now = TimeStamp::now();
        let last_time = self.last_update.get(&stage).cloned().unwrap_or(now);
        let delta = last_time.get_elapsed();
        self.last_update.insert(stage, now);
        delta
    }
}
