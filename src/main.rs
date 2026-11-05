mod color;
mod cuerpos;
mod fragment;
mod framebuffer;
mod laboratorio;
mod line;
mod obj;
mod planetas;
mod primitive;
mod rasterizer;
mod render;
mod shaders;
mod triangle;
mod vec3;
mod vertex;
// Ejercicio de la fase de vértices; el renderer no lo utiliza.
#[allow(dead_code)]
mod vertices;
mod visor;

use raylib::prelude::*;
use rayon::prelude::*;

use framebuffer::Framebuffer;
use primitive::primitive_assembly;
use rasterizer::{rasterize, TriangleMode};
use fragment::Fragment;
use shaders::{fragment_shader, vertex_shader};
use vertex::Vertex;

/// Parámetros que comparten todos los vértices y fragmentos de un mismo dibujo.
/// Pueden cambiar entre un dibujo y otro, pero no durante el dibujo.
pub struct Uniforms {
    /// Lleva las posiciones del modelo a coordenadas de pantalla.
    pub model_matrix: Matrix,
    /// Solo la rotación del modelo: lleva las normales del modelo al espacio de
    /// la vista (x a la derecha, y hacia arriba, z hacia el observador).
    pub normal_matrix: Matrix,
    /// Dirección hacia la luz en el espacio de la vista, de largo 1.
    pub light_dir: Vector3,
    /// Segundos transcurridos; los shaders lo usan para animar.
    pub time: f32,
    /// Fracción del planeta cubierta por nubes, de 0.0 a 1.0.
    pub cloud_coverage: f32,
}

impl Uniforms {
    /// Uniforms con la matriz de modelo dada y valores neutros para lo demás:
    /// sin rotación, luz desde el observador y tiempo cero.
    pub fn new(model_matrix: Matrix) -> Self {
        Self {
            model_matrix,
            normal_matrix: Matrix::identity(),
            light_dir: Vector3::new(0.0, 0.0, 1.0),
            time: 0.0,
            cloud_coverage: 0.5,
        }
    }
}

/// Firma de un fragment shader: recibe un fragmento y los uniforms del dibujo y
/// devuelve su color.
pub type FragmentShader = fn(&Fragment, &Uniforms) -> Vector3;

/// Convierte los vértices en una imagen siguiendo el pipeline:
/// vértices → triángulos → fragmentos → framebuffer.
///
/// `mode` indica si la rasterización genera solo el contorno de cada triángulo o
/// también su interior, y con qué algoritmo. `shader` es el fragment shader que
/// decide el color de cada fragmento.
fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    mode: TriangleMode,
    shader: FragmentShader,
) {
    // 1. Procesamiento de vértices
    //    El vertex shader se ejecuta una vez por vértice, con los mismos uniforms
    //    para todos; deja el resultado en `transformed_position`.
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas
    //    Cada grupo de tres vértices consecutivos forma un triángulo. En los modos
    //    rellenos se descartan los que miran hacia atrás (sus tres normales se
    //    alejan del observador): quedarían ocultos y solo costarían tiempo.
    let mut triangles = primitive_assembly(&transformed_vertices);
    let rellenar = mode != TriangleMode::Wireframe;
    if rellenar {
        triangles.retain(|t| t.iter().any(|v| v.transformed_normal.z >= 0.0));
    }

    // 3. Rasterización
    //    Cada triángulo genera los fragmentos que ocupa en la imagen.
    let fragments = rasterize(&triangles, mode);

    // 4. Procesamiento de fragmentos
    //    El fragment shader decide el color de cada fragmento. El color de un
    //    fragmento no depende de los demás, así que los shaders se ejecutan en
    //    paralelo en todos los núcleos del CPU (`par_iter` de rayon). Después
    //    cada color se escribe en orden en el framebuffer; los modos rellenos
    //    usan el z-buffer para que quede la superficie más cercana.
    let colors: Vec<Vector3> = fragments.par_iter().map(|fragment| shader(fragment, uniforms)).collect();

    for (fragment, color) in fragments.iter().zip(colors) {
        let x = fragment.position.x as i32;
        let y = fragment.position.y as i32;

        if rellenar {
            framebuffer.point_with_depth(x, y, fragment.depth, color);
        } else {
            framebuffer.point(x, y, color);
        }
    }
}

/// Tecla 1, 2 o 3: cambia el modo de rasterización.
pub fn leer_modo(window: &RaylibHandle, mode: &mut TriangleMode) -> bool {
    let anterior = *mode;
    if window.is_key_pressed(KeyboardKey::KEY_ONE) {
        *mode = TriangleMode::Wireframe;
    }
    if window.is_key_pressed(KeyboardKey::KEY_TWO) {
        *mode = TriangleMode::FilledBarycentric;
    }
    if window.is_key_pressed(KeyboardKey::KEY_THREE) {
        *mode = TriangleMode::FilledScanline;
    }
    *mode != anterior
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |nombre: &str| args.iter().any(|a| a == nombre);

    let resultado = if flag("--nave") {
        // Visor de la nave del Proyecto 1 (con `--captura`, solo guarda la imagen).
        visor::ejecutar(flag("--captura"))
    } else if flag("--benchmark-triangles") {
        // Mide la rasterización de cada modo, sin abrir la ventana.
        laboratorio::benchmark()
    } else if flag("--capturas-relleno") {
        // Guarda una imagen de la escena de prueba del relleno en cada modo.
        laboratorio::capturas();
        Ok(())
    } else if flag("--relleno") {
        // Escena de prueba del laboratorio de relleno de triángulos.
        ventana_relleno();
        Ok(())
    } else if flag("--capturas-planetas") {
        // Guarda una imagen de cada cuerpo celeste.
        planetas::capturas()
    } else {
        // Laboratorio de planetas y estrellas.
        planetas::ejecutar()
    };

    if let Err(e) = resultado {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Ventana con la escena de prueba del laboratorio de relleno de triángulos.
fn ventana_relleno() {
    let (mut window, thread) = raylib::init()
        .size(laboratorio::ANCHO as i32, laboratorio::ALTO as i32)
        .title("Relleno de triangulos")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(laboratorio::ANCHO, laboratorio::ALTO);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    // Las posiciones ya están en coordenadas de pantalla; con la matriz
    // identidad, el procesamiento de vértices las deja iguales.
    let vertex_array = laboratorio::escena_de_prueba();

    let uniforms = Uniforms::new(Matrix::identity());
    let mut mode = TriangleMode::FilledScanline;

    while !window.window_should_close() {
        leer_modo(&window, &mut mode);

        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array, mode, fragment_shader);
        let texto = format!("1: Wireframe  2: FilledBarycentric  3: FilledScanline   Modo: {}", mode.nombre());
        framebuffer.swap_buffers_with(&mut window, &thread, |d| {
            d.draw_text(&texto, 10, 10, 20, Color::LIGHTGRAY);
        });
    }
}
