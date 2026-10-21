mod color;
mod framebuffer;
mod line;
mod vertices;

use color::Color;
use framebuffer::Framebuffer;
use line::triangle;
use raylib::prelude::Vector2;

/// Rellena un rectángulo a profundidad constante usando el z-buffer.
fn fill_rect(fb: &mut Framebuffer, x0: i32, y0: i32, w: i32, h: i32, depth: f32) {
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            fb.point_with_depth(x, y, depth);
        }
    }
}

fn main() -> std::io::Result<()> {
    vertices::demo();

    let mut fb = Framebuffer::new(800, 600);

    fb.set_background_color(Color::from_hex(0x0B0E1A));
    fb.clear();

    // Unas "estrellas" sin profundidad.
    fb.set_current_color(Color::WHITE);
    for i in 0..200 {
        let x = (i * 7919) % 800;
        let y = (i * 104729) % 600;
        fb.point(x, y);
    }

    // Demostración del z-buffer: el rojo se dibuja DESPUÉS pero está más lejos,
    // así que queda detrás del azul en la zona donde se traslapan.
    fb.set_current_color(Color::from_hex(0x3A7BFF));
    fill_rect(&mut fb, 200, 150, 300, 200, 1.0);

    fb.set_current_color(Color::from_hex(0xFF4040));
    fill_rect(&mut fb, 350, 250, 300, 200, 2.0);

    // Triángulo del ejercicio.
    fb.set_current_color(Color::from_hex(0xFFD23F));
    let a = Vector2::new(320.0, 100.0);
    let b = Vector2::new(150.0, 350.0);
    let c = Vector2::new(500.0, 350.0);
    triangle(&mut fb, a, b, c);

    // Otros triángulos con distinto tamaño y orientación.
    fb.set_current_color(Color::from_hex(0x7CFF6B));
    triangle(
        &mut fb,
        Vector2::new(560.0, 80.0),
        Vector2::new(760.0, 140.0),
        Vector2::new(600.0, 220.0),
    );
    triangle(
        &mut fb,
        Vector2::new(80.0, 560.0),
        Vector2::new(300.0, 420.0),
        Vector2::new(260.0, 590.0),
    );
    // Triángulo muy delgado, casi una línea.
    triangle(
        &mut fb,
        Vector2::new(520.0, 500.0),
        Vector2::new(780.0, 520.0),
        Vector2::new(540.0, 506.0),
    );

    fb.write_bmp("out.bmp")?;
    fb.write_depth_bmp("depth.bmp")?;
    println!("Imagen guardada en out.bmp y depth.bmp");
    Ok(())
}
