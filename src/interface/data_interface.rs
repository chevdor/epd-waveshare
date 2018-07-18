use hal::{
    blocking::{
        spi::Write,
        delay::*
    },
    spi::{Mode, Phase, Polarity},
    digital::*
};

use interface::Command;

/// EPD4in2 driver
///
pub struct DataInterface<SPI, CS, BUSY, DC, RST, D> {
    /// SPI
    spi: SPI,
    /// CS for SPI
    cs: CS,
    /// Low for busy, Wait until display is ready!
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Reseting
    rst: RST,
    /// The concrete Delay implementation
    delay: D,
}


impl<SPI, CS, BUSY, DC, RST, D, E> DataInterface<SPI, CS, BUSY, DC, RST, D>
where 
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>,
{
    pub fn new(spi: SPI, cs: CS, busy: BUSY, dc: DC, rst: RST, delay: D) -> Self {
        DataInterface {spi, cs, busy, dc, rst, delay }
    }
    
    /// Basic function for sending [Commands](Command). 
    /// 
    /// Enables direct interaction with the device with the help of [EPD4in2::send_data()](EPD4in2::send_data())
    /// Should rarely be needed!
    /// //TODO: make public? 
    fn send_command<T: Command>(&mut self, command: T) -> Result<(), E> {
        // low for commands
        self.dc.set_low(); 

        // Transfer the command over spi
        self.with_cs(|epd| {
            epd.spi.write(&[command.address()])
        })
    }

    /// Basic function for sending a single u8 of data over spi
    /// 
    /// Enables direct interaction with the device with the help of [EPD4in2::send_command()](EPD4in2::send_command())
    /// 
    /// Should rarely be needed!
    /// //TODO: make public? 
    fn send_data(&mut self, val: u8) -> Result<(), E> {
        // high for data
        self.dc.set_high();

        // Transfer data (u8) over spi
        self.with_cs(|epd| {
            epd.spi.write(&[val])
        })
    }

    /// Basic function for sending an array of u8-values of data over spi
    /// 
    /// Enables direct interaction with the device with the help of [EPD4in2::send_command()](EPD4in2::send_command())
    /// 
    /// Should rarely be needed!
    /// //TODO: make public? 
    fn send_multiple_data(&mut self, data: &[u8]) -> Result<(), E> {
        // high for data
        self.dc.set_high();

        // Transfer data (u8-array) over spi
        self.with_cs(|epd| {
            epd.spi.write(data)
        })
    }

    // spi write helper/abstraction function
    fn with_cs<F>(&mut self, f: F) -> Result<(), E>
    where 
        F: FnOnce(&mut Self) -> Result<(), E>,
    {
        // activate spi with cs low
        self.cs.set_low();
        // transfer spi data
        let result = f(self);
        // deativate spi with cs high
        self.cs.set_high();
        // return result
        result
    }


    /// Waits until device isn't busy anymore (busy == HIGH)
    /// 
    /// This is normally handled by the more complicated commands themselves,
    /// but in the case you send data and commands directly you might need to check
    /// if the device is still busy
    pub fn wait_until_idle(&mut self) {
        //low: busy, high: idle
        while self.busy.is_low() {
            //TODO: shorten the time? it was 100 in the beginning
            self.delay_ms(10);
        }
    }


    /// Abstraction of setting the delay for simpler calls 
    pub fn delay_ms(&mut self, delay: u16) {
        self.delay.delay_ms(delay);
    }

        /// Resets the device.
    /// 
    /// Often used to awake the module from deep sleep. See [EPD4in2::sleep()](EPD4in2::sleep())
    /// 
    /// TODO: Takes at least 400ms of delay alone, can it be shortened?
    pub fn reset(&mut self) {
        self.rst.set_low();

        //TODO: why 200ms? (besides being in the waveshare code)
        self.delay_ms(200);

        self.rst.set_high();

        //TODO: same as 3 lines above
        self.delay_ms(200);
    }

}