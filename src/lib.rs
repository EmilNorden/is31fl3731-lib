use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

const MATRIX_WIDTH:u8 = 16;
const MATRIX_HEIGHT:u8 = 9;
const ISSI_COMMAND_REGISTER:u8 = 0xFD;
const ISSI_BANK_FUNCTION_REGISTER:u8 = 0x0B;
const ISSI_REG_SHUTDOWN:u8 = 0x0A;
const ISSI_REG_CONFIG:u8 = 0x00;
const ISSI_REG_CONFIG_PICTURE_MODE:u8 = 0x00;
const ISSI_REG_AUDIOSYNC:u8 = 0x06;

#[derive(Clone, Copy)]
pub enum Address {
    GND = 0b1110100,
    VCC = 0b1110111,
    SCL = 0b1110101,
    SDA = 0b1110110,
}

pub struct IS31FL3731<I2C, Delay>
where
    I2C: I2c,
    Delay: DelayNs
{
    i2c: I2C,
    delay: Delay,
    frame: u8,
    address: Address,
}

impl<I2C, Delay, I2cError> IS31FL3731<I2C, Delay>
where
    I2C: I2c<Error = I2cError>,
    Delay: DelayNs {
    pub fn new(i2c: I2C, delay: Delay, address: Address) -> Self {
        Self {
            i2c,
            delay,
            frame: 0,
            address
        }
    }

    pub fn reset(&mut self) -> Result<(), I2cError> {
        // shutdown
        self.write_register(ISSI_BANK_FUNCTION_REGISTER, ISSI_REG_SHUTDOWN, 0x00)?;
        self.delay.delay_ms(10);

        // out of shutdown
        self.write_register(ISSI_BANK_FUNCTION_REGISTER, ISSI_REG_SHUTDOWN, 0x01)?;

        // picture mode
        self.write_register(ISSI_BANK_FUNCTION_REGISTER, ISSI_REG_CONFIG, ISSI_REG_CONFIG_PICTURE_MODE)?;
        Ok(())
    }

    pub fn audio_sync(&mut self, enable: bool) -> Result<(), I2cError> {
        let data = if enable { 1 } else { 0 };
        self.write_register(ISSI_BANK_FUNCTION_REGISTER, ISSI_REG_AUDIOSYNC, data)?;

        Ok(())
    }

    pub fn clear(&mut self, frame: u8) -> Result<(), I2cError> {
        self.select_bank(frame)?;

        let mut erase_buf = vec![0; 25];
        for x in 0..6 {
            erase_buf[0] = 0x24 + x*24;
            self.i2c.write(self.address as u8, &erase_buf)?;
        }

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
    use embedded_hal::delay::DelayNs;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    struct DelayStub;
    impl DelayNs for DelayStub {
        fn delay_ns(&mut self, _ns: u32) {}
    }

    #[test]
    fn test_reset() {
        let mut i2c = Mock::new(&[
            Transaction::write(Address::GND as u8,
                               vec![ISSI_COMMAND_REGISTER, ISSI_BANK_FUNCTION_REGISTER]),
            Transaction::write(Address::GND as u8,
                               vec![ISSI_REG_SHUTDOWN, 0x00]),
            Transaction::write(Address::GND as u8,
                               vec![ISSI_COMMAND_REGISTER, ISSI_BANK_FUNCTION_REGISTER]),
            Transaction::write(Address::GND as u8,
                               vec![ISSI_REG_SHUTDOWN, 0x01]),
            Transaction::write(Address::GND as u8,
                               vec![ISSI_COMMAND_REGISTER, ISSI_BANK_FUNCTION_REGISTER]),
            Transaction::write(Address::GND as u8,
                               vec![ISSI_REG_CONFIG, ISSI_REG_CONFIG_PICTURE_MODE]),
        ]);

        let mut sut = IS31FL3731 {
            i2c: i2c.clone(),
            delay: DelayStub{},
            frame: 0,
            address: Address::GND,
        };

        sut.reset().unwrap();
        i2c.done();
    }

    #[test]
    fn test_clear() {
        const FRAME:u8 = 3;
        let mut i2c = Mock::new(&[
            Transaction::write(Address::GND as u8,
                               vec![ISSI_COMMAND_REGISTER, FRAME]),
            Transaction::write(Address::GND as u8,
                               vec![36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Transaction::write(Address::GND as u8,
                               vec![60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Transaction::write(Address::GND as u8,
                               vec![84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Transaction::write(Address::GND as u8,
                               vec![108, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Transaction::write(Address::GND as u8,
                               vec![132, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Transaction::write(Address::GND as u8,
                               vec![156, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        ]);

        let mut sut = IS31FL3731 {
            i2c: i2c.clone(),
            delay: DelayStub{},
            frame: 0,
            address: Address::GND,
        };

        sut.clear(FRAME).unwrap();
        i2c.done();
    }

    #[test]
    fn test_audio_sync_false() {
        let mut i2c = Mock::new(&[
            Transaction::write(Address::GND as u8,
                                                            vec![ISSI_COMMAND_REGISTER, ISSI_BANK_FUNCTION_REGISTER]),
            Transaction::write(Address::GND as u8,
                                                            vec![ISSI_REG_AUDIOSYNC, 0]),
        ]);

        let mut sut = IS31FL3731 {
            i2c: i2c.clone(),
            delay: DelayStub{},
            frame: 0,
            address: Address::GND,
        };

        sut.audio_sync(false).unwrap();
        i2c.done();
    }

    #[test]
    fn test_audio_sync_true() {
        let mut i2c = Mock::new(&[
            Transaction::write(Address::GND as u8,
                                                            vec![ISSI_COMMAND_REGISTER, ISSI_BANK_FUNCTION_REGISTER]),
            Transaction::write(Address::GND as u8,
                                                            vec![ISSI_REG_AUDIOSYNC, 1]),
        ]);

        let mut sut = IS31FL3731 {
            i2c: i2c.clone(),
            delay: DelayStub{},
            frame: 0,
            address: Address::GND,
        };

        sut.audio_sync(true).unwrap();
        i2c.done();
    }
}