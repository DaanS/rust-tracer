use approx::assert_ulps_eq;

use crate::color::{color_rgb, ColorRgb};
use crate::config::{Color, Float};
use crate::conversion::{color_component_to_u8, color_gamma, map_color_component};

pub struct PresampledFilm {
    pub width: usize,
    pub height: usize,
    pub samples: usize,
    pub pix: Vec<Color>
}

impl PresampledFilm {
    pub fn new(width: usize, height: usize, samples: usize) -> Self {
        PresampledFilm{width, height, samples, pix: vec![Color::default(); width * height]}
    }

    pub fn add_sample(&mut self, x: usize, y: usize, col: Color) {
        self.pix[x + y * self.width] += col;
    }

    pub fn to_rgb8(&self) -> Vec<u8> {
        let mut v = vec![0; self.width * self.height * 3];
        for (i, col) in self.pix.iter().enumerate() {
            let c = col.to_rgb() / self.samples as Float;
            v[i * 3] = map_color_component(c.r);
            v[i * 3 + 1] = map_color_component(c.g);
            v[i * 3 + 2] = map_color_component(c.b);
        }
        v
    }
}

#[derive(Clone, Copy)]
pub struct SampleCollector {
    pub mean: Color,
    pub sum_squared_diffs: Color,
    pub n: usize
}

impl SampleCollector {
    pub fn new() -> Self {
        SampleCollector { mean: color_rgb(0., 0., 0.), sum_squared_diffs: color_rgb(0., 0., 0.), n: 0 }
    }

    pub fn add_sample(&mut self, x: Color) {
        self.n += 1;
        let delta_n_min_1 = x - self.mean;
        self.mean += delta_n_min_1 / self.n as Float;
        let delta_n = x - self.mean;
        self.sum_squared_diffs += delta_n * delta_n_min_1;
    }

    // TODO test
    pub fn merged_with(&self, other: &SampleCollector) -> SampleCollector {
        let new_n = self.n + other.n;
        let new_mean = (self.n as f64 * self.mean + other.n as f64 * other.mean) / new_n as f64;
        let delta = other.mean - self.mean;
        let new_sum_squared_diffs = self.sum_squared_diffs + other.sum_squared_diffs + delta * delta * self.n as f64 * other.n as f64 / new_n as f64;
        SampleCollector { mean: new_mean, sum_squared_diffs: new_sum_squared_diffs, n: new_n }
    }

    pub fn mean(&self) -> Color {
        self.mean
    }

    pub fn gamma_corrected_mean(&self) -> Color {
        color_gamma(self.mean)
    }

    pub fn variance(&self) -> Color {
        if self.n > 1 {
            self.sum_squared_diffs / (self.n - 1) as Float
        } else {
            (0., 0., 0.).into()
        }
    }

    pub fn avg_variance(&self) -> Color {
        let v = self.variance();
        let l = (v.r + v.g + v.b) / 3.;
        (l, l, l).into()
    }

    pub fn max_variance(&self) -> Float {
        let v = self.variance();
        v.r.max(v.g).max(v.b)
    }
}

pub struct SamplingFilm {
    pub width: usize,
    pub height: usize,
    pub pix: Vec<SampleCollector>
}

impl SamplingFilm {
    pub fn new(width: usize, height: usize) -> Self {
        SamplingFilm { width, height, pix: vec![SampleCollector::new(); width * height] }
    }

    pub fn add_sample(&mut self, x: usize, y: usize, col: Color) {
        self.pix[x + y * self.width].add_sample(col);
    }

    pub fn overwrite_with(&mut self, top_left_x: usize, top_left_y: usize, other: &Self) {
        for x in top_left_x..(top_left_x + other.width).min(self.width) {
            for y in top_left_y..(top_left_y + other.height).min(self.height) {
                self.pix[x + y * self.width] = other.pix[x - top_left_x + (y - top_left_y) * other.width];
            }
        }
    }

    pub fn sample_collector(&self, x: usize, y: usize) -> &SampleCollector {
        &self.pix[x + y * self.width]
    }

    pub fn to_rgb8<ExtractFunc: Fn(&SampleCollector) -> Color>(&self, extract: ExtractFunc) -> Vec<u8> {
        let mut v = vec![0; self.width * self.height * 3];
        for (i, sc) in self.pix.iter().enumerate() {
            let c = extract(sc);
            v[i * 3] = color_component_to_u8(c.r);
            v[i * 3 + 1] = color_component_to_u8(c.g);
            v[i * 3 + 2] = color_component_to_u8(c.b);
        }
        v
    }
}

#[test]
fn test_presampled() {
    let mut f = PresampledFilm::new(2, 3, 1);
    f.add_sample(0, 1, (0., 0.5, 1.).into());
    let v = f.to_rgb8();

    assert_eq!(v.len(), f.width * f.height * 3);
    for x in 0..f.width {
        for y in 0..f.height {
            for c in 0..3 {
                assert_eq!(v[(x + y * f.width) * 3 + c], 
                    if x == 0 && y == 1 && c == 1 { map_color_component(0.5) }
                    else if x == 0 && y == 1 && c == 2 { 255 }
                    else { 0 }
                );
            }
        }
    }
}

