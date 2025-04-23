#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::time::{Duration, Instant};
use esp_hal::{
    gpio::{Input, InputConfig, InputPin, Pull},
    main,
};

use chronometer::speed_trap::*;

#[panic_handler]
fn panic(er: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Panic says: { }", er);
    loop {}
}

#[main]
fn main() -> ! {
    // generator version: 0.3.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::println!("Starting chronometer...");
    let mut speed_trap_1 = SpeedTrap::from(
        8.0,
        Input::new(peripherals.GPIO9, InputConfig::default()),
        Input::new(peripherals.GPIO7, InputConfig::default()),
        SpeedUnits::FeetPerSecond,
    );

    loop {
        speed_trap_1.tick();
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
