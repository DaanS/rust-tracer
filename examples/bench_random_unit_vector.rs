use std::time::Instant;
use std::hint::black_box;
use std::cell::UnsafeCell;

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;

const ITERATIONS: usize = 10_000_000;

type Float = f64;

thread_local! {
    static RNG: UnsafeCell<Xoshiro256Plus> = UnsafeCell::new(Xoshiro256Plus::seed_from_u64(0));
}

fn random_in_range(min: Float, max: Float) -> Float {
    RNG.with(|rng| unsafe { (*rng.get()).gen_range(min..=max) })
}

#[derive(Copy, Clone)]
struct Vec3 { x: Float, y: Float, z: Float }

fn vec3(x: Float, y: Float, z: Float) -> Vec3 { Vec3 { x, y, z } }

fn dot(u: Vec3, v: Vec3) -> Float { u.x * v.x + u.y * v.y + u.z * v.z }

impl Vec3 {
    fn length_squared(self) -> Float { self.x * self.x + self.y * self.y + self.z * self.z }
    fn normalize(self) -> Vec3 { let l = self.length_squared().sqrt(); vec3(self.x / l, self.y / l, self.z / l) }
}

/// Original: rejection sample in unit cube, then normalize
fn cube_rejection() -> Vec3 {
    let mut candidate = vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0));
    while candidate.length_squared() > 1.0 {
        candidate = vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0));
    }
    candidate.normalize()
}

/// Marsaglia 1972: rejection sample in unit disk, map to sphere
fn marsaglia() -> Vec3 {
    loop {
        let u = random_in_range(-1.0, 1.0);
        let v = random_in_range(-1.0, 1.0);
        let s = u * u + v * v;
        if s >= 1.0 { continue; }
        let factor = 2.0 * (1.0 - s).sqrt();
        return vec3(u * factor, v * factor, 1.0 - 2.0 * s);
    }
}

fn bench(name: &str, f: fn() -> Vec3) {
    // Warmup
    for _ in 0..ITERATIONS / 10 {
        black_box(f());
    }

    let start = Instant::now();
    let mut sum = 0.0;
    for _ in 0..ITERATIONS {
        let v = f();
        sum += v.x;
    }
    let elapsed = start.elapsed();
    black_box(sum);

    println!("{name:20} {ITERATIONS} calls in {elapsed:?} ({:.1} ns/call)",
        elapsed.as_nanos() as f64 / ITERATIONS as f64);
}

fn verify(name: &str, f: fn() -> Vec3) {
    let n = 100_000;
    let mut max_deviation = 0.0_f64;
    let mut mean = vec3(0.0, 0.0, 0.0);

    for _ in 0..n {
        let v = f();
        let len = dot(v, v).sqrt();
        let dev = (len - 1.0).abs();
        if dev > max_deviation { max_deviation = dev; }
        mean = vec3(mean.x + v.x / n as f64, mean.y + v.y / n as f64, mean.z + v.z / n as f64);
    }

    let bias = dot(mean, mean).sqrt();
    println!("{name:20} max |len-1|: {max_deviation:.2e}, mean bias: {bias:.4}");
}

fn main() {
    println!("=== Correctness ===");
    verify("cube_rejection", cube_rejection);
    verify("marsaglia", marsaglia);

    println!("\n=== Performance ===");
    bench("cube_rejection", cube_rejection);
    bench("marsaglia", marsaglia);
    // Run again to check stability
    bench("cube_rejection", cube_rejection);
    bench("marsaglia", marsaglia);
}
