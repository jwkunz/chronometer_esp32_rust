use esp_hal::{
    gpio::{Input, Output},
    time::{Duration, Instant},
};

/// Enum representing the bit ordering for SPI communication.
pub enum BitOrdering {
    /// Most significant bit is transmitted first.
    MostSignificantBitFirst,
    /// Least significant bit is transmitted first.
    LeastSignificantBitFirst,
}

/// Enum representing the polarity of a signal.
pub enum Polarity {
    /// Signal is active when high.
    ActiveHigh,
    /// Signal is active when low.
    ActiveLow,
}

/// Configuration for the SPI master.
pub struct SpiConfig {
    /// Bit ordering for SPI communication.
    pub bit_ordering: BitOrdering,
    /// Clock polarity for SPI communication.
    pub clock_polarity: Polarity,
    /// Chip select (CS) polarity for SPI communication.
    pub cs_polarity: Polarity,
    /// Data polarity for SPI communication.
    pub data_polarity: Polarity,
    /// Half the bit period in microseconds = 1E6 / (2 * baud_Hz)
    pub half_bit_period_micros: u64,
}

impl SpiConfig {
    pub fn new(
        bit_ordering: BitOrdering,
        clock_polarity: Polarity,
        cs_polarity: Polarity,
        data_polarity: Polarity,
        baud_rate_hz: f64,
    ) -> SpiConfig {
        SpiConfig {
            bit_ordering,
            clock_polarity,
            cs_polarity,
            data_polarity,
            half_bit_period_micros: SpiConfig::calculate_half_period(baud_rate_hz),
        }
    }

    fn calculate_half_period(rate_hz: f64) -> u64 {
        (1e6 / (2.0 * rate_hz)) as u64
    }

    pub fn delay_half_period(&self) {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_micros(self.half_bit_period_micros) {}
    }
}

/// Struct representing the SPI pins for the master device.
pub struct SpiPinsMaster<'a> {
    /// Master Out Slave In (MOSI) pin.
    mosi: Output<'a>,
    /// Master In Slave Out (MISO) pin (optional).
    miso: Option<Input<'a>>,
    /// Serial Clock (SCLK) pin.
    sclk: Output<'a>,
    /// Chip Select (CS) pin.
    cs: Output<'a>,
}

/// SPI master implementation with blocking operations.
pub struct SpiMasterCore<'a> {
    /// SPI pins used by the master.
    pins: SpiPinsMaster<'a>,
    /// Configuration for the SPI master.
    config: SpiConfig,
}

impl<'a> SpiMasterCore<'a> {
    /// Creates a new instance of `SpiMasterBlocking`.
    ///
    /// # Arguments
    ///
    /// * `pins` - The SPI pins to be used by the master.
    /// * `config` - The configuration for the SPI master.
    ///
    /// # Returns
    ///
    /// A new `SpiMasterBlocking` instance.
    pub fn new(pins: SpiPinsMaster<'a>, config: SpiConfig) -> Self {
        Self { pins, config }
    }

    /// Turns the clock signal on based on the configured clock polarity.
    fn clk_on(&mut self) {
        match self.config.clock_polarity {
            Polarity::ActiveHigh => self.pins.sclk.set_high(),
            Polarity::ActiveLow => self.pins.sclk.set_low(),
        }
    }

    /// Turns the clock signal off based on the configured clock polarity.
    fn clk_off(&mut self) {
        match self.config.clock_polarity {
            Polarity::ActiveHigh => self.pins.sclk.set_low(),
            Polarity::ActiveLow => self.pins.sclk.set_high(),
        }
    }

    /// Activates the chip select (CS) signal based on the configured CS polarity.
    fn cs_on(&mut self) {
        match self.config.cs_polarity {
            Polarity::ActiveHigh => self.pins.cs.set_high(),
            Polarity::ActiveLow => self.pins.cs.set_low(),
        }
    }

    /// Deactivates the chip select (CS) signal based on the configured CS polarity.
    fn cs_off(&mut self) {
        match self.config.cs_polarity {
            Polarity::ActiveHigh => self.pins.cs.set_low(),
            Polarity::ActiveLow => self.pins.cs.set_high(),
        }
    }

    /// Sets the MOSI signal to active based on the configured data polarity.
    fn mosi_on(&mut self) {
        match self.config.data_polarity {
            Polarity::ActiveHigh => self.pins.mosi.set_high(),
            Polarity::ActiveLow => self.pins.mosi.set_low(),
        }
    }

    /// Sets the MOSI signal to inactive based on the configured data polarity.
    fn mosi_off(&mut self) {
        match self.config.data_polarity {
            Polarity::ActiveHigh => self.pins.mosi.set_low(),
            Polarity::ActiveLow => self.pins.mosi.set_high(),
        }
    }

    /// Reads the MISO signal and returns its value based on the configured data polarity.
    ///
    /// # Returns
    ///
    /// * `Some(true)` if the MISO signal is active.
    /// * `Some(false)` if the MISO signal is inactive.
    /// * `None` if the MISO pin is not configured.
    fn read_miso(&self) -> Option<bool> {
        if let Some(pin_in) = &self.pins.miso {
            match self.config.data_polarity {
                Polarity::ActiveHigh => Some(pin_in.is_high()),
                Polarity::ActiveLow => Some(pin_in.is_low()),
            }
        } else {
            None
        }
    }

