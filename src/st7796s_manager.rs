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
    dc: Output<'a>,
    ref_x: u16,
    ref_y: u16,
    ref_width: u16,
    ref_height: u16,
}

impl<'a> St7796sManager<'a> {
    pub fn new(spi_master: SpiMasterCore<'a>, dc: Output<'a>) -> St7796sManager<'a> {
        St7796sManager {
            spi_master,
            dc,
            ref_x: 0,
            ref_y: 0,
            ref_width: 0,
            ref_height: 0,
        }
    }

    fn write_u8(&mut self, byte: u8) {
        read_and_write_blocking(&mut self.spi_master, byte as u64, SpiWordSize::EightBits);
    }

    fn write_u16(&mut self, word: u16) {
        read_and_write_blocking(&mut self.spi_master, word as u64, SpiWordSize::SixteenBits);
    }

    fn write_u32(&mut self, word: u32) {
        read_and_write_blocking(
            &mut self.spi_master,
            word as u64,
            SpiWordSize::ThirtyTwoBits,
        );
    }

    pub fn initialize(&mut self) {
        self.dc.set_high();
        self.write_u8(ST77XX_SWRESET);
        blocking_delay_ms(150);
        self.write_u8(ST77XX_SLPOUT);
        blocking_delay_ms(10);
        self.write_u8(ST77XX_COLMOD);
        self.write_u8(0x55);
        blocking_delay_ms(10);
        self.write_u8(ST77XX_MADCTL);
        self.write_u8(0x08);
        self.write_u8(ST77XX_CASET);
        self.write_u8(0x00);
        self.write_u8(0);
        self.write_u8(0x00);
        self.write_u8(135);
        self.write_u8(ST77XX_RASET);
        self.write_u8(0x00);
        self.write_u8(0);
        self.write_u8(0x00);
        self.write_u8(240);
        self.write_u8(ST77XX_INVON);
        blocking_delay_ms(10);
        self.write_u8(ST77XX_NORON);
        blocking_delay_ms(10);
        self.write_u8(ST77XX_DISPON);
        blocking_delay_ms(10);
    }

    pub fn set_rotation(&mut self, m: u8) {
        let mut madctl: u8;
        match m {
            0 => {
                madctl = ST77XX_MADCTL_MX | ST77XX_MADCTL_MY | ST77XX_MADCTL_RGB;
                self.ref_x = 0;
                self.ref_y = 0;
                self.ref_width = LCD_WIDTH;
                self.ref_height = LCD_HEIGHT;
            }
            1 => {
                madctl = ST77XX_MADCTL_MY | ST77XX_MADCTL_MV | ST77XX_MADCTL_RGB;
                self.ref_x = 0;
                self.ref_y = LCD_HEIGHT;
                self.ref_width = LCD_HEIGHT;
                self.ref_height = LCD_WIDTH;
            }
            2 => {
                madctl = ST77XX_MADCTL_RGB;
                self.ref_x = LCD_HEIGHT;
                self.ref_y = LCD_WIDTH;
                self.ref_width = LCD_WIDTH;
                self.ref_height = LCD_HEIGHT;
            }
            _ => {
                madctl = ST77XX_MADCTL_MX | ST77XX_MADCTL_MV | ST77XX_MADCTL_RGB;
                self.ref_x = LCD_WIDTH;
                self.ref_y = LCD_HEIGHT;
                self.ref_width = LCD_HEIGHT;
                self.ref_height = LCD_WIDTH;
            }
        }
        self.write_u8(ST77XX_MADCTL);
        self.write_u8(madctl);
    }

    pub fn draw_rectangle(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16) {
        self.set_address_window(x, y, w, h);
        self.write_u16(color);
    }

    pub fn enable_display(&mut self, enable: bool) {
        if enable {
            self.write_u8(ST77XX_DISPON);
        } else {
            self.write_u8(ST77XX_DISPOFF);
        }
    }

    fn set_address_window(&mut self, x: u16, y: u16, w: u16, h: u16) {
        let x_draw: u32 = x as u32 + self.ref_x as u32;
        let y_draw: u32 = y as u32 + self.ref_y as u32;
        let x_address = (x_draw << 16) | (x_draw + w as u32 - 1);
        let y_address = (y_draw << 16) | (y_draw + h as u32 - 1);
        self.write_u8(ST77XX_CASET);
        self.write_u32(x_address);
        self.write_u8(ST77XX_RASET);
        self.write_u32(y_address);
        self.write_u8(ST77XX_RAMWR);
    }
}
