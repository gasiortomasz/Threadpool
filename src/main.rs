use std::{thread, time};

mod threadpool;

fn main() {
    let pool = threadpool::Threadpool::new(4);

    (0..5).for_each(|i| {
        pool.submit(move || {
            println!("Start new task {}", i);
            thread::sleep(time::Duration::from_secs(2));
            println!("End task {}", i);
        });
        thread::sleep(time::Duration::from_millis(300));
    });
}
