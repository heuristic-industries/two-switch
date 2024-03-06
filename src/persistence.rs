use crate::Irqs;
use eeprom24x::{
    addr_size::OneByte, page_size::B8, unique_serial::No, Eeprom24x, SlaveAddr, Storage,
};
use embassy_stm32::{
    dma::NoDma,
    i2c::{I2c, SclPin, SdaPin},
    peripherals::I2C1,
    time::khz,
};
use embassy_time::Delay;

pub struct Persistence<'a> {
    storage: Storage<I2c<'a, I2C1>, B8, OneByte, No, Delay>,
}

impl<'a> Persistence<'a> {
    pub fn new(i2c: I2C1, scl_pin: impl SclPin<I2C1>, sda_pin: impl SdaPin<I2C1>) -> Self {
        let i2c = I2c::new(
            i2c,
            scl_pin,
            sda_pin,
            Irqs,
            NoDma,
            NoDma,
            khz(100),
            Default::default(),
        );

        let address = SlaveAddr::default();
        let eeprom: Eeprom24x<
            I2c<'_, I2C1>,
            eeprom24x::page_size::B8,
            eeprom24x::addr_size::OneByte,
            eeprom24x::unique_serial::No,
        > = Eeprom24x::new_24x02(i2c, address);

        let storage = Storage::new(eeprom, Delay);

        Persistence { storage }
    }
}
