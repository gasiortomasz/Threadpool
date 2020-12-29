use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, shared_state: Arc<(Mutex<VecDeque<Message>>, Condvar)>) -> Self {
        let thread = thread::spawn(move || {
            let (lock, cond) = &*shared_state;
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
                    Message::Task(task) => task(),
                    Message::Kill => break,
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

enum Message {
    Task(Box<dyn FnOnce() + Send>),
    Kill,
}

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
