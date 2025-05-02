use esp_hal::{
    gpio::Output,
    time::{Duration, Instant},
};

use crate::spi_master::{read_and_write_blocking, SpiMasterCore, SpiWordSize};

const LCD_HEIGHT: u16 = 240;
const LCD_WIDTH: u16 = 135;

const ST77XX_NOP: u8 = 0x00;
const ST77XX_SWRESET: u8 = 0x01;
const ST77XX_RDDID: u8 = 0x04;
const ST77XX_RDDST: u8 = 0x09;

const ST77XX_SLPIN: u8 = 0x10;
const ST77XX_SLPOUT: u8 = 0x11;
const ST77XX_PTLON: u8 = 0x12;
const ST77XX_NORON: u8 = 0x13;

const ST77XX_INVOFF: u8 = 0x20;
const ST77XX_INVON: u8 = 0x21;
const ST77XX_DISPOFF: u8 = 0x28;
const ST77XX_DISPON: u8 = 0x29;
const ST77XX_CASET: u8 = 0x2A;
const ST77XX_RASET: u8 = 0x2B;
const ST77XX_RAMWR: u8 = 0x2C;
const ST77XX_RAMRD: u8 = 0x2E;

const ST77XX_PTLAR: u8 = 0x30;
const ST77XX_TEOFF: u8 = 0x34;
const ST77XX_TEON: u8 = 0x35;
const ST77XX_MADCTL: u8 = 0x36;
const ST77XX_COLMOD: u8 = 0x3A;

const ST77XX_MADCTL_MY: u8 = 0x80;
const ST77XX_MADCTL_MX: u8 = 0x40;
const ST77XX_MADCTL_MV: u8 = 0x20;
const ST77XX_MADCTL_ML: u8 = 0x10;
const ST77XX_MADCTL_RGB: u8 = 0x00;

const ST77XX_RDID1: u8 = 0xDA;
const ST77XX_RDID2: u8 = 0xDB;
const ST77XX_RDID3: u8 = 0xDC;
const ST77XX_RDID4: u8 = 0xDD;

pub type Color565 = u16;

// Some ready-made 16-bit ('565') color settings:
pub const ST77XX_BLACK: Color565 = 0x0000;
pub const ST77XX_WHITE: Color565 = 0xFFFF;
pub const ST77XX_RED: Color565 = 0xF800;
pub const ST77XX_GREEN: Color565 = 0x07E0;
pub const ST77XX_BLUE: Color565 = 0x001F;
pub const ST77XX_CYAN: Color565 = 0x07FF;
pub const ST77XX_MAGENTA: Color565 = 0xF81F;
pub const ST77XX_YELLOW: Color565 = 0xFFE0;
pub const ST77XX_ORANGE: Color565 = 0xFC00;

fn blocking_delay_ms(x: u64) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(x) {}
}

pub struct St7796sManager<'a> {
    spi_master: SpiMasterCore<'a>,
    row_start_0: u16,
    column_start_0: u16,
    row_start_1: u16,
    column_start_1: u16,
    window_width: u16,
    window_height: u16,
    x_start: u16,
    y_start: u16,
    set_width: u16,
    set_height: u16,
}

