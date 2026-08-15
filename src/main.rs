mod caster;
mod enemy;
mod framebuffer;
mod maze;
mod player;
mod texture;
mod weapon;

use minifb::{Key, KeyRepeat, MouseButton, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

use crate::caster::{cast_ray, cast_ray_2d};
use crate::enemy::{check_player_shot_hit, render_enemies, spawn_enemies_from_maze, Enemy};

use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, Maze};
use crate::player::{process_events, Player};
use crate::texture::Texture;
use crate::weapon::Weapon;




const BLOCK_SIZE: usize = 100;

/// Cantidad de rayos, la misma cantidad de columnas que tiene la pantalla.
const NUM_RAYS: usize = 1200;

/// Amplitud del campo de visión o FOV
const FOV: f32 = PI / 3.0;

pub fn cell_color(cell: char) -> u32 {
    match cell {
        '+' => 0x00AAFF, // columnas
        '-' => 0xFF5555, // paredes horizontales
        '|' => 0xFF5555, // paredes verticales
        'g' | 'G' => 0x00FF00, // meta
        _ => 0xFFDDDD,   // cualquier otra cosa
    }
}

fn draw_cell_2d(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    cell: char,
    size: usize,
    wall_texture: &Texture,
    door_texture: &Texture,
) {
    let floor_color = 0x1B3B22; // Color de grama nocturna del piso en 3D

    if cell == ' ' {
        // Suelo del mapa en 2D con el mismo color verde grama que en 3D
        for x in xo..(xo + size).min(framebuffer.width) {
            for y in yo..(yo + size).min(framebuffer.height) {
                framebuffer.set_current_color(floor_color);
                framebuffer.point(x, y);
            }
        }
        return;
    }

    // 'g' o 'G' usa door_texture, el resto usa wall_texture
    let tex = if cell == 'g' || cell == 'G' {
        door_texture
    } else {
        wall_texture
    };

    // Paredes y puerta del mapa en 2D renderizadas con la textura PNG real
    for x in 0..size {
        let screen_x = xo + x;
        if screen_x >= framebuffer.width {
            continue;
        }

        let tx = ((x as f32 / size as f32) * tex.width as f32) as u32;

        for y in 0..size {
            let screen_y = yo + y;
            if screen_y >= framebuffer.height {
                continue;
            }

            let ty = ((y as f32 / size as f32) * tex.height as f32) as u32;
            let color = tex.get_pixel(tx, ty);

            framebuffer.set_current_color(color);
            framebuffer.point(screen_x, screen_y);
        }
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, enemies: &[Enemy]) {
    let cols = maze[0].len();
    let rows = maze.len();

    // Tamaño de celda en el minimapa
    let cell_size = 8;      // 8 pixeles por bloque
    let map_w = cols * cell_size;
    let map_h = rows * cell_size;

    let margin = 15;
    let offset_x = framebuffer.width.saturating_sub(map_w + margin);
    let offset_y = margin;

    // Fondo oscuro con borde para el minimapa
    framebuffer.draw_rect(
        offset_x.saturating_sub(3),
        offset_y.saturating_sub(3),
        map_w + 6,
        map_h + 6,
        0x11111E,
    );

    // se dibujan las paredes y meta
    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            if cell != ' ' {
                let color = match cell {
                    'g' | 'G' => 0x00FF88,
                    _ => 0x666688,
                };
                framebuffer.draw_rect(
                    offset_x + col * cell_size,
                    offset_y + row * cell_size,
                    cell_size,
                    cell_size,
                    color,
                );
            }
        }
    }

    let scale = cell_size as f32 / BLOCK_SIZE as f32;

    // Dibujar enemigos vivos en el minimapa como puntos rojos
    for enemy in enemies {
        if enemy.is_alive {
            let ex = offset_x + (enemy.pos.x * scale) as usize;
            let ey = offset_y + (enemy.pos.y * scale) as usize;
            framebuffer.draw_rect(ex.saturating_sub(2), ey.saturating_sub(2), 5, 5, 0xFF2222);
        }
    }

    // se dibuja el jugador en el minimapa
    let px = offset_x + (player.pos.x * scale) as usize;
    let py = offset_y + (player.pos.y * scale) as usize;

    framebuffer.draw_rect(px.saturating_sub(2), py.saturating_sub(2), 5, 5, 0xFFFF00);

    // Indicador de dirección de mirada del jugador
    let view_len = 10.0;
    let vx = (px as f32 + view_len * player.a.cos()) as usize;
    let vy = (py as f32 + view_len * player.a.sin()) as usize;
    framebuffer.point(vx, vy);
}

