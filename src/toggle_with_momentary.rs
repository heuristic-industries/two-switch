use embassy_stm32::{exti::AnyChannel, gpio::AnyPin};
use embassy_time::{Duration, Instant};

use crate::DebouncedButton;

pub struct ToggleWithMomentary {
    pub is_enabled: bool,
    pub hold_threshold: Duration,
    button: DebouncedButton,
    pressed_time: Instant,
}

impl ToggleWithMomentary {
    pub fn new(
        is_enabled: bool,
        input_pin: AnyPin,
        debounce_threshold: Duration,
        hold_threshold: Duration,
        channel: AnyChannel,
    ) -> Self {
        let button = DebouncedButton::new(input_pin, debounce_threshold, channel);

        ToggleWithMomentary {
            button,
            is_enabled,
            hold_threshold,
            pressed_time: Instant::MIN,
        }
    }

    pub async fn wait_for_state_change(&mut self) {
        let mut next_state = self.is_enabled;

        loop {
            self.button.wait_for_any_edge().await;
            let pressed = self.button.is_pressed;
            let timestamp = Instant::now();
            if pressed {
                next_state = !self.is_enabled;
                self.pressed_time = timestamp;
            } else {
                let elapsed = timestamp.duration_since(self.pressed_time);
                if elapsed > self.hold_threshold {
                    next_state = false;
                }
            }

            if next_state != self.is_enabled {
                self.is_enabled = next_state;
                break;
            }
        }
    }
}
