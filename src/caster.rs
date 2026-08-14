use crate::cell_color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub fn cast_ray_2d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    scale: f32,
) {
    let mut d = 0.0;
    framebuffer.set_current_color(0xFFDDDD);

    loop {
        let world_x = player.pos.x + d * a.cos();
        let world_y = player.pos.y + d * a.sin();

        let col = (world_x / block_size as f32) as usize;
        let row = (world_y / block_size as f32) as usize;

        if row >= maze.len() || col >= maze[row].len() {
            return;
        }

        if maze[row][col] != ' ' {
            return;
        }

        let screen_x = (world_x * scale) as usize;
        let screen_y = (world_y * scale) as usize;

        framebuffer.point(screen_x, screen_y);
        d += 1.0;
    }
}


pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    beta: f32,
    block_size: usize,
    i: usize,
    d_plane: f32,
) {
    let mut d = 0.0;

    loop {
        let x = (player.pos.x + d * a.cos()) as usize;
        let y = (player.pos.y + d * a.sin()) as usize;

        let col = x / block_size;
        let row = y / block_size;

        if row >= maze.len() || col >= maze[row].len() {
            return;
        }

        if maze[row][col] != ' ' {
            let cell = maze[row][col];

            // Color obtenido desde la función cell_color en main.rs
            let color = cell_color(cell);

            // Corrección de ojo de pez exacta usando el coseno del ángulo relativo a la cámara
            let distance = (d * beta.cos()).max(0.1);

            // Altura de la pared en 3D
            let wall_height = (block_size as f32 / distance) * d_plane;

            let h = framebuffer.height as f32;
            let draw_start = ((h / 2.0) - (wall_height / 2.0)).max(0.0) as usize;
            let draw_end = ((h / 2.0) + (wall_height / 2.0)).min(h) as usize;

            // Dibujar la tira vertical de la pared
            framebuffer.set_current_color(color);
            for y_pix in draw_start..draw_end {
                framebuffer.point(i, y_pix);
            }
            return;
        }

        d += 1.0;
    }
}


