//! Laboratorio de planetas y estrellas: muestra la esfera proporcionada con el
//! shader de cada cuerpo celeste.

use std::io;
use std::time::Instant;

use raylib::prelude::*;

use crate::cuerpos;
use crate::framebuffer::Framebuffer;
use crate::obj::load_obj;
use crate::rasterizer::TriangleMode;
use crate::render::{centrar, uniforms_camara, Camara, Encuadre};
use crate::vertex::Vertex;
use crate::{render, FragmentShader, Uniforms};

pub const ANCHO: usize = 800;
pub const ALTO: usize = 800;
const MODELO: &str = "assets/sphere.obj";

/// Inclinación inicial de la esfera, para ver un poco del polo norte.
const INCLINACION_INICIAL: f32 = 0.35;
/// Ángulo inicial de la luz alrededor del planeta (0 = desde el observador).
const LUZ_AZIMUT_INICIAL: f32 = -0.9;
/// Altura de la luz sobre el ecuador de la vista.
const LUZ_ELEVACION: f32 = 0.35;
/// Radianes por segundo que se mueve la luz o gira la esfera con el teclado.
const VELOCIDAD_TECLAS: f32 = 1.2;
const SENSIBILIDAD_MOUSE: f32 = 0.01;

/// Los cuerpos celestes del laboratorio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cuerpo {
    Estrella,
    Rocoso,
    Gaseoso,
}

impl Cuerpo {
    pub const TODOS: [Cuerpo; 3] = [Cuerpo::Estrella, Cuerpo::Rocoso, Cuerpo::Gaseoso];

    pub fn nombre(self) -> &'static str {
        match self {
            Cuerpo::Estrella => "Ascua (estrella)",
            Cuerpo::Rocoso => "Thalassa (planeta rocoso)",
            Cuerpo::Gaseoso => "Lirio (gigante gaseoso)",
        }
    }

    pub fn archivo(self) -> &'static str {
        match self {
            Cuerpo::Estrella => "estrella",
            Cuerpo::Rocoso => "rocoso",
            Cuerpo::Gaseoso => "gaseoso",
        }
    }

    pub fn shader(self) -> FragmentShader {
        match self {
            Cuerpo::Estrella => cuerpos::estrella,
            Cuerpo::Rocoso => cuerpos::planeta_rocoso,
            Cuerpo::Gaseoso => cuerpos::estrella,
        }
    }
}

/// Estado de la escena: lo que el usuario controla con el teclado y el mouse.
#[derive(Debug, Clone, Copy)]
pub struct Escena {
    pub cuerpo: Cuerpo,
    pub camara: Camara,
    pub luz_azimut: f32,
    pub tiempo: f32,
    pub cobertura_nubes: f32,
}

impl Default for Escena {
    fn default() -> Self {
        Escena {
            cuerpo: Cuerpo::Estrella,
            camara: Camara { yaw: 0.0, pitch: INCLINACION_INICIAL, zoom: 1.0 },
            luz_azimut: LUZ_AZIMUT_INICIAL,
            tiempo: 0.0,
            cobertura_nubes: 0.4,
        }
    }
}

impl Escena {
    /// Dirección hacia la luz, en el espacio de la vista.
    pub fn direccion_luz(&self) -> Vector3 {
        let (s, c) = self.luz_azimut.sin_cos();
        let (se, ce) = LUZ_ELEVACION.sin_cos();
        Vector3::new(s * ce, se, c * ce).normalize()
    }

    pub fn uniforms(&self, encuadre: &Encuadre) -> Uniforms {
        let mut uniforms = uniforms_camara(&self.camara, encuadre, ANCHO as f32, ALTO as f32);
        uniforms.light_dir = self.direccion_luz();
        uniforms.time = self.tiempo;
        uniforms.cloud_coverage = self.cobertura_nubes;
        uniforms
    }
}

