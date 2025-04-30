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
    let dc = Output::new(
        peripherals.GPIO2,
        esp_hal::gpio::Level::High,
        OutputConfig::default(),
    );
    let _rst = Output::new(
        peripherals.GPIO4,
        esp_hal::gpio::Level::Low,
        OutputConfig::default(),
    );
    let _blk = Output::new(
        peripherals.GPIO32,
        esp_hal::gpio::Level::High,
        OutputConfig::default(),
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
            OutputConfig::default(),
        ),
        miso: None,
        sclk: Output::new(
            peripherals.GPIO18,
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        cs: Output::new(
            peripherals.GPIO15,
            esp_hal::gpio::Level::High,
            OutputConfig::default(),
        ),
    };
    esp_println::println!("Defining spi config...");
    let spi_config = SpiConfig::new(
        spi_master::BitOrdering::MostSignificantBitFirst,
        spi_master::Polarity::ActiveHigh,
        spi_master::Polarity::ActiveLow,
        spi_master::Polarity::ActiveHigh,
        32000000,
    );
    esp_println::println!("Defining screen manager...");
    let mut screen = St7796sManager::new(SpiMasterCore::new(pins, spi_config), dc);
    esp_println::println!("Initialize...");
    screen.initialize();
    esp_println::println!("Rotation...");
    screen.set_rotation(0);
    esp_println::println!("Display Green...");
    screen.draw_rectangle(0, 0, 100, 100, ST77XX_GREEN);
    esp_println::println!("Done");

    loop {}
}
