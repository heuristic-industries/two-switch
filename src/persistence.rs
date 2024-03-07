use core::marker::PhantomData;

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

static MAX_ADDRESS: u32 = 100; // TODO

pub trait Persistable {
    fn from(input: u8) -> Self;
    fn into(&self) -> u8;
}

pub struct Persistence<'a, T>
where
    T: Persistable,
{
    storage: Storage<I2c<'a, I2C1>, B8, OneByte, No, Delay>,
    state: T,
}

impl<'a, T> Persistence<'a, T>
where
    T: Persistable,
{
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
        let eeprom = Eeprom24x::new_24x02(i2c, address);

        let storage = Storage::new(eeprom, Delay);
        // for address in 0..MAX_ADDRESS {
        //     // let value = storage.eeprom.read_data(address, U);
        //     // check if the most significant bit is 0
        //     let mut data = [0xFF; U::BYTES];
        //     storage.eeprom.read_data(address, &data);
        //     if value >> (U::BITS - 1) == U::zero() {
        //         self.value = value;
        //         self.current_address = address;
        //         break;
        //     }
        // }

        let state = T::from(0);

        Persistence { storage, state }
    }
}
