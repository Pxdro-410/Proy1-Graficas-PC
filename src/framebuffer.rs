pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
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

