use libm::{cosf, fabsf, sinf};

const N_COLUMNS: usize = 8;
const N_ROWS: usize = 8;
pub const N_DOTS: usize = N_COLUMNS * N_ROWS;

#[derive(Clone)]
pub struct Glyph {
    bit_field: u64,
    base_x: f32,
    base_y: f32,
    scale: f32,
    rotation_cw_rad: f32,
    char_position_x: i16,
    char_position_y: i16,
}
impl Default for Glyph {
    fn default() -> Self {
        Glyph::new(' ', None, None, None, None, None, None)
    }
}
impl Glyph {
    pub fn from(x: char) -> Self {
        Glyph::new(x, None, None, None, None, None, None)
    }
    pub fn new(
        x: char,
        base_x: Option<f32>,
        base_y: Option<f32>,
        scale: Option<f32>,
        rotation_deg: Option<f32>,
        char_position_x: Option<i16>,
        char_position_y: Option<i16>,
    ) -> Self {
        Glyph {
            bit_field: Glyph::char_to_bitfield(x),
            base_x: base_x.unwrap_or(0.0),
            base_y: base_y.unwrap_or(0.0),
            scale: scale.unwrap_or(1.0),
            rotation_cw_rad: core::f32::consts::PI / 180.0 * rotation_deg.unwrap_or(0.0),
            char_position_x: char_position_x.unwrap_or(0),
            char_position_y: char_position_y.unwrap_or(0),
        }
    }
    pub fn char_to_bitfield(x: char) -> u64 {
        #[allow(clippy::unusual_byte_groupings)]
        match x {
            '0' => 0x00003C464A52623C,
            '1' => 0x00007E18181A1C18,
            '2' => 0x00007E027E40427C,
            '3' => 0x00007E607860627C,
            '4' => 0x000060607E666C78,
            '5' => 0x00007E407E02027C,
            '6' => 0x00007E427E02023C,
            '7' => 0x0000060C1830607E,
            '8' => 0x00007E42427E427E,
            '9' => 0x000060607C62627C,
            '.' => 0x0018180000000000,
            ':' => 0x0018180000181800,
            '-' => 0x0000007E7E000000,
            '+' => 0x0018187E7E181800,
            '*' => 0x0000000024181824,
            '/' => 0x0000020408102040,
            'X' => 0x0042241818244200,
            ' ' => 0,
            _ => 0xFFFFFFFFFFFFFFFF,
        }
    }
    pub fn set_bitfield(&mut self, x: char) {
        self.bit_field = Glyph::char_to_bitfield(x);
    }
    pub fn get_scale(&self) -> f32 {
        self.scale
    }
    pub fn set_scale(&mut self, x: f32) {
        self.scale = fabsf(x);
    }
    pub fn set_base_position(&mut self, x: f32, y: f32) {
        self.base_x = x;
        self.base_y = y;
    }
    pub fn set_rotation_cw_deg(&mut self, rotation_deg: f32) {
        self.rotation_cw_rad = core::f32::consts::PI / 180.0 * rotation_deg;
    }
    pub fn set_char_position(&mut self, x: i16, y: i16) {
        self.char_position_x = x;
        self.char_position_y = y;
    }
    fn transform_coordinate_xy(&self, x: f32, y: f32) -> (f32, f32) {
        let start_x = x + (N_COLUMNS as f32 * self.char_position_x as f32);
        let start_y = y + (N_ROWS as f32 * self.char_position_y as f32);
        (
            self.base_x
                + self.scale
                    * (start_x * cosf(self.rotation_cw_rad) - start_y * sinf(self.rotation_cw_rad)),
            self.base_y
                + self.scale
                    * (start_x * sinf(self.rotation_cw_rad) + start_y * cosf(self.rotation_cw_rad)),
        )
    }
    fn get_row(&self, bit: u8) -> u8 {
        bit / N_COLUMNS as u8
    }
    fn get_column(&self, bit: u8) -> u8 {
        bit % N_COLUMNS as u8
    }

    fn get_dot_coordinates(&self, bit: u8) -> (f32, f32) {
        self.transform_coordinate_xy(self.get_column(bit) as f32, self.get_row(bit) as f32)
    }

    pub fn get_dot_coordinates_array(&self) -> [Option<(f32, f32)>; N_DOTS] {
        let mut result = [None; N_DOTS];

        for bit in 0..N_DOTS as u8 {
            if (self.bit_field & (0b1 << bit)) != 0 {
                result[bit as usize] = Some(self.get_dot_coordinates(bit));
            } else {
                result[bit as usize] = None;
            }
        }

        result
    }
}
