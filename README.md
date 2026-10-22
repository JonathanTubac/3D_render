# Laboratorio 5: rellenar triángulos

Renderer por software escrito en Rust para el curso de Gráficas por Computadora (UVG).
En este laboratorio la rasterización puede generar el contorno de cada triángulo o
rellenarlo con dos algoritmos: coordenadas baricéntricas y líneas horizontales
(scanline). Raylib solo se usa para abrir la ventana y mostrar el framebuffer como
textura; todos los fragmentos los genera el código del proyecto.

## Cómo ejecutarlo

Requisitos: Rust (toolchain MSVC en Windows) y CMake, que el crate `raylib` usa para
compilar Raylib. Usa `--release`: en modo debug la rasterización es mucho más lenta.

```sh
cargo run --release                          # escena de prueba en una ventana
cargo run --release -- --capturas-relleno    # guarda docs/relleno_*.png
cargo run --release -- --benchmark-triangles # mide la rasterización de cada modo
cargo test                                   # tests
```

En la ventana, las teclas cambian el modo de rasterización:

| Tecla | Modo |
|---|---|
| `1` | `Wireframe` |
| `2` | `FilledBarycentric` |
| `3` | `FilledScanline` |

> **Windows:** `.cargo/config.toml` le indica a `bindgen` dónde están los headers de
> MSVC porque en la máquina de desarrollo no los encontraba solo. Si tu instalación de
> Visual Studio está en otra ruta, ajústala ahí (o borra el archivo si compila sin él).

## Cómo funciona

`render` sigue el pipeline de cuatro etapas; el modo solo cambia la rasterización:

```
vértices → vertex shader → ensamblaje de primitivas → rasterización (TriangleMode) → fragment shader → framebuffer
```

| Archivo | Qué hace |
|---|---|
| `src/triangle.rs` | `triangle` (contorno), `triangle_filled_barycentric` y `triangle_filled_scanline` devuelven los fragmentos de un triángulo. |
| `src/rasterizer.rs` | `TriangleMode` y `rasterize`, que genera los fragmentos de todos los triángulos en el modo elegido. |
| `src/laboratorio.rs` | Escena de prueba, capturas y mediciones del laboratorio. |
| `src/main.rs` | `Uniforms`, la función `render` y la ventana con la escena de prueba. |
| `src/primitive.rs` | Ensamblaje de primitivas: agrupa los vértices de tres en tres. |
| `src/shaders.rs` | `vertex_shader` (aplica la matriz de modelo) y `fragment_shader` (passthrough). |
| `src/line.rs` | `line` (algoritmo de Bresenham) devuelve los fragmentos de una línea. |
| `src/fragment.rs` | `Fragment`: candidato a píxel con posición, color y profundidad. |
| `src/framebuffer.rs` | Buffer de color y z-buffer; `init_texture` / `swap_buffers` lo muestran en la ventana. |
| `src/obj.rs` | Cargador de OBJ; se usa para medir la rasterización con los triángulos de `spaceship.obj`. |

## Modos de dibujo

`rasterize` recibe un `TriangleMode`:

```rust
pub enum TriangleMode { Wireframe, FilledBarycentric, FilledScanline }
```

El ensamblaje de primitivas y el procesamiento de fragmentos no cambian; solo cambia
cómo cada triángulo se convierte en fragmentos.

## Imágenes de cada modo

Escena de prueba: triángulo pequeño (amarillo), grande (azul), con borde horizontal
(verde), con borde vertical (rosa), el mismo con sus vértices en orden inverso (morado,
180 px más abajo), parcialmente fuera del framebuffer (naranja) y tres vértices sobre una
misma línea (rojo).

| Wireframe | FilledBarycentric | FilledScanline |
|---|---|---|
| ![Wireframe](docs/relleno_wireframe.png) | ![Baricéntrico](docs/relleno_filledbarycentric.png) | ![Scanline](docs/relleno_filledscanline.png) |

El triángulo rojo (vértices colineales) solo aparece en wireframe: su área es cero, así
que los rellenos lo tratan como degenerado, no generan fragmentos y no dividen entre
cero. El morado tiene exactamente las mismas posiciones que el rosa.

