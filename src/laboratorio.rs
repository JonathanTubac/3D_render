//! Laboratorio de relleno de triángulos: escena de prueba, capturas de cada modo
//! y medición del tiempo de rasterización.

use std::hint::black_box;
use std::io;
use std::time::{Duration, Instant};

use raylib::prelude::*;

use crate::fragment::Fragment;
use crate::framebuffer::Framebuffer;
use crate::obj::load_obj;
use crate::primitive::primitive_assembly;
use crate::rasterizer::{rasterize, TriangleMode};
use crate::render::{centrar, uniforms_camara, Camara, Encuadre};
use crate::shaders::vertex_shader;
use crate::vertex::Vertex;
use crate::{render, Uniforms};

pub const ANCHO: usize = 800;
pub const ALTO: usize = 600;

/// Los casos que pide el laboratorio, en coordenadas de pantalla. Cada triángulo
/// tiene su propio color para distinguirlos.
pub fn escena_de_prueba() -> Vec<Vertex> {
    let triangulos: [([f32; 6], u32); 7] = [
        // Pequeño.
        ([60.0, 60.0, 78.0, 68.0, 64.0, 82.0], 0xFFD23F),
        // Grande.
        ([130.0, 110.0, 520.0, 170.0, 250.0, 520.0], 0x3A7BFF),
        // Con un borde horizontal.
        ([560.0, 60.0, 760.0, 60.0, 650.0, 220.0], 0x7CFF6B),
        // Con un borde vertical.
        ([580.0, 240.0, 580.0, 400.0, 740.0, 320.0], 0xFF5FD2),
        // El mismo triángulo con sus vértices en orden inverso, desplazado
        // 180 píxeles hacia abajo: debe verse igual.
        ([740.0, 500.0, 580.0, 580.0, 580.0, 420.0], 0xB57CFF),
        // Parcialmente fuera del framebuffer.
        ([-80.0, 380.0, 140.0, 450.0, 30.0, 700.0], 0xFF9A3C),
        // Tres vértices sobre una misma línea: relleno no genera fragmentos.
        ([40.0, 250.0, 100.0, 300.0, 160.0, 350.0], 0xFF4040),
    ];

    triangulos
        .iter()
        .flat_map(|(p, hex)| {
            let color = color_hex(*hex);
            [
                Vertex::new_with_color(Vector3::new(p[0], p[1], 0.0), color),
                Vertex::new_with_color(Vector3::new(p[2], p[3], 0.0), color),
                Vertex::new_with_color(Vector3::new(p[4], p[5], 0.0), color),
            ]
        })
        .collect()
}

fn color_hex(hex: u32) -> Vector3 {
    let canal = |desplazamiento: u32| ((hex >> desplazamiento) & 0xFF) as f32 / 255.0;
    Vector3::new(canal(16), canal(8), canal(0))
}

/// Guarda una imagen de la escena de prueba en cada modo.
pub fn capturas() {
    let uniforms = Uniforms { model_matrix: Matrix::identity() };
    let escena = escena_de_prueba();
    for mode in TriangleMode::TODOS {
        let mut framebuffer = Framebuffer::new(ANCHO, ALTO);
        framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &escena, mode);
        let archivo = format!("docs/relleno_{}.png", mode.nombre().to_lowercase());
        framebuffer.to_image().export_image(&archivo);
        println!("Guardada {archivo}");
    }
}

/// Conjunto de triángulos ya transformados sobre el que se mide la rasterización.
struct Caso {
    nombre: &'static str,
    triangles: Vec<[Vertex; 3]>,
    /// Ejecuciones por medición: más para los casos pequeños, para que cada
    /// medición dure lo suficiente como para ser confiable.
    ejecuciones: u32,
}

