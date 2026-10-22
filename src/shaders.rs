use raylib::prelude::*;

use crate::vertex::Vertex;
use crate::Uniforms;

/// Multiplica una matriz de 4 × 4 por un vector de cuatro componentes.
///
/// Raylib guarda la matriz por columnas: `m0, m1, m2, m3` es la primera columna,
/// y la traslación está en `m12, m13, m14`.
fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

/// Prepara un vértice para las siguientes etapas: aplica la matriz de modelo a su
/// posición y guarda el resultado en `transformed_position`.
///
/// Se llama una vez por cada vértice; todas las llamadas de un mismo dibujo
/// comparten los mismos `uniforms`.
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // w = 1.0 permite que la matriz de 4 × 4 también aplique traslaciones.
    let position = Vector4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);

    let transformed = multiply_matrix_vector4(&uniforms.model_matrix, &position);

    // Se conservan los demás datos del vértice (como el color); solo cambia la
    // posición transformada.
    let mut result = vertex.clone();
    result.transformed_position = Vector3::new(transformed.x, transformed.y, transformed.z);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn shader(position: Vector3, model_matrix: Matrix) -> Vector3 {
        let vertex = Vertex::new(position);
        vertex_shader(&vertex, &Uniforms { model_matrix }).transformed_position
    }

    fn cerca(a: Vector3, b: Vector3) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn identity_keeps_the_position() {
        let p = Vector3::new(300.0, 200.0, 0.0);
        assert_eq!(shader(p, Matrix::identity()), p);
    }

    #[test]
    fn translation_moves_the_vertex() {
        let p = shader(Vector3::new(300.0, 200.0, 0.0), Matrix::translate(50.0, -20.0, 0.0));
        assert!(cerca(p, Vector3::new(350.0, 180.0, 0.0)), "{p:?}");
    }

    #[test]
    fn scale_multiplies_each_component() {
        let p = shader(Vector3::new(1.0, 2.0, 3.0), Matrix::scale(2.0, 3.0, 4.0));
        assert!(cerca(p, Vector3::new(2.0, 6.0, 12.0)), "{p:?}");
    }

    #[test]
    fn rotation_turns_around_the_origin() {
        // 90° alrededor de z: el eje x pasa a ser el eje y.
        let p = shader(Vector3::new(1.0, 0.0, 0.0), Matrix::rotate_z(FRAC_PI_2));
        assert!(cerca(p, Vector3::new(0.0, 1.0, 0.0)), "{p:?}");
    }

    #[test]
    fn matches_raylib_transform() {
        let m = Matrix::scale(2.0, 0.5, 1.5)
            * Matrix::rotate_y(0.7)
            * Matrix::rotate_x(-0.3)
            * Matrix::translate(10.0, 20.0, -5.0);
        let p = Vector3::new(1.5, -2.0, 0.25);
        assert!(cerca(shader(p, m), p.transform(m)));
    }

    #[test]
    fn keeps_the_other_vertex_data() {
        let rojo = Vector3::new(1.0, 0.0, 0.0);
        let vertex = Vertex::new_with_color(Vector3::new(1.0, 2.0, 3.0), rojo);
        let result = vertex_shader(&vertex, &Uniforms { model_matrix: Matrix::translate(1.0, 0.0, 0.0) });
        assert_eq!(result.position, vertex.position);
        assert_eq!(result.color, rojo);
    }
}
