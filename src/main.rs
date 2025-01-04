#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Pin},
    i2c, peripherals,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

mod debounced_button;
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

    let mut input_1 = ToggleWithMomentary::new(
        persistence.state.switch_1,
        p.PA0.degrade(),
        DEBOUNCE_THRESHOLD,
        HOLD_THRESHOLD,
    );
    let mut output_1 = Output::new(
        p.PA1,
        Level::from(persistence.state.switch_1),
        embassy_stm32::gpio::Speed::Low,
    );
    let mut output_1_inv = Output::new(
        p.PA2,
        Level::from(!persistence.state.switch_1),
        embassy_stm32::gpio::Speed::Low,
    );

    let mut input_2 = ToggleWithMomentary::new(
        persistence.state.switch_2,
        p.PA3.degrade(),
        DEBOUNCE_THRESHOLD,
        HOLD_THRESHOLD,
    );
    let mut output_2 = Output::new(
        p.PA4,
        Level::from(persistence.state.switch_2),
        embassy_stm32::gpio::Speed::Low,
    );
    let mut output_2_inv = Output::new(
        p.PA5,
        Level::from(!persistence.state.switch_2),
        embassy_stm32::gpio::Speed::Low,
    );

    spawner.spawn(writer(persistence)).unwrap();

    let mut enabled_1 = input_1.is_enabled;
    let mut enabled_2 = input_2.is_enabled;
    let mut ticker = Ticker::every(Duration::from_hz(100));
    let mut modified = false;
    loop {
        input_1.tick();
        input_2.tick();

        let state = State::new(input_1.is_enabled, input_2.is_enabled);
        if input_1.is_enabled != enabled_1 {
            enabled_1 = input_1.is_enabled;
            output_1.set_level(Level::from(state.switch_1));
            output_1_inv.set_level(Level::from(!state.switch_1));
            modified = true;
        }

        if input_2.is_enabled != enabled_2 {
            enabled_2 = input_2.is_enabled;
            output_2.set_level(Level::from(state.switch_2));
            output_2_inv.set_level(Level::from(!state.switch_2));
            modified = true;
        }

        if modified {
            CHANNEL.send(state).await;
            modified = false;
        }

        ticker.next().await;
    }
}
