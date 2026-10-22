use crate::fragment::Fragment;
use crate::line::{fragment_count, line_into};
use crate::vertex::Vertex;

/// Genera los fragmentos del contorno de un triángulo: las tres líneas que
/// conectan sus vértices. Todavía no rellena el interior.
pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    // Se reserva de una vez el espacio de los tres lados para no volver a pedir
    // memoria mientras se agregan los fragmentos.
    let total = fragment_count(v1, v2) + fragment_count(v2, v3) + fragment_count(v3, v1);
    let mut fragments = Vec::with_capacity(total);
    line_into(v1, v2, &mut fragments);
    line_into(v2, v3, &mut fragments);
    line_into(v3, v1, &mut fragments);
    fragments
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::math::Vector3;

    #[test]
    fn contains_the_three_edges_and_not_the_interior() {
        let a = Vertex::new(Vector3::new(2.0, 2.0, 0.0));
        let b = Vertex::new(Vector3::new(17.0, 2.0, 0.0));
        let c = Vertex::new(Vector3::new(2.0, 17.0, 0.0));
        let f = triangle(&a, &b, &c);
        let contiene = |x: f32, y: f32| f.iter().any(|f| f.position.x == x && f.position.y == y);

        assert!(contiene(10.0, 2.0)); // a-b
        assert!(contiene(10.0, 9.0)); // b-c
        assert!(contiene(2.0, 10.0)); // c-a
        assert!(!contiene(6.0, 6.0)); // interior vacío
    }
}
