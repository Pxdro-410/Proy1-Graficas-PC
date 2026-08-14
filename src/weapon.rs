use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;

use crate::framebuffer::Framebuffer;

pub struct SoundSystem {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
}

impl SoundSystem {
    pub fn new() -> Self {
        if let Ok((stream, handle)) = OutputStream::try_default() {
            SoundSystem {
                _stream: Some(stream),
                stream_handle: Some(handle),
            }
        } else {
            SoundSystem {
                _stream: None,
                stream_handle: None,
            }
        }
    }

    pub fn play_sound(&self, path: &str) {
        if let Some(ref handle) = self.stream_handle {
            if let Ok(file) = File::open(path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    if let Ok(sink) = Sink::try_new(handle) {
                        sink.append(source);
                        sink.detach();
                    }
                }
            }
        }
    }
}

pub struct Weapon {
    pub is_firing: bool,
    pub fire_frame: u32,
    pub sound_system: SoundSystem,
}

impl Weapon {
    pub fn new() -> Self {
        let sound_system = SoundSystem::new();
        Weapon {
            is_firing: false,
            fire_frame: 0,
            sound_system,
        }
    }

    pub fn shoot(&mut self, sound_path: &str) {
        if !self.is_firing {
            self.is_firing = true;
            self.fire_frame = 0;
            self.sound_system.play_sound(sound_path);
        }
    }

    pub fn update(&mut self) {
        if self.is_firing {
            self.fire_frame += 1;
            if self.fire_frame >= 8 {
                self.is_firing = false;
                self.fire_frame = 0;
            }
        }
    }

    pub fn draw(&self, framebuffer: &mut Framebuffer) {
        let center_x = framebuffer.width / 2;
        let base_y = framebuffer.height;

        // Retroceso vertical del arma al disparar
        let recoil_y = if self.is_firing {
            if self.fire_frame < 4 {
                (self.fire_frame as usize) * 10
            } else {
                ((8 - self.fire_frame) as usize) * 10
            }
        } else {
            0
        };

        let gun_top = base_y.saturating_sub(230).saturating_add(recoil_y);

        // 1. Cuerpo del arma (cañón centrado en primera persona estilo Doom / Wolfenstein 3D)
        let barrel_width_bottom = 140;
        let barrel_width_top = 46;

        for y in gun_top..base_y {
            let t = (y - gun_top) as f32 / (base_y - gun_top) as f32;
            let current_half_width = ((barrel_width_top as f32 * (1.0 - t) + barrel_width_bottom as f32 * t) / 2.0) as usize;

            let left_x = center_x.saturating_sub(current_half_width);
            let right_x = (center_x + current_half_width).min(framebuffer.width);

            for x in left_x..right_x {
                let dist_from_center = (x as f32 - center_x as f32).abs();
                let norm_dist = dist_from_center / current_half_width.max(1) as f32;

                // Gradient de sombreado metálico
                let color = if norm_dist < 0.2 {
                    0x777788 // Brillo superior
                } else if norm_dist < 0.65 {
                    0x333344 // Acero metálico
                } else {
                    0x1B1B26 // Sombra lateral
                };

                framebuffer.set_current_color(color);
                framebuffer.point(x, y);
            }
        }

        // 2. Boquilla reforzada del cañón
        let muzzle_y_start = gun_top.saturating_sub(18);
        let muzzle_width = 52;
        for y in muzzle_y_start..gun_top {
            let left_x = center_x.saturating_sub(muzzle_width / 2);
            let right_x = center_x + muzzle_width / 2;
            for x in left_x..right_x {
                let color = if x > center_x - 12 && x < center_x + 12 { 0x555566 } else { 0x222233 };
                framebuffer.set_current_color(color);
                framebuffer.point(x, y);
            }
        }

        // 3. Fogonazo de disparo (Muzzle Flash) al hacer clic izquierdo
        if self.is_firing && self.fire_frame < 4 {
            let flash_center_y = muzzle_y_start.saturating_sub(30);

            // Capas del fogonazo (núcleo blanco, halo amarillo/naranja)
            framebuffer.draw_rect(center_x.saturating_sub(40), flash_center_y.saturating_sub(40), 80, 80, 0xFFCC00);
            framebuffer.draw_rect(center_x.saturating_sub(24), flash_center_y.saturating_sub(24), 48, 48, 0xFFEEAA);
            framebuffer.draw_rect(center_x.saturating_sub(12), flash_center_y.saturating_sub(12), 24, 24, 0xFFFFFF);

            // Destellos laterales
            for offset in 10..50 {
                framebuffer.set_current_color(0xFF4500);
                framebuffer.point(center_x + offset, flash_center_y - offset / 2);
                framebuffer.point(center_x - offset, flash_center_y - offset / 2);
                framebuffer.point(center_x, flash_center_y - offset);
            }
        }
    }
}
