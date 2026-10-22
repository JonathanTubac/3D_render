use raylib::prelude::{Matrix, Vector2, Vector3};

use crate::framebuffer::Framebuffer;
use crate::line::triangle;
use crate::obj::Obj;
use crate::Uniforms;

/// Fracción de la pantalla que ocupa el modelo; el resto queda como margen.
const OCUPACION: f32 = 0.9;

/// Cámara que orbita alrededor del centro del modelo.
///
/// Es el estado que controla el usuario; antes de cada dibujo se convierte en la
/// matriz de modelo. La proyección sigue siendo ortográfica (sin perspectiva).
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

/// Mueve los vértices para que el centro de la caja que envuelve al modelo quede
/// en el origen. Así la rotación de la matriz de modelo gira la nave sobre sí misma.
/// Se hace una sola vez, al cargar el modelo.
pub fn centrar(vertices: &mut [Vector3]) {
    let Some((min, max)) = caja(vertices) else { return };
    let centro = Vector3::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0, (min.z + max.z) / 2.0);
    for v in vertices {
        *v = *v - centro;
    }
}

/// Esquinas mínima y máxima de la caja que envuelve a los vértices.
fn caja(vertices: &[Vector3]) -> Option<(Vector3, Vector3)> {
    let primero = *vertices.first()?;
    Some(vertices.iter().fold((primero, primero), |(min, max), v| {
        (
            Vector3::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z)),
            Vector3::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z)),
        )
    }))
}

/// Escala que hace caber al modelo en la pantalla, y su radio. Se calcula una
/// sola vez, con los vértices ya centrados.
pub struct Encuadre {
    /// Escala con la que el modelo, visto de frente, ocupa la pantalla.
    pub escala: f32,
    /// Distancia del centro al vértice más lejano.
    radio: f32,
}

