use raylib::prelude::{Vector2, Vector3};

use crate::framebuffer::Framebuffer;
use crate::line::triangle;
use crate::obj::Obj;

/// Fracción de la pantalla que ocupa el modelo; el resto queda como margen.
const OCUPACION: f32 = 0.9;

/// Dibuja el contorno de todos los triángulos del modelo, centrado y escalado
/// para que quepa completo en el framebuffer.
///
/// Todavía no hay proyección: se usan directamente las componentes x, y de cada
/// vértice (vista de frente) y se ignora z.
pub fn draw_obj_wireframe(fb: &mut Framebuffer, obj: &Obj) {
    let pantalla = vertices_a_pantalla(&obj.vertices, fb.width as f32, fb.height as f32);

    // Cada grupo de tres índices es un triángulo; los índices señalan
    // posiciones dentro de la lista de vértices ya convertidos a pantalla.
    for t in obj.indices.chunks_exact(3) {
        let a = pantalla[t[0]];
        let b = pantalla[t[1]];
        let c = pantalla[t[2]];
        triangle(fb, a, b, c);
    }
}

/// Convierte cada vértice del modelo a coordenadas de pantalla.
///
/// Se hace una vez por vértice y no una vez por triángulo: cada vértice de un
/// modelo suele compartirse entre varios triángulos, así que se ahorra trabajo.
fn vertices_a_pantalla(vertices: &[Vector3], ancho: f32, alto: f32) -> Vec<Vector2> {
    if vertices.is_empty() {
        return Vec::new();
    }

    // Caja que envuelve al modelo en x, y.
    let mut min = Vector2::new(f32::INFINITY, f32::INFINITY);
    let mut max = Vector2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for v in vertices {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
    }

    // Misma escala en ambos ejes para no deformar el modelo; se elige la que
    // lo hace caber tanto a lo ancho como a lo alto.
    let tamano = Vector2::new(max.x - min.x, max.y - min.y);
    let escala = (ancho * OCUPACION / tamano.x).min(alto * OCUPACION / tamano.y);
    let escala = if escala.is_finite() { escala } else { 1.0 };

    let centro_modelo = Vector2::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0);
    let centro_pantalla = Vector2::new(ancho / 2.0, alto / 2.0);

    vertices
        .iter()
        .map(|v| {
            Vector2::new(
                centro_pantalla.x + (v.x - centro_modelo.x) * escala,
                // En el modelo y crece hacia arriba; en pantalla crece hacia abajo.
                centro_pantalla.y - (v.y - centro_modelo.y) * escala,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_is_centered_and_fits_the_screen() {
        let vertices = [
            Vector3::new(-2.0, -1.0, 5.0),
            Vector3::new(6.0, 3.0, -1.0),
            Vector3::new(0.0, 0.0, 0.0),
        ];
        let p = vertices_a_pantalla(&vertices, 800.0, 600.0);

        // Ancho 8 y alto 4: limita el ancho -> escala = 800 * 0.9 / 8 = 90.
        assert_eq!(p[0], Vector2::new(40.0, 480.0));
        assert_eq!(p[1], Vector2::new(760.0, 120.0));
        // El centro del modelo (2, 1) cae en el centro de la pantalla.
        let centro = Vector2::new((p[0].x + p[1].x) / 2.0, (p[0].y + p[1].y) / 2.0);
        assert_eq!(centro, Vector2::new(400.0, 300.0));
    }

    #[test]
    fn single_point_model_does_not_produce_nan() {
        let p = vertices_a_pantalla(&[Vector3::new(1.0, 1.0, 1.0)], 800.0, 600.0);
        assert_eq!(p[0], Vector2::new(400.0, 300.0));
    }
}
