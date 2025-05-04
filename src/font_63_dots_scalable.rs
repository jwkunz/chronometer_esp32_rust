use libm::{cosf, fabsf, sinf};

const N_COLUMNS: usize = 7;
const N_ROWS: usize = 9;
pub const N_DOTS: usize = N_COLUMNS * N_ROWS;

pub struct Glyph {
    bit_field: u64,
    base_x: f32,
    base_y: f32,
    scale: f32,
    rotation_rad: f32,
    char_position_x: i16,
    char_position_y: i16,
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
        #[allow(clippy::unusual_byte_groupings)]
        let bit_field: u64 = match x {
            'a' => 0b0000000_0000000_1111111_0000001_1111111_1000001_1111111_0000000_0000000_0,
            'b' => 0b1111110_1111110_1111111_1111111_1111111_1111111_1111111_1111111_1111111_0,
            _ => 0b0000000_0000000_0000000_0000000_0000000_0000000_0000000_0000000_0000000_0,
        };
        Glyph {
            bit_field,
            base_x: base_x.unwrap_or(0.0),
            base_y: base_y.unwrap_or(0.0),
            scale: scale.unwrap_or(2.0),
            rotation_rad: core::f32::consts::PI / 180.0 * rotation_deg.unwrap_or(0.0),
            char_position_x: char_position_x.unwrap_or(0),
            char_position_y: char_position_y.unwrap_or(0),
        }
    }
    pub fn get_dot_radius(&self) -> f32 {
        self.scale * 0.5
    }
    pub fn set_scale(&mut self, x: f32) {
        self.scale = fabsf(x);
    }
    pub fn set_base_position(&mut self, x: f32, y: f32) {
        self.base_x = x;
        self.base_y = y;
    }
    pub fn set_rotation_deg(&mut self, rotation_deg: f32) {
        self.rotation_rad = core::f32::consts::PI / 180.0 * rotation_deg;
    }
    pub fn set_char_position(&mut self, x: i16, y: i16) {
        self.char_position_x = x;
        self.char_position_y = y;
    }
    fn transform_coordinate_xy(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.base_x
                + self.scale * (x * cosf(-self.rotation_rad) - y * sinf(-self.rotation_rad)),
            self.base_y
                + self.scale * (x * sinf(-self.rotation_rad) + y * cosf(-self.rotation_rad)),
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
            if (self.bit_field & (0b1 << (N_DOTS - bit as usize))) != 0 {
                result[bit as usize] = Some(self.get_dot_coordinates(bit));
            } else {
                result[bit as usize] = None;
            }
        }

        result
    }
}
