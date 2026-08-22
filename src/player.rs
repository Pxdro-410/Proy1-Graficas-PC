use minifb::{Key, MouseMode, Window};

use nalgebra_glm::Vec2;
use std::f32::consts::PI;

use crate::enemy::Enemy;
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
fn process_mouse_rotation(window: &Window, player: &mut Player, _last_mouse_x: &mut Option<f32>) {
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

#[cfg(target_os = "macos")]
mod macos_cg {
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CGPoint {
        pub x: f64,
        pub y: f64,
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        pub fn CGWarpMouseCursorPosition(new_cursor_position: CGPoint) -> i32;
    }
}

#[cfg(target_os = "macos")]
fn process_mouse_rotation(window: &Window, player: &mut Player, last_mouse_x: &mut Option<f32>) {
    const MOUSE_SENSITIVITY: f32 = 0.00055;
    let (win_w, win_h) = window.get_size();
    let win_center_x = win_w as f32 / 2.0;
    let win_center_y = win_h as f32 / 2.0;

    if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Clamp) {
        if let Some(prev_x) = *last_mouse_x {
            let delta_x = mouse_x - prev_x;
            player.a += delta_x * MOUSE_SENSITIVITY;
        } else {
            let delta_x = mouse_x - win_center_x;
            player.a += delta_x * MOUSE_SENSITIVITY;
        }

        // Re-centrar el cursor nativo de macOS hacia el centro de la ventana si se desplaza
        let distance_from_center = (mouse_x - win_center_x).abs();
        if distance_from_center > 30.0 {
            let center_pt = macos_cg::CGPoint {
                x: win_center_x as f64,
                y: win_center_y as f64,
            };
            unsafe {
                macos_cg::CGWarpMouseCursorPosition(center_pt);
            }
            *last_mouse_x = Some(win_center_x);
        } else {
            *last_mouse_x = Some(mouse_x);
        }
    } else {
        *last_mouse_x = None;
    }
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
mod linux_x11 {
    use std::ffi::c_void;

    pub type Display = c_void;
    pub type Window = u64;

    #[link(name = "X11")]
    unsafe extern "C" {
        pub fn XOpenDisplay(display_name: *const i8) -> *mut Display;
        pub fn XCloseDisplay(display: *mut Display) -> i32;
        pub fn XDefaultRootWindow(display: *mut Display) -> Window;
        pub fn XWarpPointer(
            display: *mut Display,
            src_w: Window,
            dest_w: Window,
            src_x: i32,
            src_y: i32,
            src_width: u32,
            src_height: u32,
            dest_x: i32,
            dest_y: i32,
        ) -> i32;
        pub fn XFlush(display: *mut Display) -> i32;
    }
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn process_mouse_rotation(window: &Window, player: &mut Player, last_mouse_x: &mut Option<f32>) {
    const MOUSE_SENSITIVITY: f32 = 0.00055;
    let (win_w, win_h) = window.get_size();
    let win_center_x = win_w as f32 / 2.0;
    let win_center_y = win_h as f32 / 2.0;

    if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Clamp) {
        let mut delta_x = match *last_mouse_x {
            Some(prev_x) => mouse_x - prev_x,
            None => mouse_x - win_center_x,
        };

        // Si el cursor llega a los bordes de la ventana en Linux, mantenemos rotación continua
        if mouse_x >= win_w as f32 - 2.0 && delta_x >= 0.0 {
            delta_x = 15.0;
        } else if mouse_x <= 2.0 && delta_x <= 0.0 {
            delta_x = -15.0;
        }

        player.a += delta_x * MOUSE_SENSITIVITY;

        // Re-centrado físico usando X11 (XWarpPointer)
        let distance_from_center = (mouse_x - win_center_x).abs();
        if distance_from_center > 20.0 {
            unsafe {
                let dpy = linux_x11::XOpenDisplay(std::ptr::null());
                if !dpy.is_null() {
                    let root = linux_x11::XDefaultRootWindow(dpy);
                    linux_x11::XWarpPointer(
                        dpy,
                        0,
                        root,
                        0,
                        0,
                        0,
                        0,
                        win_center_x as i32,
                        win_center_y as i32,
                    );
                    linux_x11::XFlush(dpy);
                    linux_x11::XCloseDisplay(dpy);
                }
            }
            *last_mouse_x = Some(win_center_x);
        } else {
            *last_mouse_x = Some(mouse_x);
        }
    } else {
        *last_mouse_x = None;
    }
}

pub fn process_events(
    window: &mut Window,
    player: &mut Player,
    maze: &Maze,
    enemies: &[Enemy],
    block_size: usize,
    last_mouse_x: &mut Option<f32>,
) {
    const MOVE_SPEED: f32 = 7.0;
    const PLAYER_RADIUS: f32 = 15.0;
    const ENEMY_COLLISION_RADIUS: f32 = 38.0;

    process_mouse_rotation(window, player, last_mouse_x);



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

    // Verificar colisión con paredes y enemigos en el eje X
    if dx != 0.0 {
        let new_x = player.pos.x + dx;
        let mut can_move_x = true;

        // Colisión contra enemigos vivos
        for enemy in enemies {
            if enemy.is_alive {
                let edx = new_x - enemy.pos.x;
                let edy = player.pos.y - enemy.pos.y;
                if (edx * edx + edy * edy) < (PLAYER_RADIUS + ENEMY_COLLISION_RADIUS).powi(2) {
                    can_move_x = false;
                    break;
                }
            }
        }

        if can_move_x {
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
    }

    // Verificar colisión con paredes y enemigos en el eje Y
    if dy != 0.0 {
        let new_y = player.pos.y + dy;
        let mut can_move_y = true;

        // Colisión contra enemigos vivos
        for enemy in enemies {
            if enemy.is_alive {
                let edx = player.pos.x - enemy.pos.x;
                let edy = new_y - enemy.pos.y;
                if (edx * edx + edy * edy) < (PLAYER_RADIUS + ENEMY_COLLISION_RADIUS).powi(2) {
                    can_move_y = false;
                    break;
                }
            }
        }

        if can_move_y {
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
}
