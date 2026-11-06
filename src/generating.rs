//! A field being generated off the UI thread, so the window never waits for one (ADR 0001).
//!
//! It runs whatever [`generate::lay`] does for the mode, so every mode that lays on the first
//! click is covered with no change here: natively on a thread of its own, on the web in a
//! worker running the same wasm (`web/worker.js`), which sends the field back as one byte per
//! cell. Where no worker can start, the web lays the field on the UI thread instead.
//! Dropping a [`Generating`] cancels it: its field never lands, and the search stops at its
//! next [`generate::stopped`] check (the web ends the worker outright).

use rune::Waker;

use crate::game::{Game, Mode};
use crate::generate::{self, LevelSize};

/// What lays a field: [`generate::lay`], or a stand-in for tests.
pub type Lay = fn(Mode, LevelSize, u64, usize) -> Game;

/// The field (mode, level, seed, first click) name, on its way.
pub struct Generating {
    /// The first click it was asked for, opened once the field lands.
    pub first: usize,
    job: imp::Job,
}

impl Generating {
    /// Starts laying the field `lay` gives for these inputs; `wake` runs once it is ready.
    /// The web's worker runs [`generate::lay`] whatever `lay` is.
    pub fn start(lay: Lay, mode: Mode, dims: LevelSize, seed: u64, first: usize, wake: Option<Waker>) -> Generating {
        Generating { first, job: imp::Job::start(lay, mode, dims, seed, first, wake) }
    }

    /// The field, once, when it is ready.
    pub fn take(&mut self) -> Option<Game> {
        self.job.take()
    }

    /// Waits for the field (scenes and tests). The web lays it here, on the UI thread.
    pub fn wait(self) -> Game {
        self.job.wait()
    }
}

/// Gets a web worker ready before a first click needs it (nothing natively).
pub fn warm() {
    #[cfg(target_arch = "wasm32")]
    imp::warm();
}

/// A field's mines as one byte per cell, 1 for a mine: how the web's worker sends it back.
pub fn mask(g: &Game) -> Vec<u8> {
    g.cells.iter().map(|c| c.mine as u8).collect()
}

