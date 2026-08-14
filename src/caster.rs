use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::texture::Texture;


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
    wall_texture: &Texture,
    door_texture: &Texture,
) -> f32 {
    let mut d = 0.0;

    loop {
        let world_x = player.pos.x + d * a.cos();
        let world_y = player.pos.y + d * a.sin();

        let col = (world_x / block_size as f32) as usize;
        let row = (world_y / block_size as f32) as usize;

        if row >= maze.len() || col >= maze[row].len() {
            return 10000.0;
        }

        if maze[row][col] != ' ' {
            let cell = maze[row][col];

            // Corrección de ojo de pez exacta usando el coseno del ángulo relativo a la cámara
            let distance = (d * beta.cos()).max(0.1);

            // Altura de la pared en 3D
            let wall_height = (block_size as f32 / distance) * d_plane;

            let h = framebuffer.height as f32;
            let draw_start_f = (h / 2.0) - (wall_height / 2.0);
            let draw_start = (draw_start_f.max(0.0)) as usize;
            let draw_end = (((h / 2.0) + (wall_height / 2.0)).min(h)) as usize;

            // 'g' o 'G' usa door_texture, el resto usa wall_texture
            let tex = if cell == 'g' || cell == 'G' {
                door_texture
            } else {
                wall_texture
            };

            // Mapeo de coordenada X de textura (tx)
            let hit_x_cell = world_x % block_size as f32;
            let hit_y_cell = world_y % block_size as f32;

            let hit_offset = if hit_x_cell < 1.5 || hit_x_cell > (block_size as f32 - 1.5) {
                hit_y_cell
            } else {
                hit_x_cell
            };

            let tx = ((hit_offset / block_size as f32) * tex.width as f32) as u32;

            // Renderizado de la pared o puerta proyectando la textura PNG correspondiente
            for y_pix in draw_start..draw_end {
                let ty_norm = (y_pix as f32 - draw_start_f) / wall_height;
                let ty = (ty_norm * tex.height as f32) as u32;

                let color = tex.get_pixel(tx, ty);
                framebuffer.set_current_color(color);
                framebuffer.point(i, y_pix);
            }
            return distance;
        }

        d += 1.0;
    }
}





