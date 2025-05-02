use esp_hal::time::{Duration, Instant};

use crate::spi_master::{FastSpiMode0, SpiPinsMaster};

/// The height of the LCD in pixels.
const LCD_HEIGHT: u16 = 240;
/// The width of the LCD in pixels.
const LCD_WIDTH: u16 = 135;

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

    /// Fills a rectangular area on the display with a specified color.
    pub fn fill_rectangle(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16) {
        self.set_address_window(x, y, w, h);
        for _ in 0..(w * h) {
            self.spi_master.write_u16(color, Some(true));
        }
    }

    /// Fills the entire screen with a specified color.
    pub fn fill_screen(&mut self, color: u16) {
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
