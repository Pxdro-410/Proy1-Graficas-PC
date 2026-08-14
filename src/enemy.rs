use nalgebra_glm::Vec2;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::texture::Texture;

#[derive(Clone)]
pub struct Enemy {
    pub pos: Vec2,
    pub hp: i32,
    pub max_hp: i32,
    pub is_alive: bool,
    pub attack_cooldown: f32,
    pub hit_flash_timer: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32) -> Self {
        Enemy {
            pos: Vec2::new(x, y),
            hp: 100,
            max_hp: 100,
            is_alive: true,
            attack_cooldown: 0.0,
            hit_flash_timer: 0.0,
        }
    }

    /// Comprueba la línea de visión directa mediante raycasting entre el enemigo y el jugador
    pub fn has_line_of_sight(&self, player: &Player, maze: &Maze, block_size: usize) -> bool {
        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 1.0 {
            return true;
        }

        let steps = (dist / 10.0) as usize;
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let check_x = self.pos.x + dx * t;
            let check_y = self.pos.y + dy * t;

            let col = (check_x / block_size as f32) as usize;
            let row = (check_y / block_size as f32) as usize;

            if row >= maze.len() || col >= maze[row].len() || maze[row][col] != ' ' {
                return false; // Pared bloqueando la vista
            }
        }
        true
    }

    pub fn update(&mut self, player: &mut Player, dt: f32, maze: &Maze, block_size: usize) {
        if !self.is_alive {
            return;
        }

        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }

        if self.hit_flash_timer > 0.0 {
            self.hit_flash_timer -= dt;
        }

        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        let has_los = self.has_line_of_sight(player, maze, block_size);

        // Persecución fluida con deslizamiento en paredes cuando el jugador está a la vista
        if has_los && dist < 550.0 && dist > 45.0 {
            let move_speed = 65.0 * dt;
            let target_x = self.pos.x + (dx / dist) * move_speed;
            let target_y = self.pos.y + (dy / dist) * move_speed;

            // Desplazamiento X con chequeo de colisión
            let col_x = (target_x / block_size as f32) as usize;
            let row_curr = (self.pos.y / block_size as f32) as usize;
            if row_curr < maze.len() && col_x < maze[row_curr].len() && maze[row_curr][col_x] == ' ' {
                self.pos.x = target_x;
            }

            // Desplazamiento Y con chequeo de colisión
            let col_curr = (self.pos.x / block_size as f32) as usize;
            let row_y = (target_y / block_size as f32) as usize;
            if row_y < maze.len() && col_curr < maze[row_y].len() && maze[row_y][col_curr] == ' ' {
                self.pos.y = target_y;
            }
        }

        // Ataque al jugador cuando tiene línea de visión y está cerca
        if has_los && dist < 320.0 && self.attack_cooldown <= 0.0 {
            player.take_damage(20); // Quita un 20% de la vida del jugador
            self.attack_cooldown = 2.0; // Cooldown de 2.0 segundos entre ataques
        }
    }
}

/// Escanea el mapa del laberinto en busca del carácter 'e' o 'E' y genera a los enemigos en esas posiciones
pub fn spawn_enemies_from_maze(maze: &mut Maze, block_size: usize) -> Vec<Enemy> {
    let mut enemies = Vec::new();
    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            let cell = maze[row][col];
            if cell == 'e' || cell == 'E' {
                let x = col * block_size + block_size / 2;
                let y = row * block_size + block_size / 2;
                enemies.push(Enemy::new(x as f32, y as f32));
                maze[row][col] = ' '; // Reemplazar la 'e' por espacio transitable ' '
            }
        }
    }
    enemies
}

/// Verifica el disparo hitscan del jugador asegurando que se necesiten 4 disparos para eliminar al enemigo
pub fn check_player_shot_hit(player: &Player, enemies: &mut [Enemy], maze: &Maze, block_size: usize) -> bool {
    let mut ray_d = 0.0;
    let max_d = 750.0;

    while ray_d < max_d {
        ray_d += 6.0;
        let rx = player.pos.x + ray_d * player.a.cos();
        let ry = player.pos.y + ray_d * player.a.sin();

        let col = (rx / block_size as f32) as usize;
        let row = (ry / block_size as f32) as usize;

        if row >= maze.len() || col >= maze[row].len() || maze[row][col] != ' ' {
            break; // Bloqueado por pared
        }

        for enemy in enemies.iter_mut() {
            if enemy.is_alive {
                let edx = rx - enemy.pos.x;
                let edy = ry - enemy.pos.y;
                if (edx * edx + edy * edy) < 35.0 * 35.0 {
                    enemy.hp -= 24; // 24 de daño por disparo
                    enemy.hit_flash_timer = 0.18;
                    if enemy.hp <= 0 {
                        enemy.hp = 0;
                        enemy.is_alive = false;
                    }
                    return true; // Un solo impacto por cada bala
                }
            }
        }
    }
    false
}