fn render(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    mode_3d: bool,
    wall_texture: &Texture,
    door_texture: &Texture,
    enemies: &[Enemy],
    enemy_texture: Option<&Texture>,
) {
    if mode_3d {
        let width = framebuffer.width as f32;
        // Distancia exacta al plano de proyección para un campo de visión plano (sin deformaciones)
        let d_plane = (width / 2.0) / (FOV / 2.0).tan();
        let mut z_buffer = vec![10000.0f32; NUM_RAYS];

        // Vista 3D mediante Raycasting (1 rayo por cada columna de pantalla)
        for i in 0..NUM_RAYS {
            let screen_x = i as f32 - width / 2.0;
            let beta = (screen_x / d_plane).atan();
            let angle = player.a + beta;

            let dist = cast_ray(
                framebuffer,
                maze,
                player,
                angle,
                beta,
                BLOCK_SIZE,
                i,
                d_plane,
                wall_texture,
                door_texture,
            );
            z_buffer[i] = dist;
        }

        // Renderizado de enemigos 3D con Billboarding y Z-Buffer
        render_enemies(framebuffer, enemies, player, d_plane, &z_buffer, enemy_texture);

        // Minimapa en la esquina superior derecha
        render_minimap(framebuffer, maze, player, enemies);
    } else {
        // Vista 2D escalada automáticamente para encajar todo el laberinto en pantalla
        let cols = maze[0].len();
        let rows = maze.len();
        let cell_size_2d = (framebuffer.width / cols).min(framebuffer.height / rows);
        let scale_2d = cell_size_2d as f32 / BLOCK_SIZE as f32;

        for (row, line) in maze.iter().enumerate() {
            for (col, &cell) in line.iter().enumerate() {
                draw_cell_2d(
                    framebuffer,
                    col * cell_size_2d,
                    row * cell_size_2d,
                    cell,
                    cell_size_2d,
                    wall_texture,
                    door_texture,
                );
            }
        }

        // Dibujar enemigos en la vista 2D utilizando la textura real PNG (enemies.png)
        for enemy in enemies {
            if enemy.is_alive {
                let ex = (enemy.pos.x * scale_2d) as usize;
                let ey = (enemy.pos.y * scale_2d) as usize;

                let sprite_size_2d = (cell_size_2d as f32 * 0.75).max(10.0) as usize;
                let start_x = ex.saturating_sub(sprite_size_2d / 2);
                let start_y = ey.saturating_sub(sprite_size_2d / 2);

                if let Some(tex) = enemy_texture {
                    for x in 0..sprite_size_2d {
                        let sx = start_x + x;
                        if sx >= framebuffer.width { continue; }
                        let tx = ((x as f32 / sprite_size_2d as f32) * tex.width as f32) as u32;

                        for y in 0..sprite_size_2d {
                            let sy = start_y + y;
                            if sy >= framebuffer.height { continue; }
                            let ty = ((y as f32 / sprite_size_2d as f32) * tex.height as f32) as u32;

                            if tex.is_pixel_visible(tx, ty) {
                                let color = tex.get_pixel(tx, ty);
                                framebuffer.set_current_color(color);
                                framebuffer.point(sx, sy);
                            }
                        }
                    }
                } else {
                    framebuffer.draw_rect(ex.saturating_sub(4), ey.saturating_sub(4), 9, 9, 0xFF2222);
                }
            }
        }


        // Jugador en vista 2D
        framebuffer.set_current_color(0xFFFF00);
        let px = (player.pos.x * scale_2d) as usize;
        let py = (player.pos.y * scale_2d) as usize;

        for x in px.saturating_sub(3)..=px + 3 {
            for y in py.saturating_sub(3)..=py + 3 {
                framebuffer.point(x, y);
            }
        }

        // Abanico de rayos en vista 2D
        for i in 0..5 {
            let ray_fraction = i as f32 / 4.0;
            let angle = player.a - FOV / 2.0 + FOV * ray_fraction;
            cast_ray_2d(framebuffer, maze, player, angle, BLOCK_SIZE, scale_2d);
        }
    }
}


