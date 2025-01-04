use embassy_stm32::gpio::AnyPin;
use embassy_stm32::gpio::Input;
use embassy_stm32::gpio::Pull;
use embassy_time::{Duration, Instant};

pub struct DebouncedButton {
    pub threshold: Duration,
    pub is_pressed: bool,
    pub input: Input<'static>,
    time: Instant,
}

impl DebouncedButton {
    pub fn new(pin: AnyPin, threshold: Duration) -> Self {
        let input = Input::new(pin, Pull::Up);

        DebouncedButton {
            threshold,
            input,
            is_pressed: false,
            time: Instant::MIN,
        }
    }

    pub fn tick(&mut self) {
        let is_pressed = self.input.is_low();
        let time = Instant::now();
        let elapsed = time.duration_since(self.time);
        if is_pressed != self.is_pressed && elapsed > self.threshold {
            self.time = time;
            self.is_pressed = is_pressed;
        }
    }
}
