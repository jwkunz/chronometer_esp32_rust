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
    /// Data (active) or command (innactive) polarity  for interpreting SPI communication.
    pub data_or_command_polarity: Polarity,
    /// Polarity to reset the slave
    pub reset_polarity: Polarity,
    /// Half the bit period in microseconds = 1E6 / (2 * baud_Hz)
    pub half_bit_period_micros: u64,
}

impl SpiConfig {
    /// Creates a new `SpiConfig` instance.
    ///
    /// # Arguments
    ///
    /// * `bit_ordering` - The bit ordering for SPI communication.
    /// * `clock_polarity` - The clock polarity for SPI communication.
    /// * `cs_polarity` - The chip select polarity for SPI communication.
    /// * `data_polarity` - The data polarity for SPI communication.
    /// * `baud_rate_hz` - The baud rate in Hz.
    ///
    /// # Returns
    ///
    /// A new `SpiConfig` instance.
    pub fn new(
        bit_ordering: Option<BitOrdering>,
        clock_polarity: Option<Polarity>,
        cs_polarity: Option<Polarity>,
        data_polarity: Option<Polarity>,
        data_vs_command_polarity: Option<Polarity>,
        reset_polarity: Option<Polarity>,
        baud_rate_hz: Option<u64>,
    ) -> SpiConfig {
        SpiConfig {
            bit_ordering: bit_ordering.unwrap_or(BitOrdering::MostSignificantBitFirst),
            clock_polarity: clock_polarity.unwrap_or(Polarity::ActiveHigh),
            cs_polarity: cs_polarity.unwrap_or(Polarity::ActiveLow),
            data_polarity: data_polarity.unwrap_or(Polarity::ActiveHigh),
            data_or_command_polarity: data_vs_command_polarity.unwrap_or(Polarity::ActiveHigh),
            reset_polarity: reset_polarity.unwrap_or(Polarity::ActiveLow),
            half_bit_period_micros: SpiConfig::calculate_half_period(
                baud_rate_hz.unwrap_or(32_000_000),
            ),
        }
    }
    /// Calculates half the bit period in microseconds.
    ///
    /// # Arguments
    ///
    /// * `rate_hz` - The baud rate in Hz.
    ///
    /// # Returns
    ///
    /// The half bit period in microseconds.
    fn calculate_half_period(rate_hz: u64) -> u64 {
        (1e6 / (2.0 * rate_hz as f64)) as u64
    }

    /// Delays execution for half the bit period.
    pub fn delay_half_period(&self) {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_micros(self.half_bit_period_micros) {}
    }
}

#[derive(Clone, Copy)]
pub enum SpiWordSize {
    EightBits,
    SixteenBits,
    ThirtyTwoBits,
}
/// Gets the size from the WordSize
pub fn get_size(x: SpiWordSize) -> usize {
    match x {
        SpiWordSize::EightBits => 8,
        SpiWordSize::SixteenBits => 16,
        SpiWordSize::ThirtyTwoBits => 32,
    }
}

/// Struct representing the SPI pins for the master device.
pub struct SpiPinsMaster<'a> {
    /// Master Out Slave In (MOSI) pin.
    pub mosi: Output<'a>,
    /// Master In Slave Out (MISO) pin (optional).
    pub miso: Option<Input<'a>>,
    /// Serial Clock (SCLK) pin.
    pub sclk: Output<'a>,
    /// Chip Select (CS) pin.
    pub cs: Output<'a>,
    /// Data or command
    pub dc: Option<Output<'a>>,
    /// Reset
    pub rst: Option<Output<'a>>,
}

/// SPI master implementation that is highly configurable but it slow
pub struct SoftwareSpiMasterConfigurable<'a> {
    /// SPI pins used by the master.
    pub pins: SpiPinsMaster<'a>,
    /// Configuration for the SPI master.
    pub config: SpiConfig,
}

impl<'a> SoftwareSpiMasterConfigurable<'a> {
    /// Creates a new instance of `SoftwareSpiMasterConfigurable`.
    ///
    /// # Arguments
    ///
    /// * `pins` - The SPI pins to be used by the master.
    /// * `config` - The configuration for the SPI master.
    ///
    /// # Returns
    ///
    /// A new `SoftwareSpiMasterConfigurable` instance.
    pub fn new(pins: SpiPinsMaster<'a>, config: SpiConfig) -> Self {
        Self { pins, config }
    }

    /// Interpret the next write as data
    pub fn set_data_mode(&mut self) {
        if let Some(x) = self.pins.dc.as_mut() {
            match self.config.data_or_command_polarity {
                Polarity::ActiveHigh => x.set_high(),
                Polarity::ActiveLow => x.set_low(),
            }
        }
    }

    /// Interpret the next write as a command
    pub fn set_command_mode(&mut self) {
        if let Some(x) = self.pins.dc.as_mut() {
            match self.config.data_or_command_polarity {
                Polarity::ActiveHigh => x.set_low(),
                Polarity::ActiveLow => x.set_high(),
            }
        }
    }

