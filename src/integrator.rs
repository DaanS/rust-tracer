use std::{fs::File, io::{stdout, Write}, mem::replace, sync::{Arc, Mutex}};

use crate::{
    color::color_rgb, config::{Color, Film, Float}, conversion::color_gamma, film::SampleCollector, material::Scatter, png::Png, ray::Ray, scene::{Scene}, util::is_power_of_2, window::MinifbWindow, Sampler
};

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

struct RenderRegionJob<'a> {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    min_samples: usize,
    max_samples: usize,
    variance_target: Float,
    scene: &'a Scene,
    sampler: Sampler,
    film: Film
}

struct JobQueue<T> {
    jobs: Mutex<Vec<T>>
}

impl<T> JobQueue<T> {
    fn new(jobs: Vec<T>) -> Self {
        JobQueue{jobs: Mutex::new(jobs)}
    }

    fn make_shared(jobs: Vec<T>) -> Arc<Self> {
        Arc::new(JobQueue::new(jobs))
    }

    fn add_job(&self, job: T) {
        self.jobs.lock().unwrap().push(job);
    }

    fn get_job(&self) -> Option<T> {
        self.jobs.lock().unwrap().pop()
    }
}

/// integrator concepts
///
/// for pixel based approaches:
/// - dispatch parts of image (e.g. tiles) -> Film
/// - generate samples for pixels -> Sampler
/// - turn samples into rays -> Camera
/// - calculate radiance contribution for rays

// Apparently we need 'static here because otherwise the compiler assumes that implementations might contain
// non-'static references, even when they get cloned into an Arc.
pub trait RayEvaluator: Send + Sync + Clone + 'static {
    fn li(&self, scene: &Scene, r: Ray, max_bounces: usize) -> Color;
}

#[derive(Clone, Copy)]
pub struct SimpleRayEvaluator;
impl RayEvaluator for SimpleRayEvaluator {
    // TODO we could probably carry depth and attenuation info in the ray itself
    // maybe we could use this to make li non-recursive?
    // but then how do we return the final result to the dispatcher?
    // and would a larger ray struct negatively impact hit detection performance?
    fn li(&self, scene: &Scene, r: Ray, max_bounces: usize) -> Color {
        if max_bounces == 0 { return color_rgb(0., 0., 0.); }

        match scene.objects.hit(r.clone(), 0.001, Float::INFINITY) {
            Some(hit_record) => match hit_record.material.scatter(r.clone(), hit_record.clone()) {
                Some(scatter_record) => scatter_record.attenuation * self.li(scene, scatter_record.out, max_bounces - 1),
                None => color_rgb(0., 0., 0.)
            },
            None => { (scene.background_color)(r) }
        }
    }
}

// TODO do we want to assume that sampler and evaluator carry state, and if so, what is the scope of that state (tile, thread, something else entirely)?
pub trait Dispatch {
    fn dispatch(&self, scene: &Scene, sampler: Sampler, evaluator: impl RayEvaluator, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float);
}

