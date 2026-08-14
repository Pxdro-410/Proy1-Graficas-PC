pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
    stars: Vec<(usize, usize, u32)>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut stars = Vec::new();
        let mut seed: u32 = 987654321;
        for _ in 0..140 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let x = (seed as usize) % width;
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let y = (seed as usize) % ((height / 2).saturating_sub(10));
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let color = match seed % 5 {
                0 => 0xFFFFFF, // estrella blanca brillante
                1 => 0xEEEEFF, // azulado brillante
                2 => 0xCCCCCC, // blanco medio
                3 => 0x9999BB, // azul tenue
                _ => 0x777799, // muy tenue
            };
            stars.push((x, y, color));
        }

        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
            stars,
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
    }

    pub fn clear_sky_and_floor(&mut self, sky_color: u32, floor_color: u32) {
        let half = (self.height / 2) * self.width;
        let total = self.width * self.height;

        for pixel in self.buffer[..half].iter_mut() {
            *pixel = sky_color;
        }

        for pixel in self.buffer[half..total].iter_mut() {
            *pixel = floor_color;
        }

        // Pintar estrellas en el cielo nocturno
        for &(x, y, color) in &self.stars {
            if y < self.height / 2 && x < self.width {
                self.buffer[y * self.width + x] = color;
            }
        }
    }



    pub fn point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = self.current_color;
        }
    }

    pub fn draw_rect(&mut self, x_orig: usize, y_orig: usize, w: usize, h: usize, color: u32) {
        for x in x_orig..(x_orig + w).min(self.width) {
            for y in y_orig..(y_orig + h).min(self.height) {
                self.buffer[y * self.width + x] = color;
            }
        }
    }

    pub fn draw_text_scaled(&mut self, x_orig: usize, y_orig: usize, text: &str, scale: usize, color: u32) {
        self.set_current_color(color);
        let mut curr_x = x_orig;

        for ch in text.chars() {
            let glyph: [u8; 5] = match ch.to_ascii_uppercase() {
                '0' => [0x3E, 0x51, 0x49, 0x45, 0x3E],
                '1' => [0x00, 0x42, 0x7F, 0x40, 0x00],
                '2' => [0x42, 0x61, 0x51, 0x49, 0x46],
                '3' => [0x21, 0x41, 0x45, 0x4B, 0x31],
                '4' => [0x18, 0x14, 0x12, 0x7F, 0x10],
                '5' => [0x27, 0x45, 0x45, 0x45, 0x39],
                '6' => [0x3C, 0x4A, 0x49, 0x49, 0x30],
                '7' => [0x01, 0x71, 0x09, 0x05, 0x03],
                '8' => [0x36, 0x49, 0x49, 0x49, 0x36],
                '9' => [0x06, 0x49, 0x49, 0x29, 0x1E],
                'A' => [0x7C, 0x12, 0x11, 0x12, 0x7C],
                'B' => [0x7F, 0x49, 0x49, 0x49, 0x36],
                'C' => [0x3E, 0x41, 0x41, 0x41, 0x22],
                'D' => [0x7F, 0x41, 0x41, 0x22, 0x1C],
                'E' => [0x7F, 0x49, 0x49, 0x49, 0x41],
                'F' => [0x7F, 0x09, 0x09, 0x09, 0x01],
                'G' => [0x3E, 0x41, 0x49, 0x49, 0x7A],
                'H' => [0x7F, 0x08, 0x08, 0x08, 0x7F],
                'I' => [0x00, 0x41, 0x7F, 0x41, 0x00],
                'J' => [0x20, 0x40, 0x41, 0x3F, 0x01],
                'K' => [0x7F, 0x08, 0x14, 0x22, 0x41],
                'L' => [0x7F, 0x40, 0x40, 0x40, 0x40],
                'M' => [0x7F, 0x02, 0x0C, 0x02, 0x7F],
                'N' => [0x7F, 0x04, 0x08, 0x10, 0x7F],
                'O' => [0x3E, 0x41, 0x41, 0x41, 0x3E],
                'P' => [0x7F, 0x09, 0x09, 0x09, 0x06],
                'Q' => [0x3E, 0x41, 0x51, 0x21, 0x5E],
                'R' => [0x7F, 0x09, 0x19, 0x29, 0x46],
                'S' => [0x26, 0x49, 0x49, 0x49, 0x32],
                'T' => [0x01, 0x01, 0x7F, 0x01, 0x01],
                'U' => [0x3F, 0x40, 0x40, 0x40, 0x3F],
                'V' => [0x1F, 0x20, 0x40, 0x20, 0x1F],
                'W' => [0x3F, 0x40, 0x38, 0x40, 0x3F],
                'X' => [0x63, 0x14, 0x08, 0x14, 0x63],
                'Y' => [0x07, 0x08, 0x70, 0x08, 0x07],
                'Z' => [0x61, 0x51, 0x49, 0x45, 0x43],
                ':' => [0x00, 0x36, 0x36, 0x00, 0x00],
                '.' => [0x00, 0x60, 0x60, 0x00, 0x00],
                '-' => [0x08, 0x08, 0x08, 0x08, 0x08],
                '!' => [0x00, 0x00, 0x5F, 0x00, 0x00],
                ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
                _ => [0x00, 0x00, 0x00, 0x00, 0x00],
            };

            for (col, &byte) in glyph.iter().enumerate() {
                for row in 0..7 {
                    if (byte & (1 << row)) != 0 {
                        for dx in 0..scale {
                            for dy in 0..scale {
                                self.point(curr_x + col * scale + dx, y_orig + row * scale + dy);
                            }
                        }
                    }
                }
            }

            curr_x += 6 * scale;
        }
    }

    pub fn draw_text(&mut self, x_orig: usize, y_orig: usize, text: &str, color: u32) {
        self.draw_text_scaled(x_orig, y_orig, text, 2, color);
    }


    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
}