/// The field with mines where `mask` has them, nothing open.
pub fn from_mask(w: usize, h: usize, mask: &[u8]) -> Game {
    let mines: Vec<usize> = (0..mask.len()).filter(|&i| mask[i] != 0).collect();
    Game::with_mines(w, h, &mines)
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{Receiver, TryRecvError, channel};

    use super::*;

    pub struct Job {
        field: Receiver<Game>,
        stop: Arc<AtomicBool>,
    }

    impl Job {
        pub fn start(lay: Lay, mode: Mode, dims: LevelSize, seed: u64, first: usize, wake: Option<Waker>) -> Job {
            let (send, field) = channel();
            let stop = Arc::new(AtomicBool::new(false));
            let stopped = stop.clone();
            std::thread::Builder::new()
                .name("generate".into())
                .spawn(move || {
                    let g = generate::stoppable(stopped.clone(), || lay(mode, dims, seed, first));
                    // A cancelled field lands nowhere and wakes nobody.
                    if !stopped.load(Ordering::Relaxed) && send.send(g).is_ok() {
                        if let Some(wake) = wake {
                            wake();
                        }
                    }
                })
                .expect("a thread to generate on");
            Job { field, stop }
        }

        pub fn take(&mut self) -> Option<Game> {
            match self.field.try_recv() {
                Ok(g) => Some(g),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
            }
        }

        pub fn wait(self) -> Game {
            self.field.recv().expect("the generation thread ended without a field")
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use web_sys::{MessageEvent, Worker, WorkerOptions, WorkerType};

    use super::*;

    thread_local! {
        /// A worker with the wasm loaded and nothing to do, for the next first click.
        static SPARE: RefCell<Option<Worker>> = const { RefCell::new(None) };
    }

    /// A worker running `worker.js` beside the page, handed this very wasm module.
    fn spawn() -> Option<Worker> {
        let opts = WorkerOptions::new();
        opts.set_type(WorkerType::Module);
        let w = Worker::new_with_options("worker.js", &opts).ok()?;
        w.post_message(&wasm_bindgen::module()).ok()?;
        Some(w)
    }

    pub fn warm() {
        SPARE.with(|s| {
            let mut s = s.borrow_mut();
            if s.is_none() {
                *s = spawn();
            }
        });
    }

    enum Reply {
        Waiting,
        Mines(Vec<u8>),
        /// No worker, or it failed: the field is laid on the UI thread.
        Failed,
    }

    pub struct Job {
        args: (Lay, Mode, LevelSize, u64, usize),
        worker: Option<Worker>,
        reply: Rc<RefCell<Reply>>,
        _listeners: Option<(Closure<dyn FnMut(MessageEvent)>, Closure<dyn FnMut(JsValue)>)>,
    }

    impl Job {
        pub fn start(lay: Lay, mode: Mode, dims: LevelSize, seed: u64, first: usize, wake: Option<Waker>) -> Job {
            let worker = SPARE.with(|s| s.borrow_mut().take()).or_else(spawn);
            let reply = Rc::new(RefCell::new(if worker.is_some() { Reply::Waiting } else { Reply::Failed }));
            let mut listeners = None;
            if let Some(w) = &worker {
                let (r, wake_) = (reply.clone(), wake.clone());
                let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                    let data = e.data();
                    *r.borrow_mut() = match data.dyn_into::<js_sys::Uint8Array>() {
                        Ok(a) if a.length() as usize == dims.0 * dims.1 => Reply::Mines(a.to_vec()),
                        _ => Reply::Failed,
                    };
                    if let Some(wake) = &wake_ {
                        wake();
                    }
                });
                let r = reply.clone();
                let on_error = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                    *r.borrow_mut() = Reply::Failed;
                    if let Some(wake) = &wake {
                        wake();
                    }
                });
                w.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                w.set_onerror(Some(on_error.as_ref().unchecked_ref()));
                let job = js_sys::Array::new();
                job.push(&mode.tag().into());
                for n in [dims.0, dims.1, dims.2] {
                    job.push(&(n as u32).into());
                }
                job.push(&js_sys::BigInt::from(seed).into());
                job.push(&(first as u32).into());
                if w.post_message(&job).is_err() {
                    *reply.borrow_mut() = Reply::Failed;
                }
                listeners = Some((on_message, on_error));
            }
            Job { args: (lay, mode, dims, seed, first), worker, reply, _listeners: listeners }
        }

        fn inline(&self) -> Game {
            let (lay, mode, dims, seed, first) = self.args;
            lay(mode, dims, seed, first)
        }

        pub fn take(&mut self) -> Option<Game> {
            let reply = std::mem::replace(&mut *self.reply.borrow_mut(), Reply::Waiting);
            match reply {
                Reply::Waiting => None,
                Reply::Mines(m) => {
                    // Done and idle: it serves the next field.
                    if let Some(w) = self.worker.take() {
                        w.set_onmessage(None);
                        w.set_onerror(None);
                        SPARE.with(|s| {
                            let mut s = s.borrow_mut();
                            match s.as_ref() {
                                None => *s = Some(w),
                                Some(_) => w.terminate(),
                            }
                        });
                    }
                    Some(from_mask(self.args.2.0, self.args.2.1, &m))
                }
                Reply::Failed => {
                    if let Some(w) = self.worker.take() {
                        w.terminate();
                    }
                    Some(self.inline())
                }
            }
        }

        pub fn wait(mut self) -> Game {
            if let Some(w) = self.worker.take() {
                w.terminate();
            }
            self.inline()
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            // Still searching: end the worker, a fresh one serves the next field.
            if let Some(w) = self.worker.take() {
                w.terminate();
            }
        }
    }

    /// The worker's half: the field `generate::lay` gives, as one byte per cell (empty for an
    /// unknown mode).
    #[wasm_bindgen]
    pub fn lay_mines(mode: &str, w: usize, h: usize, mines: usize, seed: u64, first: usize) -> Vec<u8> {
        match Mode::from_tag(mode) {
            Some(mode) if first < w * h => mask(&generate::lay(mode, (w, h, mines), seed, first)),
            _ => Vec::new(),
        }
    }
}
