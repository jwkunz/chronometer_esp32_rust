use esp_hal::gpio::Input;
use esp_hal::time::{Duration, Instant};

use crate::st7789_display_engine::{St7789DisplayEngine, ST77XX_BLACK, ST77XX_GREEN};

/// Represents the various states of the speed trap.
enum TrapState {
    /// The speed trap is idle and waiting for an trip event.
    Idle,
    /// The start sensor is triggered forward and is waiting for the second trip.
    WaitForStopTrip,
    /// Waiting for lasers to re-lock
    WaitToPass,
    /// The speed trap has completed a measurement and is ready to report.
    Report,
    /// A wait for the toggle unit trip to clear
    WaitToggleUnit,
}

/// Represents the units of speed for reporting.
#[derive(Clone, Copy)]
pub enum SpeedUnits {
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
    /// Screen driver
    screen_driver: &'a mut St7789DisplayEngine<'a>,
    /// Memory buffer of speeds in mm / microseconds
    speed_0: f64,
    speed_1: f64,
    speed_2: f64,
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
    /// * `screen_driver` - Reference to the graphics engine
    ///
    /// # Returns
    ///
    /// A new `SpeedTrap` instance.
    pub fn from(
        separation_mm: f64,
        input_start: Input<'a>,
        input_stop: Input<'a>,
        report_format: SpeedUnits,
        screen_driver: &'a mut St7789DisplayEngine<'a>,
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
            screen_driver,
            speed_0: 0.0,
            speed_1: 0.0,
            speed_2: 0.0,
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
    ///
    /// # Returns
    ///
    /// The calculated speed in mm per microseconds
    fn get_speed(&self) -> f64 {
        let elapsed_micros = self.get_elapsed_microseconds();
        self.separation_mm / elapsed_micros
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
    fn convert_speed(speed_mm_per_us: f64, units: SpeedUnits) -> f64 {
        match units {
            SpeedUnits::MetersPerSecond => speed_mm_per_us * (1000.0),
            SpeedUnits::KilometersPerHour => speed_mm_per_us * (3600.0),
            SpeedUnits::FeetPerSecond => speed_mm_per_us * (3280.8399),
            SpeedUnits::MilesPerHour => speed_mm_per_us * (2236.93629),
        }
    }

    /// Updates the log
    fn update_memory(&mut self, new_speed: f64) {
        self.speed_2 = self.speed_1;
        self.speed_1 = self.speed_0;
        self.speed_0 = new_speed;
    }

    /// Gets the average from the log
    fn compute_average(&self) -> f64 {
        (self.speed_0 + self.speed_1 + self.speed_2) / 3.0
    }

    /// Gets the maximum span from the log
    fn compute_max_span(&self) -> f64 {
        let a = (self.speed_0 - self.speed_1).abs();
        let b = (self.speed_1 - self.speed_2).abs();
        let c = (self.speed_2 - self.speed_0).abs();
        let d = f64::max(a, b);
        f64::max(d, c)
    }

    /// Advances the unit when toggled
    fn toggle_unit(&mut self) {
        self.report_format = match self.report_format {
            SpeedUnits::MetersPerSecond => SpeedUnits::KilometersPerHour,
            SpeedUnits::KilometersPerHour => SpeedUnits::FeetPerSecond,
            SpeedUnits::FeetPerSecond => SpeedUnits::MilesPerHour,
            SpeedUnits::MilesPerHour => SpeedUnits::MetersPerSecond,
        }
    }

    /// Appends a formatted speed value to the buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The mutable buffer to append to.
    /// * `index` - The current index in the buffer.
    /// * `speed` - The speed value to format and append.
    ///
    /// # Returns
    ///
    /// The updated index after appending the formatted speed.
    fn append_speed_to_buffer(buffer: &mut [u8], mut index: usize, speed: f64) -> usize {
        // Convert the speed to an integer part and fractional part
        let integer_part = speed as u32;
        let fractional_part = libm::round((speed - integer_part as f64) * 1000.0) as u32; // 3 decimal places

        // Write the integer part into the buffer (always 5 spaces)
        let mut temp = integer_part;
        let mut digits = [b' '; 5]; // Fixed buffer for 5 spaces
        let mut digit_count = 0;
        while temp > 0 {
            digits[4 - digit_count] = b'0' + (temp % 10) as u8;
            temp /= 10;
            digit_count += 1;
        }
        for &digit in &digits {
            if index < buffer.len() {
                buffer[index] = digit;
                index += 1;
            }
        }

        // Add the decimal point
        if index < buffer.len() {
            buffer[index] = b'.';
            index += 1;
        }

        // Write the fractional part into the buffer (always 3 digits)
        let mut temp = fractional_part;
        let mut digits = [b'0'; 3]; // Fixed buffer for 3 digits
        for i in (0..3).rev() {
            digits[i] = b'0' + (temp % 10) as u8;
            temp /= 10;
        }
        for &digit in &digits {
            if index < buffer.len() {
                buffer[index] = digit;
                index += 1;
            }
        }

        index
    }

    /// Use this to add text to a char buffer
    fn append_text_to_buffer(buffer: &mut [u8], mut index: usize, text: &[u8]) -> usize {
        for &byte in text {
            if index < buffer.len() {
                buffer[index] = byte;
                index += 1;
            }
        }
        index
    }

    /// Writes the result to the display.
    ///
    /// # Arguments
    ///
    /// * `speed` - The calculated speed to display.
    ///
    /// # Description
    ///
    /// This function formats the speed value and displays it on the screen using the screen driver.
    fn write_results_to_screen(&mut self) {
        // Create a fixed-size buffer for the string
        let mut buffer = [0u8; 128]; // 128 bytes for the string
        let mut index = 0;

        // Add a label
        index = Self::append_text_to_buffer(&mut buffer, index, b"|MEM0|");

        // Append speed_0 to the buffer
        index = Self::append_speed_to_buffer(
            &mut buffer,
            index,
            Self::convert_speed(self.speed_0, self.report_format),
        );

        // Add a label
        index = Self::append_text_to_buffer(&mut buffer, index, b"\n|MEM1|");

        // Append speed_1 to the buffer
        index = Self::append_speed_to_buffer(
            &mut buffer,
            index,
            Self::convert_speed(self.speed_1, self.report_format),
        );

        // Add a label
        index = Self::append_text_to_buffer(&mut buffer, index, b"\n|MEM2|");

        // Append speed_2 to the buffer
        index = Self::append_speed_to_buffer(
            &mut buffer,
            index,
            Self::convert_speed(self.speed_2, self.report_format),
        );

        // Add a label
        index = Self::append_text_to_buffer(&mut buffer, index, b"\n\n|MEAN|");

        // Append the average speed to the buffer
        let average = self.compute_average();
        index = Self::append_speed_to_buffer(
            &mut buffer,
            index,
            Self::convert_speed(average, self.report_format),
        );

        // Add a label
        index = Self::append_text_to_buffer(&mut buffer, index, b"\n|SPAN|");

        // Append the span to the buffer
        let span = self.compute_max_span();
        index = Self::append_speed_to_buffer(
            &mut buffer,
            index,
            Self::convert_speed(span, self.report_format),
        );

        // Add a label
        index = Self::append_text_to_buffer(&mut buffer, index, b"\n|UNIT|   ");

        // Determine the unit string based on the report format
        let unit = match self.report_format {
            SpeedUnits::MetersPerSecond => b" m/s ",
            SpeedUnits::KilometersPerHour => b" km/h",
            SpeedUnits::FeetPerSecond => b" fps ",
            SpeedUnits::MilesPerHour => b" mph ",
        };
        // Add the unit
        index = Self::append_text_to_buffer(&mut buffer, index, unit);

        // Convert the buffer to a string slice
        let speed_str = core::str::from_utf8(&buffer[..index]).unwrap_or("Error");

        // Clear the screen
        self.screen_driver.fill_screen(ST77XX_BLACK);

        // Use the string (e.g., display it on the screen)
        self.screen_driver
            .draw_str(speed_str, 125.0, 0.0, 2.0, 90.0, ST77XX_GREEN);
    }

    /// Updates the state of the speed trap based on the sensor inputs.
    ///
    /// This function should be called periodically to process the sensor inputs and
    /// transition the state machine. When the speed trap reaches the `Report` state,
    /// it calculates and displays the speed, then resets to the `Idle` state.
    pub fn tick(&mut self) {
        match self.state {
            TrapState::Idle => {
                // Forward trip
                if self.input_start.is_high() {
                    // Start clock
                    self.start_clock();
                    self.state = TrapState::WaitForStopTrip

                    // Reverse trip
                } else if self.input_stop.is_high() {
                    self.toggle_unit();
                    self.state = TrapState::WaitToggleUnit;
                }
            }
            TrapState::WaitForStopTrip => {
                if self.input_stop.is_high() {
                    // Time stop
                    self.stop_clock();
                    self.state = TrapState::WaitToPass
                }
            }
            TrapState::WaitToPass => {
                // Waiting to clear
                if self.input_stop.is_low() && self.input_start.is_low() {
                    self.state = TrapState::Report
                }
            }
            TrapState::Report => {
                // Calculate the speed
                let speed = self.get_speed();
                // Update the memory buffers
                self.update_memory(speed);
                // Write the result to the display
                self.write_results_to_screen();
                // Go back to idle waiting
                self.state = TrapState::Idle
            }
            TrapState::WaitToggleUnit => {
                if self.input_stop.is_low() && self.input_start.is_low() {
                    // Update the screen with new units
                    self.write_results_to_screen();
                    // Back to idle
                    self.state = TrapState::Idle
                }
            }
        }
    }
}
