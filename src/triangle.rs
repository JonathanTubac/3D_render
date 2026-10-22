use raylib::math::{Vector2, Vector3};

use crate::fragment::Fragment;
use crate::line::{fragment_count, line_into};
use crate::vertex::Vertex;

/// Genera los fragmentos del contorno de un triángulo: las tres líneas que
/// conectan sus vértices. No rellena el interior.
pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    triangle_into(v1, v2, v3, &mut fragments);
    fragments
}

/// Igual que `triangle`, pero agrega los fragmentos a `fragments`. Así la
/// rasterización de un modelo completo junta todos sus fragmentos en un solo
/// `Vec`, sin crear y copiar uno por cada triángulo.
pub fn triangle_into(v1: &Vertex, v2: &Vertex, v3: &Vertex, fragments: &mut Vec<Fragment>) {
    // Se reserva de una vez el espacio de los tres lados para no volver a pedir
    // memoria mientras se agregan los fragmentos.
    fragments.reserve(fragment_count(v1, v2) + fragment_count(v2, v3) + fragment_count(v3, v1));
    line_into(v1, v2, fragments);
    line_into(v2, v3, fragments);
    line_into(v3, v1, fragments);
}

/// Triángulo relleno usando coordenadas baricéntricas.
pub fn triangle_filled_barycentric(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    triangle_filled_barycentric_into(v1, v2, v3, &mut fragments);
    fragments
}

/// Triángulo relleno avanzando por líneas horizontales (scanline).
pub fn triangle_filled_scanline(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    triangle_filled_scanline_into(v1, v2, v3, &mut fragments);
    fragments
}

/// Relleno con coordenadas baricéntricas: recorre el rectángulo que envuelve al
/// triángulo y, para el centro de cada píxel, calcula sus coordenadas
/// baricéntricas; si las tres indican que está adentro, genera un fragmento.
pub fn triangle_filled_barycentric_into(v1: &Vertex, v2: &Vertex, v3: &Vertex, fragments: &mut Vec<Fragment>) {
    let Some(t) = Relleno::new(v1, v2, v3) else { return };
    fragments.reserve(t.fragmentos_estimados());

    for y in t.y_min..=t.y_max {
        for x in t.x_min..=t.x_max {
            if t.dentro(x, y) {
                fragments.push(Fragment::new(x as f32, y as f32, t.color, t.depth));
            }
        }
    }
}

/// Relleno por líneas horizontales: para cada fila calcula dónde la cruzan las
/// aristas del triángulo y genera los fragmentos del tramo entre esos extremos,
/// sin evaluar uno por uno los píxeles del interior.
pub fn triangle_filled_scanline_into(v1: &Vertex, v2: &Vertex, v3: &Vertex, fragments: &mut Vec<Fragment>) {
    let Some(t) = Relleno::new(v1, v2, v3) else { return };
    fragments.reserve(t.fragmentos_estimados());

    for y in t.y_min..=t.y_max {
        let Some((inicio, fin)) = t.tramo(y) else { continue };
        for x in inicio..=fin {
            fragments.push(Fragment::new(x as f32, y as f32, t.color, t.depth));
        }
    }
}

/// Datos que comparten los dos rellenos, y la regla con la que ambos deciden si
/// el centro de un píxel está dentro del triángulo.
///
/// Usar exactamente la misma regla hace que los dos algoritmos generen las
/// mismas posiciones; solo cambia la forma de recorrerlas.
struct Relleno {
    /// Vértices en pantalla, ordenados para que `area` sea positiva.
    p: [Vector2; 3],
    /// Si cada arista es superior o izquierda (regla top-left, ver `pasa`).
    top_left: [bool; 3],
    /// Doble del área del triángulo; siempre mayor que cero.
    area: f32,
    /// Píxeles cuyo centro puede caer dentro del triángulo.
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
    /// Por ahora todos los fragmentos usan el color y la profundidad del primer vértice.
    color: Vector3,
    depth: f32,
}