pub struct SimpleDispatcher;
impl Dispatch for SimpleDispatcher {
    // TODO might be neater to factor out the window updates and progress printing
    fn dispatch(&self, scene: &Scene, sampler: Sampler, evaluator: impl RayEvaluator, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
        let mut win = MinifbWindow::new(film.width, film.height);
        let mut win2 = MinifbWindow::new(film.width, film.height);
        let mut win3 = MinifbWindow::new(film.width, film.height);

        let mut sample_count = 0;

        for n in 0..max_samples {
            win.update(&film, SampleCollector::gamma_corrected_mean);
            win2.update(&film, SampleCollector::variance);
            win3.update(&film, SampleCollector::avg_variance);

            for x in 0..film.width {
                for y in 0..film.height {
                    if n < min_samples || film.sample_collector(x, y).max_variance() > variance_target {
                        sample_count += 1;
                        let (s, t) = sampler.get_pixel_sample(x, y);
                        let r = scene.cam.get_ray(s, t);
                        film.add_sample(x, y, evaluator.li(scene, r, 8));
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

        println!("");
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (film.width * film.height * max_samples) as Float);
    }
}

pub struct SingleCoreTiledDispatcher<const TILE_WIDTH: usize, const TILE_HEIGHT: usize>;
impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize> SingleCoreTiledDispatcher<TILE_WIDTH, TILE_HEIGHT> {
    fn dispatch_tile(scene: &Scene, sampler: Sampler, evaluator: &impl RayEvaluator, film: &mut Film, tile_x: usize, tile_y: usize, min_samples: usize, max_samples: usize, variance_target: Float) -> usize {
        let mut sample_count = 0;
        for y in 0..film.height {
            for x in 0..film.width {
                for n in 0..max_samples {
                    if n >= min_samples && film.sample_collector(x, y).max_variance() <= variance_target { break; }

                    sample_count += 1;
                    let (s, t) = sampler.get_pixel_sample(x + tile_x * film.width, y + tile_y * film.height);
                    let r = scene.cam.get_ray(s, t);
                    film.add_sample(x, y, evaluator.li(scene, r, 8));
                }
            }
        }
        sample_count
    }
}
impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize> Dispatch for SingleCoreTiledDispatcher<TILE_WIDTH, TILE_HEIGHT> {
    fn dispatch(&self, scene: &Scene, sampler: Sampler, evaluator: impl RayEvaluator, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
        let mut win = MinifbWindow::new(film.width, film.height);
        let mut win2 = MinifbWindow::new(film.width, film.height);
        let mut win3 = MinifbWindow::new(film.width, film.height);

        let tiles_hor = film.width / TILE_WIDTH + 1.min(film.width % TILE_WIDTH);
        let tiles_ver = film.height / TILE_HEIGHT + 1.min(film.height % TILE_HEIGHT);

        let mut sample_count = 0;

        for x in 0..tiles_hor {
            for y in 0..tiles_ver {
                let mut tile_film = Film::new(TILE_WIDTH, TILE_HEIGHT);
                sample_count += Self::dispatch_tile(scene, sampler, &evaluator, &mut tile_film, x, y, min_samples, max_samples, variance_target);
                film.overwrite_with(x * TILE_WIDTH, y * TILE_HEIGHT, &tile_film);

                win.update(&film, SampleCollector::gamma_corrected_mean);
                win2.update(&film, SampleCollector::variance);
                win3.update(&film, SampleCollector::avg_variance);

                print_progress((y + x * tiles_ver) as Float / (tiles_hor * tiles_ver) as Float);
            }
        }

        println!("");
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (film.width * film.height * max_samples) as Float);
    }
}

pub struct MultiCoreTiledDispatcher<const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WORKER_COUNT: usize>;
impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WORKER_COUNT: usize> MultiCoreTiledDispatcher<TILE_WIDTH, TILE_HEIGHT, WORKER_COUNT> {
    pub fn worker_job(scene: Arc<Scene>, sampler: Sampler, evaluator: Arc<impl RayEvaluator>, film: Arc<Mutex<Film>>, (topleft_x, topleft_y): (usize, usize), (tile_width, tile_height): (usize, usize), min_samples: usize, max_samples: usize, variance_target: Float, _out: &mut dyn Write) -> usize {
        let mut sample_count = 0;
        let mut local_film = Film::new(tile_width, tile_height);

        for y in 0..tile_height {
            for x in 0..tile_width {
                for n in 0..max_samples {
                    if n >= min_samples && local_film.sample_collector(x, y).max_variance() <= variance_target { break; }

                    sample_count += 1;
                    let (s, t) = sampler.get_pixel_sample(topleft_x + x, topleft_y + y);
                    let r = scene.cam.get_ray(s, t);
                    local_film.add_sample(x, y, evaluator.li(&scene, r, 8));
                }
            }
        }

        Png::write(tile_width, tile_height, local_film.to_rgb8(|s| color_gamma(s.mean())), &format!("out/jobs/out-{topleft_x}-{topleft_y}.png"));
        film.lock().unwrap().overwrite_with(topleft_x, topleft_y, &local_film);
        sample_count
    }   
}
impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WORKER_COUNT: usize> Dispatch for MultiCoreTiledDispatcher<TILE_WIDTH, TILE_HEIGHT, WORKER_COUNT> {
    fn dispatch(&self, scene: &Scene, sampler: Sampler, evaluator: impl RayEvaluator, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
        let tiles_hor = film.width / TILE_WIDTH + 1.min(film.width % TILE_WIDTH);
        let tiles_ver = film.height / TILE_HEIGHT + 1.min(film.height % TILE_HEIGHT);
        let total_samples = film.width * film.height * max_samples;

        // TODO Arc and Mutex desire ownership, but we want the dispatch interface to not have to be thread-aware.
        // But this leads to excessive cloning here. Can we fix that?
        let film_arc = Arc::new(Mutex::new(replace(film, Film::new(1, 1))));
        let scene_arc = Arc::new(scene.clone());
        let evaluator_arc = Arc::new(evaluator.clone());

        let queue = JobQueue::make_shared(Vec::new());

        // TODO Would it be better to pass scene and film to the threads instead of to the jobs?
        for x in 0..tiles_hor {
            for y in 0..tiles_ver {
                let film = film_arc.clone();
                let scene = scene_arc.clone();
                let evaluator = evaluator_arc.clone();
                queue.add_job(move |out: &mut dyn Write| 
                    Self::worker_job(scene, sampler, evaluator, film, (x * TILE_WIDTH, y * TILE_HEIGHT), (TILE_WIDTH, TILE_HEIGHT), min_samples, max_samples, variance_target, out)
                );
            }
        }

        let sample_count = Arc::new(Mutex::new(0));

        let threads: Vec<_> = (0..WORKER_COUNT).map(|i| {
            let queue = queue.clone();
            let sample_count = sample_count.clone();
            std::thread::spawn(move || {
                let mut out = File::create(format!("out/worker-{i}.log")).unwrap();
                write!(out, "thread {i} reporting\n").unwrap();
                while let Some(job) = queue.get_job() {
                    let local_sample_count = job(&mut out);
                    write!(out, "finished job with {local_sample_count} samples\n").unwrap();
                    *sample_count.lock().unwrap() += local_sample_count;
                }
                write!(out, "thread {i} done\n").unwrap();
            })
        }).collect();

        let prog_thread = std::thread::spawn({
            let sample_count = sample_count.clone();
            let film_arc = film_arc.clone();
            let width = film_arc.lock().unwrap().width;
            let height = film_arc.lock().unwrap().height;
            move || {
                let mut win = MinifbWindow::new(width, height);
                win.set_position(0, 0);
                let mut win2 = MinifbWindow::new(width, height);
                win2.set_position(width as isize, 0);
                let mut win3 = MinifbWindow::new(width, height);
                win3.set_position(width as isize, height as isize);

                while !queue.jobs.lock().unwrap().is_empty() {
                    let progress = *sample_count.lock().unwrap() as Float / total_samples as Float;
                    print_progress(progress);

                    {
                        let film = film_arc.lock().unwrap();
                        win.update(&film, SampleCollector::gamma_corrected_mean);
                        win2.update(&film, SampleCollector::variance);
                        win3.update(&film, SampleCollector::avg_variance);
                    }

                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
        });

        println!("waiting for threads to finish...");
        for thread in threads { thread.join().unwrap(); }
        prog_thread.join().unwrap();
        println!("threads finished");

        let sample_count = *sample_count.lock().unwrap();
        *film = Arc::into_inner(film_arc).unwrap().into_inner().unwrap();

        println!("");
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (film.width * film.height * max_samples) as Float);
    }
}

pub struct Integrator<Eval: RayEvaluator, Disp: Dispatch> {
    eval: Eval,
    dispatch: Disp
}

impl<Eval: RayEvaluator, Disp: Dispatch> Integrator<Eval, Disp> {
    pub fn new(eval: Eval, dispatch: Disp) -> Self {
        Integrator { eval, dispatch }
    }
    pub fn dispatch(&mut self, scene: &Scene, sampler: Sampler, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
        self.dispatch.dispatch(scene, sampler, self.eval.clone(), film, min_samples, max_samples, variance_target);
    }
}