impl Encuadre {
    pub fn new(vertices: &[Vector3], ancho: f32, alto: f32) -> Self {
        let Some((min, max)) = caja(vertices) else {
            return Encuadre { escala: 1.0, radio: 0.0 };
        };

        // Misma escala en ambos ejes para no deformar el modelo; se elige la que
        // lo hace caber tanto a lo ancho como a lo alto.
        let escala = (ancho * OCUPACION / (max.x - min.x)).min(alto * OCUPACION / (max.y - min.y));
        let escala = if escala.is_finite() { escala } else { 1.0 };
        let radio = vertices.iter().map(|v| v.length()).fold(0.0, f32::max);

        Encuadre { escala, radio }
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

/// Construye la matriz de modelo: primero escala, después rota y al final traslada.
///
/// `rotation` contiene los ángulos en radianes alrededor de x, y, z. Se gira primero
/// en y (izquierda / derecha), luego en x (arriba / abajo) y al final en z.
///
/// En Raylib, `a * b` aplica primero `a` y después `b`, por eso el producto se
/// escribe en el mismo orden en que ocurren las transformaciones.
pub fn create_model_matrix(translation: Vector3, scale: f32, rotation: Vector3) -> Matrix {
    let escala = Matrix::scale(scale, scale, scale);
    let rotacion = Matrix::rotate_y(rotation.y) * Matrix::rotate_x(rotation.x) * Matrix::rotate_z(rotation.z);
    let traslacion = Matrix::translate(translation.x, translation.y, translation.z);
    escala * rotacion * traslacion
}

/// Dibuja el contorno de todos los triángulos del modelo.
pub fn draw_obj_wireframe(fb: &mut Framebuffer, uniforms: &Uniforms, obj: &Obj) {
    let pantalla = vertices_a_pantalla(&obj.vertices, uniforms, fb.height as f32);

    // Cada grupo de tres índices es un triángulo; los índices señalan
    // posiciones dentro de la lista de vértices ya convertidos a pantalla.
    for t in obj.indices.chunks_exact(3) {
        let a = pantalla[t[0]];
        let b = pantalla[t[1]];
        let c = pantalla[t[2]];
        triangle(fb, a, b, c);
    }
}

/// Transforma cada vértice con la matriz de modelo y lo pasa a coordenadas de pantalla.
///
/// Todos los vértices usan la misma `uniforms.model_matrix`; si cada uno usara una
/// distinta, el modelo se deformaría en vez de moverse como una sola pieza.
/// Se hace una vez por vértice y no una vez por triángulo, porque cada vértice
/// suele compartirse entre varios triángulos.
fn vertices_a_pantalla(vertices: &[Vector3], uniforms: &Uniforms, alto: f32) -> Vec<Vector2> {
    vertices
        .iter()
        .map(|v| {
            let p = v.transform(uniforms.model_matrix);
            // En el modelo y crece hacia arriba; en pantalla crece hacia abajo.
            Vector2::new(p.x, alto - p.y)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cerca(a: Vector2, b: Vector2) -> bool {
        (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
    }

    /// Uniforms con el modelo en el centro de una pantalla de `ancho` × `alto`.
    fn uniforms(ancho: f32, alto: f32, escala: f32, rotation: Vector3) -> Uniforms {
        let centro = Vector3::new(ancho / 2.0, alto / 2.0, 0.0);
        Uniforms { model_matrix: create_model_matrix(centro, escala, rotation) }
    }

    #[test]
    fn model_matrix_scales_then_rotates_then_translates() {
        let m = create_model_matrix(Vector3::new(10.0, 0.0, 0.0), 2.0, Vector3::new(0.0, 0.0, std::f32::consts::FRAC_PI_2));
        // (1, 0, 0) -> escala (2, 0, 0) -> gira 90° en z (0, 2, 0) -> traslada (10, 2, 0)
        let p = Vector3::new(1.0, 0.0, 0.0).transform(m);
        assert!((p - Vector3::new(10.0, 2.0, 0.0)).length() < 1e-5, "{p:?}");
    }

    #[test]
    fn centering_moves_the_box_center_to_the_origin() {
        let mut vertices = [Vector3::new(-2.0, -1.0, 5.0), Vector3::new(6.0, 3.0, -1.0)];
        centrar(&mut vertices);
        assert_eq!(vertices[0], Vector3::new(-4.0, -2.0, 3.0));
        assert_eq!(vertices[1], Vector3::new(4.0, 2.0, -3.0));
    }

    #[test]
    fn front_view_is_centered_and_fits_the_screen() {
        let mut vertices = [
            Vector3::new(-2.0, -1.0, 5.0),
            Vector3::new(6.0, 3.0, -1.0),
            Vector3::new(0.0, 0.0, 2.0),
        ];
        centrar(&mut vertices);
        let encuadre = Encuadre::new(&vertices, 800.0, 600.0);
        let u = uniforms(800.0, 600.0, encuadre.escala, Vector3::new(0.0, 0.0, 0.0));
        let p = vertices_a_pantalla(&vertices, &u, 600.0);

        // Ancho 8 y alto 4: limita el ancho -> escala = 800 * 0.9 / 8 = 90.
        assert!(cerca(p[0], Vector2::new(40.0, 480.0)), "{:?}", p[0]);
        assert!(cerca(p[1], Vector2::new(760.0, 120.0)), "{:?}", p[1]);
    }

    #[test]
    fn yaw_of_90_degrees_shows_the_side() {
        // Con 90° de yaw, un punto adelante del centro (z positivo) queda a la derecha.
        let u = uniforms(100.0, 100.0, 45.0, Vector3::new(0.0, std::f32::consts::FRAC_PI_2, 0.0));
        let p = vertices_a_pantalla(&[Vector3::new(0.0, 0.0, 1.0)], &u, 100.0);
        assert!(cerca(p[0], Vector2::new(95.0, 50.0)), "{:?}", p[0]);
    }

    #[test]
    fn pitch_of_90_degrees_shows_the_top() {
        // Con 90° de pitch, un punto adelante del centro (z positivo) queda abajo.
        let u = uniforms(100.0, 100.0, 45.0, Vector3::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0));
        let p = vertices_a_pantalla(&[Vector3::new(0.0, 0.0, 1.0)], &u, 100.0);
        assert!(cerca(p[0], Vector2::new(50.0, 95.0)), "{:?}", p[0]);
    }

    #[test]
    fn safe_zoom_keeps_every_angle_inside_the_screen() {
        // Modelo largo en z: de frente cabe, pero de lado se saldría con zoom 1.
        let vertices = [Vector3::new(-1.0, -1.0, -6.0), Vector3::new(1.0, 1.0, 6.0)];
        let encuadre = Encuadre::new(&vertices, 400.0, 400.0);
        let escala = encuadre.escala * encuadre.zoom_cualquier_angulo(400.0, 400.0);
        for i in 0..16 {
            let angulo = i as f32 * std::f32::consts::TAU / 16.0;
            let u = uniforms(400.0, 400.0, escala, Vector3::new(angulo * 0.5, angulo, 0.0));
            for p in vertices_a_pantalla(&vertices, &u, 400.0) {
                assert!(p.x >= 0.0 && p.x <= 400.0 && p.y >= 0.0 && p.y <= 400.0, "{p:?}");
            }
        }
    }
}
