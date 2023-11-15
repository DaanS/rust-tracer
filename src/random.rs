use std::cell::RefCell;

use crate::config::Float;
use rand::{SeedableRng, Rng};
use rand_xoshiro::Xoshiro256Plus;

// TODO replace refcell with something that doesnt have stupid overhead
thread_local! {
    static RNG: RefCell<Xoshiro256Plus> = RefCell::new(Xoshiro256Plus::seed_from_u64(0));
}

pub fn random_in_range(min: Float, max: Float) -> Float {
    RNG.with(|rng| rng.borrow_mut().gen_range(min..=max))
}

pub fn random_float() -> Float {
    RNG.with(|rng| rng.borrow_mut().gen())
}