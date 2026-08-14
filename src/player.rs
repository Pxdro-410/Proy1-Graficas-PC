use minifb::{Key, MouseMode, Window};

use nalgebra_glm::Vec2;
use std::f32::consts::PI;

use crate::maze::Maze;

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub hp: i32,
    pub max_hp: i32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            pos: Vec2::new(x, y),
            a: 0.0,
            hp: 100,
            max_hp: 100,
        }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.hp = (self.hp - damage).max(0);
    }
}


#[cfg(target_os = "windows")]
fn process_mouse_rotation(window: &Window, player: &mut Player) {
    use winapi::shared::windef::RECT;
    use winapi::um::winuser::{GetForegroundWindow, GetWindowRect, SetCursorPos, ShowCursor};


    unsafe {
        ShowCursor(0); // Mantener el cursor oculto

        let hwnd = GetForegroundWindow();
        if !hwnd.is_null() {
            let mut rect: RECT = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect) != 0 {
                let center_x = (rect.left + rect.right) / 2;
                let center_y = (rect.top + rect.bottom) / 2;

                if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Pass) {
                    let win_width = window.get_size().0 as f32;
                    let win_center_x = win_width / 2.0;

                    let delta_x = mouse_x - win_center_x;
                    const MOUSE_SENSITIVITY: f32 = 0.00055;
                    player.a += delta_x * MOUSE_SENSITIVITY;


                    // Re-centrar el ratón en el medio exacto de la ventana para bloquearlo dentro
                    SetCursorPos(center_x, center_y);
                }
            }
        }
    }
}

pub fn process_events(
    window: &mut Window,
    player: &mut Player,
    maze: &Maze,
    block_size: usize,
    _last_mouse_x: &mut Option<f32>,

) {
    const MOVE_SPEED: f32 = 7.0;
    const PLAYER_RADIUS: f32 = 15.0;



    #[cfg(target_os = "windows")]
    process_mouse_rotation(window, player);

    #[cfg(not(target_os = "windows"))]
    if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Pass) {
        if let Some(prev_x) = *last_mouse_x {
            let delta_x = mouse_x - prev_x;
            player.a += delta_x * MOUSE_SENSITIVITY;
        }
        *last_mouse_x = Some(mouse_x);
    } else {
        *last_mouse_x = None;
    }


    let mut dx = 0.0;
    let mut dy = 0.0;

    // Avanzar (W)
    if window.is_key_down(Key::W) {
        dx += MOVE_SPEED * player.a.cos();
        dy += MOVE_SPEED * player.a.sin();
    }

    // Retroceder (S)
    if window.is_key_down(Key::S) {
        dx -= MOVE_SPEED * player.a.cos();
        dy -= MOVE_SPEED * player.a.sin();
    }

    // Moverse a la izquierda / Strafe Left (A)
    if window.is_key_down(Key::A) {
        let left_angle = player.a - PI / 2.0;
        dx += MOVE_SPEED * left_angle.cos();
        dy += MOVE_SPEED * left_angle.sin();
    }

    // Moverse a la derecha / Strafe Right (D)
    if window.is_key_down(Key::D) {
        let right_angle = player.a + PI / 2.0;
        dx += MOVE_SPEED * right_angle.cos();
        dy += MOVE_SPEED * right_angle.sin();
    }


    // Verificar colisión en el eje X
    if dx != 0.0 {
        let new_x = player.pos.x + dx;
        let test_x = if dx > 0.0 { new_x + PLAYER_RADIUS } else { new_x - PLAYER_RADIUS };
        let col = (test_x / block_size as f32) as usize;
        let row = (player.pos.y / block_size as f32) as usize;

        if row < maze.len() && col < maze[row].len() {
            let cell = maze[row][col];
            if cell == ' ' || cell == 'g' || cell == 'G' {
                player.pos.x = new_x;
            }
        }
    }

    // Verificar colisión en el eje Y
    if dy != 0.0 {
        let new_y = player.pos.y + dy;
        let test_y = if dy > 0.0 { new_y + PLAYER_RADIUS } else { new_y - PLAYER_RADIUS };
        let col = (player.pos.x / block_size as f32) as usize;
        let row = (test_y / block_size as f32) as usize;

        if row < maze.len() && col < maze[row].len() {
            let cell = maze[row][col];
            if cell == ' ' || cell == 'g' || cell == 'G' {
                player.pos.y = new_y;
            }
        }
    }
}

