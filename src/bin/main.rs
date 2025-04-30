#![no_std]
#![no_main]

use esp_hal::{clock::CpuClock, main};

use chronometer::tb_st7796s::tb_st7796s;

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
    esp_println::println!("Starting main...");
    //chronometer();
    tb_st7796s(peripherals)
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