    /// Retrieves the value of a specific bit from a byte.
    ///
    /// # Arguments
    ///
    /// * `x` - The byte to extract the bit from.
    /// * `position` - The bit position to retrieve.
    ///
    /// # Returns
    ///
    /// `true` if the bit is set, `false` otherwise.
    fn get_bit(&self, x: u8, position: u8) -> bool {
        let bit_index = match self.config.bit_ordering {
            BitOrdering::LeastSignificantBitFirst => position,
            BitOrdering::MostSignificantBitFirst => 7 - position,
        };
        ((x >> bit_index) & 0b1) > 0
    }

    /// Sets or clears a specific bit in a byte.
    ///
    /// # Arguments
    ///
    /// * `x` - The byte to modify.
    /// * `position` - The bit position to set or clear.
    /// * `value` - `true` to set the bit, `false` to clear it.
    fn set_bit(&self, x: &mut u8, position: u8, value: bool) {
        let bit_index = match self.config.bit_ordering {
            BitOrdering::LeastSignificantBitFirst => position,
            BitOrdering::MostSignificantBitFirst => 7 - position,
        };
        if value {
            *x |= (1 << bit_index);
        } else {
            *x &= !(1 << bit_index);
        }
    }
}

/// Performs a full-duplex SPI transfer of a single byte while blocking
///
/// # Arguments
///
/// * `spi_master' - The pins and configuration to use
/// * `dout` - The byte to send.
///
/// # Returns
///
/// * `Some(u8)` containing the received byte if the MISO pin is configured.
/// * `None` if the MISO pin is not configured.
pub fn read_and_write_u8_blocking(spi_master: &mut SpiMasterCore, dout: u8) -> Option<u8> {
    let mut result: u8 = 0;
    spi_master.cs_on();
    let mut delay_start = Instant::now();
    for i in 0..8 {
        spi_master.clk_off();
        delay_start = Instant::now();
        if spi_master.get_bit(dout, i) {
            spi_master.mosi_on();
        } else {
            spi_master.mosi_off();
        }
        while delay_start.elapsed()
            < Duration::from_micros(spi_master.config.half_bit_period_micros)
        {}
        spi_master.clk_on();
        delay_start = Instant::now();
        if let Some(rx_bit) = spi_master.read_miso() {
            spi_master.set_bit(&mut result, i, rx_bit);
        }
        while delay_start.elapsed()
            < Duration::from_micros(spi_master.config.half_bit_period_micros)
        {}
    }
    spi_master.cs_off();
    if spi_master.pins.miso.is_some() {
        Some(result)
    } else {
        None
    }
}

enum SpiMasterTaskState {
    Idle,
    Start,
    Write,
    Off,
}

pub struct SpiMasterTask<'a> {
    spi_engine: SpiMasterCore<'a>,
    current_state: SpiMasterTaskState,
    send_data: Option<u8>,
    receive_data: Option<u8>,
    bit_counter: u8,
}

impl<'a> SpiMasterTask<'a> {
    pub fn new(spi_engine: SpiMasterCore) -> SpiMasterTask {
        SpiMasterTask {
            spi_engine,
            current_state: SpiMasterTaskState::Idle,
            send_data: None,
            receive_data: None,
            bit_counter: 0,
        }
    }

    pub fn load_to_send(&mut self, data: u8) {
        self.current_state = SpiMasterTaskState::Start;
        self.send_data = Some(data);
    }

    pub fn receive_data_available(&mut self) -> bool {
        self.receive_data.is_some()
    }

    pub fn get_received(&mut self) -> Option<u8> {
        match self.current_state {
            SpiMasterTaskState::Idle => {
                if let Some(x) = self.receive_data {
                    self.receive_data = None;
                    Some(x)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn tick(&mut self) {
        self.current_state = match self.current_state {
            SpiMasterTaskState::Idle => {
                self.bit_counter = 0;
                if self.send_data.is_some() {
                    SpiMasterTaskState::Start
                } else {
                    SpiMasterTaskState::Idle
                }
            }
            SpiMasterTaskState::Start => {
                self.spi_engine.cs_on();
                self.receive_data = if self.spi_engine.pins.miso.is_some() {
                    Some(0)
                } else {
                    None
                };
                SpiMasterTaskState::Off
            }
            SpiMasterTaskState::Off => {
                self.spi_engine.clk_off();
                if self
                    .spi_engine
                    .get_bit(self.send_data.unwrap(), self.bit_counter)
                {
                    self.spi_engine.mosi_on();
                } else {
                    self.spi_engine.mosi_off();
                }
                SpiMasterTaskState::Write
            }
            SpiMasterTaskState::Write => {
                self.spi_engine.clk_on();
                if let Some(rx_bit) = self.spi_engine.read_miso() {
                    self.spi_engine.set_bit(
                        &mut self.receive_data.unwrap(),
                        self.bit_counter,
                        rx_bit,
                    );
                }
                self.bit_counter += 1;
                if self.bit_counter == 8 {
                    SpiMasterTaskState::Idle
                } else {
                    SpiMasterTaskState::Off
                }
            }
        }
    }
}
