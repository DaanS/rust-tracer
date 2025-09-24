use std::{fs::File, io::{stdout, Write}, mem::replace, sync::{Arc, Mutex}, thread::Scope};

use crate::{
    color::color_rgb, config::{Color, Film, Float}, conversion::color_gamma, film::SampleCollector, material::Scatter, png::Png, ray::Ray, sampler::PixelSample, scene::Scene, util::is_power_of_2, window::MinifbWindow,
};

pub trait RayEvaluator: Default {
    fn li(&self, scene: &Scene, r: Ray, max_bounces: usize) -> Color;
}

#[derive(Clone, Copy, Default)]
pub struct SimpleRayEvaluator;
impl RayEvaluator for SimpleRayEvaluator {
    // TODO we could probably carry depth and attenuation info in the ray itself
    // maybe we could use this to make li non-recursive?
    // but then how do we return the final result to the dispatcher?
    // and would a larger ray struct negatively impact hit detection performance?
    fn li(&self, scene: &Scene, r: Ray, max_bounces: usize) -> Color {
        if max_bounces == 0 { return color_rgb(0., 0., 0.); }

        match scene.objects.hit(r, 0.001, Float::INFINITY) {
            Some(hit_record) => match hit_record.material.scatter(r, hit_record.pos, hit_record.normal) {
                Some(scatter_record) => scatter_record.attenuation * self.li(scene, scatter_record.out, max_bounces - 1),
                None => color_rgb(0., 0., 0.)
            },
            None => { (scene.background_color)(r) }
        }
    }
}

fn print_progress(prog: Float) {
    const WIDTH: usize = 70;

    let pos = (prog * WIDTH as Float) as usize;

    let mut lock = stdout().lock();
    write!(lock, "[").unwrap();
    for _i in 0..pos { write!(lock, "=").unwrap(); }
    write!(lock, ">").unwrap();
    for _i in (pos + 1)..WIDTH { write!(lock, " ").unwrap(); }
    write!(lock, "] {}%\r", (prog * 100.) as usize).unwrap();
    stdout().flush().unwrap();
}

// Integrator takes a scene and renders it onto a film, following sample size and variance targets
// We iterate over pixels, generating subpixel samples using a Sampler. Scene contains a Camera that maps these
// film-space samples to world-space rays. A RayEvaluator then computes the radiance contribution for each ray.

pub trait Integrate {
    fn integrate(scene: &Scene, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float);
}

pub struct SimpleIntegrator<Sampler: PixelSample, Evaluator: RayEvaluator> {
    _sampler: std::marker::PhantomData<Sampler>,
    _evaluator: std::marker::PhantomData<Evaluator>,
}

impl<Sampler: PixelSample, Evaluator: RayEvaluator> Integrate for SimpleIntegrator<Sampler, Evaluator> {
    // TODO might be neater to factor out the window updates and progress printing
    fn integrate(scene: &Scene, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
        let mut win = MinifbWindow::new(film.width, film.height);
        let mut win2 = MinifbWindow::new(film.width, film.height);
        let mut win3 = MinifbWindow::new(film.width, film.height);

        let mut sample_count = 0;
        let sampler = Sampler::default();
        let evaluator = Evaluator::default();

        for n in 0..max_samples {
            win.update(film, SampleCollector::gamma_corrected_mean);
            win2.update(film, SampleCollector::variance);
            win3.update(film, SampleCollector::avg_variance);

            for x in 0..film.width {
                for y in 0..film.height {
                    if n < min_samples || film.sample_collector((x, y)).max_variance() > variance_target {
                        sample_count += 1;
                        let (s, t) = sampler.pixel_sample((x, y));
                        let r = scene.cam.ray((s, t));
                        film.add_sample((x, y), evaluator.li(scene, r, 8));
                    }
                }
                print_progress((n * film.width * film.height + x * film.height) as Float / (film.width * film.height * max_samples) as Float);
            }

            if (n == 0) || is_power_of_2(n) {
                Png::write(film.width, film.height, film.to_rgb8(SampleCollector::gamma_corrected_mean), format!("out/mean-{n}.png").as_str());
                Png::write(film.width, film.height, film.to_rgb8(SampleCollector::variance), format!("out/variance-{n}.png").as_str());
                Png::write(film.width, film.height, film.to_rgb8(SampleCollector::avg_variance), format!("out/avg_variance-{n}.png").as_str());
            }
        }

        println!();
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (film.width * film.height * max_samples) as Float);
    }
}

pub struct SingleCoreTiledIntegrator<Sampler: PixelSample, Evaluator: RayEvaluator, const TILE_WIDTH: usize, const TILE_HEIGHT: usize> {
    _sampler: std::marker::PhantomData<Sampler>,
    _evaluator: std::marker::PhantomData<Evaluator>,
}

