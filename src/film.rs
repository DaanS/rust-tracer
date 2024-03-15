use crate::color::color_rgb;
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

#[derive(Clone)]
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

    // TODO probably turn this back into a float
    pub fn variance(&self) -> Color {
        if self.n > 1 {
            self.sum_squared_diffs / (self.n - 1) as Float
        } else {
            (0., 0., 0.).into()
        }
    }

    pub fn mean(&self) -> Color {
        self.mean
    }

    pub fn gamma_corrected_mean(&self) -> Color {
        color_gamma(self.mean)
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
fn test_sampling() {
    let mut f = SamplingFilm::new(2, 3);
    f.add_sample(0, 1, (0., 0.5, 1.).into());
    let v = f.to_rgb8(SampleCollector::gamma_corrected_mean);

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

    f.add_sample(0, 1, (0., 0.5, 1.).into());
    let v = f.to_rgb8(SampleCollector::gamma_corrected_mean);

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

    let mut f = SamplingFilm::new(1, 1);
    f.add_sample(0, 0, (1., 0.5, 0.).into());
    f.add_sample(0, 0, (0., 0.5, 1.).into());
    let v = f.to_rgb8(SampleCollector::gamma_corrected_mean);
    assert_eq!(v.len(), 3);
    for c in 0..3 { 
        assert_eq!(v[c], map_color_component(0.5));
    }
}