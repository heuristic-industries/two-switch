#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_stm32::{
    bind_interrupts,
    exti::Channel,
    gpio::{Level, Output, Pin},
    i2c, peripherals,
};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

mod debounced_button;
use debounced_button::DebouncedButton;
mod toggle_with_momentary;
use toggle_with_momentary::ToggleWithMomentary;
mod persistence;
use persistence::Persistence;
mod state;
use state::State;

static HOLD_THRESHOLD: Duration = Duration::from_millis(700);
static DEBOUNCE_THRESHOLD: Duration = Duration::from_millis(7);

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let mut persistence = Persistence::<State>::new(p.I2C1, p.PA9, p.PA10);

    let mut input_1 = ToggleWithMomentary::new(
        persistence.state.switch_1,
        p.PA0.degrade(),
        DEBOUNCE_THRESHOLD,
        HOLD_THRESHOLD,
        p.EXTI0.degrade(),
    );
    let mut output_1 = Output::new(
        p.PA2,
        Level::from(persistence.state.switch_1),
        embassy_stm32::gpio::Speed::Low,
    );

    let mut input_2 = ToggleWithMomentary::new(
        persistence.state.switch_2,
        p.PA1.degrade(),
        DEBOUNCE_THRESHOLD,
        HOLD_THRESHOLD,
        p.EXTI1.degrade(),
    );
    let mut output_2 = Output::new(
        p.PA3,
        Level::from(persistence.state.switch_2),
        embassy_stm32::gpio::Speed::Low,
    );

    loop {
        let input_1_future = input_1.wait_for_state_change();
        let input_2_future = input_2.wait_for_state_change();
        select(input_1_future, input_2_future).await;

        let state = State::new(input_1.is_enabled, input_2.is_enabled);
        output_1.set_level(Level::from(state.switch_1));
        output_2.set_level(Level::from(state.switch_2));
        persistence.update(state).unwrap();
    }
}
