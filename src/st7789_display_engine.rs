use core::f32::consts::PI;

use esp_hal::time::{Duration, Instant};

use crate::{
    font_64_bitmap_scalable::Glyph,
    spi_master::{FastSpiMode0, SpiPinsMaster},
};

use libm::{ceilf, cosf, roundf, sinf, sqrtf};

/// Command to perform no operation.
pub const ST77XX_NOP: u8 = 0x00;
/// Command to perform a software reset.
pub const ST77XX_SWRESET: u8 = 0x01;
/// Command to read the display ID.
pub const ST77XX_RDDID: u8 = 0x04;
/// Command to read the display status.
pub const ST77XX_RDDST: u8 = 0x09;

/// Command to enter sleep mode.
pub const ST77XX_SLPIN: u8 = 0x10;
/// Command to exit sleep mode.
pub const ST77XX_SLPOUT: u8 = 0x11;
/// Command to enable partial mode.
pub const ST77XX_PTLON: u8 = 0x12;
/// Command to enable normal display mode.
pub const ST77XX_NORON: u8 = 0x13;

/// Command to disable display inversion.
pub const ST77XX_INVOFF: u8 = 0x20;
/// Command to enable display inversion.
pub const ST77XX_INVON: u8 = 0x21;
/// Command to turn off the display.
pub const ST77XX_DISPOFF: u8 = 0x28;
/// Command to turn on the display.
pub const ST77XX_DISPON: u8 = 0x29;
/// Command to set the column address.
pub const ST77XX_CASET: u8 = 0x2A;
/// Command to set the row address.
pub const ST77XX_RASET: u8 = 0x2B;
/// Command to write data to the display RAM.
pub const ST77XX_RAMWR: u8 = 0x2C;
/// Command to read data from the display RAM.
pub const ST77XX_RAMRD: u8 = 0x2E;

/// Command to set the partial area.
pub const ST77XX_PTLAR: u8 = 0x30;
/// Command to disable tearing effect output.
pub const ST77XX_TEOFF: u8 = 0x34;
/// Command to enable tearing effect output.
pub const ST77XX_TEON: u8 = 0x35;
/// Command to set the memory access control.
pub const ST77XX_MADCTL: u8 = 0x36;
/// Command to set the pixel format.
pub const ST77XX_COLMOD: u8 = 0x3A;

/// Memory access control bit for row address order.
pub const ST77XX_MADCTL_MY: u8 = 0x80;
/// Memory access control bit for column address order.
pub const ST77XX_MADCTL_MX: u8 = 0x40;
/// Memory access control bit for row/column exchange.
pub const ST77XX_MADCTL_MV: u8 = 0x20;
/// Memory access control bit for vertical refresh order.
pub const ST77XX_MADCTL_ML: u8 = 0x10;
/// Memory access control bit for RGB/BGR order.
pub const ST77XX_MADCTL_RGB: u8 = 0x00;

/// Command to read ID1.
pub const ST77XX_RDID1: u8 = 0xDA;
/// Command to read ID2.
pub const ST77XX_RDID2: u8 = 0xDB;
/// Command to read ID3.
pub const ST77XX_RDID3: u8 = 0xDC;

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
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub x3: f32,
    pub y3: f32,
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

