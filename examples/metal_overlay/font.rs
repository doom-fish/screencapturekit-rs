//! Bitmap font for text rendering

pub struct BitmapFont {
    glyphs: [u64; 128],
}

impl BitmapFont {
    pub fn new() -> Self {
        let mut glyphs = [0u64; 128];
        glyphs[b' ' as usize] = 0x0000_0000_0000_0000;
        glyphs[b'0' as usize] = 0x3C66_6E76_6666_3C00;
        glyphs[b'1' as usize] = 0x1838_1818_1818_7E00;
        glyphs[b'2' as usize] = 0x3C66_060C_1830_7E00;
        glyphs[b'3' as usize] = 0x3C66_061C_0666_3C00;
        glyphs[b'4' as usize] = 0x0C1C_2C4C_7E0C_0C00;
        glyphs[b'5' as usize] = 0x7E60_7C06_0666_3C00;
        glyphs[b'6' as usize] = 0x3C60_607C_6666_3C00;
        glyphs[b'7' as usize] = 0x7E06_0C18_3030_3000;
        glyphs[b'8' as usize] = 0x3C66_663C_6666_3C00;
        glyphs[b'9' as usize] = 0x3C66_663E_0606_3C00;
        glyphs[b'A' as usize] = 0x183C_6666_7E66_6600;
        glyphs[b'B' as usize] = 0x7C66_667C_6666_7C00;
        glyphs[b'C' as usize] = 0x3C66_6060_6066_3C00;
        glyphs[b'D' as usize] = 0x786C_6666_666C_7800;
        glyphs[b'E' as usize] = 0x7E60_607C_6060_7E00;
        glyphs[b'F' as usize] = 0x7E60_607C_6060_6000;
        glyphs[b'G' as usize] = 0x3C66_606E_6666_3E00;
        glyphs[b'H' as usize] = 0x6666_667E_6666_6600;
        glyphs[b'I' as usize] = 0x7E18_1818_1818_7E00;
        glyphs[b'J' as usize] = 0x0606_0606_0666_3C00;
        glyphs[b'K' as usize] = 0x666C_7870_786C_6600;
        glyphs[b'L' as usize] = 0x6060_6060_6060_7E00;
        glyphs[b'M' as usize] = 0xC6EE_FED6_C6C6_C600;
        glyphs[b'N' as usize] = 0x6676_7E7E_6E66_6600;
        glyphs[b'O' as usize] = 0x3C66_6666_6666_3C00;
        glyphs[b'P' as usize] = 0x7C66_667C_6060_6000;
        glyphs[b'Q' as usize] = 0x3C66_6666_6E66_3E00;
        glyphs[b'R' as usize] = 0x7C66_667C_6C66_6600;
        glyphs[b'S' as usize] = 0x3C66_603C_0666_3C00;
        glyphs[b'T' as usize] = 0x7E18_1818_1818_1800;
        glyphs[b'U' as usize] = 0x6666_6666_6666_3C00;
        glyphs[b'V' as usize] = 0x6666_6666_663C_1800;
        glyphs[b'W' as usize] = 0xC6C6_C6D6_FEEE_C600;
        glyphs[b'X' as usize] = 0x6666_3C18_3C66_6600;
        glyphs[b'Y' as usize] = 0x6666_663C_1818_1800;
        glyphs[b'Z' as usize] = 0x7E06_0C18_3060_7E00;
        for c in b'a'..=b'z' {
            glyphs[c as usize] = glyphs[(c - 32) as usize];
        }
        glyphs[b':' as usize] = 0x0018_1800_1818_0000;
        glyphs[b'.' as usize] = 0x0000_0000_0018_1800;
        glyphs[b'-' as usize] = 0x0000_007E_0000_0000;
        glyphs[b'[' as usize] = 0x3C30_3030_3030_3C00;
        glyphs[b']' as usize] = 0x3C0C_0C0C_0C0C_3C00;
        glyphs[b'>' as usize] = 0x6030_180C_1830_6000;
        Self { glyphs }
    }

    pub const fn glyph(&self, c: char) -> u64 {
        let idx = c as usize;
        if idx < 128 {
            self.glyphs[idx]
        } else {
            0
        }
    }

    #[allow(clippy::unused_self)]
    pub const fn pixel_set(&self, glyph: u64, x: usize, y: usize) -> bool {
        if x >= 8 || y >= 8 {
            return false;
        }
        let row = (glyph >> (56 - y * 8)) & 0xFF;
        (row >> (7 - x)) & 1 == 1
    }
}
