use crate::spi_master::SpiMasterCore;

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

struct st7796s_manager<'a> {
    spi_master: SpiMasterCore<'a>,
}
