//! Visor de la nave del Proyecto 1: carga `spaceship.obj` y lo muestra en una
//! ventana con una cámara orbital. Se ejecuta con `cargo run --release -- --nave`.

use std::io;
use std::time::Instant;

use raylib::prelude::*;

use crate::framebuffer::Framebuffer;
use crate::obj::load_obj;
use crate::render::{centrar, create_model_matrix, transformar_vertices, Camara, Encuadre};
use crate::vertex::Vertex;
use crate::{render, Uniforms};

const ANCHO: usize = 1024;
const ALTO: usize = 768;
const MODELO: &str = "spaceship.obj";
/// Archivo de `--captura`. Es distinto de las capturas del README para no sobrescribirlas.
const CAPTURA: &str = "captura.png";

const COLOR_FONDO: Vector3 = Vector3 { x: 0.043, y: 0.055, z: 0.102 };
const COLOR_MODELO: Vector3 = Vector3 { x: 0.42, y: 0.894, z: 1.0 };

/// Radianes que gira la cámara por cada pixel que se arrastra el mouse.
const SENSIBILIDAD_MOUSE: f32 = 0.01;
/// Radianes por segundo que gira la cámara con las flechas.
const VELOCIDAD_TECLAS: f32 = 1.5;
/// Cuánto cambia el zoom por cada paso de la rueda del mouse.
const PASO_ZOOM: f32 = 1.1;
const ZOOM_MIN: f32 = 0.2;
const ZOOM_MAX: f32 = 8.0;

const AYUDA: &str = "Arrastrar / flechas: rotar   Rueda: zoom   R: reiniciar";

/// Abre la ventana del visor. Con `solo_captura`, guarda la vista de frente en
/// `captura.png` y termina sin abrir la ventana.
pub fn ejecutar(solo_captura: bool) -> io::Result<()> {
    let inicio = Instant::now();
    let mut modelo = load_obj(MODELO)?;
    println!(
        "{MODELO}: {} vértices, {} triángulos (cargado en {:.2?})",
        modelo.vertices.len(),
        modelo.triangle_count(),
        inicio.elapsed()
    );
    centrar(&mut modelo.vertices);
    let encuadre = Encuadre::new(&modelo.vertices, ANCHO as f32, ALTO as f32);

    let mut vertex_array: Vec<Vertex> = modelo.get_vertex_array();
    for vertex in &mut vertex_array {
        vertex.color = COLOR_MODELO;
    }

    if solo_captura {
        // La captura es la vista de frente, ocupando toda la pantalla.
        let mut framebuffer = Framebuffer::new(ANCHO, ALTO);
        framebuffer.set_background_color(COLOR_FONDO);
        let inicio = Instant::now();
        dibujar(&mut framebuffer, &mut vertex_array, &Camara::default(), &encuadre);
        println!("Dibujado en {:.2?}", inicio.elapsed());
        framebuffer.to_image().export_image(CAPTURA);
        println!("Captura guardada en {CAPTURA}");
        return Ok(());
    }

    let (mut window, thread) = raylib::init()
        .size(ANCHO as i32, ALTO as i32)
        .title("Spaceship render")
        .build();
    window.set_target_fps(60);

    // Se crea después de la ventana para que su textura se libere antes de cerrarla.
    let mut framebuffer = Framebuffer::new(ANCHO, ALTO);
    framebuffer.set_background_color(COLOR_FONDO);
    framebuffer.init_texture(&mut window, &thread);

    // Se aleja un poco la cámara para que el modelo no se salga de la pantalla al girarlo.
    let inicial = Camara {
        zoom: encuadre.zoom_cualquier_angulo(ANCHO as f32, ALTO as f32),
        ..Camara::default()
    };
    let mut camara = inicial;
    let mut redibujar = true;

    while !window.window_should_close() {
        redibujar |= actualizar_camara(&window, &mut camara, &inicial);

        // Solo se vuelve a dibujar el modelo cuando la cámara se mueve.
        if redibujar {
            redibujar = false;
            dibujar(&mut framebuffer, &mut vertex_array, &camara, &encuadre);
        }

        framebuffer.swap_buffers_with(&mut window, &thread, |d| {
            d.draw_text(AYUDA, 10, 10, 20, Color::LIGHTGRAY);
            d.draw_fps(10, ALTO as i32 - 30);
        });
    }
    Ok(())
}

/// Prepara los uniforms de la cámara, transforma los vértices y dibuja el modelo.
fn dibujar(framebuffer: &mut Framebuffer, vertex_array: &mut [Vertex], camara: &Camara, encuadre: &Encuadre) {
    let uniforms = preparar_uniforms(camara, encuadre);
    transformar_vertices(vertex_array, &uniforms, ALTO as f32);
    framebuffer.clear();
    render(framebuffer, &uniforms, vertex_array);
}

/// Prepara los uniforms de un dibujo a partir del estado de la cámara.
fn preparar_uniforms(camara: &Camara, encuadre: &Encuadre) -> Uniforms {
    let translation = Vector3::new(ANCHO as f32 / 2.0, ALTO as f32 / 2.0, 0.0);
    let scale = encuadre.escala * camara.zoom;
    let rotation = Vector3::new(camara.pitch, camara.yaw, 0.0);

    let model_matrix = create_model_matrix(translation, scale, rotation);
    Uniforms { model_matrix }
}

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
