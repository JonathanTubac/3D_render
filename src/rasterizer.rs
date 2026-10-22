use crate::fragment::Fragment;
use crate::triangle::triangle_into;
use crate::vertex::Vertex;

/// Genera los fragmentos de todos los triángulos ensamblados y los reúne en una
/// sola colección, en el mismo orden que los triángulos.
///
/// Equivale a hacer `fragments.extend(triangle(...))` por cada triángulo, pero
/// `triangle_into` agrega los fragmentos directo a `fragments`, sin crear y copiar
/// un `Vec` intermedio por triángulo (la nave tiene más de medio millón).
pub fn rasterize(triangles: &[[Vertex; 3]]) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    for tri in triangles {
        triangle_into(&tri[0], &tri[1], &tri[2], &mut fragments);
    }

    fragments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triangle::triangle;
    use raylib::prelude::Vector3;

    fn v(x: f32, y: f32) -> Vertex {
        Vertex::new(Vector3::new(x, y, 0.0))
    }

    fn posiciones(fragments: &[Fragment]) -> Vec<(f32, f32)> {
        fragments.iter().map(|f| (f.position.x, f.position.y)).collect()
    }

    #[test]
    fn joins_the_fragments_of_every_triangle_in_order() {
        let triangles = [
            [v(300.0, 200.0), v(400.0, 400.0), v(500.0, 200.0)],
            [v(550.0, 450.0), v(700.0, 300.0), v(750.0, 550.0)],
        ];

        let mut esperado = triangle(&triangles[0][0], &triangles[0][1], &triangles[0][2]);
        esperado.extend(triangle(&triangles[1][0], &triangles[1][1], &triangles[1][2]));

        assert_eq!(posiciones(&rasterize(&triangles)), posiciones(&esperado));
    }

    #[test]
    fn no_triangles_make_no_fragments() {
        assert!(rasterize(&[]).is_empty());
    }
}
