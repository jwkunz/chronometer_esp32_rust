use esp_hal::{
    gpio::{Input, InputConfig},
    peripherals::Peripherals,
};

use crate::speed_trap::{SpeedTrap, SpeedUnits};

pub fn chronometer(peripherals: Peripherals) -> ! {
    let diode_a = Input::new(peripherals.GPIO9, InputConfig::default());
    let diode_b = Input::new(peripherals.GPIO7, InputConfig::default());
    let mut speed_trap_1 = SpeedTrap::from(8.0, diode_a, diode_b, SpeedUnits::FeetPerSecond);

    loop {
        speed_trap_1.tick();
    }
}
