use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Timer};

use crate::debounced_button::DebouncedButton;
pub struct ToggleWithMomentary {
    pub is_enabled: bool,
    pub is_held: bool,
    start_timer: bool,
    button: DebouncedButton,
    hold_threshold: Duration,
}

impl ToggleWithMomentary {
    pub fn new(is_enabled: bool, button: DebouncedButton, hold_threshold: Duration) -> Self {
        ToggleWithMomentary {
            button,
            is_enabled,
            is_held: false,
            start_timer: false,
            hold_threshold,
        }
    }

    pub async fn on_change(&mut self) {
        let timer = if self.start_timer {
            Timer::after(self.hold_threshold)
        } else {
            Timer::after(Duration::from_secs(31_536_000 * 10)) // ten years, arbitrarily large
        };

        match select(self.button.on_change(), timer).await {
            Either::First(is_pressed) => {
                if is_pressed {
                    self.is_enabled = !self.is_enabled;
                    self.start_timer = true;
                } else {
                    if self.is_held {
                        self.is_enabled = false
                    }
                    self.is_held = false;
                    self.start_timer = false;
                }
            }
            Either::Second(_) => {
                self.is_held = true;
                self.start_timer = false;
            }
        }
    }
}
