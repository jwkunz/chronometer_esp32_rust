use esp_hal::{
    gpio::{Input, InputConfig, Output, OutputConfig},
    peripherals::Peripherals,
};

use crate::{
    speed_trap::{SpeedTrap, SpeedUnits},
    spi_master::SpiPinsMaster,
    st7789_display_engine::{Color565, St7789DisplayEngine, ST77XX_BLACK},
};

pub fn start_chronometer(peripherals: Peripherals) -> ! {
    // Define the backlight pin (GPIO32).
    let _blk = Output::new(
        peripherals.GPIO32,
        esp_hal::gpio::Level::High,
        OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    // Define input pins for diodes (GPIO12 and GPIO13).
    let diode_a = Input::new(
        peripherals.GPIO16,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let diode_b = Input::new(
        peripherals.GPIO5,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );

    let lcd_height = 240;
    let lcd_width = 135;

    // Define the SPI pins for the display.
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
            // Active low reset pin (GPIO4).
            peripherals.GPIO4,
            esp_hal::gpio::Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Down),
        )),
    };

    // Initialize the display engine with the SPI pins and screen dimensions.
    let mut screen = St7789DisplayEngine::new(pins, lcd_width, lcd_height);

    // Send the initialization sequence to the display.
    screen.display_initialization_sequence();

    // Set the display rotation to mode 2.
    screen.set_rotation(2);

    const BACKGROUND: Color565 = ST77XX_BLACK;
    screen.fill_screen(BACKGROUND);
    const SEPARATION_MM: f64 = 8.0;
    let mut speed_trap_1 = SpeedTrap::from(
        SEPARATION_MM,
        diode_a,
        diode_b,
        SpeedUnits::FeetPerSecond,
        &mut screen,
    );

    loop {
        speed_trap_1.tick();
    }
}
