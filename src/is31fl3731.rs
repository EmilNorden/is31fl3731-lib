use embedded_hal::i2c::SevenBitAddress;

const MATRIX_WIDTH:u8 = 16;
const MATRIX_HEIGHT:u8 = 9;
const ISSI_COMMAND_REGISTER:u8 = 0xFD;
const ISSI_BANK_FUNCTION_REGISTER:u8 = 0x0B;
const ISSI_REG_SHUTDOWN:u8 = 0x0A;

#[derive(Clone, Copy)]
pub enum Address {
    GND = 0b1110100,
    VCC = 0b1110111,
    SCL = 0b1110101,
    SDA = 0b1110110,
}

pub struct IS31FL3731<I2C>
where
    I2C: embedded_hal::i2c::I2c
{
    i2c: I2C,
    frame: u8,
    address: Address,
}

impl<I2C, I2cError> IS31FL3731<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = I2cError> {
    pub fn new(i2c: I2C, address: Address) -> Self {
        Self {
            i2c,
            frame: 0,
            address
        }
    }

    pub fn reset(&mut self) -> Result<(), I2cError> {
        // shutdown
        self.write_register(ISSI_BANK_FUNCTION_REGISTER, ISSI_REG_SHUTDOWN, 0x00)?;

        Ok(())
    }

    fn select_bank(&mut self, bank: u8) -> Result<(), I2cError> {
        self.i2c.write(self.address as u8, &[ISSI_COMMAND_REGISTER, bank])?;

        Ok(())
    }

    fn write_register(&mut self, bank: u8, reg: u8, data: u8) -> Result<(), I2cError> {
        self.select_bank(bank)?;
        self.i2c.write(self.address as u8, &[reg, data])?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_reset() {
        let mut i2c = embedded_hal_mock::eh1::i2c::Mock::new(&[
            embedded_hal_mock::eh1::i2c::Transaction::write(Address::GND as u8,
                                                            vec![ISSI_COMMAND_REGISTER, ISSI_BANK_FUNCTION_REGISTER]),
            embedded_hal_mock::eh1::i2c::Transaction::write(Address::GND as u8,
                                                            vec![ISSI_REG_SHUTDOWN, 0x00])
        ]);

        let mut sut = IS31FL3731 {
            i2c: i2c.clone(),
            frame: 0,
            address: Address::GND,
        };

        sut.reset().unwrap();
        i2c.done();
    }
}