## Los dos algoritmos

- **Baricéntrico:** recorre el rectángulo que envuelve al triángulo y, para el centro de
  cada píxel `(x + 0.5, y + 0.5)`, calcula sus tres coordenadas baricéntricas (con
  *edge functions*). Si las tres indican que el centro está adentro, genera un fragmento.
- **Scanline:** para cada fila calcula dónde la cruzan las aristas y genera los fragmentos
  entre esos dos extremos, sin evaluar los píxeles del interior.

**Una sola regla para los bordes.** Los dos usan la misma prueba para decidir si un centro
está dentro, con la regla *top-left* para los centros que caen justo sobre una arista: se
dibujan solo si la arista es superior o izquierda. Así, dos triángulos que comparten una
arista no dejan huecos ni generan el mismo píxel dos veces; un test con un cuadrado
dividido por su diagonal da exactamente 100 píxeles sin repetir.

El scanline calcula los extremos de cada fila con una división, que puede tener errores
de redondeo, y después ajusta solo esos extremos con la misma prueba del baricéntrico.
Por eso los dos algoritmos generan **exactamente las mismas posiciones**. Los tests lo
comprueban con cada caso de la escena y con 500 triángulos aleatorios, y la medición lo
comprueba con la nave completa. Las imágenes de los dos rellenos son idénticas píxel por
píxel.

Si los vértices vienen en el orden contrario, se intercambian dos para que el área sea
positiva; el triángulo no desaparece.

## Mediciones

`cargo run --release -- --benchmark-triangles`. Solo se mide `rasterize` (generar y
guardar los fragmentos): los triángulos se preparan antes y no se dibuja ninguna ventana.
Cada medición ejecuta `rasterize` varias veces con `black_box`; se reporta la mediana de 7
mediciones. La salida completa está en [`docs/benchmark.txt`](docs/benchmark.txt).

| Caso | Modo | Tiempo por ejecución | Fragmentos | ns por fragmento |
|---|---|---:|---:|---:|
| Escena de prueba (7 triángulos) | Wireframe | 0.010 ms | 3,735 | 2.72 |
| Escena de prueba (7 triángulos) | FilledBarycentric | 1.98 ms | 149,465 | 13.23 |
| Escena de prueba (7 triángulos) | FilledScanline | 1.37 ms | 149,465 | 9.16 |
| Triángulos grandes (500) | Wireframe | 1.39 ms | 124,242 | 11.22 |
| Triángulos grandes (500) | FilledBarycentric | 28.98 ms | 1,348,422 | 21.49 |
| Triángulos grandes (500) | FilledScanline | 18.66 ms | 1,348,422 | 13.84 |
| Nave (573,439 triángulos) | Wireframe | 115.05 ms | 4,441,533 | 25.90 |
| Nave (573,439 triángulos) | FilledBarycentric | 36.63 ms | 495,722 | 73.90 |
| Nave (573,439 triángulos) | FilledScanline | 56.99 ms | 495,722 | 114.97 |

Los dos rellenos generan la misma cantidad de fragmentos en cada caso, así que la
comparación es justa: ninguno es más rápido por omitir parte del triángulo.

## Diferencias encontradas

1. **Con triángulos grandes gana el scanline (~1.5× más rápido).** El baricéntrico evalúa
   tres *edge functions* en cada píxel del rectángulo que envuelve al triángulo, y cerca
   de la mitad de ese rectángulo queda fuera del triángulo: trabajo desperdiciado. El
   scanline solo calcula los dos extremos de cada fila y luego agrega fragmentos sin
   evaluar nada.
2. **Con la nave gana el baricéntrico (~1.6× más rápido).** Sus triángulos son diminutos:
   en promedio generan **menos de un fragmento** (495,722 fragmentos para 573,439
   triángulos). Ahí pesa el trabajo fijo por triángulo, no el trabajo por píxel. El
   rectángulo del baricéntrico casi siempre tiene 0 o 1 píxeles, mientras que el scanline
   calcula cruces con las tres aristas y ajusta los extremos de cada fila.