/// The `St7789DisplayEngine` struct provides an interface for controlling
/// an ST7789 display. It includes methods for initialization, drawing shapes,
/// filling areas, and managing display settings.
///
/// # Methods
///
/// - `new`: Creates a new instance of the display engine.
/// - `display_initialization_sequence`: Sends the initialization sequence to the display.
/// - `set_rotation`: Sets the display rotation based on a given mode.
/// - `set_address_window`: Configures the address window for drawing operations.
/// - `round_and_clip_x`: Rounds and clips an x-coordinate to the display boundaries.
/// - `round_and_clip_y`: Rounds and clips a y-coordinate to the display boundaries.
/// - `fill_rectangle`: Fills a rectangular area with a specified color.
/// - `fill_pixel`: Fills a single pixel without boundary checks.
/// - `fill_pixel_after_rounding_and_clipping`: Fills a single pixel with boundary checks.
/// - `draw_thin_horizontal_line`: Draws a horizontal line with a thickness of 1 pixel.
/// - `draw_thin_vertical_line`: Draws a vertical line with a thickness of 1 pixel.
/// - `trace_thin_line_start_end`: Draws a thin line between two points using Bresenham's algorithm.
/// - `trace_thick_line_start_end`: Draws a thick line between two points with a specified thickness.
/// - `fill_circle`: Fills a circle with a specified radius and color.
/// - `fill_thick_ring`: Draws a thick ring (hollow circle) with a specified thickness.
/// - `trace_thin_triangle`: Draws the outline of a triangle.
/// - `fill_triangle`: Fills a triangle with a specified color.
/// - `fill_regular_polygon`: Fills a regular polygon with a specified number of sides and radius.
/// - `fill_screen`: Fills the entire screen with a specified color.
/// - `draw_glyph`: Draws a single glyph on the display.
/// - `draw_str`: Draws a string slice using a scalable bitmap font.
/// - `enable_display`: Enables or disables the display.
///
/// # Description
///
/// This struct is designed to work with the ST7789 display controller and provides
/// a variety of methods for graphical operations. It supports basic shapes, text rendering,
/// and advanced features like rotation and clipping. The implementation ensures efficient
/// communication with the display using SPI.
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
        if window_width == 240 && window_height == 240 {
            // 1.3", 1.54" displays (right justified)
            row_start_0 = 320 - window_height;
            row_start_1 = 0;
            column_start_0 = 240 - window_width;
            column_start_1 = column_start_0;
        } else if window_width == 135 && window_height == 240 {
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

    /// Access to screen size
    pub fn get_window_width(&self) -> u16 {
        self.window_width
    }
    pub fn get_window_height(&self) -> u16 {
        self.window_height
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
        let madctl: u8;
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

    /// Helper function for rounding and clippling f32 x pixels
    fn round_and_clip_x(&self, x: f32) -> Option<u16> {
        let rounded = roundf(x);
        if rounded < 0.0 {
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
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner of the rectangle.
    /// * `y` - The y-coordinate of the top-left corner of the rectangle.
    /// * `w` - The width of the rectangle in pixels (extends to the right).
    /// * `h` - The height of the rectangle in pixels (extends downward).
    /// * `color` - The color to fill the rectangle with, in RGB565 format.
    pub fn fill_rectangle(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color565) {
        self.set_address_window(x, y, w, h);
        for _ in 0..(w * h) {
            self.spi_master.write_u16(color, Some(true))
        }
    }

    /// Fills a single pixel at the specified coordinates without clipping checks.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `color` - The color to fill the pixel with, in RGB565 format.
    ///
    /// # Description
    ///
    /// This function directly fills the pixel without verifying if the coordinates
    /// are within the display's boundaries. Use this function only when you are certain
    /// the coordinates are valid.
    pub fn fill_pixel(&mut self, x: u16, y: u16, color: Color565) {
        self.fill_rectangle(x, y, 1, 1, color)
    }

    /// Fills a single pixel at the specified coordinates with clipping checks.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `color` - The color to fill the pixel with, in RGB565 format.
    ///
    /// # Description
    ///
    /// This function ensures that the pixel coordinates are within the display's boundaries
    /// before attempting to draw the pixel.
    pub fn fill_pixel_after_rounding_and_clipping(&mut self, x: f32, y: f32, color: Color565) {
        if let Some(px) = self.round_and_clip_x(x) {
            if let Some(py) = self.round_and_clip_y(y) {
                self.fill_rectangle(px, py, 1, 1, color)
            }
        }
    }

    /// Draws a horizontal line on the display with a thickness of 1 pixel.
    ///
    /// # Arguments
    ///
    /// * `x_start` - The starting x-coordinate of the line.
    /// * `x_end` - The ending x-coordinate of the line.
    /// * `y` - The y-coordinate of the line.
    /// * `color` - The color of the line in RGB565 format.
    ///
    /// # Description
    ///
    /// This function draws a horizontal line by filling a 1-pixel-high rectangle
    /// between the specified start and end x-coordinates.
    pub fn draw_thin_horizontal_line(&mut self, x_start: f32, x_end: f32, y: f32, color: Color565) {
        if let Some(py) = self.round_and_clip_y(y) {
            if let Some(pxs) = self.round_and_clip_x(x_start) {
                if let Some(pxe) = self.round_and_clip_x(x_end) {
                    if pxs < pxe {
                        self.fill_rectangle(pxs, py, pxe - pxs, 1, color)
                    } else {
                        self.fill_rectangle(pxe, py, pxs - pxe, 1, color)
                    }
                }
            }
        }
    }

    /// Draws a vertical line on the display with a thickness of 1 pixel.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the line.
    /// * `y_start` - The starting y-coordinate of the line.
    /// * `y_end` - The ending y-coordinate of the line.
    /// * `color` - The color of the line in RGB565 format.
    ///
    /// # Description
    ///
    /// This function draws a vertical line by filling a 1-pixel-wide rectangle
    /// between the specified start and end y-coordinates.
    pub fn draw_thin_vertical_line(&mut self, x: f32, y_start: f32, y_end: f32, color: Color565) {
        if let Some(px) = self.round_and_clip_x(x) {
            if let Some(pys) = self.round_and_clip_y(y_start) {
                if let Some(pye) = self.round_and_clip_y(y_end) {
                    if pys < pye {
                        self.fill_rectangle(px, pys, 1, pye - pys, color)
                    } else {
                        self.fill_rectangle(px, pye, 1, pys - pye, color)
                    }
                }
            }
        }
    }

    /// Traces a single-pixel-width line between two points.
    ///
    /// # Arguments
    ///
    /// * `start_x` - The x-coordinate of the starting point.
    /// * `start_y` - The y-coordinate of the starting point.
    /// * `end_x` - The x-coordinate of the ending point.
    /// * `end_y` - The y-coordinate of the ending point.
    /// * `color` - The color of the line in RGB565 format.
    ///
    /// # Description
    ///
    /// This function uses Bresenham's line algorithm to trace a thin line between
    /// the specified start and end points. It ensures that the line is clipped
    /// to the display's boundaries.
    pub fn trace_thin_line_start_end(
        &mut self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        color: Color565,
    ) {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let len = sqrtf(dx * dx + dy * dy);
        let dy_dt = dy / len;
        let dx_dt = dx / len;
        for t in 0..=(roundf(len) as u16) {
            let px = start_x + dx_dt * (t as f32);
            let py = start_y + dy_dt * (t as f32);
            self.fill_pixel_after_rounding_and_clipping(px, py, color);
        }
    }

    /// Fills a thick line between two points with the specified thickness.
    ///
    /// # Arguments
    ///
    /// * `start_x` - The x-coordinate of the starting point.
    /// * `start_y` - The y-coordinate of the starting point.
    /// * `end_x` - The x-coordinate of the ending point.
    /// * `end_y` - The y-coordinate of the ending point.
    /// * `thickness` - The thickness of the line in pixels.
    /// * `color` - The color of the line in RGB565 format.
    ///
    /// # Description
    ///
    /// This function draws a thick line by oversampling and filling pixels
    /// perpendicular to the line's direction. It ensures that the line is clipped
    /// to the display's boundaries.
    pub fn trace_thick_line_start_end(
        &mut self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        thickness: f32,
        color: Color565,
    ) {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let len = roundf(sqrtf(dx * dx + dy * dy));

        if len == 0.0 {
            // Draw a filled circle if start == end
            self.fill_circle(start_x, start_y, thickness * 0.5, color);
            return;
        }

        let dy_dt = dy / len;
        let dx_dt = dx / len;
        let perp_x = -dy_dt;
        let perp_y = dx_dt;

        let half_thick = ceilf((thickness) / 2.0);
        let oversample_rate: f32 = 1.5;

        for t in 0..=(len as u16) {
            let fx = start_x + dx_dt * (t as f32);
            let fy = start_y + dy_dt * (t as f32);
            // Oversample to avoid missing pixels in gaps
            for w in (-oversample_rate * half_thick) as i16..=(oversample_rate * half_thick) as i16
            {
                let px = fx + perp_x * (w as f32) / oversample_rate;
                let py = fy + perp_y * (w as f32) / oversample_rate;
                self.fill_pixel_after_rounding_and_clipping(px, py, color);
            }
        }
    }

    /// Fills a circle starting at x, y with radius r
    pub fn fill_circle(&mut self, x: f32, y: f32, radius: f32, color: Color565) {
        let radius_squared = radius * radius;
        self.fill_pixel_after_rounding_and_clipping(x, y, color);
        let radius_ceil = ceilf(radius) as u16;
        for i in 0..radius_ceil {
            for j in 0..radius_ceil {
                let d_squared = (i * i + j * j) as f32;
                if d_squared < radius_squared {
                    let px_p = x + (i as f32);
                    let px_n = x - (i as f32);
                    let py_p = y + (j as f32);
                    let py_n = y - (j as f32);
                    self.fill_pixel_after_rounding_and_clipping(px_p, py_p, color);
                    self.fill_pixel_after_rounding_and_clipping(px_p, py_n, color);
                    self.fill_pixel_after_rounding_and_clipping(px_n, py_p, color);
                    self.fill_pixel_after_rounding_and_clipping(px_n, py_n, color);
                }
            }
        }
    }

    /// Traces a circle starting at x, y with radius r
    pub fn fill_thick_ring(
        &mut self,
        x: f32,
        y: f32,
        center_radius: f32,
        thickness: f32,
        color: Color565,
    ) {
        let half_thickness = thickness * 0.5;
        // Oversampling the deltas
        let d_theta = 0.7 * 1.0 / (center_radius + half_thickness);
        let d_r = 0.7 * thickness / ceilf(thickness);
        let mut theta = 0.0;
        while theta < 2.0 * PI {
            let mut r = center_radius - half_thickness;
            while r < center_radius + half_thickness {
                let px = x + r * cosf(theta);
                let py = y + r * sinf(theta);
                self.fill_pixel_after_rounding_and_clipping(px, py, color);
                r += d_r;
            }
            theta += d_theta;
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
        vertices
            .sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));

        let (x1, y1) = vertices[0];
        let (x2, y2) = vertices[1];
        let (x3, y3) = vertices[2];

        // Calculate the slopes of the triangle edges.
        let slope1 = if y2 != y1 { (x2 - x1) / (y2 - y1) } else { 0.0 };
        let slope2 = if y3 != y1 { (x3 - x1) / (y3 - y1) } else { 0.0 };
        let slope3 = if y3 != y2 { (x3 - x2) / (y3 - y2) } else { 0.0 };

        let mut y;
        let mut x_start;
        let mut x_end;

        // Fill the top part of the triangle (from y1 to y2).
        x_start = x1;
        x_end = x1;
        y = y1;
        while y < y2 {
            self.trace_thin_line_start_end(x_start, y, x_end, y, color);
            x_start += slope1;
            x_end += slope2;
            y += 1.0;
        }

        // Fill the bottom part of the triangle (from y2 to y3).
        x_start = x2;
        x_end = x1 + slope2 * (y2 - y1);
        y = y2;
        while y < y3 {
            self.trace_thin_line_start_end(x_start, y, x_end, y, color);
            x_start += slope3;
            x_end += slope2;
            y += 1.0;
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
        center_x: f32,
        center_y: f32,
        radius: f32,
        rotation_deg: f32,
        sides: u8,
        color: Color565,
    ) {
        if sides < 3 {
            return; // A polygon must have at least 3 sides.
        }

        let angle_rad = rotation_deg * core::f32::consts::PI / 180.0;
        let angle_step = 2.0 * core::f32::consts::PI / (sides as f32);

        // Iterate through each side of the polygon and draw the corresponding triangle.
        let mut prev_x = center_x + radius * libm::cosf(angle_rad);
        let mut prev_y = center_y + radius * libm::sinf(angle_rad);

        for i in 1..=sides {
            let angle = i as f32 * angle_step + angle_rad;
            let curr_x = center_x + radius * libm::cosf(angle);
            let curr_y = center_y + radius * libm::sinf(angle);

            // Draw the triangle formed by the center, the previous vertex, and the current vertex.
            self.fill_triangle(
                Triangle {
                    x1: center_x,
                    y1: center_y,
                    x2: prev_x,
                    y2: prev_y,
                    x3: curr_x,
                    y3: curr_y,
                },
                color,
            );

            // Update the previous vertex to the current vertex.
            prev_x = curr_x;
            prev_y = curr_y;
        }
    }

    /// Fills the entire screen with a specified color.
    pub fn fill_screen(&mut self, color: Color565) {
        self.fill_rectangle(0, 0, self.window_width, self.window_height, color);
    }

    /// Draws a glyph on the display.
    ///
    /// # Arguments
    ///
    /// * `glyph` - The `Glyph` object containing the dot coordinates and scaling information.
    /// * `color` - The color to draw the glyph with, in RGB565 format.
    ///
    /// # Description
    ///
    /// This function retrieves the dot coordinates of the glyph using `get_dot_coordinates_array`.
    /// For each dot, it calculates the scaled size and fills a rectangle at the corresponding
    /// position on the display.
    fn draw_glyph(&mut self, glyph: Glyph, color: Color565) {
        let dots = glyph.get_dot_coordinates_array();
        for (px, py) in dots.into_iter().flatten() {
            let w = roundf(glyph.get_scale()) as u16;
            self.fill_rectangle(roundf(px) as u16, roundf(py) as u16, w, w, color);
        }
    }

    /// Draws a string slice on the display using the 64 bitmap scalable font.
    ///
    /// # Arguments
    ///
    /// * `text` - The string slice to draw.
    /// * `base_x` - The x-coordinate of the base position for the text.
    /// * `base_y` - The y-coordinate of the base position for the text.
    /// * `scale` - The scaling factor for the text.
    /// * `rotation_cw_deg` - The clockwise rotation angle in degrees for the text.
    /// * `color` - The color to draw the text with, in RGB565 format.
    ///
    /// # Description
    ///
    /// This function iterates over the characters in the string slice and draws each character
    /// as a glyph. It handles newlines by resetting the x-coordinate and incrementing the y-coordinate.
    /// Each glyph is positioned and scaled based on the provided base position, scale, and rotation.
    pub fn draw_str(
        &mut self,
        text: &str,
        base_x: f32,
        base_y: f32,
        scale: f32,
        rotation_cw_deg: f32,
        color: Color565,
    ) {
        let mut glyph = Glyph::default();
        glyph.set_base_position(base_x, base_y);
        glyph.set_scale(scale);
        glyph.set_rotation_cw_deg(rotation_cw_deg);
        let mut char_position_x: i16 = 0;
        let mut char_position_y: i16 = 0;
        for c in text.chars() {
            if c == '\n' {
                char_position_x = 0;
                char_position_y += 1;
            } else {
                glyph.set_bitfield(c);
                glyph.set_char_position(char_position_x, char_position_y);
                self.draw_glyph(glyph.clone(), color);
                char_position_x += 1;
            }
        }
    }

    /// Enables or disables the display.
    pub fn enable_display(&mut self, enable: bool) {
        if enable {
            self.spi_master.write_u8(ST77XX_DISPON, Some(false));
        } else {
            self.spi_master.write_u8(ST77XX_DISPOFF, Some(false));
        }
    }
}