/// Renderizado de enemigos en 3D con escalado proporcional
pub fn render_enemies(
    framebuffer: &mut Framebuffer,
    enemies: &[Enemy],
    player: &Player,
    d_plane: f32,
    z_buffer: &[f32],
    enemy_texture: Option<&Texture>,
) {
    let mut sorted_indices: Vec<usize> = (0..enemies.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        let da = (enemies[a].pos - player.pos).norm_squared();
        let db = (enemies[b].pos - player.pos).norm_squared();
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });

    for idx in sorted_indices {
        let enemy = &enemies[idx];
        if !enemy.is_alive {
            continue;
        }

        let dx = enemy.pos.x - player.pos.x;
        let dy = enemy.pos.y - player.pos.y;

        let sprite_angle = dy.atan2(dx) - player.a;
        let mut norm_angle = sprite_angle;
        while norm_angle > std::f32::consts::PI { norm_angle -= 2.0 * std::f32::consts::PI; }
        while norm_angle < -std::f32::consts::PI { norm_angle += 2.0 * std::f32::consts::PI; }

        let raw_dist = (dx * dx + dy * dy).sqrt() * norm_angle.cos();
        if raw_dist < 15.0 {
            continue;
        }

        let distance = raw_dist.max(25.0);

        let screen_x_center = (framebuffer.width as f32 / 2.0) + (norm_angle.tan() * d_plane);
        
        // Limitar el tamaño al 65% para evitar gigantismo
        let raw_sprite_size = (65.0 / distance) * d_plane;
        let max_sprite_size = framebuffer.height as f32 * 0.70;
        let sprite_size = raw_sprite_size.min(max_sprite_size);

        let sprite_w = sprite_size as usize;
        let sprite_h = sprite_size as usize;

        let start_x = (screen_x_center - sprite_size / 2.0) as isize;
        let start_y = (framebuffer.height as f32 / 2.0 - sprite_size / 4.0) as isize;


        for x in 0..sprite_w {
            let screen_x = start_x + x as isize;
            if screen_x < 0 || screen_x >= framebuffer.width as isize {
                continue;
            }

            let sx = screen_x as usize;
            if z_buffer.get(sx).map_or(false, |&z| distance >= z) {
                continue;
            }

            for y in 0..sprite_h {
                let screen_y = start_y + y as isize;
                if screen_y < 0 || screen_y >= framebuffer.height as isize {
                    continue;
                }

                let sy = screen_y as usize;

                let color = if let Some(tex) = enemy_texture {
                    let tx = ((x as f32 / sprite_w as f32) * tex.width as f32) as u32;
                    let ty = ((y as f32 / sprite_h as f32) * tex.height as f32) as u32;

                    if !tex.is_pixel_visible(tx, ty) {
                        continue;
                    }

                    if enemy.hit_flash_timer > 0.0 {
                        0xFF4444
                    } else {
                        tex.get_pixel(tx, ty)
                    }
                } else {
                    let rel_x = x as f32 / sprite_w as f32;
                    let rel_y = y as f32 / sprite_h as f32;

                    if enemy.hit_flash_timer > 0.0 {
                        0xFFFFFF
                    } else if (rel_y > 0.25 && rel_y < 0.38) && ((rel_x > 0.25 && rel_x < 0.4) || (rel_x > 0.6 && rel_x < 0.75)) {
                        0xFF0000
                    } else if (rel_x - 0.5).powi(2) + (rel_y - 0.5).powi(2) < 0.22 {
                        0xCC2222
                    } else {
                        continue;
                    }
                };

                framebuffer.set_current_color(color);
                framebuffer.point(sx, sy);
            }
        }

        // Barra de salud en 3D sobre el enemigo
        let hp_bar_w = sprite_w.min(50);
        let hp_bar_x = (screen_x_center - hp_bar_w as f32 / 2.0) as isize;
        let hp_bar_y = start_y - 12;

        if hp_bar_y > 0 && hp_bar_x > 0 && (hp_bar_x + hp_bar_w as isize) < framebuffer.width as isize {
            let hp_ratio = enemy.hp as f32 / enemy.max_hp as f32;
            let fill_w = (hp_bar_w as f32 * hp_ratio) as usize;

            framebuffer.draw_rect(hp_bar_x as usize, hp_bar_y as usize, hp_bar_w, 5, 0x330000);
            framebuffer.draw_rect(hp_bar_x as usize, hp_bar_y as usize, fill_w, 5, 0xFF2222);
        }
    }
}
