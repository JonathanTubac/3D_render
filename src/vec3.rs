//! Operaciones con `Vector3` escritas en Rust.
//!
//! Los operadores de `Vector3` de raylib (`+`, `*`, `normalize`...) llaman a
//! funciones de la biblioteca de C. El compilador no puede optimizarlas en
//! línea, y en el código que se ejecuta una vez por fragmento eso costaba más
//! que el resto del trabajo. Estas funciones hacen lo mismo y se optimizan en
//! línea.

use raylib::prelude::Vector3;

#[inline]
pub fn v3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x, y, z }
}

#[inline]
pub fn add(a: Vector3, b: Vector3) -> Vector3 {
    v3(a.x + b.x, a.y + b.y, a.z + b.z)
}

#[inline]
pub fn sub(a: Vector3, b: Vector3) -> Vector3 {
    v3(a.x - b.x, a.y - b.y, a.z - b.z)
}

#[inline]
pub fn scale(a: Vector3, s: f32) -> Vector3 {
    v3(a.x * s, a.y * s, a.z * s)
}

/// Multiplica componente por componente (por ejemplo, un color por otro).
#[inline]
pub fn mul(a: Vector3, b: Vector3) -> Vector3 {
    v3(a.x * b.x, a.y * b.y, a.z * b.z)
}

#[inline]
pub fn dot(a: Vector3, b: Vector3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[inline]
pub fn length(a: Vector3) -> f32 {
    dot(a, a).sqrt()
}

/// Vector de largo 1 en la misma dirección; el vector cero se queda igual.
#[inline]
pub fn normalize(a: Vector3) -> Vector3 {
    let largo = length(a);
    if largo > 0.0 {
        scale(a, 1.0 / largo)
    } else {
        a
    }
}

/// Interpolación lineal: `a` cuando `t` es 0 y `b` cuando `t` es 1.
#[inline]
pub fn lerp(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    add(a, scale(sub(b, a), t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coincide_con_raylib() {
        let a = v3(1.0, -2.0, 0.5);
        let b = v3(0.25, 3.0, -1.0);
        assert_eq!(add(a, b), a + b);
        assert_eq!(sub(a, b), a - b);
        assert_eq!(scale(a, 2.5), a * 2.5);
        assert!((dot(a, b) - a.dot(b)).abs() < 1e-6);
        assert!(length(sub(normalize(a), a.normalize())) < 1e-6);
        assert_eq!(normalize(v3(0.0, 0.0, 0.0)), v3(0.0, 0.0, 0.0));
    }
}
