use esp_hal::{
    gpio::{Input, InputConfig, Output, OutputConfig},
    peripherals::Peripherals,
};

use crate::{
    spi_master::{self, SpiConfig, SpiMasterCore, SpiPinsMaster},
    st7796s_manager::{St7796sManager, ST77XX_GREEN},
};

pub fn tb_st7796s(peripherals: Peripherals) -> ! {
    esp_println::println!("Defining pins...");
    let _rst = Output::new(
        // Active low
        peripherals.GPIO4,
        esp_hal::gpio::Level::High,
        OutputConfig::default().with_pull(esp_hal::gpio::Pull::Down),
    );
    let _blk = Output::new(
        peripherals.GPIO32,
        esp_hal::gpio::Level::High,
        OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let _diode_a = Input::new(peripherals.GPIO12, InputConfig::default());
    let _diode_b = Input::new(peripherals.GPIO13, InputConfig::default());

    let _lcd_height = 240;
    let _lcd_width = 135;
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
        dc: Output::new(
            peripherals.GPIO2,
            esp_hal::gpio::Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
        ),
    };
    esp_println::println!("Defining spi config...");
    let spi_config = SpiConfig::new(
        spi_master::BitOrdering::MostSignificantBitFirst,
        spi_master::Polarity::ActiveHigh,
        spi_master::Polarity::ActiveLow,
        spi_master::Polarity::ActiveHigh,
        spi_master::Polarity::ActiveHigh,
        32000000,
    );
    esp_println::println!("Defining screen manager...");
    let mut screen = St7796sManager::new(SpiMasterCore::new(pins, spi_config));
    esp_println::println!("Initialize...");
    screen.initialize();
    esp_println::println!("Rotation...");
    screen.set_rotation(2);
    esp_println::println!("Display Green...");
    screen.draw_rectangle(0, 0, 100, 100, ST77XX_GREEN);
    esp_println::println!("Done");
    screen.enable_display(true);

    loop {}
}
