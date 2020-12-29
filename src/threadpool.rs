use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

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

        let thread_handles = (0..pool_size)
            .map(|_| {
                let cloned_shared_state = shared_state.clone();

                thread::spawn(move || {
                    let (ref lock, ref cond) = *cloned_shared_state;
                    loop {
                        let task = {
                            let mut msg_queue = lock.lock().unwrap();
                            while msg_queue.is_empty() {
                                msg_queue = cond.wait(msg_queue).unwrap();
                            }
                            msg_queue.pop_front().unwrap()
                        };
                        cond.notify_one();
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
        let (lock, cond) = &*self.shared_state;
        {
            let mut task_queue = lock.lock().unwrap();
            let task = Box::new(f);
            task_queue.push_back(Message::Task(task));
        }
        cond.notify_one();
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        let (lock, cond) = &*self.shared_state;
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
