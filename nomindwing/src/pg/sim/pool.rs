//! Worker threads for the planner (`arch.rs`). A job is one fight played forward: it works from
//! its own copy of the fight and gives back a number, so it comes out the same on any thread, at
//! any speed, in any order. Nothing in a match may depend on when a job finishes, only on what it
//! gives: the planner asks for an answer on a tick fixed in advance, and waits if it isn't ready.
//!
//! A job nobody has begun is run by whoever gets to it first: a worker, or the thread that asks
//! for its answer. So asking never waits on a queue, a machine with one core still plays, and a
//! tool that fills every core with matches of its own loses nothing.

use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

type Work = Box<dyn FnOnce() -> i32 + Send>;

enum State {
    /// Not begun: whoever gets to it first runs it.
    Queued(Work),
    Running,
    Done(i32),
    /// It panicked (a check inside the fight it played failed): whoever asks is told so.
    Failed(String),
}

/// One fight being played forward, somewhere.
pub struct Job {
    state: Mutex<State>,
    done: Condvar,
    /// How long it took to work out, once it has been, in nanoseconds. Only tools read it.
    pub nanos: AtomicU64,
}

/// Where a thread's jobs are run. Only tests and tools change it, and it changes nothing a job
/// gives back: that's what the tests that change it are for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum Mode {
    /// On the worker threads.
    Pool,
    /// On no thread but this one, each when its answer is asked for.
    Inline,
    /// On the worker threads, each held up for a while first (up to 12 ms, by turns none at
    /// all), as a busy or a slow machine would hold it.
    Slow,
}

thread_local! {
    pub static MODE: Cell<Mode> = const { Cell::new(Mode::Pool) };
    static SLOW_TURN: Cell<u64> = const { Cell::new(0) };
}

/// Jobs that had to be waited for or run by the thread that asked, and the nanoseconds that
/// took, over the whole program. Only `--frame-cost` and `--ways` read them.
pub static WAITED: AtomicU64 = AtomicU64::new(0);
pub static WAIT_NANOS: AtomicU64 = AtomicU64::new(0);

impl Job {
    /// The answer: at once if it's ready, after a wait if a worker has it in hand, and worked
    /// out here and now if nobody has begun it.
    pub fn get(&self) -> i32 {
        let mut st = self.state.lock().unwrap();
        let mut clock: Option<std::time::Instant> = None;
        loop {
            match &*st {
                State::Done(v) => {
                    let v = *v;
                    if let Some(c) = clock {
                        WAITED.fetch_add(1, Ordering::Relaxed);
                        WAIT_NANOS.fetch_add(c.elapsed().as_nanos() as u64, Ordering::Relaxed);
                    }
                    return v;
                }
                State::Failed(why) => panic!("a fight the planner played forward failed: {why}"),
                State::Running => {
                    clock.get_or_insert_with(std::time::Instant::now);
                    st = self.done.wait(st).unwrap();
                }
                State::Queued(_) => {
                    clock.get_or_insert_with(std::time::Instant::now);
                    let State::Queued(work) = std::mem::replace(&mut *st, State::Running) else { unreachable!() };
                    drop(st);
                    let out = run(work, &self.nanos);
                    st = self.state.lock().unwrap();
                    *st = out;
                    self.done.notify_all();
                }
            }
        }
    }

    /// Whether the answer is in.
    pub fn ready(&self) -> bool {
        matches!(*self.state.lock().unwrap(), State::Done(_) | State::Failed(_))
    }
}

fn run(work: Work, nanos: &AtomicU64) -> State {
    let clock = std::time::Instant::now();
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    nanos.store(clock.elapsed().as_nanos() as u64, Ordering::Relaxed);
    match out {
        Ok(v) => State::Done(v),
        Err(e) => State::Failed(e.downcast_ref::<String>().cloned().or(e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| "it panicked".into())),
    }
}

struct Pool {
    queue: Mutex<VecDeque<(Arc<Job>, Duration)>>,
    wake: Condvar,
}

