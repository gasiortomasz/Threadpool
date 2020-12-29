extern crate threadpool;
use std::{thread, time};

use threadpool::Threadpool;

fn main() {
    let pool = Threadpool::new(4);

    (0..5).for_each(|i| {
        pool.submit(move || {
            println!("Start new task {}", i);
            thread::sleep(time::Duration::from_secs(2));
            println!("End task {}", i);
        });
        thread::sleep(time::Duration::from_millis(300));
    });
}
