use core::cmp::min;

use esp_hal::time::{Duration, Instant};

use crate::spi_master::{FastSpiMode0, SpiPinsMaster};

use libm::{ceilf, cosf, roundf, sinf, sqrtf};

/// Command to perform no operation.
const ST77XX_NOP: u8 = 0x00;
/// Command to perform a software reset.
const ST77XX_SWRESET: u8 = 0x01;
/// Command to read the display ID.
const ST77XX_RDDID: u8 = 0x04;
/// Command to read the display status.
const ST77XX_RDDST: u8 = 0x09;

/// Command to enter sleep mode.
const ST77XX_SLPIN: u8 = 0x10;
/// Command to exit sleep mode.
const ST77XX_SLPOUT: u8 = 0x11;
/// Command to enable partial mode.
const ST77XX_PTLON: u8 = 0x12;
/// Command to enable normal display mode.
const ST77XX_NORON: u8 = 0x13;

/// Command to disable display inversion.
const ST77XX_INVOFF: u8 = 0x20;
/// Command to enable display inversion.
const ST77XX_INVON: u8 = 0x21;
/// Command to turn off the display.
const ST77XX_DISPOFF: u8 = 0x28;
/// Command to turn on the display.
const ST77XX_DISPON: u8 = 0x29;
/// Command to set the column address.
const ST77XX_CASET: u8 = 0x2A;
/// Command to set the row address.
const ST77XX_RASET: u8 = 0x2B;
/// Command to write data to the display RAM.
const ST77XX_RAMWR: u8 = 0x2C;
/// Command to read data from the display RAM.
const ST77XX_RAMRD: u8 = 0x2E;

/// Command to set the partial area.
const ST77XX_PTLAR: u8 = 0x30;
/// Command to disable tearing effect output.
const ST77XX_TEOFF: u8 = 0x34;
/// Command to enable tearing effect output.
const ST77XX_TEON: u8 = 0x35;
/// Command to set the memory access control.
const ST77XX_MADCTL: u8 = 0x36;
/// Command to set the pixel format.
const ST77XX_COLMOD: u8 = 0x3A;

/// Memory access control bit for row address order.
const ST77XX_MADCTL_MY: u8 = 0x80;
/// Memory access control bit for column address order.
const ST77XX_MADCTL_MX: u8 = 0x40;
/// Memory access control bit for row/column exchange.
const ST77XX_MADCTL_MV: u8 = 0x20;
/// Memory access control bit for vertical refresh order.
const ST77XX_MADCTL_ML: u8 = 0x10;
/// Memory access control bit for RGB/BGR order.
const ST77XX_MADCTL_RGB: u8 = 0x00;

/// Command to read ID1.
const ST77XX_RDID1: u8 = 0xDA;
/// Command to read ID2.
const ST77XX_RDID2: u8 = 0xDB;
/// Command to read ID3.
const ST77XX_RDID3: u8 = 0xDC;
/// Command to read ID4.
const ST77XX_RDID4: u8 = 0xDD;

/// Type alias for 16-bit color values in RGB565 format.
pub type Color565 = u16;

/// Predefined 16-bit color values in RGB565 format.
pub const ST77XX_BLACK: Color565 = 0x0000;
pub const ST77XX_WHITE: Color565 = 0xFFFF;
pub const ST77XX_RED: Color565 = 0xF800;
pub const ST77XX_GREEN: Color565 = 0x07E0;
pub const ST77XX_BLUE: Color565 = 0x001F;
pub const ST77XX_CYAN: Color565 = 0x07FF;
pub const ST77XX_MAGENTA: Color565 = 0xF81F;
pub const ST77XX_YELLOW: Color565 = 0xFFE0;
pub const ST77XX_ORANGE: Color565 = 0xFC00;

/// Introduces a blocking delay for the specified number of milliseconds.
///
/// # Arguments
///
/// * `x` - The number of milliseconds to delay.
fn blocking_delay_ms(x: u64) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(x) {}
}

/// A primitive triangle to use
pub struct Triangle {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
    pub x3: u16,
    pub y3: u16,
}

/// A display engine for the ST7789 LCD controller, providing methods for initialization,
/// configuration, and drawing operations.
pub struct St7789DisplayEngine<'a> {
    /// The SPI master used to communicate with the display.
    spi_master: FastSpiMode0<'a>,
    /// The starting row for rotation mode 0.
    row_start_0: u16,
    /// The starting column for rotation mode 0.
    column_start_0: u16,
    /// The starting row for rotation mode 1.
    row_start_1: u16,
    /// The starting column for rotation mode 1.
    column_start_1: u16,
    /// The width of the display window in pixels.
    window_width: u16,
    /// The height of the display window in pixels.
    window_height: u16,
    /// The x-coordinate of the starting address window.
    x_start: u16,
    /// The y-coordinate of the starting address window.
    y_start: u16,
    /// The width of the address window.
    set_width: u16,
    /// The height of the address window.
    set_height: u16,
}

