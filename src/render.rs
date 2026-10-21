use raylib::prelude::{Vector2, Vector3};

use crate::framebuffer::Framebuffer;
use crate::line::triangle;
use crate::obj::Obj;

/// Fracción de la pantalla que ocupa el modelo; el resto queda como margen.
const OCUPACION: f32 = 0.9;

/// Cámara que orbita alrededor del centro del modelo.
///
/// La proyección sigue siendo ortográfica (sin perspectiva): solo se rota el
/// modelo y después se usan sus componentes x, y.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camara {
    /// Giro horizontal, alrededor del eje y, en radianes.
    pub yaw: f32,
    /// Giro vertical, alrededor del eje x, en radianes.
    pub pitch: f32,
    /// Multiplicador del tamaño del modelo en pantalla.
    pub zoom: f32,
}

impl Default for Camara {
    fn default() -> Self {
        Camara { yaw: 0.0, pitch: 0.0, zoom: 1.0 }
    }
}

/// Dónde está el modelo y qué escala lo hace caber en la pantalla viéndolo de frente.
/// Se calcula una sola vez al cargar el modelo.
pub struct Encuadre {
    centro: Vector3,
    escala: f32,
    /// Distancia del centro al vértice más lejano.
    radio: f32,
}

impl Encuadre {
    pub fn new(vertices: &[Vector3], ancho: f32, alto: f32) -> Self {
        if vertices.is_empty() {
            return Encuadre { centro: Vector3::new(0.0, 0.0, 0.0), escala: 1.0, radio: 0.0 };
        }

        // Caja que envuelve al modelo.
        let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for v in vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        // Misma escala en ambos ejes para no deformar el modelo; se elige la que
        // lo hace caber tanto a lo ancho como a lo alto.
        let escala = (ancho * OCUPACION / (max.x - min.x)).min(alto * OCUPACION / (max.y - min.y));
        let escala = if escala.is_finite() { escala } else { 1.0 };

        let centro = Vector3::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0, (min.z + max.z) / 2.0);
        let radio = vertices
            .iter()
            .map(|v| {
                let (x, y, z) = (v.x - centro.x, v.y - centro.y, v.z - centro.z);
                (x * x + y * y + z * z).sqrt()
            })
            .fold(0.0, f32::max);

        Encuadre { centro, escala, radio }
    }

    /// Zoom con el que el modelo cabe completo en pantalla desde cualquier ángulo.
    ///
    /// Al rotar, ningún vértice se aleja del centro más que `radio`, así que basta
    /// con que un círculo de ese radio quepa en la pantalla.
    pub fn zoom_cualquier_angulo(&self, ancho: f32, alto: f32) -> f32 {
        let zoom = ancho.min(alto) * OCUPACION / 2.0 / self.radio / self.escala;
        if zoom.is_finite() { zoom } else { 1.0 }
    }
}

