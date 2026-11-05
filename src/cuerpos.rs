//! Fragment shaders procedurales de los cuerpos celestes. Los tres usan la misma
//! esfera; solo cambian las reglas con que cada shader calcula el color.
//!
//! Convenciones:
//! - `p` es la dirección del fragmento desde el centro de la esfera, en las
//!   coordenadas del modelo (`normalize(vertex_position)`). Los patrones se
//!   calculan con `p`, así quedan pegados a la superficie aunque la esfera gire.
//! - La esfera proporcionada tiene pocas caras y normales planas, así que la
//!   normal de iluminación se calcula como `p` (la normal de una esfera perfecta).
//! - El ruido devuelve valores aproximadamente entre -0.45 y 0.45 (fBm) y se
//!   reescala a [0, 1] donde hace falta.

use std::sync::OnceLock;

use fastnoise_lite::{CellularReturnType, DomainWarpType, FastNoiseLite, FractalType, NoiseType};
use raylib::prelude::{Matrix, Vector3};

use crate::fragment::Fragment;
use crate::vec3::{self, add, scale, sub, v3};
use crate::Uniforms;

// ---------------------------------------------------------------------------
// Herramientas
// ---------------------------------------------------------------------------

/// Interpolación lineal entre dos colores.
fn mezclar(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    vec3::lerp(a, b, t.clamp(0.0, 1.0))
}