#[derive(PartialEq)]
enum GameState {
    WelcomeMenu,
    Playing,
    Victory,
    GameOver,
}

fn render_welcome_menu(framebuffer: &mut Framebuffer) {
    // Fondo de noche limpia sobria
    framebuffer.clear_full_screen(0x0A0B10);

    // Título Principal estilo
    framebuffer.draw_text_centered_shadow(50, "BACKROOMS ESCAPE", 6, 0xFFFFFF, 0x555555);
    framebuffer.draw_text_centered_shadow(145, "Pedro Caso 241286", 2, 0xCCCCCC, 0x333333);


    // Tarjeta central metálica sobria
    let card_w = 660;
    let card_h = 310;
    let card_x = (framebuffer.width - card_w) / 2;
    let card_y = 205;

    framebuffer.draw_rect(card_x - 3, card_y - 3, card_w + 6, card_h + 6, 0x555566);
    framebuffer.draw_rect(card_x, card_y, card_w, card_h, 0x141620);

    framebuffer.draw_text_centered(230, "SELECCIONA UN NIVEL", 3, 0xFFD700);

    // Opciones del menú de color Amarillo brillante
    framebuffer.draw_text_centered(295, "[ 1 ] - LABERINTO I", 3, 0xFFD700);
    framebuffer.draw_text_centered(360, "[ 2 ] - LABERINTO II", 3, 0xFFD700);
    framebuffer.draw_text_centered(425, "[ 3 ] - LABERINTO III", 3, 0xFFD700);

    // Instrucción de inicio en amarillo
    framebuffer.draw_text_centered_shadow(565, "PRESIONA [1] [2] o [3] PARA INICIAR LA PARTIDA", 2, 0xFFD700, 0x333300);
}

fn render_crosshair(framebuffer: &mut Framebuffer) {
    let center_x = framebuffer.width / 2;
    let center_y = framebuffer.height / 2;

    let arm_length = 8;
    let gap = 4;

    // Borde oscuro para alto contraste sobre cualquier fondo
    for i in (gap - 1)..=(arm_length + gap) {
        framebuffer.draw_rect(center_x.saturating_sub(i + 1), center_y - 1, 3, 3, 0x000000);
        framebuffer.draw_rect(center_x + i - 1, center_y - 1, 3, 3, 0x000000);
        framebuffer.draw_rect(center_x - 1, center_y.saturating_sub(i + 1), 3, 3, 0x000000);
        framebuffer.draw_rect(center_x - 1, center_y + i - 1, 3, 3, 0x000000);
    }
    framebuffer.draw_rect(center_x - 2, center_y - 2, 5, 5, 0x000000);

    // Color de la retícula
    for i in gap..=(arm_length + gap) {
        framebuffer.draw_rect(center_x.saturating_sub(i), center_y, 1, 1, 0xFFFFFF);
        framebuffer.draw_rect(center_x + i, center_y, 1, 1, 0xFFFFFF);
        framebuffer.draw_rect(center_x, center_y.saturating_sub(i), 1, 1, 0xFFFFFF);
        framebuffer.draw_rect(center_x, center_y + i, 1, 1, 0xFFFFFF);
    }

    // Punto central
    framebuffer.draw_rect(center_x - 1, center_y - 1, 3, 3, 0x00FF88);
}

fn render_victory_screen(framebuffer: &mut Framebuffer) {
    // Fondo de noche limpia
    framebuffer.clear_full_screen(0x09140E);

    // Título 
    framebuffer.draw_text_centered_shadow(70, "PASASTE EL NIVEL", 6, 0xFFFFFF, 0x555555);
    framebuffer.draw_text_centered_shadow(155, "META ALCANZADA CON EXITO", 3, 0xFFD700, 0x333300);

    // Tarjeta central sobria
    let card_w = 700;
    let card_h = 240;
    let card_x = (framebuffer.width - card_w) / 2;
    let card_y = 230;

    framebuffer.draw_rect(card_x - 3, card_y - 3, card_w + 6, card_h + 6, 0x555566);
    framebuffer.draw_rect(card_x, card_y, card_w, card_h, 0x141620);

    // Opciones e información en color amarillo
    framebuffer.draw_text_centered(280, "SOBREVIVISTE AL LABERINTO", 3, 0xFFD700);
    framebuffer.draw_text_centered(350, "TODOS LOS ENEMIGOS FUERON EVADIDOS", 2, 0xFFD700);

    // Instrucción en amarillo
    framebuffer.draw_text_centered_shadow(560, "PRESIONA ENTER O ESPACIO PARA VOLVER AL MENU", 2, 0xFFD700, 0x333300);
}

