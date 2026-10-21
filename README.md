# Spaceship Render

**English** · [Español](README.es.md)

A software renderer written in Rust for the Computer Graphics course at Universidad del Valle de Guatemala (UVG).
It loads a spaceship modeled in Blender from an `.obj` file and draws the outline of every one
of its triangles using hand-written algorithms: an OBJ parser, Bresenham line drawing and a custom
framebuffer.

Raylib is only used to open the window, display the framebuffer as a texture and read mouse and
keyboard input. Raylib's model loading and drawing functions are **not** used. Every pixel of
the model is drawn by this project's own code.

## Screenshots

![Spaceship wireframe, angled view](screenshot.png)

| | |
|---|---|
| ![Close-up view](screenshot1.png) | ![Front view](screenshot2.png) |

The model is `spaceship.obj`, with 286,689 vertices and 573,439 triangles.

## Features

- **OBJ loader:** reads vertex positions (`v`) and faces (`f`), converts the 1-based OBJ
  indices to 0-based, accepts the `v/vt/vn` face format and splits faces with more than
  three vertices into triangles (fan triangulation).
- **Wireframe rendering:** every triangle is drawn as three Bresenham lines into the framebuffer.
- **Automatic framing:** the model is centered and scaled to fit the window.
- **Orbit camera:** you can rotate and zoom the model in real time. The projection is
  orthographic for now; perspective comes in a later stage of the course.
- **Redraw on demand:** the model is only redrawn when the camera moves.

## Running

Requirements: Rust (MSVC toolchain on Windows) and CMake, which the `raylib` crate uses to build Raylib.

```sh
cargo run --release
```

This opens a 1024×768 window with the model and saves a front view to `screenshot.png`.
Use `--release`: drawing 573k triangles in a debug build is much slower.

To only generate the image, without opening the window:

```sh
cargo run --release -- --no-window
```

To run the tests:

```sh
cargo test
```

> **Windows:** `.cargo/config.toml` tells `bindgen` where the MSVC headers are, because it
> could not find them on its own on the development machine. If your Visual Studio is installed
> in a different path, update that file, or delete it if the project builds without it.

### Controls

| Input | Action |
|---|---|
| Left-click drag / arrow keys | Rotate the model |
| Mouse wheel | Zoom |
| `R` | Reset the view |

## How it works

```
spaceship.obj → vertices + indices → triangles → lines → pixels → window
```

| File | Purpose |
|---|---|
| `src/obj.rs` | OBJ loader that builds an `Obj { vertices, indices }`. |
| `src/render.rs` | Camera, framing and the vertex-to-screen transform; walks the index list three at a time and draws each triangle. |
| `src/line.rs` | `line` (Bresenham) and `triangle` (three calls to `line`). |
| `src/framebuffer.rs` | Color buffer and z-buffer; exports to a Raylib image or texture. |
| `src/color.rs` | 8-bit RGB color. |
| `src/main.rs` | Loads the model, saves the screenshot, runs the window and handles input. |
| `src/vertices.rs` | Warm-up exercise with `Vector2` and `Vector3`. |
