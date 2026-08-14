use image::GenericImageView;

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
    pub alpha: Vec<u8>,
}

impl Texture {
    pub fn load(path: &str) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("Error cargando textura {}: {}", path, e))?;
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        let mut alpha = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                // Convertir canal R, G, B a formato u32 (0x00RRGGBB)
                let color = ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);
                let a = pixel[3];
                pixels.push(color);
                alpha.push(a);
            }
        }

        Ok(Texture {
            width,
            height,
            pixels,
            alpha,
        })
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if self.width == 0 || self.height == 0 {
            return 0xFF00FF;
        }
        let px = (x % self.width) as usize;
        let py = (y % self.height) as usize;
        self.pixels[py * self.width as usize + px]
    }

    pub fn is_pixel_visible(&self, x: u32, y: u32) -> bool {
        if self.width == 0 || self.height == 0 {
            return false;
        }
        let px = (x % self.width) as usize;
        let py = (y % self.height) as usize;
        let idx = py * self.width as usize + px;
        let c = self.pixels[idx];

        // Verificar transparencia por canal alfa de la imagen PNG
        if let Some(&a) = self.alpha.get(idx) {
            if a < 30 {
                return false;
            }
        }
        !(c == 0x00FFFF || c == 0xFF00FF)
    }
}
