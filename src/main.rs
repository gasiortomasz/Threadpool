use std::{thread, time};

mod threadpool;

fn main() {
    let pool = threadpool::Threadpool::new(4);
    let ten_millis = time::Duration::from_millis(1000);

    thread::sleep(ten_millis);
    (0..4).for_each(|i| {
        pool.submit(Box::new(move || {
            println!("Start new task {}", i);
            thread::sleep(time::Duration::from_secs(4));
            println!("End task {}", i);
        }));
        thread::sleep(time::Duration::from_millis(300));
    });
}
