#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    i2c, peripherals,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel};
use embassy_time::{Duration, Timer};
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

static CHANNEL: channel::Channel<ThreadModeRawMutex, State, 4> = channel::Channel::new();

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

/// EEPROM writes can take up to 5ms to complete, which is longer than we'd like
/// to block the main thread (each update is 10ms), so we'll process the actual
/// writes with this task, and take advantage of Channel's queue to avoid data races.
#[embassy_executor::task]
async fn writer(mut persistence: Persistence<'static, State>) {
    loop {
        let state = CHANNEL.receive().await;
        persistence.update(state).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let persistence = Persistence::<State>::new(p.I2C1, p.PA9, p.PA10);

    let input = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up);
    let mut bypass_switch = ToggleWithMomentary::new(
        persistence.state.bypass,
        input,
        DEBOUNCE_THRESHOLD,
        HOLD_THRESHOLD,
    );
    let mut output = Output::new(p.PA1, Level::from(persistence.state.bypass), Speed::Low);
    let mut output_inv = Output::new(p.PA2, Level::from(!persistence.state.bypass), Speed::Low);

    let mut enable_output = Output::new(p.PA3, Level::Low, Speed::Low);

    Timer::after_millis(100).await;
    enable_output.set_high();

    spawner.spawn(writer(persistence)).unwrap();

    loop {
        bypass_switch.wait_for_state_change().await;

        let state = State::new(bypass_switch.is_enabled);
        output.set_level(Level::from(state.bypass));
        output_inv.set_level(Level::from(!state.bypass));
        CHANNEL.send(state).await;
    }
}
