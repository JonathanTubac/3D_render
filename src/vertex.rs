use raylib::math::Vector3;

/// Vértice de un modelo.
///
/// - `position`: posición original, en las coordenadas del modelo.
/// - `normal`: normal original, en las coordenadas del modelo.
/// - `color`: rojo, verde y azul en el rango [0.0, 1.0].
/// - `transformed_position`: posición después de aplicar las transformaciones,
///   ya en coordenadas de pantalla. Es la que usan `line` y `triangle`.
/// - `transformed_normal`: normal después de rotar el modelo, en el espacio de
///   la vista (x a la derecha, y hacia arriba, z hacia el observador).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub color: Vector3,
    pub transformed_position: Vector3,
    pub transformed_normal: Vector3,
}

impl Vertex {
    /// Vértice blanco.
    pub fn new(position: Vector3) -> Self {
        Self::new_with_color(position, Vector3::new(1.0, 1.0, 1.0))
    }

    /// Vértice con color. `transformed_position` comienza igual a `position` y la
    /// normal apunta hacia el observador hasta que se le asigne otra.
    pub fn new_with_color(position: Vector3, color: Vector3) -> Self {
        let normal = Vector3::new(0.0, 0.0, 1.0);
        Self {
            position,
            normal,
            color,
            transformed_position: position,
            transformed_normal: normal,
        }
    }
}
