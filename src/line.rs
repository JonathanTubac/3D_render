use raylib::prelude::Vector2;

use crate::framebuffer::Framebuffer;

/// Dibuja una línea de `start` a `end` con el color actual del framebuffer.
///
/// Usa el algoritmo de Bresenham: dentro del ciclo solo hay sumas y comparaciones
/// de enteros, sin divisiones ni flotantes, porque se ejecuta una vez por pixel.
pub fn line(fb: &mut Framebuffer, start: Vector2, end: Vector2) {
    let mut x1 = start.x.round() as i32;
    let mut y1 = start.y.round() as i32;

    let x2 = end.x.round() as i32;
    let y2 = end.y.round() as i32;

    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        fb.point(x1, y1);
        if x1 == x2 && y1 == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
    }
}

/// Dibuja el contorno de un triángulo conectando sus tres vértices con `line`.
pub fn triangle(fb: &mut Framebuffer, a: Vector2, b: Vector2, c: Vector2) {
    line(fb, a, b);
    line(fb, b, c);
    line(fb, c, a);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    fn painted(fb: &Framebuffer) -> usize {
        let mut n = 0;
        for y in 0..fb.height as i32 {
            for x in 0..fb.width as i32 {
                if fb.get_color(x, y) == Some(Color::WHITE) {
                    n += 1;
                }
            }
        }
        n
    }

    #[test]
    fn draws_both_endpoints_in_any_direction() {
        let mut fb = Framebuffer::new(10, 10);
        line(&mut fb, Vector2::new(8.0, 7.0), Vector2::new(1.0, 2.0));
        assert_eq!(fb.get_color(8, 7), Some(Color::WHITE));
        assert_eq!(fb.get_color(1, 2), Some(Color::WHITE));
    }

    #[test]
    fn one_pixel_per_step_on_major_axis() {
        let mut fb = Framebuffer::new(20, 20);
        line(&mut fb, Vector2::new(0.0, 0.0), Vector2::new(15.0, 4.0));
        assert_eq!(painted(&fb), 16);

        let mut fb = Framebuffer::new(20, 20);
        line(&mut fb, Vector2::new(3.0, 19.0), Vector2::new(5.0, 0.0));
        assert_eq!(painted(&fb), 20);
    }

    #[test]
    fn triangle_draws_all_three_edges() {
        let mut fb = Framebuffer::new(20, 20);
        let (a, b, c) = (Vector2::new(2.0, 2.0), Vector2::new(17.0, 2.0), Vector2::new(2.0, 17.0));
        triangle(&mut fb, a, b, c);
        assert_eq!(fb.get_color(10, 2), Some(Color::WHITE)); // a-b
        assert_eq!(fb.get_color(2, 10), Some(Color::WHITE)); // c-a
        assert_eq!(fb.get_color(10, 9), Some(Color::WHITE)); // b-c
        assert_eq!(fb.get_color(6, 6), Some(Color::BLACK)); // interior vacío
    }

    #[test]
    fn degenerate_line_is_a_single_point() {
        let mut fb = Framebuffer::new(5, 5);
        line(&mut fb, Vector2::new(2.4, 2.6), Vector2::new(2.0, 3.0));
        assert_eq!(painted(&fb), 1);
    }
}