/// La esfera lista para dibujar y su encuadre en la pantalla.
pub struct Esfera {
    pub vertex_array: Vec<Vertex>,
    pub encuadre: Encuadre,
}

impl Esfera {
    pub fn cargar() -> io::Result<Self> {
        let mut modelo = load_obj(MODELO)?;
        centrar(&mut modelo.vertices);
        let encuadre = Encuadre::new(&modelo.vertices, ANCHO as f32, ALTO as f32);
        Ok(Esfera { vertex_array: modelo.get_vertex_array(), encuadre })
    }

    /// Dibuja el fondo y el cuerpo celeste de `escena` en el framebuffer.
    pub fn dibujar(&self, framebuffer: &mut Framebuffer, escena: &Escena, fondo: &[(i32, i32, f32)]) {
        framebuffer.clear();
        for &(x, y, brillo) in fondo {
            framebuffer.point(x, y, Vector3::new(brillo, brillo, brillo * 1.1));
        }
        let uniforms = escena.uniforms(&self.encuadre);
        render(
            framebuffer,
            &uniforms,
            &self.vertex_array,
            TriangleMode::FilledBarycentric,
            escena.cuerpo.shader(),
        );
    }
}

/// Estrellas fijas del fondo: posición y brillo, siempre las mismas.
pub fn fondo_estrellado() -> Vec<(i32, i32, f32)> {
    let mut semilla: u32 = 99;
    let mut azar = move || {
        semilla = semilla.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (semilla >> 8) as f32 / (1 << 24) as f32
    };
    (0..350)
        .map(|_| {
            let x = (azar() * ANCHO as f32) as i32;
            let y = (azar() * ALTO as f32) as i32;
            // La mayoría tenues y unas pocas brillantes.
            let brillo = 0.15 + 0.85 * azar().powi(6);
            (x, y, brillo)
        })
        .collect()
}

pub fn nuevo_framebuffer() -> Framebuffer {
    let mut framebuffer = Framebuffer::new(ANCHO, ALTO);
    framebuffer.set_background_color(Vector3::new(0.01, 0.01, 0.03));
    framebuffer
}

const AYUDA: &str = "1/2/3: cuerpo   Arrastrar/flechas: girar   A/D: luz   W/S: nubes   Espacio: pausa";

/// Abre la ventana con los cuerpos celestes.
pub fn ejecutar() -> io::Result<()> {
    let esfera = Esfera::cargar()?;
    let fondo = fondo_estrellado();

    let (mut window, thread) = raylib::init()
        .size(ANCHO as i32, ALTO as i32)
        .title("Planetas y estrellas")
        .build();
    window.set_target_fps(60);

    // Se crea después de la ventana para que su textura se libere antes de cerrarla.
    let mut framebuffer = nuevo_framebuffer();
    framebuffer.init_texture(&mut window, &thread);

    let mut escena = Escena::default();
    let mut pausa = false;

    while !window.window_should_close() {
        let dt = window.get_frame_time();
        leer_controles(&window, &mut escena, &mut pausa, dt);
        if !pausa {
            escena.tiempo += dt;
        }

        let inicio = Instant::now();
        esfera.dibujar(&mut framebuffer, &escena, &fondo);
        let ms = inicio.elapsed().as_secs_f32() * 1000.0;

        let titulo = escena.cuerpo.nombre();
        let estado = format!(
            "Nubes: {:.0} %   Dibujo: {ms:.0} ms{}",
            escena.cobertura_nubes * 100.0,
            if pausa { "   (pausa)" } else { "" }
        );
        framebuffer.swap_buffers_with(&mut window, &thread, |d| {
            d.draw_text(titulo, 10, 10, 24, Color::WHITE);
            d.draw_text(&estado, 10, 40, 18, Color::LIGHTGRAY);
            d.draw_text(AYUDA, 10, ALTO as i32 - 26, 16, Color::GRAY);
        });
    }
    Ok(())
}