impl Pool {
    fn work(&self) {
        loop {
            let (job, hold) = {
                let mut q = self.queue.lock().unwrap();
                loop {
                    match q.pop_front() {
                        Some(j) => break j,
                        None => q = self.wake.wait(q).unwrap(),
                    }
                }
            };
            // A job nobody is waiting on any more (its fight was started again, or the match is
            // over) isn't worth doing: the queue's is the only hold on it left.
            if Arc::strong_count(&job) == 1 {
                continue;
            }
            // Take it, unless the thread that wants it has already.
            let work = {
                let mut st = job.state.lock().unwrap();
                match std::mem::replace(&mut *st, State::Running) {
                    State::Queued(work) => work,
                    other => {
                        *st = other;
                        continue;
                    }
                }
            };
            if !hold.is_zero() {
                std::thread::sleep(hold);
            }
            let out = run(work, &job.nanos);
            *job.state.lock().unwrap() = out;
            job.done.notify_all();
        }
    }
}

/// `--workers N`, for measuring: how many worker threads to start, if it's set before the first
/// job (0: as many as `workers` says).
pub static WORKERS: AtomicU64 = AtomicU64::new(0);

/// How many worker threads there are: every core but one, which is the game's own (at least
/// one, at most sixteen). It changes how soon answers are ready and nothing else.
pub fn workers() -> usize {
    match WORKERS.load(Ordering::Relaxed) {
        0 => std::thread::available_parallelism().map_or(1, |n| n.get()).saturating_sub(1).clamp(1, 16),
        n => n as usize,
    }
}

fn pool() -> &'static Pool {
    static POOL: OnceLock<&'static Pool> = OnceLock::new();
    POOL.get_or_init(|| {
        let p: &'static Pool = Box::leak(Box::new(Pool { queue: Mutex::new(VecDeque::new()), wake: Condvar::new() }));
        for k in 0..workers() {
            std::thread::Builder::new()
                .name(format!("planner {k}"))
                .spawn(move || {
                    below_normal();
                    p.work()
                })
                .expect("couldn't start a planner thread");
        }
        p
    })
}

/// The workers give way to the game's own thread, and to the sound and the window.
fn below_normal() {
    unsafe {
        use winapi::um::processthreadsapi::{GetCurrentThread, SetThreadPriority};
        SetThreadPriority(GetCurrentThread(), winapi::um::winbase::THREAD_PRIORITY_BELOW_NORMAL as i32);
    }
}

/// Hands a job over. The answer is asked for with `Job::get`.
pub fn submit(work: Work) -> Arc<Job> {
    let job = Arc::new(Job { state: Mutex::new(State::Queued(work)), done: Condvar::new(), nanos: AtomicU64::new(0) });
    let hold = match MODE.get() {
        Mode::Inline => return job,
        Mode::Pool => Duration::ZERO,
        Mode::Slow => {
            let k = SLOW_TURN.get();
            SLOW_TURN.set(k + 1);
            Duration::from_millis([0, 3, 0, 12, 1, 0, 7][(k % 7) as usize])
        }
    };
    let p = pool();
    p.queue.lock().unwrap().push_back((job.clone(), hold));
    p.wake.notify_one();
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_gives_the_same_answer_wherever_it_runs_and_to_everyone_who_asks() {
        for mode in [Mode::Pool, Mode::Inline, Mode::Slow] {
            MODE.set(mode);
            let jobs: Vec<Arc<Job>> = (0..40).map(|k| submit(Box::new(move || (0..=k).sum::<i32>()))).collect();
            for (k, j) in jobs.iter().enumerate() {
                let want = (0..=k as i32).sum::<i32>();
                // Asked twice, and through a second handle: the same each time.
                assert_eq!((j.get(), j.clone().get(), j.get()), (want, want, want), "{mode:?}");
                assert!(j.ready());
            }
        }
        MODE.set(Mode::Pool);
    }

    #[test]
    fn a_job_that_panics_tells_whoever_asks() {
        MODE.set(Mode::Inline);
        let j = submit(Box::new(|| panic!("the fight broke a rule")));
        let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| j.get()));
        MODE.set(Mode::Pool);
        assert!(got.is_err());
    }
}
