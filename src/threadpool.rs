use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::{collections::VecDeque, sync::MutexGuard};

use thread::JoinHandle;

enum Message {
    Task(Box<dyn FnOnce() + Send>),
    Kill,
}

pub struct Threadpool {
    pool_size: u32,
    thread_handles: Vec<JoinHandle<()>>,
    shared_state: Arc<(Mutex<VecDeque<Message>>, Condvar)>,
}

impl Threadpool {
    pub fn new(pool_size: u32) -> Self {
        let shared_state = Arc::new((Mutex::new(VecDeque::<Message>::new()), Condvar::new()));

        let predicate = |guard: &MutexGuard<VecDeque<Message>>| guard.deref().is_empty();

        let thread_handles = (0..pool_size)
            .map(|_| {
                let cloned = shared_state.clone();

                thread::spawn(move || {
                    let (ref lock, ref cond) = *cloned;
                    loop {
                        let task = {
                            let mut task_queue = lock.lock().unwrap();
                            while predicate(&task_queue) {
                                task_queue = cond.wait(task_queue).unwrap();
                            }
                            task_queue.pop_front().unwrap()
                        };
                        cond.notify_all();
                        match task {
                            Message::Task(f) => f(),
                            Message::Kill => break,
                        }
                    }
                })
            })
            .collect();

        Self {
            thread_handles,
            pool_size,
            shared_state,
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (ref lock, ref cond) = *self.shared_state;
        {
            let mut task_queue = lock.lock().unwrap();
            let task = Box::new(f);
            task_queue.push_back(Message::Task(task));
        }
        cond.notify_all();
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        let (ref lock, ref cond) = *self.shared_state;
        {
            let mut task_queue = lock.lock().unwrap();
            (0..self.pool_size).for_each(|_| task_queue.push_back(Message::Kill));
        }
        cond.notify_all();

        self.thread_handles
            .drain(..)
            .for_each(|th| th.join().unwrap())
    }
}
