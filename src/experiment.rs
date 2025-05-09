use crate::{random::random_float, config::Float};

mod random;
mod config;
mod color;
mod film;
mod sampler;
mod conversion;
mod util;

struct VarianceEstimator {
    mean: Float,
    m2: Float,
    n: usize
}

impl VarianceEstimator {
    pub fn new() -> Self { VarianceEstimator { mean: 0.0, m2: 0.0, n: 0 }}
    pub fn add(& mut self, x: Float) {
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as Float;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;
    }
    pub fn add_all(&mut self, other: VarianceEstimator) {
        let new_n = self.n + other.n;
        let new_mean = (self.n as f64 * self.mean + other.n as f64 * other.mean) / new_n as f64;
        let delta = other.mean - self.mean;
        let new_m2 = self.m2 + other.m2 + delta * delta * self.n as f64 * other.n as f64 / new_n as f64;
        self.n = new_n;
        self.mean = new_mean;
        self.m2 = new_m2;
    }
}

fn main() {
    const N: usize = 10000;
    let mut samples: Vec<Float> = Vec::with_capacity(N);

    let mut v = VarianceEstimator::new();
    let mut v2 = VarianceEstimator::new();
    for i in 1..=N {
        let x = random_float();
        let y = x * 2. + 2.;
        samples.push(y);
        let est = samples.iter().sum::<Float>() / i as Float;
        //let avg_of_squares = samples.iter().map(|x| x * x).sum::<Float>() / i as Float;
        //let avg = samples.iter().sum::<Float>() / i as Float;
        //let square_of_avg = avg * avg;
        if i % 2 == 0 {v.add(y);} else { v2.add(y);}
        if i > 1 {
            //let var = (avg_of_squares - square_of_avg) / (i - 1) as Float;
            let var = v.m2 / (i - 1) as Float;
            println!("{:5}: est = {:.5}, var = {:.5}", i, est, var);
        }
    }

    println!("v: est = {:.5}, var = {:.5}", v.mean, v.m2 / (v.n - 1) as f64);
    println!("v2: est = {:.5}, var = {:.5}", v2.mean, v2.m2 / (v2.n - 1) as f64);
    let mut v3 = VarianceEstimator::new();
    v3.add_all(v);
    v3.add_all(v2);
    println!("v3: est = {:.5}, var = {:.5}", v3.mean, v3.m2 / (v3.n - 1) as f64);
}