3. **Para la nave, rellenar es ~3× más rápido que el wireframe.** Cada línea genera al
   menos un fragmento por extremo, así que un triángulo diminuto produce varios
   fragmentos en contorno (4.4M en total) y casi ninguno relleno.
4. **Guardar los fragmentos cuesta tanto como calcularlos.** Para investigarlo, se repitió
   la medición de los triángulos grandes reutilizando el mismo `Vec` en lugar de crear uno
   nuevo en cada ejecución: el scanline bajó de ~20 ms a ~5 ms y el baricéntrico de
   ~31 ms a ~15 ms. Con triángulos grandes, ~75 % del tiempo del scanline se va en pedir
   memoria nueva al sistema y llenarla (un `Vec` de ~32 MB por ejecución), no en el
   algoritmo. Sin ese costo, el scanline es ~3× más rápido que el baricéntrico.

## Perfil

```sh
cargo install flamegraph
cargo flamegraph -- --benchmark-triangles   # en una terminal de administrador
```

En Windows, `cargo flamegraph` necesita permisos de administrador para leer los eventos
de rendimiento del sistema. `Cargo.toml` activa `debug = true` en el perfil release para
que el perfil muestre los nombres de las funciones; no cambia la optimización.

[![Flamegraph de --benchmark-triangles](docs/flamegraph.svg)](docs/flamegraph.svg)

Abre [`docs/flamegraph.svg`](docs/flamegraph.svg) en el navegador para hacer zoom y
buscar funciones. El ancho de cada bloque es la proporción de muestras en las que apareció
esa función, no el orden en que se ejecutó.

`rasterize` aparece en el 93 % de las muestras; el resto es preparar los triángulos de la
nave y comparar las posiciones. Dentro de cada relleno, el tiempo se reparte así:

| Dentro de… | Prueba "¿está adentro?" (`dentro`, `edge`, `pasa`) | Escribir el fragmento (`Vec::push`) | Hacer crecer el `Vec` (`reserve` → `grow_amortized` → sistema) | Cruces de aristas (`tramo`) |
|---|---:|---:|---:|---:|
| `triangle_filled_barycentric_into` (52 % del total) | 38 % | 29 % | 23 % | — |
| `triangle_filled_scanline_into` (37 % del total) | 2 % | 49 % | 33 % | 8 % |

- **Baricéntrico:** el cálculo geométrico es su parte más grande (38 %): las tres *edge
  functions* que evalúa en cada píxel del rectángulo, incluidos los que quedan fuera.
- **Scanline:** casi no hace cálculos geométricos (10 % entre `tramo` y `dentro`); el
  **82 % de su tiempo es memoria**: escribir los fragmentos y hacer crecer el `Vec`.
- Los bloques sin nombre (`0x7FF9…`) son funciones de Windows (`ntdll`) que aparecen al
  copiar el `Vec` a un bloque más grande y al liberarlo.

Esto confirma el experimento de la sección anterior: en este renderer, crear y guardar
fragmentos cuesta más que decidir qué píxeles cubre el triángulo. La mejora más grande ya
no estaría en el algoritmo de relleno, sino en no guardar todos los fragmentos en un `Vec`
(por ejemplo, reutilizar el mismo `Vec` entre cuadros o escribir cada fragmento en cuanto se
genera).

## Preguntas del laboratorio

**¿Cómo determinan qué posiciones de la imagen están dentro de un triángulo?**
Se evalúa el centro de cada píxel, `(x + 0.5, y + 0.5)`. Con las tres *edge functions* se
obtienen sus coordenadas baricéntricas sin normalizar, `w0`, `w1` y `w2`; las
baricéntricas son `w0 / área`, `w1 / área` y `w2 / área`. Si las tres son positivas, el
centro está dentro. Si alguna es cero, el centro cae justo sobre una arista y decide la
regla top-left (ver abajo).

**Si invierten el orden de los vértices, ¿desaparece el triángulo?**
No. Con el orden invertido el área sale negativa y todas las `w` cambian de signo, así que
la prueba "las tres positivas" fallaría siempre. Para evitarlo, si el área es negativa se
intercambian dos vértices antes de rellenar. En la escena, el triángulo morado es el rosa
con sus vértices en orden inverso y cubre exactamente los mismos 12,800 píxeles.

