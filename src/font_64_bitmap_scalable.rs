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
            'A' => 0x000042427E422418,
            'a' => 0x00003C243C203800,
            'B' => 0x00007E46463E467E,
            'b' => 0x00007C44443C0404,
            'C' => 0x00007E420202427E,
            'c' => 0x00003C04043C0000,
            'D' => 0x00007E444444443E,
            'd' => 0x00003C2424382020,
            'E' => 0x00003E02020E023E,
            'e' => 0x0000380838283800,
            'F' => 0x000006061E06067E,
            'f' => 0x000004041C04141C,
            'G' => 0x00007E427202027E,
            'g' => 0x3C203C24243C0000,
            'H' => 0x000066667E666666,
            'h' => 0x000024243C040404,
            'I' => 0x00007E181818187E,
            'i' => 0x0000181818180018,
            'J' => 0x00007E6660606060,
            'j' => 0x00003C2420200020,
            'K' => 0x000066160E162646,
            'k' => 0x000024140C140404,
            'L' => 0x00007E0606060606,
            'l' => 0x0000181818181818,
            'M' => 0x00004242425A6642,
            'm' => 0x0000425A5A7E0000,
            'N' => 0x00004262524A4642,
            'n' => 0x00002424243C0000,
            'O' => 0x00007E424242427E,
            'o' => 0x0000182424180000,
            'P' => 0x00000C0C7C4C4C7C,
            'p' => 0x04043C24243C0000,
            'Q' => 0x00807E624242427E,
            'q' => 0x60203C24243C0000,
            'R' => 0x000066367E66667E,
            'r' => 0x0000080828380000,
            'S' => 0x00007E407E02427E,
            's' => 0x00003C203C043C00,
            'T' => 0x000018181818187E,
            't' => 0x00001808081C0800,
            'U' => 0x00007E4242424242,
            'u' => 0x00003C2424240000,
            'V' => 0x0000181824242442,
            'v' => 0x0000181824240000,
            'W' => 0x00007E5A5A5A5A42,
            'w' => 0x00007E5A5A5A0000,
            'X' => 0x0000422418182442,
            'x' => 0x0000241818240000,
            'Y' => 0x00001818187E4242,
            'y' => 0x0038203C24240000,
            'Z' => 0x00007E040810207E,
            'z' => 0x00003C08103C0000,
            ' ' => 0,
            '!' => 0x0018180018181818,
            '@' => 0xFE017DA5A5BD817E,
            '"' => 0x0000000000226666,
            '#' => 0x00247E24247E2400,
            '$' => 0x00287C407C047C28,
            '%' => 0x007254781E2A4E00,
            '^' => 0x0000000000422418,
            '&' => 0x0000582458042418,
            '(' => 0x0804020202020408,
            ')' => 0x1020404040402010,
            '-' => 0x0000007E7E000000,
            '_' => 0xFFFF000000000000,
            '+' => 0x0018187E7E181800,
            '=' => 0x0000003C003C0000,
            '~' => 0x000000102A440000,
            '\'' => 0x0000000000081000,
            '`' => 0x0000000000100800,
            ':' => 0x0018180000181800,
            ';' => 0x0018100000181800,
            ',' => 0x0008101000000000,
            '<' => 0x0010080404081000,
            '>' => 0x0008102020100800,
            '.' => 0x0018180000000000,
            '*' => 0x0000000024181824,
            '/' => 0x0000020408102040,
            '\\' => 0x0000402010080402,
            '?' => 0x1818001818202418,
            '{' => 0x0804040202040408,
            '}' => 0x1020204040202010,
            '[' => 0x0E0202020202020E,
            ']' => 0x7040404040404070,
            '|' => 0x1818181818181818,
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
