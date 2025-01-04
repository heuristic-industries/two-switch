use embassy_stm32::gpio::AnyPin;
use embassy_time::{Duration, Instant};
use num_enum::{FromPrimitive, IntoPrimitive};

use crate::debounced_button::DebouncedButton;

#[derive(Copy, Clone, FromPrimitive, IntoPrimitive, Debug, PartialEq)]
#[repr(u8)]
pub enum SwitchState {
    #[num_enum(default)]
    Off,
    On,
    Held,
}

pub struct ToggleWithMomentary {
    pub is_enabled: bool,
    pub is_held: bool,
    last_state: bool,
    button: DebouncedButton,
    time: Instant,
    hold_threshold: Duration,
}

impl ToggleWithMomentary {
    pub fn new(
        is_enabled: bool,
        pin: AnyPin,
        debounce_threshold: Duration,
        hold_threshold: Duration,
    ) -> Self {
        let button = DebouncedButton::new(pin, debounce_threshold);

        ToggleWithMomentary {
            button,
            is_enabled,
            is_held: false,
            last_state: false,
            hold_threshold,
            time: Instant::MIN,
        }
    }

    pub fn get_enabled(&self) -> SwitchState {
        if self.is_enabled {
            SwitchState::On
        } else {
            SwitchState::Off
        }
    }

    pub fn get_state(&self) -> SwitchState {
        if self.is_held {
            return SwitchState::Held;
        }

        return self.get_enabled();
    }

    pub fn tick(&mut self) -> SwitchState {
        self.button.tick();
        let now = Instant::now();
        let duration = now.duration_since(self.time);
        if self.is_enabled && self.button.is_pressed && duration > self.hold_threshold {
            self.is_held = true;
        }

        if self.button.is_pressed == self.last_state {
            return self.get_state();
        }

        if self.button.is_pressed {
            self.time = now;
            self.is_enabled = !self.is_enabled
        } else {
            if self.is_held {
                self.is_enabled = false
            }
            self.is_held = false
        }
        self.last_state = self.button.is_pressed;

        return self.get_state();
    }
}
