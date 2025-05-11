use esp_hal::{
    gpio::{Input, InputConfig, Output, OutputConfig},
    peripherals::Peripherals,
};

use crate::{
    speed_trap::{SpeedTrap, SpeedUnits},
    spi_master::SpiPinsMaster,
    st7789_display_engine::{Color565, St7789DisplayEngine, ST77XX_BLACK, ST77XX_GREEN},
};

pub fn main_speed_trap(peripherals: Peripherals) -> ! {
    // Define the backlight pin (GPIO32).
    let _blk = Output::new(
        peripherals.GPIO32,
        esp_hal::gpio::Level::High,
        OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    // Define input pins for phototransitors.
    let laser_in_a = Input::new(
        peripherals.GPIO16,
        // Pin stays low when the light is on the transistor, opening it, and grounding this pin
        // If the light is off, the transistor closes, and this pin pulls up.
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let laser_in_b = Input::new(
        peripherals.GPIO5,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );

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

    // Screen parameters
    let lcd_height = 240;
    let lcd_width = 135;

    // Initialize the display engine with the SPI pins and screen dimensions.
    let mut screen = St7789DisplayEngine::new(pins, lcd_width, lcd_height);

    // Send the initialization sequence to the display.
    screen.display_initialization_sequence();

    // Set the display rotation to mode 2.
    screen.set_rotation(2);

    // Draw the splash screen
    const BACKGROUND: Color565 = ST77XX_BLACK;
    screen.fill_screen(BACKGROUND);
    screen.draw_str(
        "  Speed Trap \n\nMeasures screen\nto back\nReverse toggles\nthe units\n\n Trip to start",
        125.0,
        5.0,
        2.0,
        90.0,
        ST77XX_GREEN,
    );

    // Initialize the speed trap state machine
    const SEPARATION_MM: f64 = 8.0;
    let mut speed_trap_1 = SpeedTrap::from(
        SEPARATION_MM,
        laser_in_a,
        laser_in_b,
        SpeedUnits::FeetPerSecond,
        &mut screen,
    );

    // Tick the state machine for ever
    loop {
        speed_trap_1.tick();
    }
}
