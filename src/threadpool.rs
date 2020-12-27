use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

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

        let thread_handles = (0..pool_size)
            .map(|_| {
                let cloned = shared_state.clone();
                let atomic = poison_pills.clone();

                thread::spawn(move || {
                    let (ref lock, ref cond) = *cloned;
                    loop {
                        let task = {
                            let mut shared_value = lock.lock().unwrap();
                            while shared_value.deref().is_empty()
                                && atomic.load(Ordering::SeqCst) == 0
                            {
                                shared_value = cond.wait(shared_value).unwrap();
                            }
                            if shared_value.deref().is_empty() {
                                atomic.fetch_sub(1, Ordering::SeqCst);
                                break;
                            }
                            shared_value.pop_front().unwrap()
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

        for thread_handle in self.thread_handles.drain(..) {
            thread_handle.join().unwrap();
        }
    }
}
