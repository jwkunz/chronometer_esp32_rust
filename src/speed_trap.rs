use esp_hal::gpio::Input;
use esp_hal::time::{Duration, Instant};

/// Represents the various states of the speed trap.
enum TrapState {
    /// The speed trap is idle and waiting for an event.
    Idle,
    /// The start sensor is triggered and is in a low state.
    LowStart,
    /// The stop sensor is triggered and is in a low state.
    LowStop,
    /// The start sensor is triggered and is in a high state.
    HighStart,
    /// The stop sensor is triggered and is in a high state.
    HighStop,
    /// The speed trap has completed a measurement and is ready to report.
    Report,
}

/// Represents the units of speed for reporting.
#[derive(Clone, Copy)]
pub enum SpeedUnits {
    /// Speed in millimeters per second.
    MillimetersPerSecond,
    /// Speed in meters per second.
    MetersPerSecond,
    /// Speed in feet per second.
    FeetPerSecond,
    /// Speed in miles per hour.
    MilesPerHour,
    /// Speed in kilometers per hour.
    KilometersPerHour,
}

/// A speed trap for measuring the speed of an object passing between two sensors.
///
/// The `SpeedTrap` uses two GPIO inputs to detect the start and stop events, calculates
/// the elapsed time, and computes the speed based on the separation distance between
/// the sensors.
pub struct SpeedTrap<'a> {
    /// The timestamp when the start sensor was triggered.
    start: Instant,
    /// The timestamp when the stop sensor was triggered.
    stop: Instant,
    /// The separation distance between the start and stop sensors in millimeters.
    separation_mm: f64,
    /// The GPIO input for the start sensor.
    input_start: Input<'a>,
    /// The GPIO input for the stop sensor.
    input_stop: Input<'a>,
    /// The current state of the speed trap.
    state: TrapState,
    /// The format in which the speed should be reported.
    report_format: SpeedUnits,
}

impl<'a> SpeedTrap<'a> {
    /// Creates a new `SpeedTrap` instance.
    ///
    /// # Arguments
    ///
    /// * `separation_mm` - The separation distance between the start and stop sensors in millimeters.
    /// * `input_start` - The GPIO input for the start sensor.
    /// * `input_stop` - The GPIO input for the stop sensor.
    /// * `report_format` - The format in which the speed should be reported.
    ///
    /// # Returns
    ///
    /// A new `SpeedTrap` instance.
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

    /// Starts the clock by recording the current timestamp.
    fn start_clock(&mut self) {
        self.start = Instant::now();
    }

    /// Stops the clock by recording the current timestamp.
    fn stop_clock(&mut self) {
        self.stop = Instant::now();
    }

    /// Calculates the elapsed time between the start and stop events in microseconds.
    ///
    /// # Returns
    ///
    /// The elapsed time in microseconds as a floating-point value.
    fn get_elapsed_microseconds(&self) -> f64 {
        let elapsed: Duration = self.stop - self.start;
        elapsed.as_micros() as f64
    }

    /// Calculates the speed of the object based on the elapsed time and separation distance.
    ///
    /// # Arguments
    ///
    /// * `units` - The units in which the speed should be calculated.
    ///
    /// # Returns
    ///
    /// The calculated speed in the specified units.
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

    /// Updates the state of the speed trap based on the sensor inputs.
    ///
    /// This function should be called periodically to process the sensor inputs and
    /// transition the state machine. When the speed trap reaches the `Report` state,
    /// it calculates and prints the speed, then resets to the `Idle` state.
    pub fn tick(&mut self) {
        match self.state {
            TrapState::Idle => {
                if self.input_start.is_low() {
                    self.start_clock();
                    self.state = TrapState::LowStart
                }
            }
            TrapState::LowStart => {
                if self.input_stop.is_low() {
                    self.state = TrapState::LowStop
                }
            }
            TrapState::LowStop => {
                if self.input_start.is_high() {
                    self.state = TrapState::HighStart
                }
            }
            TrapState::HighStart => {
                if self.input_stop.is_high() {
                    self.state = TrapState::HighStop
                }
            }
            TrapState::HighStop => {
                if self.input_start.is_low() {
                    self.stop_clock();
                    self.state = TrapState::Report
                }
            }
            TrapState::Report => {
                esp_println::println!("Speed: {}", self.get_speed(self.report_format));
                self.state = TrapState::Idle
            }
        }
    }
}