/// Mide cuánto tarda `rasterize` con cada modo.
///
/// Solo se mide la generación de fragmentos: los triángulos se preparan antes y
/// no se dibuja nada en una ventana. Cada medición ejecuta `rasterize` varias
/// veces seguidas; se repite la medición y se reporta la mediana.
pub fn benchmark() -> io::Result<()> {
    const MEDICIONES: usize = 7;

    let casos = [
        Caso {
            nombre: "Escena de prueba (7 triángulos)",
            triangles: primitive_assembly(&escena_de_prueba()),
            ejecuciones: 2000,
        },
        Caso { nombre: "Triángulos grandes (500)", triangles: triangulos_grandes(500), ejecuciones: 20 },
        Caso { nombre: "Nave (573,439 triángulos)", triangles: triangulos_nave()?, ejecuciones: 3 },
    ];

    println!("Mediana de {MEDICIONES} mediciones; cada medición ejecuta rasterize varias veces.\n");
    println!("| Caso | Modo | Tiempo por ejecución | Fragmentos | ns por fragmento |");
    println!("|---|---|---:|---:|---:|");

    for caso in &casos {
        for mode in TriangleMode::TODOS {
            let fragmentos = rasterize(&caso.triangles, mode).len();

            let mut tiempos: Vec<Duration> = (0..MEDICIONES)
                .map(|_| {
                    let inicio = Instant::now();
                    for _ in 0..caso.ejecuciones {
                        let fragments = rasterize(black_box(&caso.triangles), mode);
                        black_box(fragments);
                    }
                    inicio.elapsed() / caso.ejecuciones
                })
                .collect();
            tiempos.sort();
            let mediana = tiempos[MEDICIONES / 2];

            println!(
                "| {} | {} | {:.3?} | {} | {:.2} |",
                caso.nombre,
                mode.nombre(),
                mediana,
                fragmentos,
                mediana.as_nanos() as f64 / fragmentos.max(1) as f64
            );
        }
    }

    println!();
    for caso in &casos {
        let baricentrico = posiciones(&rasterize(&caso.triangles, TriangleMode::FilledBarycentric));
        let scanline = posiciones(&rasterize(&caso.triangles, TriangleMode::FilledScanline));
        let iguales = baricentrico == scanline;
        println!(
            "{}: posiciones de FilledBarycentric y FilledScanline {}",
            caso.nombre,
            if iguales { "idénticas" } else { "DIFERENTES" }
        );
    }
    Ok(())
}

/// Posiciones de los fragmentos ordenadas, para comparar dos algoritmos sin
/// depender del orden en que generan los fragmentos.
fn posiciones(fragments: &[Fragment]) -> Vec<(i32, i32)> {
    let mut p: Vec<(i32, i32)> = fragments
        .iter()
        .map(|f| (f.position.x as i32, f.position.y as i32))
        .collect();
    p.sort_unstable();
    p
}

/// Triángulos grandes repartidos por la pantalla, siempre los mismos.
fn triangulos_grandes(cantidad: usize) -> Vec<[Vertex; 3]> {
    // Generador pseudoaleatorio simple, con semilla fija para que las mediciones
    // sean comparables entre ejecuciones.
    let mut semilla: u32 = 2024;
    let mut azar = move || {
        semilla = semilla.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (semilla >> 8) as f32 / (1 << 24) as f32
    };

    (0..cantidad)
        .map(|_| {
            let centro = Vector2::new(azar() * ANCHO as f32, azar() * ALTO as f32);
            let tamano = 50.0 + azar() * 250.0;
            let mut vertice = || {
                let p = Vector3::new(
                    centro.x + (azar() - 0.5) * tamano,
                    centro.y + (azar() - 0.5) * tamano,
                    0.0,
                );
                Vertex::new(p)
            };
            [vertice(), vertice(), vertice()]
        })
        .collect()
}

/// Triángulos de la nave ya transformados, vista de frente en una pantalla de
/// 1024 × 768 (como la captura del visor).
fn triangulos_nave() -> io::Result<Vec<[Vertex; 3]>> {
    let (ancho, alto) = (1024.0, 768.0);
    let mut modelo = load_obj("spaceship.obj")?;
    centrar(&mut modelo.vertices);
    let encuadre = Encuadre::new(&modelo.vertices, ancho, alto);
    let uniforms = uniforms_camara(&Camara::default(), &encuadre, ancho, alto);

    let transformed: Vec<Vertex> = modelo
        .get_vertex_array()
        .iter()
        .map(|v| vertex_shader(v, &uniforms))
        .collect();
    Ok(primitive_assembly(&transformed))
}
