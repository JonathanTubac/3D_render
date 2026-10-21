use raylib::math::{Vector2, Vector3};

/// Abstracción de un píxel antes de escribirlo en el framebuffer.
///
/// - `position`: x, y dentro de la imagen (origen arriba a la izquierda, y hacia abajo).
/// - `color`: rojo, verde y azul en el rango [0.0, 1.0].
/// - `depth`: profundidad; servirá para decidir qué superficie queda delante.
pub struct Fragment {
    pub position: Vector2,
    pub color: Vector3,
    pub depth: f32,
}

impl Fragment {
    pub fn new(x: f32, y: f32, color: Vector3, depth: f32) -> Self {
        Self {
            position: Vector2::new(x, y),
            color,
            depth,
        }
    }
}
