use embassy_stm32::exti::{AnyChannel, ExtiInput};
use embassy_stm32::gpio::{AnyPin, Input, Pull};
use embassy_time::{Duration, Instant};

pub struct DebouncedButton {
    pub threshold: Duration,
    pub is_pressed: bool,
    input: ExtiInput<'static, AnyPin>,
    time: Instant,
}

impl DebouncedButton {
    pub fn new(input_pin: AnyPin, threshold: Duration, channel: AnyChannel) -> Self {
        let button = Input::new(input_pin, Pull::Up);
        let input = ExtiInput::new(button, channel);

        DebouncedButton {
            threshold,
            input,
            is_pressed: false,
            time: Instant::MIN,
        }
    }

    pub async fn wait_for_any_edge(&mut self) {
        loop {
            self.input.wait_for_any_edge().await;
            let time = Instant::now();
            let is_pressed = self.input.is_low();
            let elapsed = time.duration_since(self.time);

            if is_pressed != self.is_pressed && elapsed > self.threshold {
                self.is_pressed = is_pressed;
                self.time = time;
                break;
            }
        }
    }
}
