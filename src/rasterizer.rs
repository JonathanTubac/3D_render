use crate::fragment::Fragment;
use crate::triangle::{triangle_filled_barycentric_into, triangle_filled_scanline_into, triangle_into};
use crate::vertex::Vertex;

/// Cómo convierte la rasterización cada triángulo en fragmentos. Los tres modos
/// reciben los mismos vértices; solo cambia qué posiciones generan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleMode {
    /// Solo el contorno: tres llamadas a `line`.
    Wireframe,
    /// Relleno evaluando las coordenadas baricéntricas de cada píxel.
    FilledBarycentric,
    /// Relleno por líneas horizontales.
    FilledScanline,
}

impl TriangleMode {
    pub const TODOS: [TriangleMode; 3] =
        [TriangleMode::Wireframe, TriangleMode::FilledBarycentric, TriangleMode::FilledScanline];

    pub fn nombre(self) -> &'static str {
        match self {
            TriangleMode::Wireframe => "Wireframe",
            TriangleMode::FilledBarycentric => "FilledBarycentric",
            TriangleMode::FilledScanline => "FilledScanline",
        }
    }
}

/// Genera los fragmentos de todos los triángulos ensamblados y los reúne en una
/// sola colección, en el mismo orden que los triángulos.
///
/// Cada función `*_into` agrega sus fragmentos directo a `fragments`, sin crear y
/// copiar un `Vec` intermedio por triángulo (la nave tiene más de medio millón).
pub fn rasterize(triangles: &[[Vertex; 3]], mode: TriangleMode) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    for tri in triangles {
        let [a, b, c] = tri;
        match mode {
            TriangleMode::Wireframe => triangle_into(a, b, c, &mut fragments),
            TriangleMode::FilledBarycentric => triangle_filled_barycentric_into(a, b, c, &mut fragments),
            TriangleMode::FilledScanline => triangle_filled_scanline_into(a, b, c, &mut fragments),
        }
    }

    fragments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triangle::{triangle, triangle_filled_barycentric, triangle_filled_scanline};
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
        let funciones: [(TriangleMode, fn(&Vertex, &Vertex, &Vertex) -> Vec<Fragment>); 3] = [
            (TriangleMode::Wireframe, triangle),
            (TriangleMode::FilledBarycentric, triangle_filled_barycentric),
            (TriangleMode::FilledScanline, triangle_filled_scanline),
        ];

        for (mode, funcion) in funciones {
            let mut esperado = funcion(&triangles[0][0], &triangles[0][1], &triangles[0][2]);
            esperado.extend(funcion(&triangles[1][0], &triangles[1][1], &triangles[1][2]));
            assert_eq!(posiciones(&rasterize(&triangles, mode)), posiciones(&esperado), "{mode:?}");
        }
    }

    #[test]
    fn no_triangles_make_no_fragments() {
        for mode in TriangleMode::TODOS {
            assert!(rasterize(&[], mode).is_empty());
        }
    }
}
