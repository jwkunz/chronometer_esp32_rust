use esp_hal::time::{Duration, Instant};

use crate::spi_master::{read_and_write_blocking, SpiMasterCore, SpiWordSize};

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
    let mut delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(x) {}
}

pub struct st7796s_manager<'a> {
    spi_master: SpiMasterCore<'a>,
}

impl<'a> st7796s_manager<'a> {
    pub fn new(spi_master: SpiMasterCore<'a>) -> st7796s_manager<'a> {
        st7796s_manager { spi_master }
    }

    fn initialize(&mut self) {
        //  1: Software reset, no args
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_SWRESET as u64,
            SpiWordSize::EightBits,
        );
        blocking_delay_ms(150);
        //  2: Out of sleep mode, no args
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_SLPOUT as u64,
            SpiWordSize::EightBits,
        );
        blocking_delay_ms(10);
        //  3: Set color mode to 16 bit color
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_COLMOD as u64,
            SpiWordSize::EightBits,
        );
        read_and_write_blocking(
            &mut self.spi_master,
            0x55 as u64, // 16 bit
            SpiWordSize::EightBits,
        );
        blocking_delay_ms(10);
        //  4: Mem access ctrl (directions)
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_MADCTL as u64,
            SpiWordSize::EightBits,
        );
        read_and_write_blocking(
            &mut self.spi_master,
            0x08 as u64, // Row/col addr, bottom-top refresh
            SpiWordSize::EightBits,
        );
        //  5: Column addr set
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_CASET as u64,
            SpiWordSize::EightBits,
        );
        read_and_write_blocking(&mut self.spi_master, 0x00 as u64, SpiWordSize::EightBits);
        read_and_write_blocking(
            &mut self.spi_master,
            0 as u64, // XSTART = 0
            SpiWordSize::EightBits,
        );
        read_and_write_blocking(&mut self.spi_master, 0 as u64, SpiWordSize::EightBits);
        read_and_write_blocking(
            &mut self.spi_master,
            240 as u64, // XEND = 240
            SpiWordSize::EightBits,
        );
        // 6: Row addr set
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_RASET as u64,
            SpiWordSize::EightBits,
        );
        read_and_write_blocking(&mut self.spi_master, 0x00 as u64, SpiWordSize::EightBits);
        read_and_write_blocking(
            &mut self.spi_master,
            0 as u64, // YSTART = 0
            SpiWordSize::EightBits,
        );
        read_and_write_blocking(
            &mut self.spi_master,
            320 >> 8 as u64,
            SpiWordSize::EightBits,
        );
        read_and_write_blocking(
            &mut self.spi_master,
            320 & 0xff as u64, // YEND = 320
            SpiWordSize::EightBits,
        );
        //  7: hack
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_INVON as u64, //
            SpiWordSize::EightBits,
        );
        blocking_delay_ms(10);
        //  8: Normal display on
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_NORON as u64, //
            SpiWordSize::EightBits,
        );
        blocking_delay_ms(10);
        //  9: Main screen turn on
        read_and_write_blocking(
            &mut self.spi_master,
            ST77XX_DISPON as u64, //
            SpiWordSize::EightBits,
        );
        blocking_delay_ms(10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
