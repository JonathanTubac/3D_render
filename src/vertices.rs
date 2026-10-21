use raylib::prelude::{Vector2, Vector3};

/// Imprime las tres componentes de un vértice 3D.
pub fn imprimir_vertice(v: Vector3) {
    println!("Vértice ({}, {}, {})", v.x, v.y, v.z);
    println!("  x = {}", v.x);
    println!("  y = {}", v.y);
    println!("  z = {}", v.z);
}

/// Ejercicio de la fase de vértices: crear e imprimir Vector2 y Vector3.
pub fn demo() {
    // Vértice 2D en la posición (100, 200).
    let punto = Vector2::new(100.0, 200.0);
    println!("Vector2: x = {}, y = {}", punto.x, punto.y);

    // Vértice 3D en la posición (3, 5, -2).
    let vertice = Vector3::new(3.0, 5.0, -2.0);
    println!("x = {}", vertice.x);
    println!("y = {}", vertice.y);
    println!("z = {}", vertice.z);

    // Reto: la misma información usando la función.
    imprimir_vertice(vertice);
}
