use raylib::math::{Vector2, Vector3};

/// Abstracción de un píxel antes de escribirlo en el framebuffer.
///
/// - `position`: x, y dentro de la imagen (origen arriba a la izquierda, y hacia abajo).
/// - `color`: rojo, verde y azul en el rango [0.0, 1.0].
/// - `depth`: distancia al observador; un valor menor está más cerca.
/// - `normal`: normal interpolada, en el espacio de la vista.
/// - `vertex_position`: posición interpolada en las coordenadas del modelo. Los
///   shaders procedurales la usan para que los patrones queden pegados a la
///   superficie aunque el modelo gire.
pub struct Fragment {
    pub position: Vector2,
    pub color: Vector3,
    pub depth: f32,
    pub normal: Vector3,
    pub vertex_position: Vector3,
}

impl Fragment {
    pub fn new(x: f32, y: f32, color: Vector3, depth: f32) -> Self {
        Self {
            position: Vector2::new(x, y),
            color,
            depth,
            normal: Vector3::new(0.0, 0.0, 1.0),
            vertex_position: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}