#[test]
fn test_sample_collector() {
    let c = color_rgb(0., 0.5, 1.0);
    let mut s = SampleCollector::new();

    s.add_sample(c);
    assert_eq!(s.mean(), c);
    assert_eq!(s.gamma_corrected_mean(), color_gamma(c));
    assert_eq!(s.variance(), (0., 0., 0.).into());
    assert_eq!(s.avg_variance(), (0., 0., 0.).into());
    assert_eq!(s.max_variance(), 0.);

    s.add_sample(c);
    assert_eq!(s.mean(), c);
    assert_eq!(s.gamma_corrected_mean(), color_gamma(c));
    assert_eq!(s.variance(), (0., 0., 0.).into());
    assert_eq!(s.avg_variance(), (0., 0., 0.).into());
    assert_eq!(s.max_variance(), 0.);

    let mut s2 = SampleCollector::new();
    for _ in 0..10 { s2.add_sample(c); }
    assert_eq!(s2.mean(), c);
    assert_eq!(s2.gamma_corrected_mean(), color_gamma(c));
    assert_eq!(s2.variance(), (0., 0., 0.).into());
    assert_eq!(s2.avg_variance(), (0., 0., 0.).into());
    assert_eq!(s2.max_variance(), 0.);

    let merged = s.merged_with(&s2);
    assert_eq!(merged.mean(), c);
    assert_eq!(merged.gamma_corrected_mean(), color_gamma(c));
    assert_eq!(merged.variance(), (0., 0., 0.).into());
    assert_eq!(merged.avg_variance(), (0., 0., 0.).into());
    assert_eq!(merged.max_variance(), 0.);

    let mut sa = SampleCollector::new();
    let mut sb = SampleCollector::new();
    let mut sm = SampleCollector::new();
    sa.add_sample((0.5, 1.0, 0.0).into());
    sa.add_sample((0.0, 0.5, 1.0).into());
    sa.add_sample((1.0, 0.0, 0.5).into());

    sb.add_sample((1.0, 0.0, 0.0).into());
    sb.add_sample((0.0, 1.0, 0.0).into());
    sb.add_sample((0.0, 0.0, 1.0).into());

    sm.add_sample((0.5, 1.0, 0.0).into());
    sm.add_sample((0.0, 0.5, 1.0).into());
    sm.add_sample((1.0, 0.0, 0.5).into());
    sm.add_sample((1.0, 0.0, 0.0).into());
    sm.add_sample((0.0, 1.0, 0.0).into());
    sm.add_sample((0.0, 0.0, 1.0).into());

    let sm2 = sa.merged_with(&sb);
    // TODO ulps
    assert_ulps_eq!(sm.mean(), sm2.mean());
    assert_ulps_eq!(sm.gamma_corrected_mean(), sm2.gamma_corrected_mean());
    assert_ulps_eq!(sm.variance(), sm2.variance());
    assert_ulps_eq!(sm.avg_variance(), sm2.avg_variance());
    assert_ulps_eq!(sm.max_variance(), sm2.max_variance());
}

#[test]
fn test_sampling() {
    // add single sample to film and get gamma corrected rgb8
    let mut f = SamplingFilm::new(2, 3);
    f.add_sample(0, 1, (0., 0.5, 1.).into());
    let v = f.to_rgb8(SampleCollector::gamma_corrected_mean);

    // check if sample was stored and mapped correctly
    assert_eq!(v.len(), f.width * f.height * 3);
    for x in 0..f.width {
        for y in 0..f.height {
            for c in 0..3 {
                assert_eq!(v[(x + y * f.width) * 3 + c], 
                    if x == 0 && y == 1 && c == 1 { map_color_component(0.5) }
                    else if x == 0 && y == 1 && c == 2 { 255 }
                    else { 0 }
                );
            }
        }
    }

    // add a second sample
    f.add_sample(0, 1, (0., 0.5, 1.).into());
    let v = f.to_rgb8(SampleCollector::gamma_corrected_mean);

    // check if output still matches
    assert_eq!(v.len(), f.width * f.height * 3);
    for x in 0..f.width {
        for y in 0..f.height {
            for c in 0..3 {
                assert_eq!(v[(x + y * f.width) * 3 + c], 
                    if x == 0 && y == 1 && c == 1 { map_color_component(0.5) }
                    else if x == 0 && y == 1 && c == 2 { 255 }
                    else { 0 }
                );
            }
        }
    }

    // add two different samples and check if they average out correctly
    let mut f = SamplingFilm::new(1, 1);
    f.add_sample(0, 0, (1., 0.5, 0.).into());
    f.add_sample(0, 0, (0., 0.5, 1.).into());
    let v = f.to_rgb8(SampleCollector::gamma_corrected_mean);
    assert_eq!(v.len(), 3);
    for c in 0..3 { 
        assert_eq!(v[c], map_color_component(0.5));
    }
}