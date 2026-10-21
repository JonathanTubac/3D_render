/// Color RGB de 8 bits por canal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    /// Crea un color a partir de un hex 0xRRGGBB.
    pub const fn from_hex(hex: u32) -> Self {
        Color {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }

    /// Crea un color a partir de valores flotantes en el rango [0.0, 1.0].
    pub fn from_float(r: f32, g: f32, b: f32) -> Self {
        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        Color::new(to_u8(r), to_u8(g), to_u8(b))
    }
}

impl From<Color> for raylib::ffi::Color {
    fn from(c: Color) -> Self {
        raylib::ffi::Color { r: c.r, g: c.g, b: c.b, a: 255 }
    }
}

/// Convierte un color guardado como `Vector3` (r, g, b en [0.0, 1.0]), como el de
/// un fragmento, a un color de 8 bits por canal.
impl From<raylib::math::Vector3> for Color {
    fn from(v: raylib::math::Vector3) -> Self {
        Color::from_float(v.x, v.y, v.z)
    }
}
