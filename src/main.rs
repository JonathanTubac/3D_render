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
use obj::load_obj;
use render::draw_obj_wireframe;

const ANCHO: usize = 1024;
const ALTO: usize = 768;
const MODELO: &str = "spaceship.obj";
const CAPTURA: &str = "screenshot.png";

fn main() -> std::io::Result<()> {
    let inicio = Instant::now();
    let modelo = load_obj(MODELO)?;
    println!(
        "{MODELO}: {} vértices, {} triángulos (cargado en {:.2?})",
        modelo.vertices.len(),
        modelo.triangle_count(),
        inicio.elapsed()
    );

    let mut fb = Framebuffer::new(ANCHO, ALTO);
    fb.set_background_color(Color::from_hex(0x0B0E1A));
    fb.clear();
    fb.set_current_color(Color::from_hex(0x6BE4FF));

    let inicio = Instant::now();
    draw_obj_wireframe(&mut fb, &modelo);
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

    // El modelo se dibuja una sola vez en nuestro framebuffer; la ventana solo
    // muestra ese resultado como textura.
    let textura = rl
        .load_texture_from_image(&thread, &imagen)
        .expect("no se pudo crear la textura del framebuffer");

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.draw_texture(&textura, 0, 0, raylib::prelude::Color::WHITE);
    }
    Ok(())
}