**¿Qué hacen si el área del triángulo es cero?**
Lo tratan como degenerado: `Relleno::new` devuelve `None` y no se genera ningún fragmento,
así que nunca se divide entre el área. Es el caso de los tres vértices sobre una misma línea
(el triángulo rojo): en wireframe se ve como una línea y relleno no aparece.

**¿Cómo tratan las aristas horizontales y los vértices que coinciden exactamente con una
fila?**
- **Aristas horizontales:** en el scanline, una arista horizontal no tiene un cruce con la
  fila (dividir entre su altura sería dividir entre cero). Si está justo sobre la fila, se
  toman sus dos extremos como límites del tramo; si no, se ignora.
- **Vértices sobre una fila:** cuando la fila pasa exactamente por un vértice, las dos
  aristas que llegan a él dan el mismo cruce, y el tramo se calcula una sola vez por fila,
  así que no se duplican fragmentos.
- **Centros sobre una arista:** se dibujan solo si la arista es **superior** (horizontal,
  con el triángulo debajo) o **izquierda**. Es la regla *top-left* que usan las GPUs. Sin
  una regla así, dos triángulos que comparten una arista dejan huecos (si ninguno dibuja
  esos píxeles) o fragmentos duplicados (si los dos los dibujan). El test
  `shared_edge_has_no_gaps_and_no_duplicates` divide un cuadrado de 10 × 10 por su diagonal,
  que pasa justo por centros de píxeles, y obtiene exactamente 100 fragmentos sin repetir.

**¿Producen los dos algoritmos la misma figura? Si hay diferencias en un borde, ¿qué regla
usa cada implementación?**
Producen **exactamente las mismas posiciones**, porque usan la misma regla para los bordes.
El baricéntrico aplica la prueba a cada píxel del rectángulo que envuelve al triángulo. El
scanline calcula los extremos de cada fila con una división; como esa división puede tener
errores de redondeo justo en los bordes, después ajusta **solo los extremos** con la misma
prueba del baricéntrico. Los píxeles del interior del tramo no se evalúan.

Sin ese ajuste, la división podría dejar fuera o meter de más un píxel cuyo centro cae
justo sobre una arista, y ahí los dos algoritmos diferirían. Se
comparan las posiciones, no solo la imagen: los tests lo comprueban en cada caso de la escena
y con 500 triángulos aleatorios, `--benchmark-triangles` lo comprueba con los 573,439
triángulos de la nave, y las dos imágenes rellenas son idénticas píxel por píxel.

**¿Por qué registrar cuántos fragmentos produce cada algoritmo?**
Porque una versión que omite parte del triángulo parecería más rápida solo por hacer menos
trabajo. Aquí los dos rellenos producen la misma cantidad de fragmentos en cada caso (por
ejemplo, 1,348,422 con los triángulos grandes), así que la diferencia de tiempo se debe al
algoritmo.

**En el flamegraph, ¿qué ocupa más espacio: los cálculos geométricos, la creación de
fragmentos o el crecimiento del `Vec`?**
Depende del algoritmo (ver la tabla de la sección "Perfil"):
- **Baricéntrico:** lo más grande son los **cálculos geométricos** (38 %), por evaluar tres
  *edge functions* en cada píxel del rectángulo; le siguen la creación de fragmentos (29 %)
  y el crecimiento del `Vec` (23 %).
- **Scanline:** casi no hace cálculos geométricos (10 %); lo que domina es la **creación de
  fragmentos** (49 %) y el **crecimiento del `Vec`** (33 %).

Sumando las dos, el manejo de memoria es la mayor parte del tiempo de rasterización en este
renderer.

**¿Cuál algoritmo conviene?**
Depende del tamaño de los triángulos. Con triángulos grandes, el scanline es ~1.5× más
rápido porque no evalúa los píxeles que quedan fuera del triángulo. Con triángulos diminutos,
como los de la nave (menos de un fragmento por triángulo en promedio), gana el baricéntrico
porque su trabajo fijo por triángulo es menor.
