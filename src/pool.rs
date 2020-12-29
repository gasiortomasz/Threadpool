use crate::worker::{Message, Worker};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Threadpool {
    pool_size: u8,
    workers: Vec<Worker>,
    shared_state: Arc<(Mutex<VecDeque<Message>>, Condvar)>,
}

impl Threadpool {
    pub fn new(pool_size: u8) -> Self {
        let shared_state = Arc::new((Mutex::new(VecDeque::<Message>::new()), Condvar::new()));

        let workers = (0..pool_size)
            .map(|id| Worker::new(id as usize, shared_state.clone()))
            .collect();

        Self {
            pool_size,
            workers,
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

        for worker in self.workers.iter_mut() {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
