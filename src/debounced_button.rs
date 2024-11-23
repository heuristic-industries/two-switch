use embassy_stm32::exti::ExtiInput;
use embassy_time::{Duration, Instant};

pub struct DebouncedButton {
    pub threshold: Duration,
    pub is_pressed: bool,
    input: ExtiInput<'static>,
    time: Instant,
}

impl DebouncedButton {
    pub fn new(input: ExtiInput<'static>, threshold: Duration) -> Self {
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
