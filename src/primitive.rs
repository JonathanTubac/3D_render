use crate::vertex::Vertex;

/// Agrupa los vértices procesados de tres en tres para formar triángulos, como
/// `GL_TRIANGLES` en OpenGL: los vértices 1, 2, 3 son el primer triángulo; 4, 5, 6
/// el segundo, y así sucesivamente.
///
/// Falla si la cantidad de vértices no es múltiplo de 3, en lugar de perder sin
/// avisar los vértices del último grupo incompleto.
pub fn primitive_assembly(transformed_vertices: &[Vertex]) -> Vec<[Vertex; 3]> {
    assert!(
        transformed_vertices.len() % 3 == 0,
        "La cantidad de vértices debe ser múltiplo de 3",
    );

    transformed_vertices
        .chunks_exact(3)
        .map(|group| [group[0].clone(), group[1].clone(), group[2].clone()])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::prelude::Vector3;

    /// `n` vértices numerados 1, 2, 3, ... en su coordenada x.
    fn numerados(n: usize) -> Vec<Vertex> {
        (1..=n).map(|i| Vertex::new(Vector3::new(i as f32, 0.0, 0.0))).collect()
    }

    #[test]
    fn six_vertices_make_two_triangles_in_order() {
        let triangles = primitive_assembly(&numerados(6));
        let numeros: Vec<[f32; 3]> = triangles
            .iter()
            .map(|t| [t[0].position.x, t[1].position.x, t[2].position.x])
            .collect();
        assert_eq!(numeros, vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    }

    #[test]
    #[should_panic(expected = "múltiplo de 3")]
    fn seven_vertices_are_rejected() {
        primitive_assembly(&numerados(7));
    }

    #[test]
    fn no_vertices_make_no_triangles() {
        assert!(primitive_assembly(&[]).is_empty());
    }
}
