#![no_std]
use esp_hal::gpio::Input;
use esp_hal::time::{Duration, Instant};
enum TrapState {
    Idle,
    LowStart,
    LowStop,
    HighStart,
    HighStop,
    Report,
}

#[derive(Clone, Copy)]
pub enum SpeedUnits {
    MillimetersPerSecond,
    MetersPerSecond,
    FeetPerSecond,
    MilesPerHour,
    KilometersPerHour,
}

pub struct SpeedTrap<'a> {
    start: Instant,
    stop: Instant,
    separation_mm: f64,
    input_start: Input<'a>,
    input_stop: Input<'a>,
    state: TrapState,
    report_format: SpeedUnits,
}

impl<'a> SpeedTrap<'a> {
    pub fn from(
        separation_mm: f64,
        input_start: Input<'a>,
        input_stop: Input<'a>,
        report_format: SpeedUnits,
    ) -> SpeedTrap<'a> {
        let now = Instant::now();
        SpeedTrap {
            start: now,
            stop: now,
            separation_mm,
            input_start,
            input_stop,
            state: TrapState::Idle,
            report_format,
        }
    }

    fn start_clock(&mut self) {
        self.start = Instant::now();
    }

    fn stop_clock(&mut self) {
        self.stop = Instant::now();
    }

    fn get_elapsed_microseconds(&self) -> f64 {
        let elapsed: Duration = self.stop - self.start;
        elapsed.as_micros() as f64
    }

    fn get_speed(&self, units: SpeedUnits) -> f64 {
        let elapsed_nanos = self.get_elapsed_microseconds();
        let speed = self.separation_mm / elapsed_nanos;
        match units {
            SpeedUnits::MillimetersPerSecond => speed * 1E6,
            SpeedUnits::MetersPerSecond => speed * (0.001 * 1E6),
            SpeedUnits::KilometersPerHour => speed * (0.0036 * 1E6),
            SpeedUnits::FeetPerSecond => speed * (0.00328083989501 * 1E6),
            SpeedUnits::MilesPerHour => speed * (0.00223693629205 * 1E6),
        }
    }

    pub fn tick(&mut self) {
        match self.state {
            TrapState::Idle => {
                if (self.input_start.is_low()) {
                    self.start_clock();
                    self.state = TrapState::LowStart
                }
            }
            TrapState::LowStart => {
                if (self.input_stop.is_low()) {
                    self.state = TrapState::LowStop
                }
            }
            TrapState::LowStop => {
                if (self.input_start.is_high()) {
                    self.state = TrapState::HighStart
                }
            }
            TrapState::HighStart => {
                if (self.input_stop.is_high()) {
                    self.state = TrapState::HighStop
                }
            }
            TrapState::HighStop => {
                if (self.input_start.is_low()) {
                    self.stop_clock();
                    self.state = TrapState::Report
                }
            }
            TrapState::Report => {
                esp_println::println!("Speed: { }", self.get_speed(self.report_format));
                self.state = TrapState::Idle
            }
        }
    }
}