fn render_game_over_screen(framebuffer: &mut Framebuffer) {
    // Fondo de noche limpia
    framebuffer.clear_full_screen(0x140909);

    // Título 
    framebuffer.draw_text_centered_shadow(70, "¡HAS MUERTO!", 6, 0xFFFFFF, 0x555555);

    framebuffer.draw_text_centered_shadow(155, "TE QUEDASTE SIN SALUD", 3, 0xFFD700, 0x333300);

    // Tarjeta central sobria
    let card_w = 700;
    let card_h = 240;
    let card_x = (framebuffer.width - card_w) / 2;
    let card_y = 230;

    framebuffer.draw_rect(card_x - 3, card_y - 3, card_w + 6, card_h + 6, 0x555566);
    framebuffer.draw_rect(card_x, card_y, card_w, card_h, 0x141620);

    // Opciones e información en color amarillo
    framebuffer.draw_text_centered(280, "LOS ENEMIGOS TE HAN DERROTADO", 3, 0xFFD700);
    framebuffer.draw_text_centered(350, "INTENTALO DE NUEVO", 2, 0xFFD700);

    // Instrucción en amarillo
    framebuffer.draw_text_centered_shadow(560, "PRESIONA ENTER O ESPACIO PARA REINTENTAR", 2, 0xFFD700, 0x333300);
}



