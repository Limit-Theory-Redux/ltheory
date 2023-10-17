use crate::system::TimeStamp;

pub struct ControlState {
    is_connected: bool,
    last_event_timestamp: TimeStamp,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            is_connected: true,
            last_event_timestamp: Default::default(),
        }
    }
}

impl ControlState {
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn connect(&mut self) {
        self.is_connected = true;
    }

    pub fn disconnect(&mut self) {
        self.is_connected = false;
    }

    pub fn last_event_timestamp(&self) -> TimeStamp {
        self.last_event_timestamp
    }

    pub fn update(&mut self) -> bool {
        if self.is_connected {
            self.last_event_timestamp = TimeStamp::now();
            true
        } else {
            false
        }
    }
}
