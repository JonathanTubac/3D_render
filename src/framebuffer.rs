use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use raylib::prelude::Image;

use crate::color::Color;

/// Framebuffer con buffer de color y z-buffer (buffer de profundidad).
///
/// Convenciones:
/// - El origen (0, 0) es la esquina superior izquierda; `y` crece hacia abajo.
/// - Profundidad: un valor de `z` MENOR significa más cerca de la cámara.
///   El z-buffer se limpia a `f32::INFINITY`.
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    buffer: Vec<Color>,
    zbuffer: Vec<f32>,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Framebuffer {
            width,
            height,
            buffer: vec![Color::BLACK; size],
            zbuffer: vec![f32::INFINITY; size],
            background_color: Color::BLACK,
            current_color: Color::WHITE,
        }
    }

    /// Llena el buffer de color con el color de fondo y reinicia el z-buffer.
    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
        self.zbuffer.fill(f32::INFINITY);
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        Some(y as usize * self.width + x as usize)
    }

    /// Pinta un pixel con el color actual, sin prueba de profundidad.
    /// Las coordenadas fuera de pantalla se ignoran.
    pub fn point(&mut self, x: i32, y: i32) {
        if let Some(i) = self.index(x, y) {
            self.buffer[i] = self.current_color;
        }
    }

    /// Pinta un pixel solo si `depth` está más cerca que lo que ya hay en el z-buffer.
    /// Devuelve `true` si el pixel se pintó.
    pub fn point_with_depth(&mut self, x: i32, y: i32, depth: f32) -> bool {
        match self.index(x, y) {
            Some(i) if depth < self.zbuffer[i] => {
                self.zbuffer[i] = depth;
                self.buffer[i] = self.current_color;
                true
            }
            _ => false,
        }
    }

    pub fn get_color(&self, x: i32, y: i32) -> Option<Color> {
        self.index(x, y).map(|i| self.buffer[i])
    }

    pub fn get_depth(&self, x: i32, y: i32) -> Option<f32> {
        self.index(x, y).map(|i| self.zbuffer[i])
    }

    /// Copia el buffer de color a una `Image` de Raylib, para mostrarlo en una
    /// ventana o exportarlo como PNG.
    pub fn to_image(&self) -> Image {
        let mut image = Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.buffer[y * self.width + x];
                image.draw_pixel(x as i32, y as i32, c);
            }
        }
        image
    }

    /// Guarda el buffer de color como BMP de 24 bits sin compresión.
    pub fn write_bmp<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut out = BufWriter::new(File::create(path)?);
        write_bmp_data(&mut out, self.width, self.height, |x, y| {
            self.buffer[y * self.width + x]
        })?;
        out.flush()
    }

    /// Guarda el z-buffer como imagen en escala de grises (blanco = más cerca).
    /// Útil para depurar. Los pixeles sin profundidad quedan negros.
    pub fn write_depth_bmp<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let finite = self.zbuffer.iter().copied().filter(|z| z.is_finite());
        let min = finite.clone().fold(f32::INFINITY, f32::min);
        let max = finite.fold(f32::NEG_INFINITY, f32::max);
        let range = if max > min { max - min } else { 1.0 };

        let mut out = BufWriter::new(File::create(path)?);
        write_bmp_data(&mut out, self.width, self.height, |x, y| {
            let z = self.zbuffer[y * self.width + x];
            if !z.is_finite() {
                return Color::BLACK;
            }
            let v = 1.0 - (z - min) / range;
            // Se deja un mínimo de 0.2 para distinguir "lejos" de "vacío".
            let v = 0.2 + 0.8 * v;
            Color::from_float(v, v, v)
        })?;
        out.flush()
    }
}

/// Escribe cabeceras y pixeles de un BMP de 24 bits.
/// `pixel(x, y)` usa la convención de y hacia abajo; BMP guarda las filas de abajo hacia arriba.
fn write_bmp_data<W: Write>(
    out: &mut W,
    width: usize,
    height: usize,
    pixel: impl Fn(usize, usize) -> Color,
) -> io::Result<()> {
    const FILE_HEADER_SIZE: u32 = 14;
    const INFO_HEADER_SIZE: u32 = 40;

    let row_size = (width * 3 + 3) & !3; // cada fila se rellena a múltiplo de 4 bytes
    let padding = row_size - width * 3;
    let image_size = (row_size * height) as u32;
    let offset = FILE_HEADER_SIZE + INFO_HEADER_SIZE;

    // BITMAPFILEHEADER
    out.write_all(b"BM")?;
    out.write_all(&(offset + image_size).to_le_bytes())?;
    out.write_all(&0u32.to_le_bytes())?; // reservado
    out.write_all(&offset.to_le_bytes())?;

    // BITMAPINFOHEADER
    out.write_all(&INFO_HEADER_SIZE.to_le_bytes())?;
    out.write_all(&(width as i32).to_le_bytes())?;
    out.write_all(&(height as i32).to_le_bytes())?;
    out.write_all(&1u16.to_le_bytes())?; // planos
    out.write_all(&24u16.to_le_bytes())?; // bits por pixel
    out.write_all(&0u32.to_le_bytes())?; // sin compresión
    out.write_all(&image_size.to_le_bytes())?;
    out.write_all(&2835i32.to_le_bytes())?; // 72 DPI horizontal
    out.write_all(&2835i32.to_le_bytes())?; // 72 DPI vertical
    out.write_all(&0u32.to_le_bytes())?; // colores en paleta
    out.write_all(&0u32.to_le_bytes())?; // colores importantes

    let pad = [0u8; 3];
    for y in (0..height).rev() {
        for x in 0..width {
            let c = pixel(x, y);
            out.write_all(&[c.b, c.g, c.r])?; // BMP usa orden BGR
        }
        out.write_all(&pad[..padding])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_fills_background_and_resets_depth() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_background_color(Color::new(10, 20, 30));
        fb.point_with_depth(1, 1, 0.5);
        fb.clear();
        assert_eq!(fb.get_color(1, 1), Some(Color::new(10, 20, 30)));
        assert_eq!(fb.get_depth(1, 1), Some(f32::INFINITY));
    }

    #[test]
    fn zbuffer_keeps_closest_fragment() {
        let mut fb = Framebuffer::new(2, 2);
        fb.set_current_color(Color::new(255, 0, 0));
        assert!(fb.point_with_depth(0, 0, 5.0));
        fb.set_current_color(Color::new(0, 255, 0));
        assert!(!fb.point_with_depth(0, 0, 10.0)); // más lejos: se descarta
        assert_eq!(fb.get_color(0, 0), Some(Color::new(255, 0, 0)));
        assert!(fb.point_with_depth(0, 0, 1.0)); // más cerca: se pinta
        assert_eq!(fb.get_color(0, 0), Some(Color::new(0, 255, 0)));
        assert_eq!(fb.get_depth(0, 0), Some(1.0));
    }

    #[test]
    fn out_of_bounds_is_ignored() {
        let mut fb = Framebuffer::new(2, 2);
        fb.point(-1, 0);
        fb.point(2, 0);
        assert!(!fb.point_with_depth(0, 5, 0.0));
        assert_eq!(fb.get_color(5, 5), None);
    }

    #[test]
    fn bmp_has_expected_size_with_padding() {
        // ancho 3 -> 9 bytes por fila, relleno a 12
        let mut bytes = Vec::new();
        write_bmp_data(&mut bytes, 3, 2, |_, _| Color::WHITE).unwrap();
        assert_eq!(bytes.len(), 54 + 12 * 2);
        assert_eq!(&bytes[0..2], b"BM");
    }
}