    /// Apply reset then release it
    pub fn apply_reset(&mut self) {
        if let Some(x) = self.pins.rst.as_mut() {
            match self.config.reset_polarity {
                Polarity::ActiveHigh => {
                    x.set_low();
                    let delay_start = Instant::now();
                    while delay_start.elapsed()
                        < Duration::from_micros(self.config.half_bit_period_micros)
                    {
                    }
                    x.set_high()
                }
                Polarity::ActiveLow => {
                    x.set_high();
                    let delay_start = Instant::now();
                    while delay_start.elapsed()
                        < Duration::from_micros(self.config.half_bit_period_micros)
                    {
                    }
                    x.set_low()
                }
            }
        }
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
    /// * `x` - The word to extract the bit from.
    /// * `position` - The bit position to retrieve.
    /// * `n_bits` - The number of bits in the word
    ///
    /// # Returns
    ///
    /// `true` if the bit is set, `false` otherwise.
    fn get_bit(&self, x: u32, position: usize, n_bits: SpiWordSize) -> bool {
        let bit_index = match self.config.bit_ordering {
            BitOrdering::LeastSignificantBitFirst => position,
            BitOrdering::MostSignificantBitFirst => {
                let limit: usize = get_size(n_bits) - 1;
                limit - position
            }
        };
        ((x >> bit_index) & 0b1) > 0
    }

    /// Sets or clears a specific bit in a byte.
    ///
    /// # Arguments
    ///
    /// * `x` - The word to modify.
    /// * `position` - The bit position to set or clear.
    /// * `n_bits` - The number of bits in the word
    /// * `value` - `true` to set the bit, `false` to clear it.
    fn set_bit(&self, x: &mut u32, position: usize, n_bits: SpiWordSize, value: bool) {
        let bit_index = match self.config.bit_ordering {
            BitOrdering::LeastSignificantBitFirst => position,
            BitOrdering::MostSignificantBitFirst => {
                let limit: usize = get_size(n_bits) - 1;
                limit - position
            }
        };
        if value {
            *x |= 1 << bit_index;
        } else {
            *x &= !(1 << bit_index);
        }
    }

    /// Performs a full-duplex SPI transfer of a single byte while blocking
    ///
    /// # Arguments
    ///
    /// * `spi_master' - The pins and configuration to use
    /// * `dout` - The word containing at least n_bits to send (starting at lsb)
    /// * `n_bits` - Number of bits to send
    /// * `is_data` - If provided, will toggle the ds pin to indicate whether is a data (true) or command (false) packet.
    ///
    /// # Returns
    ///
    /// * `Some(u8)` containing the received byte if the MISO pin is configured.
    /// * `None` if the MISO pin is not configured.
    pub fn read_and_write_blocking(
        &mut self,
        dout: u32,
        n_bits: SpiWordSize,
        is_data: Option<bool>,
    ) -> Option<u32> {
        let mut result: u32 = 0;
        self.cs_on();
        if let Some(x) = is_data {
            if x {
                self.set_data_mode();
            } else {
                self.set_command_mode();
            }
        }
        let mut delay_start;
        let limit = get_size(n_bits);
        for i in 0..limit {
            self.clk_off();
            delay_start = Instant::now();
            if self.get_bit(dout, i, n_bits) {
                self.mosi_on();
            } else {
                self.mosi_off();
            }
            while delay_start.elapsed() < Duration::from_micros(self.config.half_bit_period_micros)
            {
            }
            self.clk_on();
            delay_start = Instant::now();
            if let Some(rx_bit) = self.read_miso() {
                self.set_bit(&mut result, i, n_bits, rx_bit);
            }
            while delay_start.elapsed() < Duration::from_micros(self.config.half_bit_period_micros)
            {
            }
        }
        self.cs_off();
        if self.pins.miso.is_some() {
            Some(result)
        } else {
            None
        }
    }
}

/// This is an fast optimized SPI implementation for the format of
/// Most Significant Bit First
/// Data mode = dc pin high (or does not exist with None options)
/// Reset is active low
/// CS is active low
/// SCLK and MOSI are active high
pub struct FastSpiMode0<'a> {
    /// SPI pins used by the master.
    pub pins: SpiPinsMaster<'a>,
}

/// Wse_data_mode determines whether to toggle the dc pin (if it exists) for data (true) or command (false)
impl<'a> FastSpiMode0<'a> {
    /// Fast write a u8.
    pub fn write_u8(&mut self, mut word: u8, use_data_mode: Option<bool>) {
        self.pins.cs.set_low();
        if let Some(x) = self.pins.dc.as_mut() {
            if let Some(y) = use_data_mode {
                if y {
                    x.set_high();
                } else {
                    x.set_low();
                }
            }
        }
        for _ in 0..8 {
            self.pins.sclk.set_low();
            if (word & 0x80) > 0 {
                self.pins.mosi.set_high()
            } else {
                self.pins.mosi.set_low()
            }
            self.pins.sclk.set_high();
            word <<= 1;
        }
        self.pins.cs.set_high();
    }

    /// Fast write a u16.
    pub fn write_u16(&mut self, mut word: u16, use_data_mode: Option<bool>) {
        self.pins.cs.set_low();
        if let Some(x) = self.pins.dc.as_mut() {
            if let Some(y) = use_data_mode {
                if y {
                    x.set_high();
                } else {
                    x.set_low();
                }
            }
        }
        for _ in 0..16 {
            self.pins.sclk.set_low();
            if (word & 0x8000) > 0 {
                self.pins.mosi.set_high()
            } else {
                self.pins.mosi.set_low()
            }
            self.pins.sclk.set_high();
            word <<= 1;
        }
        self.pins.cs.set_high();
    }

    /// Fast write a u32.
    pub fn write_u32(&mut self, mut word: u32, use_data_mode: Option<bool>) {
        self.pins.cs.set_low();
        if let Some(x) = self.pins.dc.as_mut() {
            if let Some(y) = use_data_mode {
                if y {
                    x.set_high();
                } else {
                    x.set_low();
                }
            }
        }
        for _ in 0..32 {
            self.pins.sclk.set_low();
            if (word & 0x8000_0000) > 0 {
                self.pins.mosi.set_high()
            } else {
                self.pins.mosi.set_low()
            }
            self.pins.sclk.set_high();
            word <<= 1;
        }
        self.pins.cs.set_high();
    }
}
