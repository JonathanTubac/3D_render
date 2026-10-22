mod color;
mod fragment;
mod framebuffer;
mod laboratorio;
mod line;
mod obj;
mod primitive;
mod rasterizer;
mod render;
mod shaders;
mod triangle;
mod vertex;
// Ejercicio de la fase de vértices; el renderer no lo utiliza.
#[allow(dead_code)]
mod vertices;
mod visor;

use raylib::prelude::*;

use framebuffer::Framebuffer;
use primitive::primitive_assembly;
use rasterizer::{rasterize, TriangleMode};
use shaders::{fragment_shader, vertex_shader};
use vertex::Vertex;

/// Parámetros que comparten todos los vértices y fragmentos de un mismo dibujo.
/// Pueden cambiar entre un dibujo y otro, pero no durante el dibujo.
pub struct Uniforms {
    pub model_matrix: Matrix,
}

/// Convierte los vértices en una imagen siguiendo el pipeline:
/// vértices → triángulos → fragmentos → framebuffer.
///
/// `mode` indica si la rasterización genera solo el contorno de cada triángulo o
/// también su interior, y con qué algoritmo.
fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], mode: TriangleMode) {
    // 1. Procesamiento de vértices
    //    El vertex shader se ejecuta una vez por vértice, con los mismos uniforms
    //    para todos; deja el resultado en `transformed_position`.
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas
    //    Cada grupo de tres vértices consecutivos forma un triángulo.
    let triangles = primitive_assembly(&transformed_vertices);

    // 3. Rasterización
    //    Cada triángulo genera los fragmentos que ocupa en la imagen.
    let fragments = rasterize(&triangles, mode);

    // 4. Procesamiento de fragmentos
    //    El fragment shader decide el color de cada fragmento y ese color se
    //    escribe en su posición del framebuffer.
    for fragment in fragments {
        let color = fragment_shader(&fragment);

        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            color,
        );
    }
}

/// Tecla 1, 2 o 3: cambia el modo de rasterización.
pub fn leer_modo(window: &RaylibHandle, mode: &mut TriangleMode) -> bool {
    let anterior = *mode;
    if window.is_key_pressed(KeyboardKey::KEY_ONE) {
        *mode = TriangleMode::Wireframe;
    }
    if window.is_key_pressed(KeyboardKey::KEY_TWO) {
        *mode = TriangleMode::FilledBarycentric;
    }
    if window.is_key_pressed(KeyboardKey::KEY_THREE) {
        *mode = TriangleMode::FilledScanline;
    }
    *mode != anterior
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |nombre: &str| args.iter().any(|a| a == nombre);

    // `--nave` abre el visor de la nave del Proyecto 1 (con `--captura`, solo guarda la imagen).
    if flag("--nave") {
        if let Err(e) = visor::ejecutar(flag("--captura")) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        return;
    }
    // Mide la rasterización de cada modo, sin abrir la ventana.
    if flag("--benchmark-triangles") {
        if let Err(e) = laboratorio::benchmark() {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        return;
    }
    // Guarda una imagen de la escena de prueba en cada modo.
    if flag("--capturas-relleno") {
        laboratorio::capturas();
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(laboratorio::ANCHO as i32, laboratorio::ALTO as i32)
        .title("Relleno de triangulos")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(laboratorio::ANCHO, laboratorio::ALTO);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    // Las posiciones ya están en coordenadas de pantalla; con la matriz
    // identidad, el procesamiento de vértices las deja iguales.
    let vertex_array = laboratorio::escena_de_prueba();

    let uniforms = Uniforms {
        model_matrix: Matrix::identity(),
    };
    let mut mode = TriangleMode::FilledScanline;

    while !window.window_should_close() {
        leer_modo(&window, &mut mode);

        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array, mode);
        let texto = format!("1: Wireframe  2: FilledBarycentric  3: FilledScanline   Modo: {}", mode.nombre());
        framebuffer.swap_buffers_with(&mut window, &thread, |d| {
            d.draw_text(&texto, 10, 10, 20, Color::LIGHTGRAY);
        });
    }
}
