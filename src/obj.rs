use std::fs;
use std::io;

use raylib::prelude::Vector3;

/// Modelo cargado de un archivo OBJ.
///
/// `indices` se lee de tres en tres: cada grupo `[a, b, c]` es un triángulo y
/// cada valor es una posición dentro de `vertices` (ya convertida a base 0).
pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<usize>,
}

impl Obj {
    /// Cantidad de triángulos del modelo.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Devuelve los tres vértices del triángulo número `i`.
    pub fn triangle(&self, i: usize) -> [Vector3; 3] {
        let t = &self.indices[i * 3..i * 3 + 3];
        [self.vertices[t[0]], self.vertices[t[1]], self.vertices[t[2]]]
    }
}

/// Lee un archivo OBJ y devuelve sus posiciones (`v`) y triángulos (`f`).
/// Cualquier otro tipo de línea (`vt`, `vn`, `o`, `mtllib`, comentarios...) se ignora.
pub fn load_obj(path: &str) -> io::Result<Obj> {
    let contenido = fs::read_to_string(path)?;
    parse_obj(&contenido)
}

fn parse_obj(contenido: &str) -> io::Result<Obj> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (n, linea) in contenido.lines().enumerate() {
        let num_linea = n + 1;
        // split_whitespace tolera varios espacios o tabs entre valores.
        let mut partes = linea.split_whitespace();

        match partes.next() {
            Some("v") => {
                let x = parse_f32(partes.next(), num_linea)?;
                let y = parse_f32(partes.next(), num_linea)?;
                let z = parse_f32(partes.next(), num_linea)?;
                vertices.push(Vector3::new(x, y, z));
            }
            Some("f") => {
                let cara = partes
                    .map(|grupo| parse_indice(grupo, num_linea))
                    .collect::<io::Result<Vec<usize>>>()?;
                if cara.len() < 3 {
                    return Err(error(num_linea, "una cara necesita al menos 3 vértices"));
                }
                // Si la cara ya es un triángulo esto agrega solo (0, 1, 2).
                // Si trae más vértices, se divide en abanico: (0, i, i+1).
                for i in 1..cara.len() - 1 {
                    indices.extend_from_slice(&[cara[0], cara[i], cara[i + 1]]);
                }
            }
            _ => {}
        }
    }

    if let Some(&max) = indices.iter().max() {
        if max >= vertices.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("índice {} fuera de rango ({} vértices)", max + 1, vertices.len()),
            ));
        }
    }

    Ok(Obj { vertices, indices })
}

fn parse_f32(texto: Option<&str>, num_linea: usize) -> io::Result<f32> {
    let texto = texto.ok_or_else(|| error(num_linea, "faltan coordenadas en el vértice"))?;
    texto
        .parse::<f32>()
        .map_err(|_| error(num_linea, &format!("'{texto}' no es un número")))
}

/// Convierte un grupo como `5`, `5/7` o `5/7/2` en el índice de vértice en base 0.
fn parse_indice(grupo: &str, num_linea: usize) -> io::Result<usize> {
    // Solo nos interesa lo que está antes de la primera '/'.
    let texto = grupo.split('/').next().unwrap_or("");
    let indice: usize = texto
        .parse()
        .map_err(|_| error(num_linea, &format!("'{grupo}' no es un índice válido")))?;
    // OBJ cuenta desde 1 y Vec desde 0, por eso se resta 1. El 0 no existe en OBJ.
    indice
        .checked_sub(1)
        .ok_or_else(|| error(num_linea, "los índices OBJ comienzan en 1"))
}

fn error(num_linea: usize, mensaje: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("línea {num_linea}: {mensaje}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_triangulated_cube() {
        let obj = load_obj("assets/cube.obj").unwrap();
        assert_eq!(obj.vertices.len(), 8);
        assert_eq!(obj.indices.len(), 36);
        assert_eq!(obj.triangle_count(), 12);

        // Primera cara: f 5/1/1 3/2/1 1/3/1  ->  vértices 5, 3, 1  ->  índices 4, 2, 0
        assert_eq!(&obj.indices[0..3], &[4, 2, 0]);
        let [a, b, c] = obj.triangle(0);
        assert_eq!(a, Vector3::new(-0.516802, 1.627034, -0.292704));
        assert_eq!(b, Vector3::new(1.568286, 0.015753, 0.735004));
        assert_eq!(c, Vector3::new(1.125986, 0.781798, -1.058748));
    }

    #[test]
    fn accepts_plain_and_slashed_indices() {
        let obj = parse_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\nf 3//1 2/5 1/4/2\n").unwrap();
        assert_eq!(obj.indices, vec![0, 1, 2, 2, 1, 0]);
    }

    #[test]
    fn splits_quads_into_two_triangles() {
        let obj = parse_obj("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n").unwrap();
        assert_eq!(obj.indices, vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse_obj("v 1 2\n").is_err());
        assert!(parse_obj("v 1 dos 3\n").is_err());
        assert!(parse_obj("v 0 0 0\nf 0 1 1\n").is_err());
        assert!(parse_obj("v 0 0 0\nf 1 1 2\n").is_err());
    }
}