impl Relleno {
    /// Devuelve `None` si el triángulo es degenerado (área cero: sus tres
    /// vértices están sobre una misma línea), porque no cubre ningún píxel y
    /// las coordenadas baricéntricas requerirían dividir entre cero.
    fn new(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Option<Self> {
        let a = Vector2::new(v1.transformed_position.x, v1.transformed_position.y);
        let mut b = Vector2::new(v2.transformed_position.x, v2.transformed_position.y);
        let mut c = Vector2::new(v3.transformed_position.x, v3.transformed_position.y);

        let mut area = edge(a, b, c);
        if area == 0.0 || !area.is_finite() {
            return None;
        }
        // Si los vértices vienen en el orden contrario, se intercambian dos:
        // invertir el orden no debe hacer desaparecer el triángulo.
        if area < 0.0 {
            std::mem::swap(&mut b, &mut c);
            area = -area;
        }
        let p = [a, b, c];

        // Un píxel (x, y) tiene su centro en (x + 0.5, y + 0.5). Estos son los
        // píxeles cuyo centro cae dentro del rectángulo que envuelve al triángulo.
        let min_x = a.x.min(b.x).min(c.x);
        let max_x = a.x.max(b.x).max(c.x);
        let min_y = a.y.min(b.y).min(c.y);
        let max_y = a.y.max(b.y).max(c.y);

        Some(Relleno {
            p,
            top_left: [es_top_left(b, c), es_top_left(c, a), es_top_left(a, b)],
            area,
            x_min: (min_x - 0.5).ceil() as i32,
            x_max: (max_x - 0.5).floor() as i32,
            y_min: (min_y - 0.5).ceil() as i32,
            y_max: (max_y - 0.5).floor() as i32,
            color: v1.color,
            depth: v1.transformed_position.z,
        })
    }

    /// ¿El centro del píxel (x, y) está dentro del triángulo?
    ///
    /// `w0`, `w1`, `w2` son las coordenadas baricéntricas sin normalizar: las
    /// coordenadas baricéntricas son `w0 / area`, `w1 / area` y `w2 / area`.
    /// Como `area` es positiva, tienen el mismo signo, así que basta revisar `w`.
    /// No se divide aquí porque solo hace falta saber si están adentro; la
    /// división servirá cuando interpolemos atributos.
    #[inline]
    fn dentro(&self, x: i32, y: i32) -> bool {
        let centro = Vector2::new(x as f32 + 0.5, y as f32 + 0.5);
        let [a, b, c] = self.p;
        let w0 = edge(b, c, centro);
        let w1 = edge(c, a, centro);
        let w2 = edge(a, b, centro);
        pasa(w0, self.top_left[0]) && pasa(w1, self.top_left[1]) && pasa(w2, self.top_left[2])
    }

    /// Primer y último píxel de la fila `y` cuyo centro está dentro del triángulo.
    fn tramo(&self, y: i32) -> Option<(i32, i32)> {
        let yc = y as f32 + 0.5;

        // Dónde cruza la línea horizontal y = yc a cada arista.
        let mut izquierda = f32::INFINITY;
        let mut derecha = f32::NEG_INFINITY;
        for (a, b) in [(self.p[0], self.p[1]), (self.p[1], self.p[2]), (self.p[2], self.p[0])] {
            if yc < a.y.min(b.y) || yc > a.y.max(b.y) {
                continue; // la fila no toca esta arista
            }
            if a.y == b.y {
                // Arista horizontal sobre la fila: cuentan sus dos extremos.
                izquierda = izquierda.min(a.x.min(b.x));
                derecha = derecha.max(a.x.max(b.x));
            } else {
                let x = a.x + (yc - a.y) * (b.x - a.x) / (b.y - a.y);
                izquierda = izquierda.min(x);
                derecha = derecha.max(x);
            }
        }
        if izquierda > derecha {
            return None;
        }

        // Píxeles cuyo centro cae entre los dos cruces.
        let mut inicio = ((izquierda - 0.5).ceil() as i32).max(self.x_min);
        let mut fin = ((derecha - 0.5).floor() as i32).min(self.x_max);

        // La división anterior puede tener errores de redondeo, y un centro que
        // cae justo sobre una arista depende de la regla top-left. Por eso los
        // extremos se ajustan con la misma prueba `dentro` del relleno
        // baricéntrico. Solo se revisan los extremos: el interior del tramo
        // nunca se evalúa píxel por píxel.
        while inicio > self.x_min && self.dentro(inicio - 1, y) {
            inicio -= 1;
        }
        while inicio <= fin && !self.dentro(inicio, y) {
            inicio += 1;
        }
        while fin < self.x_max && self.dentro(fin + 1, y) {
            fin += 1;
        }
        while fin >= inicio && !self.dentro(fin, y) {
            fin -= 1;
        }

        (inicio <= fin).then_some((inicio, fin))
    }

    /// Cantidad aproximada de fragmentos (el área del triángulo), para reservar
    /// memoria una sola vez.
    fn fragmentos_estimados(&self) -> usize {
        (self.area / 2.0) as usize + 1
    }
}

/// Función de arista: positiva si `c` está de un lado de la recta que va de `a`
/// a `b`, negativa del otro y cero sobre ella. Su valor es el doble del área
/// (con signo) del triángulo `a`, `b`, `c`.
#[inline]
fn edge(a: Vector2, b: Vector2, c: Vector2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// Regla top-left: un centro que cae exactamente sobre una arista se dibuja solo
/// si la arista es superior o izquierda.
///
/// Así, cuando dos triángulos comparten una arista, cada píxel de esa arista
/// pertenece a uno solo de ellos: no quedan huecos ni fragmentos duplicados.
#[inline]
fn pasa(w: f32, top_left: bool) -> bool {
    w > 0.0 || (w == 0.0 && top_left)
}

/// Con los vértices ordenados para que el área sea positiva (y crece hacia abajo),
/// una arista es superior si es horizontal y va hacia la derecha, e izquierda
/// si sube.
fn es_top_left(a: Vector2, b: Vector2) -> bool {
    let d = Vector2::new(b.x - a.x, b.y - a.y);
    (d.y == 0.0 && d.x > 0.0) || d.y < 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vertex {
        Vertex::new(Vector3::new(x, y, 0.0))
    }

    /// Posiciones de los fragmentos, ordenadas, para comparar algoritmos.
    fn posiciones(fragments: &[Fragment]) -> Vec<(i32, i32)> {
        let mut p: Vec<(i32, i32)> = fragments
            .iter()
            .map(|f| (f.position.x as i32, f.position.y as i32))
            .collect();
        p.sort_unstable();
        p
    }

    fn baricentrico(t: &[Vertex; 3]) -> Vec<(i32, i32)> {
        posiciones(&triangle_filled_barycentric(&t[0], &t[1], &t[2]))
    }

    fn scanline(t: &[Vertex; 3]) -> Vec<(i32, i32)> {
        posiciones(&triangle_filled_scanline(&t[0], &t[1], &t[2]))
    }

    /// Los casos que pide el laboratorio.
    fn casos() -> Vec<(&'static str, [Vertex; 3])> {
        vec![
            ("pequeño", [v(10.2, 10.1), v(14.7, 12.3), v(11.5, 15.9)]),
            ("grande", [v(40.0, 30.0), v(760.0, 120.0), v(300.0, 580.0)]),
            ("borde horizontal", [v(100.0, 100.0), v(300.0, 100.0), v(180.0, 250.0)]),
            ("borde vertical", [v(100.0, 100.0), v(100.0, 300.0), v(260.0, 180.0)]),
            ("parcialmente fuera", [v(-50.0, -30.0), v(120.0, 40.0), v(30.0, 150.0)]),
        ]
    }

    #[test]
    fn both_algorithms_produce_the_same_positions() {
        for (nombre, t) in casos() {
            let b = baricentrico(&t);
            assert!(!b.is_empty(), "{nombre}: vacío");
            assert_eq!(b, scanline(&t), "{nombre}: los algoritmos difieren");
        }
    }

    #[test]
    fn reversed_order_produces_the_same_triangle() {
        for (nombre, [a, b, c]) in casos() {
            let invertido = [c, b, a];
            assert_eq!(baricentrico(&[a, b, c]), baricentrico(&invertido), "{nombre}");
            assert_eq!(scanline(&[a, b, c]), scanline(&invertido), "{nombre}");
        }
    }

    #[test]
    fn collinear_vertices_produce_no_fragments() {
        let t = [v(10.0, 10.0), v(50.0, 30.0), v(90.0, 50.0)];
        assert!(baricentrico(&t).is_empty());
        assert!(scanline(&t).is_empty());
    }

    #[test]
    fn no_duplicated_positions_inside_a_triangle() {
        for (nombre, t) in casos() {
            let mut p = scanline(&t);
            let total = p.len();
            p.dedup();
            assert_eq!(p.len(), total, "{nombre}");
        }
    }

    #[test]
    fn shared_edge_has_no_gaps_and_no_duplicates() {
        // Un cuadrado de 10 × 10 dividido en dos triángulos por su diagonal.
        // La diagonal pasa justo por centros de píxeles: sin una regla
        // consistente quedarían huecos o fragmentos repetidos.
        let (a, b, c, d) = (v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 10.0));
        for relleno in [baricentrico as fn(&[Vertex; 3]) -> Vec<(i32, i32)>, scanline] {
            let mut p = relleno(&[a, b, c]);
            p.extend(relleno(&[a, c, d]));
            p.sort_unstable();
            let total = p.len();
            p.dedup();
            assert_eq!(total, 100, "fragmentos repetidos o faltantes");
            assert_eq!(p.len(), 100);
        }
    }

    #[test]
    fn small_horizontal_and_vertical_cases_cover_the_expected_pixels() {
        // Triángulo rectángulo con catetos de 4 píxeles sobre la cuadrícula. La
        // hipotenusa pasa justo por los centros con x + y = 3, pero es una arista
        // inferior derecha, así que la regla top-left los deja fuera.
        let t = [v(0.0, 0.0), v(4.0, 0.0), v(0.0, 4.0)];
        let esperado = vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (2, 0)];
        assert_eq!(baricentrico(&t), esperado);
        assert_eq!(scanline(&t), esperado);
    }

    #[test]
    fn many_random_triangles_match() {
        // Generador pseudoaleatorio simple para no depender de otra librería.
        let mut semilla: u32 = 12345;
        let mut azar = move || {
            semilla = semilla.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (semilla >> 8) as f32 / (1 << 24) as f32
        };
        for _ in 0..500 {
            let escala = if azar() < 0.5 { 20.0 } else { 300.0 };
            let mut punto = || v(azar() * escala - 10.0, azar() * escala - 10.0);
            let t = [punto(), punto(), punto()];
            assert_eq!(baricentrico(&t), scanline(&t));
        }
    }

    #[test]
    fn wireframe_contains_the_three_edges_and_not_the_interior() {
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
