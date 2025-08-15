use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToggleDetector {
    // last saved state.
    last_state: Option<bool>,
    toggle_count: u8,
    // time of first toggle in current sequence.
    first_toggle_time: Option<Instant>,
    required_toggles: u8,
    time_window: Duration,
}

impl ToggleDetector {
    pub fn new() -> Self {
        Self {
            last_state: None,
            toggle_count: 0,
            first_toggle_time: None,
            required_toggles: 6,
            time_window: Duration::from_secs(10),
        }
    }

    pub fn update(&mut self, current_el_state: bool) -> bool {
        // setup initializer value
        if self.last_state.is_none() {
            self.last_state = Some(current_el_state);
            return false;
        }

        if current_el_state != self.last_state.unwrap() {
            let now = Instant::now();

            if self.first_toggle_time.is_none() {
                self.first_toggle_time = Some(now);
                self.toggle_count = 1;
            } else if now.duration_since(self.first_toggle_time.unwrap()) <= self.time_window {
                self.toggle_count += 1;
                if self.toggle_count >= self.required_toggles {
                    self.reset_detector();
                    return true;
                }
            } else {
                // Window expired - start completely fresh
                self.reset_detector();
                self.first_toggle_time = Some(now);
            }
        } else if let Some(first_time) = self.first_toggle_time {
            if Instant::now().duration_since(first_time) > self.time_window {
                self.reset_detector();
            }
        }
        self.last_state = Some(current_el_state);
        false
    }

    fn reset_detector(&mut self) {
        self.toggle_count = 0;
        self.first_toggle_time = None;
    }
}

impl Default for ToggleDetector {
    fn default() -> Self {
        Self::new()
    }
}
