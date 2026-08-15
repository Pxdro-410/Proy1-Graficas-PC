use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;

use crate::framebuffer::Framebuffer;

pub struct SoundSystem {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    bgm_sink: Option<Sink>,
}

impl SoundSystem {
    pub fn new() -> Self {
        if let Ok((stream, handle)) = OutputStream::try_default() {
            SoundSystem {
                _stream: Some(stream),
                stream_handle: Some(handle),
                bgm_sink: None,
            }
        } else {
            SoundSystem {
                _stream: None,
                stream_handle: None,
                bgm_sink: None,
            }
        }
    }

    pub fn play_sound(&self, path: &str) {
        if let Some(ref handle) = self.stream_handle {
            if let Ok(file) = File::open(path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    if let Ok(sink) = Sink::try_new(handle) {
                        sink.set_volume(0.6);
                        sink.append(source);
                        sink.detach();
                    }
                }
            }
        }
    }

    pub fn start_bgm_loop(&mut self, path: &str, volume: f32) {
        if self.bgm_sink.is_some() {
            return;
        }

        if let Some(ref handle) = self.stream_handle {
            if let Ok(file) = File::open(path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    if let Ok(sink) = Sink::try_new(handle) {
                        sink.set_volume(volume); // Volumen moderado
                        sink.append(source.repeat_infinite()); // Bucle continuo
                        self.bgm_sink = Some(sink);
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
        let mut sound_system = SoundSystem::new();
        // musica al 25%
        sound_system.start_bgm_loop("./assets/music.mp3", 0.25);
        Weapon {
            is_firing: false,
            fire_frame: 0,
            sound_system,
        }
    }


    pub fn shoot(&mut self, sound_path: &str) -> bool {
        if !self.is_firing {
            self.is_firing = true;
            self.fire_frame = 0;
            self.sound_system.play_sound(sound_path);
            true
        } else {
            false
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

        // arma centrada
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

        // Boquilla reforzada del cañón
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

        // Fogonazo de disparo al hacer clic izquierdo
        if self.is_firing && self.fire_frame < 4 {
            let flash_center_y = muzzle_y_start.saturating_sub(30);

            // Capas del fogonazo
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
