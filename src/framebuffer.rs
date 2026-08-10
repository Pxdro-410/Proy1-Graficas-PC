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

    pub fn draw_text(&mut self, x_orig: usize, y_orig: usize, text: &str, color: u32) {
        self.set_current_color(color);
        let mut curr_x = x_orig;

        for ch in text.chars() {
            let glyph: [u8; 5] = match ch {
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
                'F' => [0x7F, 0x09, 0x09, 0x09, 0x01],
                'P' => [0x7F, 0x09, 0x09, 0x09, 0x06],
                'S' => [0x26, 0x49, 0x49, 0x49, 0x32],
                ':' => [0x00, 0x36, 0x36, 0x00, 0x00],
                '.' => [0x00, 0x60, 0x60, 0x00, 0x00],
                ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
                _ => [0x00, 0x00, 0x00, 0x00, 0x00],
            };

            for (col, &byte) in glyph.iter().enumerate() {
                for row in 0..7 {
                    if (byte & (1 << row)) != 0 {
                        for dx in 0..2 {
                            for dy in 0..2 {
                                self.point(curr_x + col * 2 + dx, y_orig + row * 2 + dy);
                            }
                        }
                    }
                }
            }

            curr_x += 13;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
}

