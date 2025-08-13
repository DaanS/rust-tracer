use std::{fs::File, io::Write, sync::{Arc, Mutex}, thread};

mod random;
mod config;
mod color;
mod film;
mod sampler;
mod conversion;
mod util;

fn do_work_dyn(a: i64, b: i64, out: &mut dyn Write) {
    writeln!(out, "{a} + {b} = {}", a + b).unwrap();
}

fn do_work_impl(a: i64, b: i64, mut out: impl Write) {
    writeln!(out, "{a} + {b} = {}", a + b).unwrap();
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

fn main() {
    let queue = JobQueue::make_shared(
        (0..64).map(|i| move |out: &mut dyn Write| do_work_dyn(i, i + 1, out)).collect()
    );

    let queue = JobQueue::make_shared(Vec::new());
    for i in 0..64 {
        queue.add_job(move |out: &mut dyn Write| do_work_dyn(i, i + 1, out));
    }


    let threads: Vec<_> = (1..=8).map(|i| {
        let mut out = File::create(format!("worker-{i}.log")).unwrap();
        let queue = Arc::clone(&queue);
        thread::spawn(move || {
            writeln!(out, "thread {i} reporting").unwrap();
            while let Some(job) = queue.get_job() {
                job(&mut out);
            }
        })
    }).collect();

    for t in threads { t.join().unwrap(); }
}