impl<'a> St7796sManager<'a> {
    pub fn new(
        spi_master: SpiMasterCore<'a>,
        window_width: u16,
        window_height: u16,
    ) -> St7796sManager<'a> {
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
        St7796sManager {
            spi_master,
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

    fn write_u8(&mut self, mut word: u8, is_data: bool) {
        /*
        read_and_write_blocking(
            &mut self.spi_master,
            word as u32,
            SpiWordSize::EightBits,
            Some(is_data),
        );
        */

        self.spi_master.pins.cs.set_low();
        if is_data {
            self.spi_master.set_data_mode();
        } else {
            self.spi_master.set_command_mode();
        }
        for _ in 0..8 {
            self.spi_master.pins.sclk.set_low();
            if (word & 0x80) > 0 {
                self.spi_master.pins.mosi.set_high()
            } else {
                self.spi_master.pins.mosi.set_low()
            }
            self.spi_master.pins.sclk.set_high();
            word <<= 1;
        }
        self.spi_master.pins.cs.set_high();
    }

    fn write_u16(&mut self, mut word: u16, is_data: bool) {
        /*
        read_and_write_blocking(
            &mut self.spi_master,
            word as u32,
            SpiWordSize::SixteenBits,
            Some(is_data),
        );
        */
        self.spi_master.pins.cs.set_low();
        if is_data {
            self.spi_master.set_data_mode();
        } else {
            self.spi_master.set_command_mode();
        }
        for _ in 0..16 {
            self.spi_master.pins.sclk.set_low();
            if (word & 0x8000) > 0 {
                self.spi_master.pins.mosi.set_high()
            } else {
                self.spi_master.pins.mosi.set_low()
            }
            self.spi_master.pins.sclk.set_high();
            word <<= 1;
        }
        self.spi_master.pins.cs.set_high();
    }

    fn write_u32(&mut self, mut word: u32, is_data: bool) {
        /*
        read_and_write_blocking(
            &mut self.spi_master,
            word as u32,
            SpiWordSize::ThirtyTwoBits,
            Some(is_data),
        );
        */
        self.spi_master.pins.cs.set_low();
        if is_data {
            self.spi_master.set_data_mode();
        } else {
            self.spi_master.set_command_mode();
        }
        for _ in 0..32 {
            self.spi_master.pins.sclk.set_low();
            if (word & 0x8000_0000) > 0 {
                self.spi_master.pins.mosi.set_high()
            } else {
                self.spi_master.pins.mosi.set_low()
            }
            self.spi_master.pins.sclk.set_high();
            word <<= 1;
        }
        self.spi_master.pins.cs.set_high();
    }

    pub fn display_initialization_sequence(&mut self) {
        self.write_u8(ST77XX_SWRESET, false);
        blocking_delay_ms(150);
        self.write_u8(ST77XX_SLPOUT, false);
        blocking_delay_ms(10);
        self.write_u8(ST77XX_COLMOD, false);
        self.write_u8(0x55, true);
        blocking_delay_ms(10);
        self.write_u8(ST77XX_MADCTL, false);
        self.write_u8(0x08, true);
        self.write_u8(ST77XX_CASET, false);
        self.write_u32(240, true);
        self.write_u8(ST77XX_RASET, false);
        self.write_u32(320, true);
        self.write_u8(ST77XX_INVON, false);
        self.write_u8(ST77XX_NORON, false);
        self.write_u8(ST77XX_DISPON, false);
        blocking_delay_ms(10);
    }

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
        self.write_u8(ST77XX_MADCTL, false);
        self.write_u8(madctl, true);
    }

    pub fn fill_rectangle(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16) {
        self.set_address_window(x, y, w, h);
        for _ in 0..(w * h) {
            self.write_u16(color, true);
        }
    }

    pub fn fill_screen(&mut self, color: u16) {
        self.fill_rectangle(0, 0, self.window_width, self.window_height, color);
    }

    pub fn enable_display(&mut self, enable: bool) {
        if enable {
            self.write_u8(ST77XX_DISPON, false);
        } else {
            self.write_u8(ST77XX_DISPOFF, false);
        }
    }

    fn set_address_window(&mut self, x: u16, y: u16, w: u16, h: u16) {
        let x_draw: u32 = x as u32 + self.x_start as u32;
        let y_draw: u32 = y as u32 + self.y_start as u32;
        let x_address = (x_draw << 16) | (x_draw + w as u32 - 1);
        let y_address = (y_draw << 16) | (y_draw + h as u32 - 1);
        self.write_u8(ST77XX_CASET, false);
        self.write_u32(x_address, true);
        self.write_u8(ST77XX_RASET, false);
        self.write_u32(y_address, true);
        self.write_u8(ST77XX_RAMWR, false);
    }
}
