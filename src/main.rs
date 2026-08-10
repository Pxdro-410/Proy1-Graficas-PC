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

/// Amplitud del campo de visión (field of view), en radianes.
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

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, cell: char) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(cell_color(cell));

    for x in xo..xo + BLOCK_SIZE {
        for y in yo..yo + BLOCK_SIZE {
            framebuffer.point(x, y);
        }
    }
}

fn render(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, mode_3d: bool) {
    if mode_3d {
        // Vista 3D mediante Raycasting (1 rayo por cada columna de pantalla)
        for i in 0..NUM_RAYS {
            let ray_fraction = i as f32 / (NUM_RAYS - 1) as f32; // de 0.0 a 1.0
            let angle = player.a - FOV / 2.0 + FOV * ray_fraction;
            cast_ray(framebuffer, maze, player, angle, BLOCK_SIZE, i);
        }
    } else {
        // Vista 2D original (laberinto + jugador + 5 rayos)
        for (row, line) in maze.iter().enumerate() {
            for (col, &cell) in line.iter().enumerate() {
                draw_cell(framebuffer, col * BLOCK_SIZE, row * BLOCK_SIZE, cell);
            }
        }

        framebuffer.set_current_color(0xFFFF00);
        let px = player.pos.x as usize;
        let py = player.pos.y as usize;

        for x in px.saturating_sub(3)..=px + 3 {
            for y in py.saturating_sub(3)..=py + 3 {
                framebuffer.point(x, y);
            }
        }

        for i in 0..5 {
            let ray_fraction = i as f32 / 4.0;
            let angle = player.a - FOV / 2.0 + FOV * ray_fraction;
            cast_ray_2d(framebuffer, maze, player, angle, BLOCK_SIZE);
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

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Cálculo y estabilización de FPS (se promedia y actualiza cada 0.5 segundos)
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

        process_events(&window, &mut player, &maze, BLOCK_SIZE);

        // ¿el jugador llegó a la meta?
        let i = player.pos.x as usize / BLOCK_SIZE;
        let j = player.pos.y as usize / BLOCK_SIZE;
        if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

        framebuffer.clear();

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