impl<Sampler: PixelSample, Evaluator: RayEvaluator, const TILE_WIDTH: usize, const TILE_HEIGHT: usize> SingleCoreTiledIntegrator<Sampler, Evaluator, TILE_WIDTH, TILE_HEIGHT> {
    fn integrate_tile(scene: &Scene, film: &mut Film, tile_x: usize, tile_y: usize, min_samples: usize, max_samples: usize, variance_target: Float) -> usize {
        let mut sample_count = 0;
        let sampler = Sampler::default();
        let evaluator = Evaluator::default();
        for y in 0..film.height {
            for x in 0..film.width {
                for n in 0..max_samples {
                    if n >= min_samples && film.sample_collector((x, y)).max_variance() <= variance_target { break; }

                    sample_count += 1;
                    let (s, t) = sampler.pixel_sample((x + tile_x * film.width, y + tile_y * film.height));
                    let r = scene.cam.ray((s, t));
                    film.add_sample((x, y), evaluator.li(scene, r, 8));
                }
            }
        }
        sample_count
    }
}
impl<Sampler: PixelSample, Evaluator: RayEvaluator, const TILE_WIDTH: usize, const TILE_HEIGHT: usize> Integrate for SingleCoreTiledIntegrator<Sampler, Evaluator, TILE_WIDTH, TILE_HEIGHT> {
    fn integrate(scene: &Scene, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
        let mut win = MinifbWindow::new(film.width, film.height);
        let mut win2 = MinifbWindow::new(film.width, film.height);
        let mut win3 = MinifbWindow::new(film.width, film.height);

        let tiles_hor = film.width / TILE_WIDTH + 1.min(film.width % TILE_WIDTH);
        let tiles_ver = film.height / TILE_HEIGHT + 1.min(film.height % TILE_HEIGHT);

        let mut sample_count = 0;

        for x in 0..tiles_hor {
            for y in 0..tiles_ver {
                let mut tile_film = Film::new((TILE_WIDTH, TILE_HEIGHT));
                sample_count += Self::integrate_tile(scene, &mut tile_film, x, y, min_samples, max_samples, variance_target);
                film.overwrite_with((x * TILE_WIDTH, y * TILE_HEIGHT), &tile_film);

                win.update(film, SampleCollector::gamma_corrected_mean);
                win2.update(film, SampleCollector::variance);
                win3.update(film, SampleCollector::avg_variance);

                print_progress((y + x * tiles_ver) as Float / (tiles_hor * tiles_ver) as Float);
            }
        }

        println!();
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (film.width * film.height * max_samples) as Float);
    }
}

struct JobQueue<'a, F: Send + 'a> {
    jobs: Mutex<Vec<F>>,
    __marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, F: Send + 'a> JobQueue<'a, F> {
    fn new(jobs: Vec<F>) -> Self {
        JobQueue{jobs: Mutex::new(jobs), __marker: std::marker::PhantomData}
    }

    fn make_shared(jobs: Vec<F>) -> Arc<Self> {
        Arc::new(JobQueue::new(jobs))
    }

    fn add_job(&self, job: F) {
        self.jobs.lock().unwrap().push(job);
    }

    fn get_job(&self) -> Option<F> {
        self.jobs.lock().unwrap().pop()
    }

    fn is_empty(&self) -> bool {
        self.jobs.lock().unwrap().is_empty()
    }
}

struct CoordinateRange(std::ops::Range<usize>, std::ops::Range<usize>);

impl CoordinateRange {
    fn iter(&self) -> CoordinateRangeIterator {
        let mut first = self.0.clone();
        let start = first.next().unwrap_or(0);
        CoordinateRangeIterator(first, self.1.clone(), start)
    }
}

struct CoordinateRangeIterator(std::ops::Range<usize>, std::ops::Range<usize>, usize);

impl Iterator for CoordinateRangeIterator {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(y) = self.1.next() {
            Some((self.2, y))
        } else if let Some(x) = self.0.next() {
            self.1 = 0..self.1.end;
            self.2 = x;
            Some((x, self.1.next().unwrap()))
        } else {
            None
        }
    }
}

pub struct MultiCoreTiledIntegrator<Sampler: PixelSample, Evaluator: RayEvaluator, const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WORKER_COUNT: usize> {
    _sampler: std::marker::PhantomData<Sampler>,
    _evaluator: std::marker::PhantomData<Evaluator>,
}

