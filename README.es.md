# Spaceship Render

[English](README.md) · **Español**

Renderer por software escrito en Rust para el curso de Gráficas por Computadora (UVG).
Carga un modelo `.obj` hecho en Blender y dibuja el contorno de todos sus triángulos
con algoritmos propios. Raylib solo se usa para abrir la ventana, mostrar el
framebuffer como textura y guardar la captura en PNG; **no** se usan sus funciones
para cargar ni dibujar modelos.

## Proyecto 1 — Parte 1: dibujando un modelo OBJ

![Nave renderizada en wireframe](screenshot.png)

| | |
|---|---|
| ![Vista de cerca](screenshot1.png) | ![Vista de frente](screenshot2.png) |

Modelo: `spaceship.obj`, 286,689 vértices y 573,439 triángulos.

Como todavía no hay proyección, se dibujan directamente las componentes `x` y `y`
de cada vértice (vista de frente). Los vértices se escalan con la misma escala en
ambos ejes y se trasladan para que el modelo quede centrado y ocupe el 90 % de la
ventana sin salirse.

## Cómo ejecutarlo

Requisitos: Rust (toolchain MSVC en Windows) y CMake, que el crate `raylib` usa para compilar Raylib.

Para abrir el visor de la nave:

```sh
cargo run --release -- --nave
```

Abre una ventana de 1024×768 con el modelo. Usa `--release`: en modo debug dibujar
573k triángulos es mucho más lento.

### Controles

| Entrada | Acción |
|---|---|
| Arrastrar con clic izquierdo / flechas | Rotar el modelo |
| Rueda del mouse | Zoom |
| `R` | Regresar a la vista inicial |

La ventana arranca con el zoom necesario para que el modelo quepa completo desde
cualquier ángulo. Solo se vuelve a dibujar cuando la cámara se mueve.

Para guardar la vista de frente en `captura.png` sin abrir la ventana:

```sh
cargo run --release -- --nave --captura
```

Sin `--nave` (`cargo run`) se abre la prueba "primer triángulo" del curso: dos contornos
blancos de triángulos definidos directamente en coordenadas de pantalla.

Para correr los tests:

```sh
cargo test
```

> **Windows:** `.cargo/config.toml` le indica a `bindgen` dónde están los headers de
> MSVC porque en la máquina de desarrollo no los encontraba solo. Si tu instalación de
> Visual Studio está en otra ruta, ajústala ahí (o borra el archivo si compila sin él).

## Cómo funciona

```
spaceship.obj → Obj → arreglo de vértices → matriz de modelo (Uniforms) → triángulos → fragmentos → framebuffer → ventana
```

| Archivo | Qué hace |
|---|---|
| `src/obj.rs` | Cargador de OBJ: lee las líneas `v` (posiciones) y `f` (caras), convierte los índices de base 1 a base 0 y acepta grupos como `5/7/2`. `get_vertex_array()` devuelve los vértices ordenados por caras. |
| `src/vertex.rs` | `Vertex` con su `position` original, `color` y `transformed_position` en coordenadas de pantalla. |
| `src/render.rs` | Matriz de modelo (`create_model_matrix`), cámara orbital, encuadre y la transformación que llena `transformed_position`. |
| `src/triangle.rs` | `triangle` devuelve los fragmentos del contorno de un triángulo. |
| `src/line.rs` | `line` (algoritmo de Bresenham) devuelve los fragmentos de una línea. |
| `src/fragment.rs` | `Fragment`: candidato a píxel con posición, color y profundidad. |
| `src/framebuffer.rs` | Buffer de color y z-buffer propios; `init_texture` / `swap_buffers` lo muestran en la ventana. |
| `src/color.rs` | Color RGB de 8 bits por canal. |
| `src/main.rs` | `Uniforms`, la función `render` y la prueba del primer triángulo. |
| `src/visor.rs` | Visor de la nave: carga el modelo y maneja la cámara y la entrada. |
| `src/vertices.rs` | Ejercicio de la fase de vértices (`Vector2` y `Vector3`). |
