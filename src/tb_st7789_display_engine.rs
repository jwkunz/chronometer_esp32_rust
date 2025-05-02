use esp_hal::{
    gpio::{Input, InputConfig, Output, OutputConfig},
    peripherals::Peripherals,
    time::{Duration, Instant},
};

use crate::{
    spi_master::{FastSpiMode0, SoftwareSpiMasterConfigurable, SpiConfig, SpiPinsMaster},
    st7789_display_engine::{St7789DisplayEngine, ST77XX_GREEN},
};

/// Implements a maximum length sequence Linear Feedback Shift Register (LFSR) in Galois form for a `u16`.
///
/// # Arguments
///
/// * `x` - A mutable reference to the current state of the LFSR.
///
/// # Description
///
/// This function updates the LFSR state in-place using a feedback polynomial.
/// The feedback polynomial used here is `x^16 + x^14 + x^13 + x^11 + 1`, which is known to generate
/// a maximum length sequence for a 16-bit LFSR.
fn lfsr_16(x: &mut u16) {
    // Extract the least significant bit (LSB) as the feedback bit.
    let lsb = *x & 1;
    // Right shift the register by 1.
    *x >>= 1;
    // Apply the feedback polynomial if the LSB was 1.
    if lsb == 1 {
        *x ^= 0xB400; // Feedback polynomial: x^16 + x^14 + x^13 + x^11 + 1
    }
}

pub fn tb_st7789_display_engine(peripherals: Peripherals) -> ! {
    esp_println::println!("Defining pins...");
    let _blk = Output::new(
        peripherals.GPIO32,
        esp_hal::gpio::Level::High,
        OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let _diode_a = Input::new(peripherals.GPIO12, InputConfig::default());
    let _diode_b = Input::new(peripherals.GPIO13, InputConfig::default());

    let lcd_height = 240;
    let lcd_width = 135;
    esp_println::println!("Defining spi pins...");
    let pins = SpiPinsMaster {
        mosi: Output::new(
            peripherals.GPIO23,
            esp_hal::gpio::Level::Low,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Down),
        ),
        miso: None,
        sclk: Output::new(
            peripherals.GPIO18,
            esp_hal::gpio::Level::Low,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Down),
        ),
        cs: Output::new(
            peripherals.GPIO15,
            esp_hal::gpio::Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
        ),
        dc: Some(Output::new(
            peripherals.GPIO2,
            esp_hal::gpio::Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
        )),
        rst: Some(Output::new(
            // Active low
            peripherals.GPIO4,
            esp_hal::gpio::Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Down),
        )),
    };
    esp_println::println!("Defining screen manager...");
    let mut screen = St7789DisplayEngine::new(pins, lcd_width, lcd_height);
    esp_println::println!("Initialize...");
    screen.display_initialization_sequence();
    esp_println::println!("Rotation...");
    screen.set_rotation(2);
    esp_println::println!("Display Colors...");
    let mut color: u16 = ST77XX_GREEN;
    loop {
        let start_time = Instant::now();
        lfsr_16(&mut color);
        screen.fill_screen(color);
        screen.enable_display(true);
        esp_println::println!(
            "Render time: { } seconds",
            start_time.elapsed().as_micros() as f64 / 1E6
        );
        while start_time.elapsed() < esp_hal::time::Duration::from_secs(1) {}
    }
}