/// 0 antes de `borde0`, 1 después de `borde1` y una transición suave en medio.
fn suave(borde0: f32, borde1: f32, x: f32) -> f32 {
    let t = ((x - borde0) / (borde1 - borde0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Rampa de colores: `paradas` son pares (posición, color) ordenados; devuelve el
/// color de `t` interpolando entre las dos paradas que lo rodean.
fn rampa(t: f32, paradas: &[(f32, Vector3)]) -> Vector3 {
    let (primera, ultima) = (paradas[0], paradas[paradas.len() - 1]);
    if t <= primera.0 {
        return primera.1;
    }
    for par in paradas.windows(2) {
        let ((t0, c0), (t1, c1)) = (par[0], par[1]);
        if t <= t1 {
            return mezclar(c0, c1, (t - t0) / (t1 - t0));
        }
    }
    ultima.1
}

fn rgb(hex: u32) -> Vector3 {
    let canal = |desplazamiento: u32| ((hex >> desplazamiento) & 0xFF) as f32 / 255.0;
    v3(canal(16), canal(8), canal(0))
}

/// Rota `v` con la matriz de rotación `m` (columnas `m0..m2`, `m4..m6`, `m8..m10`):
/// lleva una dirección del modelo al espacio de la vista.
fn rotar(m: &Matrix, v: Vector3) -> Vector3 {
    v3(
        m.m0 * v.x + m.m4 * v.y + m.m8 * v.z,
        m.m1 * v.x + m.m5 * v.y + m.m9 * v.z,
        m.m2 * v.x + m.m6 * v.y + m.m10 * v.z,
    )
}

/// Rotación inversa (la transpuesta): lleva una dirección de la vista al modelo.
fn rotar_inversa(m: &Matrix, v: Vector3) -> Vector3 {
    v3(
        m.m0 * v.x + m.m1 * v.y + m.m2 * v.z,
        m.m4 * v.x + m.m5 * v.y + m.m6 * v.z,
        m.m8 * v.x + m.m9 * v.y + m.m10 * v.z,
    )
}

/// Rota `p` alrededor del eje y.
fn rotar_y(p: Vector3, angulo: f32) -> Vector3 {
    let (s, c) = angulo.sin_cos();
    v3(p.x * c + p.z * s, p.y, -p.x * s + p.z * c)
}

fn ruido(r: &FastNoiseLite, p: Vector3) -> f32 {
    r.get_noise_3d(p.x, p.y, p.z)
}

/// Generadores de ruido. Se configuran una sola vez y se comparten entre todos
/// los fragmentos (y entre los hilos que ejecutan los shaders).
struct Ruidos {
    /// OpenSimplex2 con 5 octavas de fBm: detalle a varias escalas.
    fbm: FastNoiseLite,
    /// OpenSimplex2 con 3 octavas: formas grandes y suaves.
    fbm_suave: FastNoiseLite,
    /// Deformación del dominio con OpenSimplex2: tuerce las coordenadas.
    deformacion: FastNoiseLite,
    /// Ruido celular: distancia al borde de la celda más cercana (F2 - F1).
    celdas: FastNoiseLite,
}

impl Ruidos {
    fn new() -> Self {
        let fbm = |semilla: i32, octavas: i32| {
            let mut r = FastNoiseLite::with_seed(semilla);
            r.set_noise_type(Some(NoiseType::OpenSimplex2));
            r.set_fractal_type(Some(FractalType::FBm));
            r.set_fractal_octaves(Some(octavas));
            r.set_frequency(Some(1.0));
            r
        };

        let mut deformacion = FastNoiseLite::with_seed(7);
        deformacion.set_domain_warp_type(Some(DomainWarpType::OpenSimplex2));
        deformacion.set_domain_warp_amp(Some(1.0));
        deformacion.set_frequency(Some(1.0));

        let mut celdas = FastNoiseLite::with_seed(11);
        celdas.set_noise_type(Some(NoiseType::Cellular));
        celdas.set_cellular_return_type(Some(CellularReturnType::Distance2Sub));
        celdas.set_frequency(Some(1.0));

        Ruidos { fbm: fbm(1, 5), fbm_suave: fbm(3, 3), deformacion, celdas }
    }
}

fn ruidos() -> &'static Ruidos {
    static RUIDOS: OnceLock<Ruidos> = OnceLock::new();
    RUIDOS.get_or_init(Ruidos::new)
}

/// Dirección del fragmento sobre la esfera, en las coordenadas del modelo.
fn direccion(fragment: &Fragment) -> Vector3 {
    vec3::normalize(fragment.vertex_position)
}

// ---------------------------------------------------------------------------
// Estrella: Ascua
// ---------------------------------------------------------------------------

/// Tamaño de los remolinos de plasma: mayor = remolinos más pequeños.
const ESTRELLA_ESCALA_PLASMA: f32 = 2.4;
/// Cuánto tuerce la deformación del dominio al plasma.
const ESTRELLA_TURBULENCIA: f32 = 1.2;
/// Tamaño de las celdas de convección (granulación): mayor = celdas más pequeñas.
const ESTRELLA_ESCALA_GRANOS: f32 = 16.0;
/// Velocidad con la que el plasma y las celdas se transforman.
const ESTRELLA_VELOCIDAD: f32 = 0.12;
/// Umbral de las manchas dentro de una región activa: mayor = manchas más pequeñas.
const ESTRELLA_UMBRAL_MANCHAS: f32 = 0.12;
/// Umbral de las regiones activas: mayor = menos regiones con manchas.
const ESTRELLA_UMBRAL_REGIONES: f32 = 0.17;

/// Estrella "Ascua": una superficie de plasma en ebullición con celdas de
/// convección, grupos de manchas frías y el borde más oscuro y rojizo que el
/// centro.
///
/// Es emisiva: no usa la luz de la escena.
pub fn estrella(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let r = ruidos();
    let p = direccion(fragment);

    // El ruido se muestrea en un punto que se desplaza con el tiempo. La esfera
    // atraviesa un campo 3D en movimiento, así que los patrones no se deslizan
    // en bloque: cambian de forma, como algo que hierve.
    let deriva = scale(v3(1.0, 0.6, -0.8), u.time * ESTRELLA_VELOCIDAD);

    // 1. Plasma: fBm sobre coordenadas torcidas por la deformación del dominio.
    let q = add(scale(p, ESTRELLA_ESCALA_PLASMA), deriva);
    let (wx, wy, wz) = r.deformacion.domain_warp_3d(q.x, q.y, q.z);
    let torcido = vec3::lerp(q, v3(wx, wy, wz), ESTRELLA_TURBULENCIA);
    let plasma = (ruido(&r.fbm, torcido) / 0.45).clamp(-1.0, 1.0); // -1..1

    // 2. Granulación: celdas de convección con el borde más frío. La distancia
    //    al borde (F2 - F1) vale -1 justo en el borde.
    let g = add(scale(p, ESTRELLA_ESCALA_GRANOS), scale(deriva, 3.0));
    let borde_celda = 1.0 - suave(-1.0, -0.55, ruido(&r.celdas, g)); // 1 en el borde

    // 3. Manchas: zonas frías con un centro oscuro (umbra) y un halo (penumbra).
    //    Un ruido de frecuencia baja marca las regiones activas y uno más fino
    //    da forma a las manchas; solo hay manchas donde coinciden los dos, así
    //    salen pocas y en grupos, como en el Sol.
    let region = ruido(&r.fbm_suave, add(scale(p, 1.3), v3(4.0, 0.0, 0.0)));
    let activa = suave(ESTRELLA_UMBRAL_REGIONES, ESTRELLA_UMBRAL_REGIONES + 0.12, region);
    let forma = ruido(&r.fbm_suave, add(scale(p, 6.0), scale(deriva, 0.3)));
    let umbra = activa * suave(ESTRELLA_UMBRAL_MANCHAS + 0.05, ESTRELLA_UMBRAL_MANCHAS + 0.1, forma);
    let penumbra = activa * suave(ESTRELLA_UMBRAL_MANCHAS - 0.04, ESTRELLA_UMBRAL_MANCHAS + 0.05, forma);

    // Temperatura de 0 (fría) a 1 (caliente).
    let mut temperatura =
        0.8 + 0.22 * plasma - 0.1 * borde_celda * (1.0 - penumbra) - 0.25 * penumbra - 0.4 * umbra;

    // 4. Oscurecimiento del borde: en el borde se ve plasma más alto y frío.
    //    `mu` es el coseno entre la normal y la dirección hacia el observador.
    let mu = rotar(&u.normal_matrix, p).z.max(0.0);
    temperatura -= 0.35 * (1.0 - mu);

    let color = rampa(
        temperatura,
        &[
            (0.0, rgb(0x1E0200)),
            (0.25, rgb(0x7A1200)),
            (0.45, rgb(0xD9400A)),
            (0.62, rgb(0xFF8A1F)),
            (0.8, rgb(0xFFD36B)),
            (1.0, rgb(0xFFF8E1)),
        ],
    );
    scale(color, 0.6 + 0.4 * mu.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rampa_interpola_entre_paradas() {
        let negro = Vector3::new(0.0, 0.0, 0.0);
        let blanco = Vector3::new(1.0, 1.0, 1.0);
        let paradas = [(0.0, negro), (1.0, blanco)];
        assert_eq!(rampa(-1.0, &paradas), negro);
        assert_eq!(rampa(0.5, &paradas), Vector3::new(0.5, 0.5, 0.5));
        assert_eq!(rampa(2.0, &paradas), blanco);
    }

    #[test]
    fn rotar_inversa_deshace_rotar() {
        let m = Matrix::rotate_y(0.7) * Matrix::rotate_x(-0.4);
        let v = Vector3::new(0.3, -0.5, 0.8);
        let w = rotar_inversa(&m, rotar(&m, v));
        assert!((w - v).length() < 1e-5);
    }

    #[test]
    fn rotar_coincide_con_raylib() {
        let m = Matrix::rotate_y(0.7) * Matrix::rotate_x(-0.4);
        let v = Vector3::new(0.3, -0.5, 0.8);
        assert!((rotar(&m, v) - v.transform(m)).length() < 1e-5);
    }
}