fn main() {
    let window_width = 1200;
    let window_height = 680;
    let framebuffer_width = 1200;
    let framebuffer_height = 680;
    let target_frame_duration = Duration::from_micros(22222); // para 45 fps estables

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new(
        "Maze Runner 3D",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();


    let wall_texture = Texture::load("./assets/wall.png").unwrap_or_else(|e| {
        eprintln!("Aviso: {}", e);
        Texture {
            width: 1,
            height: 1,
            pixels: vec![0xFF5555],
            alpha: vec![255],
        }
    });

    let door_texture = Texture::load("./assets/door.png").unwrap_or_else(|e| {
        eprintln!("Aviso: {}", e);
        Texture {
            width: 1,
            height: 1,
            pixels: vec![0x00FF88],
            alpha: vec![255],
        }
    });


    let enemy_texture = Texture::load("./assets/enemies.png").ok();



    let mut weapon = Weapon::new();

    let mut state = GameState::WelcomeMenu;
    let mut mode_3d = true;
    let mut last_fps_update = Instant::now();
    let mut frame_count: u32 = 0;
    let mut displayed_fps: f32 = 0.0;
    let mut last_mouse_x: Option<f32> = None;

    let mut maze: Maze = Vec::new();
    let mut player = Player::new(0.0, 0.0);
    let mut enemies: Vec<Enemy> = Vec::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = Instant::now();

        // Cálculo y estabilización de FPS
        frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(last_fps_update).as_secs_f32();

        if elapsed >= 0.8 {
            displayed_fps = frame_count as f32 / elapsed;
            frame_count = 0;
            last_fps_update = now;
        }

        match state {
            GameState::WelcomeMenu => {
                if window.is_key_pressed(Key::Key1, KeyRepeat::No) || window.is_key_pressed(Key::NumPad1, KeyRepeat::No) {
                    let loaded = load_maze("./maze.txt", BLOCK_SIZE);
                    maze = loaded.0;
                    player = Player::new(loaded.1.pos.x, loaded.1.pos.y);
                    enemies = spawn_enemies_from_maze(&mut maze, BLOCK_SIZE);
                    state = GameState::Playing;
                } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) || window.is_key_pressed(Key::NumPad2, KeyRepeat::No) {
                    let loaded = load_maze("./maze2.txt", BLOCK_SIZE);
                    maze = loaded.0;
                    player = Player::new(loaded.1.pos.x, loaded.1.pos.y);
                    enemies = spawn_enemies_from_maze(&mut maze, BLOCK_SIZE);
                    state = GameState::Playing;
                } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) || window.is_key_pressed(Key::NumPad3, KeyRepeat::No) {
                    let loaded = load_maze("./maze3.txt", BLOCK_SIZE);
                    maze = loaded.0;
                    player = Player::new(loaded.1.pos.x, loaded.1.pos.y);
                    enemies = spawn_enemies_from_maze(&mut maze, BLOCK_SIZE);
                    state = GameState::Playing;
                }

                render_welcome_menu(&mut framebuffer);
            }
            GameState::Playing => {
                let dt = 0.022; // ~45 FPS

                // Actualizar enemigos
                for enemy in enemies.iter_mut() {
                    enemy.update(&mut player, dt, &maze, BLOCK_SIZE);
                }

                // Alternar entre modo 2D y modo 3D con la tecla M
                if window.is_key_pressed(Key::M, KeyRepeat::No) {
                    mode_3d = !mode_3d;
                }

                // Disparar el arma con clic izquierdo e infligir daño a enemigos solo al iniciar cada disparo
                if window.get_mouse_down(MouseButton::Left) {
                    if weapon.shoot("./assets/gun_shot.mp3") {
                        check_player_shot_hit(&player, &mut enemies, &maze, BLOCK_SIZE);
                    }
                }


                weapon.update();

                process_events(&mut window, &mut player, &maze, &enemies, BLOCK_SIZE, &mut last_mouse_x);


                // Si el jugador pierde toda la vida es game over
                if player.hp <= 0 {
                    state = GameState::GameOver;
                }

                // Verificar si se llegó a la meta
                let i = player.pos.x as usize / BLOCK_SIZE;
                let j = player.pos.y as usize / BLOCK_SIZE;
                if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
                    state = GameState::Victory;
                }

                if mode_3d {
                    framebuffer.clear_sky_and_floor(0x0B0F26, 0x1B3B22);
                } else {
                    framebuffer.clear();
                }

                render(
                    &mut framebuffer,
                    &maze,
                    &player,
                    mode_3d,
                    &wall_texture,
                    &door_texture,
                    &enemies,
                    enemy_texture.as_ref(),
                );

                // Dibujar el arma y la retícula de mira en 3D
                if mode_3d {
                    weapon.draw(&mut framebuffer);
                    render_crosshair(&mut framebuffer);
                }

                // HUD de Salud del Jugador en la esquina inferior derecha
                let hp_ratio = player.hp as f32 / player.max_hp as f32;
                let hp_bar_w = 220;
                let fill_w = (hp_bar_w as f32 * hp_ratio) as usize;
                let hp_color = if hp_ratio > 0.5 { 0x00FF88 } else if hp_ratio > 0.25 { 0xFFFF00 } else { 0xFF2222 };

                let hud_x = framebuffer.width.saturating_sub(hp_bar_w + 20);
                let hud_y = framebuffer.height.saturating_sub(44);

                framebuffer.draw_rect(hud_x, hud_y, hp_bar_w, 24, 0x221111);
                framebuffer.draw_rect(hud_x, hud_y, fill_w, 24, hp_color);
                let hp_str = format!("SALUD: {}%", player.hp);
                framebuffer.draw_text(hud_x + 10, hud_y + 5, &hp_str, 0xFFFFFF);


                // Mostrar recuadro e información de FPS directamente sobre el juego
                framebuffer.draw_rect(10, 10, 130, 24, 0x11111E);
                let fps_str = format!("FPS: {:.1}", displayed_fps);
                framebuffer.draw_text(15, 15, &fps_str, 0x00FF88);
            }
            GameState::Victory => {
                if window.is_key_pressed(Key::Enter, KeyRepeat::No) || window.is_key_pressed(Key::Space, KeyRepeat::No) {
                    state = GameState::WelcomeMenu;
                }

                render_victory_screen(&mut framebuffer);
            }
            GameState::GameOver => {
                if window.is_key_pressed(Key::Enter, KeyRepeat::No) || window.is_key_pressed(Key::Space, KeyRepeat::No) {
                    state = GameState::WelcomeMenu;
                }

                render_game_over_screen(&mut framebuffer);
            }
        }


        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        // Pausa dinámica para FPS estables
        let work_duration = frame_start.elapsed();
        if work_duration < target_frame_duration {
            std::thread::sleep(target_frame_duration - work_duration);
        }
    }
}