fn leer_controles(window: &RaylibHandle, escena: &mut Escena, pausa: &mut bool, dt: f32) {
    for (tecla, cuerpo) in [
        (KeyboardKey::KEY_ONE, Cuerpo::Estrella),
        (KeyboardKey::KEY_TWO, Cuerpo::Rocoso),
        (KeyboardKey::KEY_THREE, Cuerpo::Gaseoso),
    ] {
        if window.is_key_pressed(tecla) {
            escena.cuerpo = cuerpo;
        }
    }

    if window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        let delta = window.get_mouse_delta();
        escena.camara.yaw += delta.x * SENSIBILIDAD_MOUSE;
        escena.camara.pitch += delta.y * SENSIBILIDAD_MOUSE;
    }

    // -1, 0 o 1 según cuál de las dos teclas está presionada.
    let eje = |menos: KeyboardKey, mas: KeyboardKey| {
        window.is_key_down(mas) as i32 as f32 - window.is_key_down(menos) as i32 as f32
    };
    let paso = VELOCIDAD_TECLAS * dt;
    escena.camara.yaw += paso * eje(KeyboardKey::KEY_LEFT, KeyboardKey::KEY_RIGHT);
    escena.camara.pitch += paso * eje(KeyboardKey::KEY_UP, KeyboardKey::KEY_DOWN);
    escena.luz_azimut += paso * eje(KeyboardKey::KEY_A, KeyboardKey::KEY_D);
    escena.cobertura_nubes =
        (escena.cobertura_nubes + 0.4 * dt * eje(KeyboardKey::KEY_S, KeyboardKey::KEY_W)).clamp(0.0, 1.0);
    if window.is_key_pressed(KeyboardKey::KEY_SPACE) {
        *pausa = !*pausa;
    }
}

/// Guarda una imagen de cada cuerpo celeste en `docs/planetas/`.
pub fn capturas() -> io::Result<()> {
    let esfera = Esfera::cargar()?;
    let fondo = fondo_estrellado();
    std::fs::create_dir_all("docs/planetas")?;
    for cuerpo in Cuerpo::TODOS {
        let escena = Escena { cuerpo, tiempo: 3.0, ..Escena::default() };
        let mut framebuffer = nuevo_framebuffer();
        let inicio = Instant::now();
        esfera.dibujar(&mut framebuffer, &escena, &fondo);
        println!("{}: {:.1?}", cuerpo.nombre(), inicio.elapsed());
        let archivo = format!("docs/planetas/{}.png", cuerpo.archivo());
        framebuffer.to_image().export_image(&archivo);
    }
    Ok(())
}

/// Imágenes que muestran cada efecto con una configuración pensada para verlo.
pub fn evidencias() -> io::Result<()> {
    let esfera = Esfera::cargar()?;
    let fondo = fondo_estrellado();
    std::fs::create_dir_all("docs/planetas")?;

    let base = Escena { tiempo: 3.0, ..Escena::default() };
    let rocoso = Escena { cuerpo: Cuerpo::Rocoso, ..base };
    let casos = [
        // Luz desde atrás a la izquierda: casi todo lo visible es de noche.
        ("rocoso_noche", Escena { luz_azimut: -2.3, cobertura_nubes: 0.3, ..rocoso }),
        // La misma vista con poca y con mucha cobertura de nubes.
        ("rocoso_nubes_0", Escena { cobertura_nubes: 0.0, ..rocoso }),
        ("rocoso_nubes_80", Escena { cobertura_nubes: 0.8, ..rocoso }),
    ];
    for (archivo, escena) in casos {
        let mut framebuffer = nuevo_framebuffer();
        esfera.dibujar(&mut framebuffer, &escena, &fondo);
        framebuffer.to_image().export_image(&format!("docs/planetas/{archivo}.png"));
        println!("Guardada docs/planetas/{archivo}.png");
    }
    Ok(())
}
