use image::GenericImageView;

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Texture {
    pub fn load(path: &str) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("Error cargando textura {}: {}", path, e))?;
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                // Convertir canal R, G, B a formato u32 (0x00RRGGBB)
                let color = ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);
                pixels.push(color);
            }
        }

        Ok(Texture { width, height, pixels })
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if self.width == 0 || self.height == 0 {
            return 0xFF00FF;
        }
        let px = (x % self.width) as usize;
        let py = (y % self.height) as usize;
        self.pixels[py * self.width as usize + px]
    }
}
