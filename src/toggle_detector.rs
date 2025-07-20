use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToggleDetector {
    last_state: bool,
    toggle_count: u8,
    first_toggle_time: Option<Instant>,
    required_toggles: u8,
    time_window: Duration,
}

impl ToggleDetector {
    pub fn new() -> Self {
        Self {
            last_state: false,
            toggle_count: 0,
            first_toggle_time: None,
            required_toggles: 6,
            time_window: Duration::from_secs(8),
        }
    }

    pub fn update(&mut self, current_el_state: bool) -> bool {
        if current_el_state != self.last_state {
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
                self.first_toggle_time = Some(now);
                self.toggle_count = 1;
            }
        } else if let Some(first_time) = self.first_toggle_time {
            if Instant::now().duration_since(first_time) > self.time_window {
                self.reset_detector();
            }
        }
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
