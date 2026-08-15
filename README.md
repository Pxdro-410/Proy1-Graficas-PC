# Backrooms Escape - Motor de Raycasting 2D simulado en 3D en Rust

Proyecto 1 - Graficas por computadora
Estudiante: Pedro Caso - 241286  

---

## Descripción del Proyecto

Backrooms Escape es un motor gráfico 2D simulado en3D desarrollado desde cero en Rust utilizando técnicas de Raycasting en modo software rendering. El proyecto simula una perspectiva tridimensional en primera persona inspirada en títulos clásicos como Wolfenstein 3D y Doom, incorporando texturizado de paredes y puertas, animaciones de armas, efectos de sonido, música de fondo en bucle, navegación por mapa 2D/minimapa y un sistema de inteligencia y combate con enemigos.

---

## Características Principales

### Motor 3D y Renderizado
* Renderizado 3D por Raycasting en modo software sobre framebuffer.
* Proyección plana de paredes con corrección matemática del efecto ojo de pez (Fisheye Correction).
* Control de oclusión y profundidad mediante Z-Buffer para ordenar el dibujado de elementos.
* Texturizado de paredes mediante imágenes PNG (`wall.png`) y textura única para la meta (`door.png`).
* Fondos de cielo estelar completo en pantallas de menú y ambiente nocturno en gameplay.

### Mecánicas de Juego y Controles
* Movimiento en primera persona con teclas WASD (avance, retroceso y desplazamientos laterales).
* Rotación libre de cámara de 360 grados mediante el movimiento del ratón con bloqueo de puntero dentro de la ventana.
* Retícula de mira (Crosshair) fija en el centro de la pantalla para precisión de disparo.
* Alternancia fluida entre la vista 3D y la vista de mapa 2D presionando la tecla M.
* Minimapa de navegación en la esquina superior derecha de la pantalla.

### Sistema de Combate y Enemigos
* Generación de enemigos ubicados estratégicamente desde los archivos de mapa mediante el carácter 'e'.
* Renderizado de enemigos en 3D mediante Billboarding (sprites orientados hacia la cámara) y textura PNG (`enemies.png`).
* Enemigos con verificación de línea de visión directa (Line-of-Sight Raycasting) para evitar ataques a través de paredes sólidas.
* Colisión física entre el jugador y los enemigos para evitar traspasos de cuerpos.
* Disparos tipo Hitscan con animación de fogonazo, retroceso del arma y reproducción de efectos de sonido.
* Balance de combate:
  - Vida del Jugador: 100 HP (Barra de salud visible en la esquina inferior derecha).
  - Daño por ataque enemigo: 20% de salud (cooldown de 2.0 segundos entre ataques).
  - Daño por disparo del jugador: 25% de salud del enemigo (requiere exactamente 4 disparos certeros para eliminar a cada enemigo).
* Barras de salud flotantes sobre cada enemigo.

### Interfaz de Usuario y Menús
* Menú de bienvenida y selección de niveles (Laberinto I, II y III).
* Pantallas de Victoria (Meta alcanzada) y Game Over (Muerte del jugador).
* Tipografía retro con centrado horizontal pixel-perfect y sombras proyectadas en relieve.

---

## Controles de Juego

| Tecla / Acción | Función |
| --- | --- |
| W | Avanzar |
| S | Retroceder |
| A | Desplazamiento lateral izquierdo (Strafe Left) |
| D | Desplazamiento lateral derecho (Strafe Right) |
| Ratón | Girar cámara (Rotación de 360 grados) |
| Clic Izquierdo | Disparar el arma |
| M | Alternar entre vista 3D y vista de mapa 2D |
| Teclas 1, 2, 3 | Seleccionar nivel en el menú principal |
| Enter / Espacio | Volver al menú desde las pantallas de Victoria o Game Over |
| Escape | Salir del juego |

---

## Requisitos e Instalación

### Requisitos Previos
* Rust y Cargo (Edición 2021 o superior).
* Compilador de C/C++ compatible con el sistema operativo (MSVC en Windows / GCC o Clang en Linux/macOS).

### Instrucciones de Compilación y Ejecución

1. Clonar el repositorio:
   ```bash
   git clone https://github.com/Pxdro-410/Proy1-Graficas-PC.git
   cd Proy1-Graficas-PC
   ```

2. Verificar el código:
   ```bash
   cargo check
   ```

3. Compilar y ejecutar el proyecto:
   ```bash
   cargo run
   ```

4. (Opcional) Compilar versión optimizada de producción:
   ```bash
   cargo run --release
   ```

---

## Estructura de Archivos del Proyecto

```
Proy1-Graficas-PC/
├── assets/
│   ├── door.png        # Textura de la puerta de meta
│   ├── enemies.png     # Textura del enemigo
│   ├── gun_shot.mp3    # Sonido de disparo del arma
│   ├── music.mp3       # Música de fondo en bucle
│   └── wall.png        # Textura de las paredes del laberinto
├── src/
│   ├── caster.rs       # Algoritmo de Raycasting 3D y 2D
│   ├── enemy.rs        # Lógica de enemigos, IA, combate y billboarding
│   ├── framebuffer.rs  # Gestión del búfer de píxeles, dibujo y renderizado de texto
│   ├── main.rs         # Bucle principal del juego, menús y estados
│   ├── maze.rs         # Carga y estructuración de niveles desde archivos de texto
│   ├── player.rs       # Estado del jugador, movimiento y colisiones
│   ├── texture.rs      # Decodificación y consulta de píxeles de texturas PNG
│   └── weapon.rs       # Sistema de armas, animaciones y audio con Rodio
├── maze.txt            # Laberinto Nivel 1
├── maze2.txt           # Laberinto Nivel 2
├── maze3.txt           # Laberinto Nivel 3
└── Cargo.toml          # Configuración de dependencias de Rust
```

---

## Dependencias Principales

* `minifb`: Ventanado y gestión del framebuffer por software.
* `nalgebra-glm`: Operaciones matemáticas vectoriales.
* `image`: Decodificación y procesamiento de imágenes PNG.
* `rodio`: Reproducción de audio y música de fondo.
* `winapi`: Control de eventos de puntero y bloqueo de ratón en Windows.
