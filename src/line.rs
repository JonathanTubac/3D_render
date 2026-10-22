use crate::fragment::Fragment;
use crate::vertex::Vertex;

/// Genera los fragmentos de una línea entre las posiciones transformadas de `a` y `b`.
///
/// Usa el algoritmo de Bresenham: dentro del ciclo solo hay sumas y comparaciones
/// de enteros, sin divisiones ni flotantes, porque se ejecuta una vez por pixel.
/// Todos los fragmentos usan el color y la profundidad de `a`.
pub fn line(a: &Vertex, b: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::with_capacity(fragment_count(a, b));
    line_into(a, b, &mut fragments);
    fragments
}

/// Cantidad de fragmentos que genera la línea de `a` a `b`: uno por cada paso
/// en su eje más largo.
pub fn fragment_count(a: &Vertex, b: &Vertex) -> usize {
    let dx = (b.transformed_position.x.round() - a.transformed_position.x.round()).abs();
    let dy = (b.transformed_position.y.round() - a.transformed_position.y.round()).abs();
    dx.max(dy) as usize + 1
}

/// Igual que `line`, pero agrega los fragmentos a `fragments` en lugar de crear
/// un `Vec` nuevo. Así `triangle` puede reservar memoria una sola vez para sus
/// tres lados, lo que importa cuando se dibujan cientos de miles de triángulos.
pub fn line_into(a: &Vertex, b: &Vertex, fragments: &mut Vec<Fragment>) {
    let mut x1 = a.transformed_position.x.round() as i32;
    let mut y1 = a.transformed_position.y.round() as i32;

    let x2 = b.transformed_position.x.round() as i32;
    let y2 = b.transformed_position.y.round() as i32;

    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    let depth = a.transformed_position.z;

    loop {
        fragments.push(Fragment::new(x1 as f32, y1 as f32, a.color, depth));
        if x1 == x2 && y1 == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::math::Vector3;

    fn v(x: f32, y: f32) -> Vertex {
        Vertex::new(Vector3::new(x, y, 0.0))
    }

    fn contiene(fragments: &[Fragment], x: f32, y: f32) -> bool {
        fragments.iter().any(|f| f.position.x == x && f.position.y == y)
    }

    #[test]
    fn includes_both_endpoints_in_any_direction() {
        let f = line(&v(8.0, 7.0), &v(1.0, 2.0));
        assert!(contiene(&f, 8.0, 7.0));
        assert!(contiene(&f, 1.0, 2.0));
    }

    #[test]
    fn one_fragment_per_step_on_major_axis() {
        assert_eq!(line(&v(0.0, 0.0), &v(15.0, 4.0)).len(), 16);
        assert_eq!(line(&v(3.0, 19.0), &v(5.0, 0.0)).len(), 20);
    }

    #[test]
    fn fragment_count_matches_generated_fragments() {
        for (a, b) in [((0.0, 0.0), (15.0, 4.0)), ((3.0, 19.0), (5.0, 0.0)), ((2.4, 2.6), (2.0, 3.0))] {
            let (a, b) = (v(a.0, a.1), v(b.0, b.1));
            assert_eq!(fragment_count(&a, &b), line(&a, &b).len());
        }
    }

    #[test]
    fn degenerate_line_is_a_single_fragment() {
        assert_eq!(line(&v(2.4, 2.6), &v(2.0, 3.0)).len(), 1);
    }

    #[test]
    fn uses_transformed_position_and_vertex_color() {
        let rojo = Vector3::new(1.0, 0.0, 0.0);
        let mut a = Vertex::new_with_color(Vector3::new(0.0, 0.0, 0.0), rojo);
        a.transformed_position = Vector3::new(5.0, 5.0, 0.0);
        let f = line(&a, &a);
        assert_eq!((f[0].position.x, f[0].position.y), (5.0, 5.0));
        assert_eq!(f[0].color, rojo);
    }
}