/// Dibuja el contorno de todos los triángulos del modelo visto desde `camara`.
pub fn draw_obj_wireframe(fb: &mut Framebuffer, obj: &Obj, encuadre: &Encuadre, camara: &Camara) {
    let pantalla = vertices_a_pantalla(&obj.vertices, encuadre, camara, fb.width as f32, fb.height as f32);

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
fn vertices_a_pantalla(
    vertices: &[Vector3],
    encuadre: &Encuadre,
    camara: &Camara,
    ancho: f32,
    alto: f32,
) -> Vec<Vector2> {
    // Senos y cosenos se calculan una vez, no por cada vértice.
    let (sin_yaw, cos_yaw) = camara.yaw.sin_cos();
    let (sin_pitch, cos_pitch) = camara.pitch.sin_cos();
    let escala = encuadre.escala * camara.zoom;
    let centro_pantalla = Vector2::new(ancho / 2.0, alto / 2.0);

    vertices
        .iter()
        .map(|v| {
            // Se rota alrededor del centro del modelo, no del origen.
            let x = v.x - encuadre.centro.x;
            let y = v.y - encuadre.centro.y;
            let z = v.z - encuadre.centro.z;

            // Rotación alrededor del eje y (izquierda / derecha).
            let x1 = x * cos_yaw + z * sin_yaw;
            let z1 = -x * sin_yaw + z * cos_yaw;
            // Rotación alrededor del eje x (arriba / abajo).
            let y2 = y * cos_pitch - z1 * sin_pitch;

            Vector2::new(
                centro_pantalla.x + x1 * escala,
                // En el modelo y crece hacia arriba; en pantalla crece hacia abajo.
                centro_pantalla.y - y2 * escala,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cerca(a: Vector2, b: Vector2) -> bool {
        (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
    }

    #[test]
    fn front_view_is_centered_and_fits_the_screen() {
        let vertices = [
            Vector3::new(-2.0, -1.0, 5.0),
            Vector3::new(6.0, 3.0, -1.0),
            Vector3::new(0.0, 0.0, 2.0),
        ];
        let encuadre = Encuadre::new(&vertices, 800.0, 600.0);
        let p = vertices_a_pantalla(&vertices, &encuadre, &Camara::default(), 800.0, 600.0);

        // Ancho 8 y alto 4: limita el ancho -> escala = 800 * 0.9 / 8 = 90.
        assert!(cerca(p[0], Vector2::new(40.0, 480.0)));
        assert!(cerca(p[1], Vector2::new(760.0, 120.0)));
    }

    #[test]
    fn yaw_of_90_degrees_shows_the_side() {
        // Con 90° de yaw, un punto adelante del centro (z positivo) queda a la derecha.
        let vertices = [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)];
        let encuadre = Encuadre::new(&vertices, 100.0, 100.0); // escala 45
        let camara = Camara { yaw: std::f32::consts::FRAC_PI_2, ..Camara::default() };
        let p = vertices_a_pantalla(&[Vector3::new(0.0, 0.0, 1.0)], &encuadre, &camara, 100.0, 100.0);
        assert!(cerca(p[0], Vector2::new(95.0, 50.0)));
    }

    #[test]
    fn zoom_scales_around_the_center() {
        let vertices = [Vector3::new(-1.0, -1.0, 0.0), Vector3::new(1.0, 1.0, 0.0)];
        let encuadre = Encuadre::new(&vertices, 100.0, 100.0);
        let camara = Camara { zoom: 0.5, ..Camara::default() };
        let p = vertices_a_pantalla(&vertices, &encuadre, &camara, 100.0, 100.0);
        assert!(cerca(p[1], Vector2::new(72.5, 27.5)));
    }

    #[test]
    fn safe_zoom_keeps_every_angle_inside_the_screen() {
        // Modelo largo en z: de frente cabe, pero de lado se saldría con zoom 1.
        let vertices = [Vector3::new(-1.0, -1.0, -6.0), Vector3::new(1.0, 1.0, 6.0)];
        let encuadre = Encuadre::new(&vertices, 400.0, 400.0);
        let zoom = encuadre.zoom_cualquier_angulo(400.0, 400.0);
        for i in 0..16 {
            let angulo = i as f32 * std::f32::consts::TAU / 16.0;
            let camara = Camara { yaw: angulo, pitch: angulo * 0.5, zoom };
            for p in vertices_a_pantalla(&vertices, &encuadre, &camara, 400.0, 400.0) {
                assert!(p.x >= 0.0 && p.x <= 400.0 && p.y >= 0.0 && p.y <= 400.0, "{p:?}");
            }
        }
    }

    #[test]
    fn single_point_model_does_not_produce_nan() {
        let vertices = [Vector3::new(1.0, 1.0, 1.0)];
        let encuadre = Encuadre::new(&vertices, 800.0, 600.0);
        let p = vertices_a_pantalla(&vertices, &encuadre, &Camara::default(), 800.0, 600.0);
        assert_eq!(p[0], Vector2::new(400.0, 300.0));
    }
}
