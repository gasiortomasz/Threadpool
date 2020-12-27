use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::{collections::VecDeque, sync::MutexGuard};

use thread::JoinHandle;

type Task = Box<dyn FnOnce() + Send>;

pub struct Threadpool {
    pool_size: u32,
    thread_handles: Vec<JoinHandle<()>>,
    shared_state: Arc<(Mutex<VecDeque<Task>>, Condvar)>,
    poison_pills: Arc<AtomicU32>,
}

impl Threadpool {
    pub fn new(pool_size: u32) -> Self {
        let shared_state = Arc::new((Mutex::new(VecDeque::<Task>::new()), Condvar::new()));
        let poison_pills = Arc::new(AtomicU32::new(0));

        let predicate = |guard: &MutexGuard<VecDeque<Task>>, poison_counter: &Arc<AtomicU32>| {
            guard.deref().is_empty() && poison_counter.load(Ordering::SeqCst) == 0
        };

        let thread_handles = (0..pool_size)
            .map(|_| {
                let cloned = shared_state.clone();
                let atomic = poison_pills.clone();

                thread::spawn(move || {
                    let (ref lock, ref cond) = *cloned;
                    loop {
                        let task = {
                            let mut task_queue = lock.lock().unwrap();
                            while predicate(&task_queue, &atomic) {
                                task_queue = cond.wait(task_queue).unwrap();
                            }

                            match task_queue.deref().is_empty() {
                                true => {
                                    atomic.fetch_sub(1, Ordering::SeqCst);
                                    break;
                                }
                                false => task_queue.pop_front().unwrap(),
                            }
                        };
                        cond.notify_all();
                        task();
                    }
                })
            })
            .collect();

        Self {
            thread_handles,
            pool_size,
            shared_state,
            poison_pills,
        }
    }

    pub fn submit(&self, task: Task) {
        let (ref lock, ref cond) = *self.shared_state;
        {
            let mut task_queue = lock.lock().unwrap();
            task_queue.push_back(task);
        }
        cond.notify_all();
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        let (_, ref cond) = *self.shared_state;
        self.poison_pills
            .fetch_add(self.pool_size, Ordering::SeqCst);
        cond.notify_all();

        self.thread_handles
            .drain(..)
            .for_each(|th| th.join().unwrap())
    }
}
