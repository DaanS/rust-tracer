use std::cell::UnsafeCell;

use crate::config::Float;
use rand::{SeedableRng, Rng};
use rand_xoshiro::Xoshiro256Plus;

// Use UnsafeCell for fast thread-local mutable RNG
thread_local! {
    static RNG: UnsafeCell<Xoshiro256Plus> = UnsafeCell::new(Xoshiro256Plus::seed_from_u64(0));
}

pub fn random_in_range(min: Float, max: Float) -> Float {
    RNG.with(|rng| unsafe { (*rng.get()).gen_range(min..=max) })
}

pub fn random_float() -> Float {
    RNG.with(|rng| unsafe { (*rng.get()).gen() })
}