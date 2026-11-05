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

// ---------------------------------------------------------------------------
// Planeta rocoso: Thalassa
// ---------------------------------------------------------------------------

/// Tamaño de los continentes: mayor = continentes más pequeños y numerosos.
const ROCOSO_ESCALA_CONTINENTES: f32 = 1.5;
/// Nivel del mar: mayor = más océano y menos tierra.
const ROCOSO_NIVEL_MAR: f32 = 0.06;
/// Latitud (|y| en la esfera) donde empiezan los casquetes polares.
const ROCOSO_LATITUD_HIELO: f32 = 0.8;
/// Tamaño de las montañas del relieve: mayor = montañas más pequeñas.
const ROCOSO_ESCALA_RELIEVE: f32 = 7.0;
/// Fuerza del relieve: cuánto inclina las normales. 0 = superficie lisa.
const ROCOSO_FUERZA_RELIEVE: f32 = 0.45;
/// Brillo del reflejo especular del océano.
const ROCOSO_BRILLO_OCEANO: f32 = 0.9;
/// Exponente especular del océano: mayor = reflejo más pequeño y concentrado.
const ROCOSO_EXPONENTE_OCEANO: f32 = 70.0;

/// Superficie de Thalassa en un punto, antes de iluminarla.
struct Superficie {
    color: Vector3,
    /// Normal en las coordenadas del modelo, ya con el relieve aplicado.
    normal: Vector3,
    /// 1 en mar abierto, 0 en tierra o hielo.
    oceano: f32,
    /// Qué tan habitable es el punto, de 0 a 1 (tierra baja, sin hielo ni desierto).
    habitable: f32,
}

/// Altura de las montañas que usa el relieve.
fn altura_montanas(r: &Ruidos, p: Vector3) -> f32 {
    ruido(&r.fbm_suave, add(scale(p, ROCOSO_ESCALA_RELIEVE), v3(11.0, 0.0, 0.0)))
}

/// Relieve procedural (bump mapping): inclina la normal según la pendiente del
/// campo de altura de las montañas, sin mover ningún vértice.
///
/// La pendiente se estima con diferencias finitas: cuánto cambia la altura al
/// moverse un poco en x, y y z. Se descarta la parte de la pendiente que apunta
/// hacia afuera de la esfera y lo que queda se resta de la normal: la normal se
/// inclina alejándose de las cuestas, como en una montaña real.
fn relieve(r: &Ruidos, p: Vector3, fuerza: f32) -> Vector3 {
    const EPS: f32 = 0.004;
    let h0 = altura_montanas(r, p);
    let gradiente = scale(
        v3(
            altura_montanas(r, add(p, v3(EPS, 0.0, 0.0))) - h0,
            altura_montanas(r, add(p, v3(0.0, EPS, 0.0))) - h0,
            altura_montanas(r, add(p, v3(0.0, 0.0, EPS))) - h0,
        ),
        1.0 / EPS,
    );
    let tangente = sub(gradiente, scale(p, vec3::dot(gradiente, p)));
    vec3::normalize(sub(p, scale(tangente, fuerza / ROCOSO_ESCALA_RELIEVE)))
}

fn superficie_thalassa(r: &Ruidos, p: Vector3) -> Superficie {
    // Continentes: fBm sobre coordenadas deformadas, para costas irregulares.
    let q = scale(p, ROCOSO_ESCALA_CONTINENTES);
    let (wx, wy, wz) = r.deformacion.domain_warp_3d(q.x, q.y, q.z);
    let h = ruido(&r.fbm, v3(wx, wy, wz));

    // Casquetes polares con borde irregular.
    let latitud = p.y.abs() + 0.08 * ruido(&r.fbm_suave, scale(p, 5.0));
    let hielo = suave(ROCOSO_LATITUD_HIELO, ROCOSO_LATITUD_HIELO + 0.04, latitud);
    let color_hielo = rgb(0xEEF4FA);

    if h < ROCOSO_NIVEL_MAR {
        // Océano: más claro cerca de la costa y oscuro en lo profundo.
        let profundidad = ROCOSO_NIVEL_MAR - h;
        let agua = rampa(
            profundidad,
            &[(0.0, rgb(0x3FC9B9)), (0.05, rgb(0x1A8A9E)), (0.18, rgb(0x0E4A7A)), (0.4, rgb(0x081C45))],
        );
        return Superficie {
            color: mezclar(agua, color_hielo, hielo),
            normal: p,
            oceano: 1.0 - hielo,
            habitable: 0.0,
        };
    }

    // Tierra. `e` es la elevación sobre el nivel del mar, de 0 a ~1.
    let e = (h - ROCOSO_NIVEL_MAR) / 0.45;
    // Humedad: separa bosques de desiertos.
    let humedad = ruido(&r.fbm_suave, add(scale(p, 2.2), v3(0.0, 7.0, 0.0)));
    let montana = altura_montanas(r, p);

    let vegetacion = mezclar(rgb(0x4C8A3A), rgb(0x1F4D2A), suave(-0.1, 0.25, humedad + 0.3 * montana));
    let desierto = mezclar(rgb(0xE0B96E), rgb(0xB5713E), suave(-0.2, 0.2, montana));
    let mut color = mezclar(desierto, vegetacion, suave(-0.12, 0.02, humedad));
    // Playa en la costa, roca y nieve en lo alto.
    color = mezclar(rgb(0xE8D8A8), color, suave(0.0, 0.05, e));
    color = mezclar(color, rgb(0x6E6258), suave(0.45, 0.65, e + 0.2 * montana));
    let nieve = suave(0.75, 0.82, e + 0.15 * montana).max(hielo);
    color = mezclar(color, color_hielo, nieve);

    // Más relieve en las zonas altas.
    let fuerza = ROCOSO_FUERZA_RELIEVE * (0.35 + e).min(1.2);
    Superficie {
        color,
        normal: relieve(r, p, fuerza),
        oceano: 0.0,
        habitable: (1.0 - nieve) * suave(-0.12, 0.02, humedad) * (1.0 - suave(0.25, 0.45, e)),
    }
}

/// Planeta rocoso "Thalassa": un mundo oceánico con archipiélagos, desiertos,
/// montañas nevadas y casquetes polares.
///
/// El océano y la tierra reaccionan distinto a la luz: el agua tiene un reflejo
/// especular y la tierra es mate y muestra relieve.
pub fn planeta_rocoso(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let r = ruidos();
    let p = direccion(fragment);
    let s = superficie_thalassa(r, p);

    // Iluminación en el espacio de la vista.
    let n = vec3::normalize(rotar(&u.normal_matrix, s.normal));
    let luz = u.light_dir;
    let difusa = vec3::dot(n, luz).max(0.0);

    let mut color = scale(s.color, 0.03 + 0.97 * difusa);

    // Reflejo especular (Blinn-Phong) solo en el océano. El observador está en
    // +z, así que el vector medio entre la luz y la vista es normalize(L + V).
    if s.oceano > 0.0 {
        let medio = vec3::normalize(add(luz, v3(0.0, 0.0, 1.0)));
        let especular = vec3::dot(n, medio).max(0.0).powf(ROCOSO_EXPONENTE_OCEANO);
        let reflejo = ROCOSO_BRILLO_OCEANO * especular * s.oceano * suave(0.0, 0.1, difusa);
        color = add(color, scale(rgb(0xFFF1D6), reflejo));
    }
    let _ = s.habitable;
    color
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
