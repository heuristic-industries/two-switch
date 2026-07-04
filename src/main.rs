#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use crate::debounced_button::DebouncedButton;
use cortex_m::asm;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::{ExtiInput, InterruptHandler as ExtiInterruptHandler},
    gpio::{Level, Output, Pull},
    i2c,
    interrupt::typelevel::{EXTI0_1, EXTI2_3},
    mode::Async,
    peripherals::I2C1,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel};
use embassy_time::Duration;

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

#[derive(Clone, Copy)]
enum Switch {
    Switch1,
    Switch2,
}
struct UpdateEvent {
    switch: Switch,
    enabled: bool,
}

static CHANNEL: channel::Channel<ThreadModeRawMutex, UpdateEvent, 2> = channel::Channel::new();

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<I2C1>, i2c::ErrorInterruptHandler<I2C1>;
    EXTI0_1 => ExtiInterruptHandler<EXTI0_1>;
    EXTI2_3 => ExtiInterruptHandler<EXTI2_3>;
});

/// EEPROM writes can take up to 5ms to complete, which is longer than we'd like
/// to block the main thread (each update is 10ms), so we'll process the actual
/// writes with this task, and take advantage of Channel's queue to avoid data races.
#[embassy_executor::task]
async fn writer(mut persistence: Persistence<'static, State>) {
    loop {
        let event = CHANNEL.receive().await;
        match event.switch {
            Switch::Switch1 => persistence.state.switch_1 = event.enabled,
            Switch::Switch2 => persistence.state.switch_2 = event.enabled,
        }
        persistence.update(persistence.state).await;
    }
}

#[embassy_executor::task]
async fn button_reader(
    switch: Switch,
    input: ExtiInput<'static, Async>,
    mut output: Output<'static>,
    mut output_inv: Output<'static>,
    initial: bool,
) {
    let button = DebouncedButton::new(input, DEBOUNCE_THRESHOLD);
    let mut toggle = ToggleWithMomentary::new(initial, button, HOLD_THRESHOLD);

    output.set_level(initial.into());

    loop {
        toggle.on_change().await;
        output.set_level(toggle.is_enabled.into());
        output_inv.set_level((!toggle.is_enabled).into());
        CHANNEL
            .send(UpdateEvent {
                switch,
                enabled: toggle.is_enabled,
            })
            .await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let persistence = Persistence::<State>::new(p.I2C1, p.PA9, p.PA10);

    let input_1 = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up, Irqs);
    let output_1 = Output::new(
        p.PA1,
        Level::from(persistence.state.switch_1),
        embassy_stm32::gpio::Speed::Low,
    );
    let output_1_inv = Output::new(
        p.PA2,
        Level::from(!persistence.state.switch_1),
        embassy_stm32::gpio::Speed::Low,
    );

    let input_2 = ExtiInput::new(p.PA3, p.EXTI3, Pull::Up, Irqs);
    let output_2 = Output::new(
        p.PA4,
        Level::from(persistence.state.switch_2),
        embassy_stm32::gpio::Speed::Low,
    );
    let output_2_inv = Output::new(
        p.PA5,
        Level::from(!persistence.state.switch_2),
        embassy_stm32::gpio::Speed::Low,
    );

    spawner.spawn(
        button_reader(
            Switch::Switch1,
            input_1,
            output_1,
            output_1_inv,
            persistence.state.switch_1,
        )
        .unwrap(),
    );
    spawner.spawn(
        button_reader(
            Switch::Switch2,
            input_2,
            output_2,
            output_2_inv,
            persistence.state.switch_2,
        )
        .unwrap(),
    );
    spawner.spawn(writer(persistence).unwrap());

    loop {
        asm::wfi();
    }
}
