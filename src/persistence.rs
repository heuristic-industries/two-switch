use crate::Irqs;
use eeprom24x::{
    addr_size::OneByte, page_size::B8, unique_serial::No, Eeprom24x, Error, SlaveAddr,
};
use embassy_stm32::{
    dma::NoDma,
    i2c::{self, I2c, SclPin, SdaPin},
    peripherals::I2C1,
    time::khz,
};

static MAX_ADDRESS: u32 = 2047;

pub trait Persistable {
    fn from_u8(input: u8) -> Self;
    fn into_u8(&self) -> u8;
}

pub struct Persistence<'a, T>
where
    T: Persistable,
{
    eeprom: Eeprom24x<I2c<'a, I2C1>, B8, OneByte, No>,
    current_address: u32,
    pub state: T,
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
        let mut eeprom = Eeprom24x::new_24x02(i2c, address);

        let mut current_address: u32 = 0;
        let mut value: u8 = 0;
        for address in 0..MAX_ADDRESS {
            // let value = storage.eeprom.read_data(address, U);
            // check if the most significant bit is 0
            value = eeprom.read_byte(address).unwrap();
            if value >> 7 == 0 {
                current_address = address;
                break;
            }
        }

        let state = T::from_u8(value);

        Persistence {
            eeprom,
            state,
            current_address,
        }
    }

    pub fn update(&mut self, state: T) -> Result<(), Error<i2c::Error>> {
        self.state = state;
        let previous_address = self.current_address;
        self.current_address += 1;
        if self.current_address > MAX_ADDRESS {
            self.current_address = 0;
        }

        let data = self.state.into_u8();

        let result = self.eeprom.write_byte(self.current_address, data);
        self.eeprom.write_byte(previous_address, 0xFF).unwrap();
        return result;
    }
}
