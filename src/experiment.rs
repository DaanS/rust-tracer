use crate::{random::random_float, config::Float};

mod random;
mod config;
mod color;

struct VarianceEstimator {
    mean: Float,
    S: Float,
    n: usize
}

impl VarianceEstimator {
    pub fn new() -> Self { VarianceEstimator { mean: 0.0, S: 0.0, n: 0 }}
    pub fn add(& mut self, x: Float) {
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as Float;
        let delta2 = x - self.mean;
        self.S += delta * delta2;
    }
}

fn main() {
    const N: usize = 10000;
    let mut samples: Vec<Float> = Vec::with_capacity(N);

    let mut sum = 0.;
    let mut sum_sq = 0.;
    let mut v = VarianceEstimator::new();
    for i in 1..=N {
        let x = random_float();
        let y = x * x;
        samples.push(y);
        let est = samples.iter().sum::<Float>() / i as Float;
        //let avg_of_squares = samples.iter().map(|x| x * x).sum::<Float>() / i as Float;
        //let avg = samples.iter().sum::<Float>() / i as Float;
        //let square_of_avg = avg * avg;
        v.add(y);
        if i > 1 {
            //let var = (avg_of_squares - square_of_avg) / (i - 1) as Float;
            let var = v.S / (i - 1) as Float;
            println!("{:5}: est = {:.5}, var = {:.5}", i, est, var);
        }
    }
}