impl<Sampler: PixelSample, Evaluator: RayEvaluator, const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WORKER_COUNT: usize> MultiCoreTiledIntegrator<Sampler, Evaluator, TILE_WIDTH, TILE_HEIGHT, WORKER_COUNT> {
    pub fn integrate_inner(scene: &Scene, film: &Mutex<Film>, min_samples: usize, max_samples: usize, variance_target: Float) {
        let width = film.lock().unwrap().width;
        let height = film.lock().unwrap().height;

        let queue = Self::queue_jobs(scene, film, (width, height), min_samples, max_samples, variance_target);

        let sample_count = Mutex::new(0);
        std::thread::scope(|scope| {
            Self::spawn_workers(scope, &queue, &sample_count);
            Self::spawn_progress_thread(scope, &queue, film, &sample_count, (width, height), max_samples);
        });

        let sample_count = *sample_count.lock().unwrap();

        println!();
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (width * height * max_samples) as Float);
    }

    fn queue_jobs<'scene>(scene: &'scene Scene, film: &'scene Mutex<Film>, (width, height): (usize, usize), min_samples: usize, max_samples: usize, variance_target: Float) -> JobQueue<'scene, impl FnOnce(&mut dyn Write) -> usize + Send + 'scene> {
        let tiles_hor = width / TILE_WIDTH + 1.min(width % TILE_WIDTH);
        let tiles_ver = height / TILE_HEIGHT + 1.min(height % TILE_HEIGHT);

        JobQueue::new(CoordinateRange(0..tiles_hor, 0..tiles_ver).iter().map(|(x, y)| {
            move |out: &mut dyn Write|
                Self::render_tile(scene, film, (x * TILE_WIDTH, y * TILE_HEIGHT), (TILE_WIDTH, TILE_HEIGHT), min_samples, max_samples, variance_target, out)
        }).collect())
    }

    fn render_tile(scene: &Scene, film: &Mutex<Film>, (topleft_x, topleft_y): (usize, usize), (tile_width, tile_height): (usize, usize), min_samples: usize, max_samples: usize, variance_target: Float, _out: &mut dyn Write) -> usize {
        let mut sample_count = 0;
        let mut local_film = Film::new((tile_width, tile_height));
        let sampler = Sampler::default();
        let evaluator = Evaluator::default();

        for y in 0..tile_height {
            for x in 0..tile_width {
                for n in 0..max_samples {
                    if n >= min_samples && local_film.sample_collector((x, y)).max_variance() <= variance_target { break; }

                    sample_count += 1;
                    let (s, t) = sampler.pixel_sample((topleft_x + x, topleft_y + y));
                    let r = scene.cam.ray((s, t));
                    local_film.add_sample((x, y), evaluator.li(scene, r, 8));
                }
            }
        }

        Png::write(tile_width, tile_height, local_film.to_rgb8(|s| color_gamma(s.mean())), &format!("out/jobs/out-{topleft_x}-{topleft_y}.png"));
        film.lock().unwrap().overwrite_with((topleft_x, topleft_y), &local_film);
        sample_count
    }

    fn spawn_workers<'scope, 'env>(scope: &'scope Scope<'scope, 'env>, queue: &'scope JobQueue<impl FnOnce(&mut dyn Write) -> usize + Send + 'scope>, sample_count: &'scope Mutex<usize>) {
        for i in 0..WORKER_COUNT {
            scope.spawn(move || {
                let mut out = File::create(format!("out/worker-{i}.log")).unwrap();
                writeln!(out, "thread {i} reporting").unwrap();
                while let Some(job) = queue.get_job() {
                    let local_sample_count = job(&mut out);
                    writeln!(out, "finished job with {local_sample_count} samples").unwrap();
                    *sample_count.lock().unwrap() += local_sample_count;
                }
                writeln!(out, "thread {i} done").unwrap();
            });
        };
    }

    fn spawn_progress_thread<'scope, 'env>(scope: &'scope Scope<'scope, 'env>, queue: &'scope JobQueue<impl FnOnce(&mut dyn Write) -> usize + Send + 'scope>, film: &'scope Mutex<Film>, sample_count: &'scope Mutex<usize>, (width, height): (usize, usize), max_samples: usize) {
        scope.spawn(move || {
            let mut win = MinifbWindow::positioned(width, height, 0, 0);
            let mut win2 = MinifbWindow::positioned(width, height, width as isize, 0);
            let mut win3 = MinifbWindow::positioned(width, height, width as isize, height as isize);
            let total_samples = width * height * max_samples;

            while !queue.is_empty() {
                {
                    let progress = *sample_count.lock().unwrap() as Float / total_samples as Float;
                    print_progress(progress);
                }

                {
                    let film = film.lock().unwrap();
                    win.update(&film, SampleCollector::gamma_corrected_mean);
                    win2.update(&film, SampleCollector::variance);
                    win3.update(&film, SampleCollector::avg_variance);
                }

                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        });
    }
}

impl<Sampler: PixelSample, Evaluator: RayEvaluator, const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WORKER_COUNT: usize> Integrate for MultiCoreTiledIntegrator<Sampler, Evaluator, TILE_WIDTH, TILE_HEIGHT, WORKER_COUNT> {
    fn integrate(scene: &Scene, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
        // the replace / into_inner is still a bit ugly, but it avoids cloning the film

        let film_mutex = Mutex::new(replace(film, Film::new((1, 1))));

        Self::integrate_inner(scene, &film_mutex, min_samples, max_samples, variance_target);

        *film = film_mutex.into_inner().unwrap();
    }

}

#[test]
fn test_coordinate_range() {
    let mut cr = CoordinateRange(0..2, 0..3).iter();
    assert_eq!(cr.next(), Some((0, 0)));
    assert_eq!(cr.next(), Some((0, 1)));
    assert_eq!(cr.next(), Some((0, 2)));
    assert_eq!(cr.next(), Some((1, 0)));
    assert_eq!(cr.next(), Some((1, 1)));
    assert_eq!(cr.next(), Some((1, 2)));
    assert_eq!(cr.next(), None);
}