mod color;
mod framebuffer;
mod line;
mod obj;
mod render;
// Ejercicio de la fase de vértices; el renderer no lo utiliza.
#[allow(dead_code)]
mod vertices;

use std::time::Instant;

use raylib::prelude::*;

use color::Color;
use framebuffer::Framebuffer;
use obj::{load_obj, Obj};
use render::{draw_obj_wireframe, Camara, Encuadre};

const ANCHO: usize = 1024;
const ALTO: usize = 768;
const MODELO: &str = "spaceship.obj";
const CAPTURA: &str = "screenshot.png";

/// Radianes que gira la cámara por cada pixel que se arrastra el mouse.
const SENSIBILIDAD_MOUSE: f32 = 0.01;
/// Radianes por segundo que gira la cámara con las flechas.
const VELOCIDAD_TECLAS: f32 = 1.5;
/// Cuánto cambia el zoom por cada paso de la rueda del mouse.
const PASO_ZOOM: f32 = 1.1;
const ZOOM_MIN: f32 = 0.2;
const ZOOM_MAX: f32 = 8.0;

const AYUDA: &str = "Arrastrar / flechas: rotar   Rueda: zoom   R: reiniciar";

/// Ajusta la cámara según el teclado y el mouse; `R` la regresa a `inicial`.
/// Devuelve `true` si cambió.
fn actualizar_camara(rl: &RaylibHandle, camara: &mut Camara, inicial: &Camara) -> bool {
    let anterior = *camara;

    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        let delta = rl.get_mouse_delta();
        camara.yaw += delta.x * SENSIBILIDAD_MOUSE;
        camara.pitch += delta.y * SENSIBILIDAD_MOUSE;
    }

    let paso = VELOCIDAD_TECLAS * rl.get_frame_time();
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        camara.yaw -= paso;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        camara.yaw += paso;
    }
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        camara.pitch -= paso;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        camara.pitch += paso;
    }

    let rueda = rl.get_mouse_wheel_move();
    if rueda != 0.0 {
        camara.zoom = (camara.zoom * PASO_ZOOM.powf(rueda)).clamp(ZOOM_MIN, ZOOM_MAX);
    }

    if rl.is_key_pressed(KeyboardKey::KEY_R) {
        *camara = *inicial;
    }

    *camara != anterior
}

fn render(fb: &mut Framebuffer, modelo: &Obj, encuadre: &Encuadre, camara: &Camara) {
    fb.clear();
    draw_obj_wireframe(fb, modelo, encuadre, camara);
}

fn main() -> std::io::Result<()> {
    let inicio = Instant::now();
    let modelo = load_obj(MODELO)?;
    println!(
        "{MODELO}: {} vértices, {} triángulos (cargado en {:.2?})",
        modelo.vertices.len(),
        modelo.triangle_count(),
        inicio.elapsed()
    );
    let encuadre = Encuadre::new(&modelo.vertices, ANCHO as f32, ALTO as f32);

    let mut fb = Framebuffer::new(ANCHO, ALTO);
    fb.set_background_color(Color::from_hex(0x0B0E1A));
    fb.set_current_color(Color::from_hex(0x6BE4FF));

    // La captura siempre es la vista de frente, ocupando toda la pantalla.
    let inicio = Instant::now();
    render(&mut fb, &modelo, &encuadre, &Camara::default());
    println!("Dibujado en {:.2?}", inicio.elapsed());

    let imagen = fb.to_image();
    imagen.export_image(CAPTURA);
    println!("Captura guardada en {CAPTURA}");

    // `--no-window` solo genera la captura, sin abrir la ventana.
    if std::env::args().any(|a| a == "--no-window") {
        return Ok(());
    }

    let (mut rl, thread) = raylib::init()
        .size(ANCHO as i32, ALTO as i32)
        .title("Spaceship render")
        .build();
    rl.set_target_fps(60);

    // El modelo se dibuja en nuestro framebuffer; la ventana solo muestra ese
    // resultado como textura.
    let mut textura = rl
        .load_texture_from_image(&thread, &imagen)
        .expect("no se pudo crear la textura del framebuffer");
    let mut pixeles = Vec::new();

    // En la ventana se aleja un poco la cámara para que el modelo no se salga
    // de la pantalla al girarlo.
    let inicial = Camara {
        zoom: encuadre.zoom_cualquier_angulo(ANCHO as f32, ALTO as f32),
        ..Camara::default()
    };
    let mut camara = inicial;
    let mut redibujar = true;

    while !rl.window_should_close() {
        redibujar |= actualizar_camara(&rl, &mut camara, &inicial);

        // Solo se vuelve a dibujar el modelo cuando la cámara se mueve.
        if redibujar {
            redibujar = false;
            render(&mut fb, &modelo, &encuadre, &camara);
            fb.write_rgba(&mut pixeles);
            textura
                .update_texture(&pixeles)
                .expect("el framebuffer no coincide con el tamaño de la textura");
        }

        let mut d = rl.begin_drawing(&thread);
        d.draw_texture(&textura, 0, 0, raylib::prelude::Color::WHITE);
        d.draw_text(AYUDA, 10, 10, 20, raylib::prelude::Color::LIGHTGRAY);
        d.draw_fps(10, ALTO as i32 - 30);
    }
    Ok(())
}
