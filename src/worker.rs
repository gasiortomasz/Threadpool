use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

pub enum Message {
    Task(Box<dyn FnOnce() + Send>),
    Kill,
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, shared_state: Arc<(Mutex<VecDeque<Message>>, Condvar)>) -> Self {
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
