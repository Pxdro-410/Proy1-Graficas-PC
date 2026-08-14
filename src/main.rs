mod caster;
mod framebuffer;
mod maze;
mod player;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

use crate::caster::{cast_ray, cast_ray_2d};
use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, Maze};
use crate::player::{process_events, Player};

const BLOCK_SIZE: usize = 100;

/// Cantidad de rayos, la misma cantidad de columnas que tiene la pantalla.
const NUM_RAYS: usize = 1300;

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

fn draw_cell_2d(framebuffer: &mut Framebuffer, xo: usize, yo: usize, cell: char, size: usize) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(cell_color(cell));

    for x in xo..(xo + size).min(framebuffer.width) {
        for y in yo..(yo + size).min(framebuffer.height) {
            framebuffer.point(x, y);
        }
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
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

    // se dubijan las paredes y meta
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

    // se dibuja el jugador en el minimapa
    let scale = cell_size as f32 / BLOCK_SIZE as f32;
    let px = offset_x + (player.pos.x * scale) as usize;
    let py = offset_y + (player.pos.y * scale) as usize;

    framebuffer.draw_rect(px.saturating_sub(2), py.saturating_sub(2), 5, 5, 0xFFFF00);

    // Indicador de dirección de mirada del jugador
    let view_len = 10.0;
    let vx = (px as f32 + view_len * player.a.cos()) as usize;
    let vy = (py as f32 + view_len * player.a.sin()) as usize;
    framebuffer.point(vx, vy);
}

fn render(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, mode_3d: bool) {
    if mode_3d {
        let width = framebuffer.width as f32;
        // Distancia exacta al plano de proyección para un campo de visión plano (sin deformaciones)
        let d_plane = (width / 2.0) / (FOV / 2.0).tan();

        // Vista 3D mediante Raycasting (1 rayo por cada columna de pantalla)
        for i in 0..NUM_RAYS {
            let screen_x = i as f32 - width / 2.0;
            let beta = (screen_x / d_plane).atan();
            let angle = player.a + beta;

            cast_ray(framebuffer, maze, player, angle, beta, BLOCK_SIZE, i, d_plane);
        }

        // Minimapa en la esquina superior derecha
        render_minimap(framebuffer, maze, player);
    } else {

        // Vista 2D escalada automáticamente para encajar todo el laberinto en pantalla
        let cols = maze[0].len();
        let rows = maze.len();
        let cell_size_2d = (framebuffer.width / cols).min(framebuffer.height / rows);
        let scale_2d = cell_size_2d as f32 / BLOCK_SIZE as f32;

        for (row, line) in maze.iter().enumerate() {
            for (col, &cell) in line.iter().enumerate() {
                draw_cell_2d(framebuffer, col * cell_size_2d, row * cell_size_2d, cell, cell_size_2d);
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



fn main() {
    let window_width = 1300;
    let window_height = 900;
    let framebuffer_width = 1300;
    let framebuffer_height = 900;
    let frame_delay = Duration::from_millis(16);

    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new(
        "Maze Runner (Presiona 'M' para alternar 2D / 3D)",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let mut mode_3d = true;
    let mut last_fps_update = Instant::now();
    let mut frame_count: u32 = 0;
    let mut displayed_fps: f32 = 0.0;
    let mut last_mouse_x: Option<f32> = None;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Cálculo y estabilización de FPS
        frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(last_fps_update).as_secs_f32();

        if elapsed >= 0.8 {
            displayed_fps = frame_count as f32 / elapsed;
            frame_count = 0;
            last_fps_update = now;
        }

        // Alternar entre modo 2D y modo 3D con la tecla M
        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            mode_3d = !mode_3d;
        }

        process_events(&mut window, &mut player, &maze, BLOCK_SIZE, &mut last_mouse_x);



        // ¿el jugador llegó a la meta?
        let i = player.pos.x as usize / BLOCK_SIZE;
        let j = player.pos.y as usize / BLOCK_SIZE;
        if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

        if mode_3d {
            framebuffer.clear_sky_and_floor(0x0B0F26, 0x1B3B22);
        } else {
            framebuffer.clear();
        }


        render(&mut framebuffer, &maze, &player, mode_3d);


        // Mostrar recuadro e información de FPS directamente sobre el juego (esquina superior izquierda)
        framebuffer.draw_rect(10, 10, 130, 24, 0x11111E);
        let fps_str = format!("FPS: {:.1}", displayed_fps);
        framebuffer.draw_text(15, 15, &fps_str, 0x00FF88);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

