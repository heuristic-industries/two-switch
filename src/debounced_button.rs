use embassy_stm32::{exti::ExtiInput, mode::Async};
use embassy_time::{Duration, Timer};

pub struct DebouncedButton {
    pub threshold: Duration,
    input: ExtiInput<'static, Async>,
}

impl DebouncedButton {
    pub fn new(input: ExtiInput<'static, Async>, threshold: Duration) -> Self {
        DebouncedButton { threshold, input }
    }

    pub async fn on_change(&mut self) -> bool {
        loop {
            let l1 = self.pressed();
            self.input.wait_for_any_edge().await;
            Timer::after(self.threshold).await;
            let l2 = self.pressed();
            if l1 != l2 {
                break l2;
            }
        }
    }

    fn pressed(&mut self) -> bool {
        self.input.is_low()
    }
}