impl<'a> St7789DisplayEngine<'a> {
    /// Creates a new `St7789DisplayEngine` instance.
    ///
    /// # Arguments
    ///
    /// * `pins` - The SPI pins used to communicate with the display.
    /// * `window_width` - The width of the display window in pixels.
    /// * `window_height` - The height of the display window in pixels.
    ///
    /// # Returns
    ///
    /// A new `St7789DisplayEngine` instance.
    pub fn new(
        pins: SpiPinsMaster<'a>,
        window_width: u16,
        window_height: u16,
    ) -> St7789DisplayEngine<'a> {
        let row_start_0;
        let row_start_1;
        let column_start_0;
        let column_start_1;
        if (window_width == 240 && window_height == 240) {
            // 1.3", 1.54" displays (right justified)
            row_start_0 = 320 - window_height;
            row_start_1 = 0;
            column_start_0 = 240 - window_width;
            column_start_1 = column_start_0;
        } else if (window_width == 135 && window_height == 240) {
            // 1.14" display (centered, with odd size)
            row_start_0 = (320 - window_height) / 2;
            row_start_1 = row_start_0;
            column_start_0 = (240 - window_width + 1) / 2;
            column_start_1 = (240 - window_width) / 2;
        } else {
            // 1.47", 1.69, 1.9", 2.0" displays (centered)
            row_start_0 = (320 - window_height) / 2;
            row_start_1 = row_start_0;
            column_start_0 = (240 - window_width) / 2;
            column_start_1 = column_start_0;
        }
        St7789DisplayEngine {
            spi_master: FastSpiMode0 { pins },
            row_start_0,
            column_start_0,
            row_start_1,
            column_start_1,
            window_width,
            window_height,
            x_start: 0,
            y_start: 0,
            set_width: 0,
            set_height: 0,
        }
    }

    /// Sends the initialization sequence to the ST7789 controller.
    pub fn display_initialization_sequence(&mut self) {
        self.spi_master.write_u8(ST77XX_SWRESET, Some(false));
        blocking_delay_ms(150);
        self.spi_master.write_u8(ST77XX_SLPOUT, Some(false));
        blocking_delay_ms(10);
        self.spi_master.write_u8(ST77XX_COLMOD, Some(false));
        self.spi_master.write_u8(0x55, Some(true));
        blocking_delay_ms(10);
        self.spi_master.write_u8(ST77XX_MADCTL, Some(false));
        self.spi_master.write_u8(0x08, Some(true));
        self.spi_master.write_u8(ST77XX_CASET, Some(false));
        self.spi_master.write_u32(240, Some(true));
        self.spi_master.write_u8(ST77XX_RASET, Some(false));
        self.spi_master.write_u32(320, Some(true));
        self.spi_master.write_u8(ST77XX_INVON, Some(false));
        self.spi_master.write_u8(ST77XX_NORON, Some(false));
        self.spi_master.write_u8(ST77XX_DISPON, Some(false));
        blocking_delay_ms(10);
    }

    /// Sets the display rotation based on the given mode (0-3).
    pub fn set_rotation(&mut self, m: u8) {
        let mut madctl: u8;
        match m {
            0 => {
                madctl = ST77XX_MADCTL_MX | ST77XX_MADCTL_MY | ST77XX_MADCTL_RGB;
                self.x_start = self.column_start_0;
                self.y_start = self.row_start_0;
                self.set_width = self.window_width;
                self.set_height = self.window_height;
            }
            1 => {
                madctl = ST77XX_MADCTL_MY | ST77XX_MADCTL_MV | ST77XX_MADCTL_RGB;
                self.x_start = self.row_start_0;
                self.y_start = self.column_start_1;
                self.set_width = self.window_height;
                self.set_height = self.window_width;
            }
            2 => {
                madctl = ST77XX_MADCTL_RGB;
                self.x_start = self.column_start_1;
                self.y_start = self.row_start_1;
                self.set_width = self.window_width;
                self.set_height = self.window_height;
            }
            _ => {
                madctl = ST77XX_MADCTL_MX | ST77XX_MADCTL_MV | ST77XX_MADCTL_RGB;
                self.x_start = self.row_start_1;
                self.y_start = self.column_start_0;
                self.set_width = self.window_height;
                self.set_height = self.window_width;
            }
        }
        self.spi_master.write_u8(ST77XX_MADCTL, Some(false));
        self.spi_master.write_u8(madctl, Some(true));
    }

    /// Helper function for rounding and clippling f32 x pixels
    fn round_and_clip_x(&self, x: f32) -> Option<u16> {
        let rounded = roundf(x);
        if (rounded < 0.0) {
            return None;
        }
        if rounded > self.window_width as f32 {
            return None;
        }
        Some(rounded as u16)
    }

    /// Helper function for rounding and clippling f32 y pixels
    fn round_and_clip_y(&self, y: f32) -> Option<u16> {
        let rounded = roundf(y);
        if rounded < 0.0 {
            return None;
        }
        if rounded > self.window_height as f32 {
            return None;
        }
        Some(rounded as u16)
    }

    /// Fills a rectangular area on the display with a specified color.
    /// x and y correspond to the top left corner
    /// w goes left to right
    /// h goes top to bottom
    pub fn fill_rectangle(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color565) {
        self.set_address_window(x, y, w, h);
        for _ in 0..(w * h) {
            self.spi_master.write_u16(color, Some(true))
        }
    }

    /// Fills a single pixel at x, y and checks for clipping
    pub fn fill_pixel_with_clip_check(&mut self, x: u16, y: u16, color: Color565) {
        if (x < self.window_width) && (y < self.window_height) {
            self.fill_rectangle(x, y, 1, 1, color)
        }
    }

    /// Fills a single pixel at x, y
    pub fn fill_pixel(&mut self, x: u16, y: u16, color: Color565) {
        self.fill_rectangle(x, y, 1, 1, color)
    }

    /// Draws a horizontal line on the display.
    ///
    /// # Arguments
    ///
    /// * `x_start` - The starting x-coordinate of the line.
    /// * `x_end` - The ending x-coordinate of the line.
    /// * `y` - The y-coordinate of the line.
    /// * `color` - The color of the line in RGB565 format.
    pub fn draw_thin_horizontal_line(&mut self, x_start: u16, x_end: u16, y: u16, color: Color565) {
        if y < self.window_height {
            if x_end > x_start {
                self.fill_rectangle(x_start, y, x_end - x_start, 1, color);
            } else {
                self.fill_rectangle(x_end, y, x_start - x_end, 1, color);
            }
        }
    }

    /// Draws a vertical line on the display.
    ///
    /// # Arguments
    ///
    /// * `x` - The y-coordinate of the line.
    /// * `y_start` - The starting y-coordinate of the line.
    /// * `y_end` - The ending y-coordinate of the line.
    /// * `color` - The color of the line in RGB565 format.
    pub fn draw_thin_vertical_line(&mut self, x: u16, y_start: u16, y_end: u16, color: Color565) {
        if x < self.window_width {
            if y_end > y_start {
                self.fill_rectangle(x, y_start, 1, y_end - y_start, color);
            } else {
                self.fill_rectangle(x, y_end, 1, y_start - y_end, color);
            }
        }
    }

    /// Traces a single pixel width line given the start and end points
    pub fn trace_thin_line_start_end(
        &mut self,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
        color: Color565,
    ) {
        let dx = end_x as f32 - start_x as f32;
        let dy = end_y as f32 - start_y as f32;
        let len = sqrtf(dx * dx + dy * dy);
        let dy_dt = dy / len;
        let dx_dt = dx / len;
        for t in 0..=(ceilf(len) as u16) {
            let px = start_x as f32 + dx_dt * (t as f32);
            if let Some(pxr) = self.round_and_clip_x(px) {
                let py = start_y as f32 + dy_dt * (t as f32);
                if let Some(pyr) = self.round_and_clip_y(py) {
                    self.fill_pixel(pxr, pyr, color)
                }
            }
        }
    }

    /// Fills a thick line given the start and end points
    pub fn trace_thick_line_start_end(
        &mut self,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
        thickness: u16,
        color: Color565,
    ) {
        let dx = end_x as f32 - start_x as f32;
        let dy = end_y as f32 - start_y as f32;
        let len = sqrtf(dx * dx + dy * dy);

        if len == 0.0 {
            // Draw a filled circle if start == end
            for tx in -(thickness as i16) / 2..=thickness as i16 / 2 {
                for ty in -(thickness as i16) / 2..=thickness as i16 / 2 {
                    if let Some(pxr) = self.round_and_clip_x(start_x as f32 + tx as f32) {
                        if let Some(pyr) = self.round_and_clip_y(start_y as f32 + ty as f32) {
                            self.fill_pixel(pxr, pyr, color)
                        }
                    }
                }
            }
            return;
        }

        let dy_dt = dy / len;
        let dx_dt = dx / len;
        let perp_x = -dy_dt;
        let perp_y = dx_dt;

        let half_thick = ceilf((thickness as f32) / 2.0);
        let oversample_rate: f32 = 1.5;

        for t in 0..=(ceilf(len) as u16) {
            let fx = start_x as f32 + dx_dt * (t as f32);
            let fy = start_y as f32 + dy_dt * (t as f32);
            // oversample to avoid missing pixels in gaps
            for w in (-oversample_rate * half_thick) as i32..=(oversample_rate * half_thick) as i32
            {
                if let Some(pxr) = self.round_and_clip_x(fx + perp_x * (w as f32) / oversample_rate)
                {
                    if let Some(pyr) =
                        self.round_and_clip_y(fy + perp_y * (w as f32) / oversample_rate)
                    {
                        self.fill_pixel(pxr, pyr, color)
                    }
                }
            }
        }
    }

    /// Fills a circle starting at x, y with radius r
    pub fn fill_circle(&mut self, x: u16, y: u16, radius: u16, color: Color565) {
        let radius_squared = (radius as f32) * (radius as f32);
        self.fill_pixel_with_clip_check(x, y, color);
        for i in 0..radius {
            for j in 0..radius {
                let d_squared = (i * i + j * j) as f32;
                if d_squared < radius_squared {
                    self.fill_pixel_with_clip_check(x + i, y + j, color);
                    let pos_x = i < x;
                    let pos_y = j < y;
                    if pos_x && pos_y {
                        self.fill_pixel_with_clip_check(x - i, y - j, color);
                    }
                    if pos_x {
                        self.fill_pixel_with_clip_check(x - i, y + j, color);
                    }
                    if pos_y {
                        self.fill_pixel_with_clip_check(x + i, y - j, color);
                    }
                }
            }
        }
    }

    /// Traces a circle starting at x, y with radius r
    pub fn trace_circle(&mut self, x: u16, y: u16, radius: u16, thickness: u16, color: Color565) {
        let radius_squared = (radius as f32) * (radius as f32);
        let thickness_squared = (thickness as f32) * (thickness as f32);
        for i in 0..(radius + thickness) {
            for j in 0..(radius + thickness) {
                let d_squared = (i * i + j * j) as f32;
                if (d_squared - radius_squared).abs() < thickness_squared {
                    self.fill_pixel_with_clip_check(x + i, y + j, color);
                    let pos_x = i < x;
                    let pos_y = j < y;
                    if pos_x && pos_y {
                        self.fill_pixel_with_clip_check(x - i, y - j, color);
                    }
                    if pos_x {
                        self.fill_pixel_with_clip_check(x - i, y + j, color);
                    }
                    if pos_y {
                        self.fill_pixel_with_clip_check(x + i, y - j, color);
                    }
                }
            }
        }
    }

    /// Traces a triangle with thin lines
    pub fn trace_thin_triangle(&mut self, triangle: Triangle, color: Color565) {
        self.trace_thin_line_start_end(triangle.x1, triangle.y1, triangle.x2, triangle.y2, color);
        self.trace_thin_line_start_end(triangle.x2, triangle.y2, triangle.x3, triangle.y3, color);
        self.trace_thin_line_start_end(triangle.x3, triangle.y3, triangle.x1, triangle.y1, color);
    }

    /// Fills a triangle with the specified color by shading its interior.
    ///
    /// # Arguments
    ///
    /// * `triangle` - The triangle to fill, defined by its three vertices.
    /// * `color` - The color to fill the triangle with.
    pub fn fill_triangle(&mut self, triangle: Triangle, color: Color565) {
        // Sort the vertices by their y-coordinates (y1 <= y2 <= y3).
        let mut vertices = [
            (triangle.x1, triangle.y1),
            (triangle.x2, triangle.y2),
            (triangle.x3, triangle.y3),
        ];
        vertices.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        let (x1, y1) = vertices[0];
        let (x2, y2) = vertices[1];
        let (x3, y3) = vertices[2];

        // Calculate the slopes of the triangle edges.
        let slope1 = if y2 != y1 {
            (x2 as f32 - x1 as f32) / (y2 as f32 - y1 as f32)
        } else {
            0.0
        };
        let slope2 = if y3 != y1 {
            (x3 as f32 - x1 as f32) / (y3 as f32 - y1 as f32)
        } else {
            0.0
        };
        let slope3 = if y3 != y2 {
            (x3 as f32 - x2 as f32) / (y3 as f32 - y2 as f32)
        } else {
            0.0
        };

        // Fill the top part of the triangle (from y1 to y2).
        let mut x_start = x1 as f32;
        let mut x_end = x1 as f32;
        for y in y1..=y2 {
            self.draw_thin_horizontal_line(roundf(x_start) as u16, roundf(x_end) as u16, y, color);
            x_start += slope1;
            x_end += slope2;
        }

        // Fill the bottom part of the triangle (from y2 to y3).
        x_start = x2 as f32;
        x_end = x1 as f32 + slope2 * (y2 as f32 - y1 as f32);
        for y in y2..=y3 {
            self.draw_thin_horizontal_line(roundf(x_start) as u16, roundf(x_end) as u16, y, color);
            x_start += slope3;
            x_end += slope2;
        }
    }

    /// Fills a regular polygon with the specified number of sides and radius.
    ///
    /// # Arguments
    ///
    /// * `center_x` - The x-coordinate of the center of the polygon.
    /// * `center_y` - The y-coordinate of the center of the polygon.
    /// * `radius` - The radius of the polygon.
    /// * `sides` - The number of sides of the polygon (must be >= 3).
    /// * `color` - The color to fill the polygon with.
    pub fn fill_regular_polygon(
        &mut self,
        center_x: u16,
        center_y: u16,
        radius: u16,
        rotation_deg: f32,
        sides: u16,
        color: Color565,
    ) {
        if sides < 3 {
            return; // A polygon must have at least 3 sides.
        }

        let angle_rad = rotation_deg * core::f32::consts::PI / 180.0;
        let angle_step = 2.0 * core::f32::consts::PI / (sides as f32);

        // Iterate through each side of the polygon and draw the corresponding triangle.
        let mut prev_x = center_x as f32 + radius as f32 * libm::cosf(angle_rad);
        let mut prev_y = center_y as f32 + radius as f32 * libm::sinf(angle_rad);

        for i in 1..=sides {
            let angle = i as f32 * angle_step + angle_rad;
            let curr_x = center_x as f32 + radius as f32 * libm::cosf(angle);
            let curr_y = center_y as f32 + radius as f32 * libm::sinf(angle);

            // If the triangle fits on the screen
            if let Some(prev_x_clipped) = self.round_and_clip_x(prev_x) {
                if let Some(prev_y_clipped) = self.round_and_clip_y(prev_y) {
                    if let Some(curr_x_clipped) = self.round_and_clip_x(curr_x) {
                        if let Some(curr_y_clipped) = self.round_and_clip_y(curr_y) {
                            // Draw the triangle formed by the center, the previous vertex, and the current vertex.
                            self.fill_triangle(
                                Triangle {
                                    x1: center_x,
                                    y1: center_y,
                                    x2: prev_x_clipped,
                                    y2: prev_y_clipped,
                                    x3: curr_x_clipped,
                                    y3: curr_y_clipped,
                                },
                                color,
                            );
                        }
                    }
                }
            }

            // Update the previous vertex to the current vertex.
            prev_x = curr_x;
            prev_y = curr_y;
        }
    }

    /// Fills the entire screen with a specified color.
    pub fn fill_screen(&mut self, color: Color565) {
        self.fill_rectangle(0, 0, self.window_width, self.window_height, color);
    }

    /// Enables or disables the display.
    pub fn enable_display(&mut self, enable: bool) {
        if enable {
            self.spi_master.write_u8(ST77XX_DISPON, Some(false));
        } else {
            self.spi_master.write_u8(ST77XX_DISPOFF, Some(false));
        }
    }

    /// Configures the address window for subsequent drawing operations.
    fn set_address_window(&mut self, x: u16, y: u16, w: u16, h: u16) {
        let x_draw: u32 = x as u32 + self.x_start as u32;
        let y_draw: u32 = y as u32 + self.y_start as u32;
        let x_address = (x_draw << 16) | (x_draw + w as u32 - 1);
        let y_address = (y_draw << 16) | (y_draw + h as u32 - 1);
        self.spi_master.write_u8(ST77XX_CASET, Some(false));
        self.spi_master.write_u32(x_address, Some(true));
        self.spi_master.write_u8(ST77XX_RASET, Some(false));
        self.spi_master.write_u32(y_address, Some(true));
        self.spi_master.write_u8(ST77XX_RAMWR, Some(false));
    }
}
