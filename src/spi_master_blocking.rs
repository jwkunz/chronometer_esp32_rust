#![no_std]

use esp_hal::gpio::{Input, Output};

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
pub struct SpiMasterBlocking<'a> {
    /// SPI pins used by the master.
    pins: SpiPinsMaster<'a>,
    /// Configuration for the SPI master.
    config: SpiConfig,
}

impl<'a> SpiMasterBlocking<'a> {
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
    fn read_miso(&mut self) -> Option<bool> {
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
    fn get_bit(&mut self, x: u8, position: u8) -> bool {
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
    fn set_bit(&mut self, x: &mut u8, position: u8, value: bool) {
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

    /// Performs a full-duplex SPI transfer of a single byte.
    ///
    /// # Arguments
    ///
    /// * `dout` - The byte to send.
    ///
    /// # Returns
    ///
    /// * `Some(u8)` containing the received byte if the MISO pin is configured.
    /// * `None` if the MISO pin is not configured.
    pub fn read_and_write_u8(&mut self, dout: u8) -> Option<u8> {
        let mut result: u8 = 0;
        self.cs_on();
        for i in 0..8 {
            self.clk_off();
            if self.get_bit(dout, i) {
                self.mosi_on();
            } else {
                self.mosi_off();
            }
            self.clk_on();
            if let Some(rx_bit) = self.read_miso() {
                self.set_bit(&mut result, i, rx_bit);
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
