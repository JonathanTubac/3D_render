mod color;
mod fragment;
mod framebuffer;
mod line;
mod obj;
mod render;
mod triangle;
mod vertex;
// Ejercicio de la fase de vértices; el renderer no lo utiliza.
#[allow(dead_code)]
mod vertices;
mod visor;

use raylib::prelude::*;

use framebuffer::Framebuffer;
use triangle::triangle;
use vertex::Vertex;

/// Parámetros que comparten todos los vértices y fragmentos de un mismo dibujo.
/// Pueden cambiar entre un dibujo y otro, pero no durante el dibujo.
pub struct Uniforms {
    pub model_matrix: Matrix,
}

/// Dibuja los triángulos de `vertex_array`: cada grupo de tres vértices
/// consecutivos es un triángulo. Usa `transformed_position` de cada vértice.
fn render(framebuffer: &mut Framebuffer, _uniforms: &Uniforms, vertex_array: &[Vertex]) {
    for vertices in vertex_array.chunks_exact(3) {
        let fragments = triangle(&vertices[0], &vertices[1], &vertices[2]);

        for fragment in fragments {
            framebuffer.point(
                fragment.position.x as i32,
                fragment.position.y as i32,
                fragment.color,
            );
        }
    }
}

fn main() {
    // `--nave` abre el visor de la nave del Proyecto 1 (con `--captura`, solo guarda la imagen).
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--nave") {
        if let Err(e) = visor::ejecutar(args.iter().any(|a| a == "--captura")) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(800, 600)
        .title("Primer triangulo")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(800, 600);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    let positions = [
        // Primer triángulo.
        Vector3::new(300.0, 200.0, 0.0),
        Vector3::new(400.0, 400.0, 0.0),
        Vector3::new(500.0, 200.0, 0.0),
        // Segundo triángulo, en la misma llamada a render.
        Vector3::new(550.0, 450.0, 0.0),
        Vector3::new(700.0, 300.0, 0.0),
        Vector3::new(750.0, 550.0, 0.0),
    ];

    let vertex_array: Vec<Vertex> = positions
        .into_iter()
        .map(|position| {
            let mut vertex = Vertex::new_with_color(position, Vector3::new(1.0, 1.0, 1.0));

            // Las posiciones ya están en coordenadas de pantalla.
            vertex.transformed_position = position;
            vertex
        })
        .collect();

    let uniforms = Uniforms {
        model_matrix: Matrix::identity(),
    };

    while !window.window_should_close() {
        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array);
        framebuffer.swap_buffers(&mut window, &thread);
    }
}
