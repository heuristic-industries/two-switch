#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_stm32::{
    bind_interrupts,
    dma::NoDma,
    exti::{Channel, ExtiInput},
    gpio::{Input, Level, Output, Pin, Pull, Speed},
    i2c::{self, I2c},
    peripherals,
};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};
// use stm32f0xx_hal::

mod debounced_button;
use debounced_button::DebouncedButton;
mod toggle_with_momentary;
use toggle_with_momentary::ToggleWithMomentary;
mod persistence;
use persistence::Persistence;

static HOLD_THRESHOLD: Duration = Duration::from_millis(700);
static DEBOUNCE_THRESHOLD: Duration = Duration::from_millis(7);

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let persistence = Persistence::new(p.I2C1, p.PA9, p.PA10);

    let input_1 = ToggleWithMomentary::new(
        false,
        p.PA0.degrade(),
        DEBOUNCE_THRESHOLD,
        HOLD_THRESHOLD,
        p.EXTI0.degrade(),
    );
    let input_1 = ToggleWithMomentary::new(
        false,
        p.PA1.degrade(),
        DEBOUNCE_THRESHOLD,
        HOLD_THRESHOLD,
        p.EXTI1.degrade(),
    );

    loop {}
}
